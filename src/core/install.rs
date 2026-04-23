use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use colored::Colorize;
use futures::future::join_all;
use walkdir::WalkDir;

use crate::core::checksum::{find_checksum_file, verify_file};
use crate::core::config::Config;
use crate::core::constants::{APP_NAME, TEMP_DIR_PREFIX};
use crate::core::extract::extract_archive;
use crate::core::registry::{InstalledPackage, Registry};
use crate::core::semver::{parse_tag_version, VersionConstraint};
use crate::core::util::{bin_dir_from, registry_path_from};
use crate::network::github::{
    find_matching_asset, parse_package, Asset, GithubClient, Platform, Release,
};
use crate::output;
use semver::Version;

pub async fn handle_install(
    package: &str,
    force: bool,
    dry_run: bool,
    verify: bool,
    config: &Config,
    channel: Option<crate::core::channel::Channel>,
) -> Result<()> {
    let resolved = if !package.contains('/') {
        if let Some(alias_target) = crate::core::alias::AliasMap::load(config)?.resolve(package) {
            output::print_info(&format!("Alias '{}' -> '{}'.", package, alias_target));
            alias_target.to_string()
        } else {
            package.to_string()
        }
    } else {
        package.to_string()
    };

    let (owner, repo, version) = parse_package(&resolved)?;
    let key = format!("{}/{}", owner, repo);

    let aliases = crate::core::alias::AliasMap::load(config)?;
    if let Some(alias_target) = aliases.resolve(&repo) {
        if alias_target != key {
            output::print_warn(&format!(
                "Package repo '{}' conflicts with alias '{}' -> '{}'.",
                repo, repo, alias_target
            ));
            output::print_info(
                "The alias will resolve 'bat' to the aliased target, not this package.",
            );
        }
    }
    drop(aliases);

    let registry_path = registry_path_from(&config.install_dir);
    let mut reg = Registry::load_from(&registry_path)?;

    if !force && reg.is_installed(&key) {
        let pkg = reg.packages.get(&key).unwrap();
        output::print_warn(&format!(
            "{} already installed ({}). Use --force to reinstall.",
            key, pkg.version
        ));
        return Ok(());
    }

    let client = GithubClient::new(config.github_token.clone())?;

    let release = match (&version, channel) {
        (_, Some(ch)) => {
            let releases = client.get_releases(&owner, &repo).await?;
            let filtered = crate::core::channel::filter_releases(&releases, ch, None);
            if filtered.is_empty() {
                bail!("No {} release found for {}/{}.", ch, owner, repo);
            }
            filtered.into_iter().next().unwrap()
        }
        (Some(v), None) => {
            if is_semver_constraint(v) {
                let constraint = VersionConstraint::parse(v)?;
                find_matching_release(&client, &owner, &repo, &constraint).await?
            } else {
                client.get_release(&owner, &repo, v).await?
            }
        }
        (None, None) => client.get_release(&owner, &repo, "latest").await?,
    };

    let asset = select_best_asset(&release)?;

    let pkg_install_dir = config.install_dir.join("packages").join(&key);
    let bin_dir = bin_dir_from(&config.install_dir);

    if dry_run {
        output::print_info(&format!("[DRY RUN] Would install {}.", key.cyan()));
        output::print_kv("Release", &release.tag_name);
        output::print_kv("Asset", &asset.name);
        output::print_kv("Install dir", &pkg_install_dir.display().to_string());
        output::print_kv("Binary", &format!("{}/{}", pkg_install_dir.display(), repo));
        output::print_kv("Symlink", &format!("{}/{}", bin_dir.display(), repo));
        return Ok(());
    }

    if !config.output.quiet {
        output::print_info(&format!("Installing {}.", key.cyan().bold()));
        output::print_kv("Release", &release.tag_name);
        output::print_kv("Asset", &asset.name);
    }

    let cache_key = crate::core::cache::cache_key(&owner, &repo, &release.tag_name, &asset.name);
    let cached = crate::core::cache::get_cached(config, &cache_key, None);

    let download_path = if let Some(cached_path) = cached {
        if !config.output.quiet {
            output::print_info("Using cached archive.");
        }
        cached_path
    } else {
        let temp_dir = std::env::temp_dir().join(format!("{}{}-{}", TEMP_DIR_PREFIX, owner, repo));
        std::fs::create_dir_all(&temp_dir)?;
        let temp_path = temp_dir.join(&asset.name);

        client
            .download_asset(asset, &temp_path, config.download.show_progress)
            .await?;

        let cached_path = crate::core::cache::store(config, &cache_key, &temp_path)?;

        // clean up temp
        let _ = fs::remove_file(&temp_path);

        cached_path
    };

    println!();

    if verify || config.download.verify_checksums {
        if let Some((algo, checksum_url)) = find_checksum_file(&asset.name, &release.assets) {
            let checksum_data = client.download_text(&checksum_url).await?;

            if let Some(expected) =
                crate::core::checksum::parse_checksum_file(&checksum_data, &asset.name)
            {
                if !config.output.quiet {
                    output::print_info("Verifying checksum.");
                }

                verify_file(&download_path, &expected, algo)?;

                if !config.output.quiet {
                    output::print_success("Checksum verified.");
                }
            }
        } else if verify {
            bail!("Checksum verification requested but no checksum file found.");
        }
    }

    let pkg_install_dir = config.install_dir.join("packages").join(&key);
    fs::create_dir_all(&pkg_install_dir)?;

    if !config.output.quiet {
        println!();
        output::print_info(&format!("Extracting {}.", asset.name));
    }

    extract_archive(
        &download_path,
        &pkg_install_dir,
        config.download.prefer_strip,
    )?;

    let binary = find_binary(&pkg_install_dir, &repo)?;

    let pkg = InstalledPackage {
        name: key.clone(),
        owner: owner.clone(),
        repo: repo.clone(),
        version: release.tag_name.clone(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: binary.clone(),
        install_dir: pkg_install_dir.clone(),
        asset_name: asset.name.clone(),
        identifier: repo.clone(),
        channel: channel.map(|c| c.to_string()),
    };

    reg.add(pkg);
    reg.save()?;

    let bin_dir = bin_dir_from(&config.install_dir);
    let binary_absolute = std::fs::canonicalize(&binary)?;
    create_symlink(&binary_absolute, &repo, &bin_dir)?;

    if !config.output.quiet {
        output::print_install_complete(&key, &binary.display().to_string());
    }

    Ok(())
}

pub async fn handle_update(package: Option<&str>, config: &Config) -> Result<()> {
    match package {
        Some(p) => update_one(p, config).await,
        None => update_all(config).await,
    }
}

async fn update_one(package: &str, config: &Config) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;
    let key = format!("{}/{}", owner, repo);
    let registry_path = registry_path_from(&config.install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if !reg.is_installed(&key) {
        bail!("{} not installed. Use '{} install' first.", key, APP_NAME);
    }

    let installed = reg.packages.get(&key).unwrap();

    if !config.output.quiet {
        output::print_info(&format!(
            "Checking {} (current: {}).",
            key.dimmed(),
            installed.version.dimmed()
        ));
    }

    let client = GithubClient::new(config.github_token.clone())?;

    let ch = match installed.channel.as_deref() {
        Some(c) => Some(c.parse::<crate::core::channel::Channel>()?),
        None => None,
    };

    let latest = match ch {
        Some(channel) => {
            let releases = client.get_releases(&owner, &repo).await?;
            let filtered = crate::core::channel::filter_releases(&releases, channel, None);
            if filtered.is_empty() {
                bail!("No {} release found for {}/{}.", channel, owner, repo);
            }
            filtered.into_iter().next().unwrap()
        }
        None => client.get_release(&owner, &repo, "latest").await?,
    };

    if latest.tag_name == installed.version {
        if !config.output.quiet {
            output::print_success(&format!("{} is up to date ({}).", key, installed.version));
        }
        return Ok(());
    }

    if !config.output.quiet {
        output::print_info(&format!(
            "Update available: {} -> {}.",
            installed.version.dimmed(),
            latest.tag_name.green().bold()
        ));
    }

    crate::core::registry::uninstall(package, &config.install_dir, config)?;
    handle_install(package, false, false, false, config, ch).await
}

async fn update_all(config: &Config) -> Result<()> {
    let registry_path = registry_path_from(&config.install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if reg.packages.is_empty() {
        if !config.output.quiet {
            output::print_info("No packages installed.");
        }
        return Ok(());
    }

    let names: Vec<String> = reg.packages.keys().cloned().collect();
    let mut updated = 0u32;
    let mut current = 0u32;

    for name in &names {
        match update_one(name, config).await {
            Ok(()) => updated += 1,
            Err(e) => {
                if e.to_string().contains("up to date") {
                    current += 1;
                } else {
                    output::print_warn(&format!("Update {} failed: {}.", name, e));
                }
            }
        }
    }

    if !config.output.quiet {
        println!();
        output::print_info(&format!("Done: {} updated, {} current.", updated, current));
    }

    Ok(())
}

fn select_best_asset(release: &Release) -> Result<&Asset> {
    if release.assets.is_empty() {
        bail!("Release {} has no assets", release.tag_name);
    }

    let platform = Platform::current()?;

    match find_matching_asset(release, platform) {
        Ok(asset) => Ok(asset),
        Err(_) if release.assets.len() == 1 => {
            output::print_warn("No platform match; using sole asset.");
            Ok(&release.assets[0])
        }
        Err(e) => bail!("No suitable asset for {}: {:?}", platform, e),
    }
}

fn find_binary(dir: &Path, repo_name: &str) -> Result<PathBuf> {
    for entry in WalkDir::new(dir).max_depth(3) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let stem = entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();

        if stem != repo_name {
            continue;
        }

        if fs::metadata(entry.path())
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
        {
            return Ok(entry.path().to_path_buf());
        }
    }

    for entry in WalkDir::new(dir).max_depth(3) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        if fs::metadata(entry.path())
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
        {
            return Ok(entry.path().to_path_buf());
        }
    }

    bail!("No binary found in {}", dir.display())
}

fn create_symlink(binary: &Path, name: &str, bin_dir: &Path) -> Result<()> {
    fs::create_dir_all(bin_dir)?;
    let link = bin_dir.join(name);

    if link.exists() || link.is_symlink() {
        fs::remove_file(&link)?;
    }

    std::os::unix::fs::symlink(binary, &link)?;
    Ok(())
}

pub async fn handle_install_multiple(
    packages: &[String],
    force: bool,
    dry_run: bool,
    verify: bool,
    config: &Config,
    channel: Option<crate::core::channel::Channel>,
) -> Result<()> {
    let total = packages.len();

    if !config.output.quiet {
        output::print_info(&format!("Installing {} packages.", total));
    }

    let mut tasks = Vec::new();

    for pkg in packages {
        let pkg = pkg.clone();
        let config = config.clone();
        let ch = channel;

        let task =
            tokio::spawn(
                async move { handle_install(&pkg, force, dry_run, verify, &config, ch).await },
            );

        tasks.push(task);
    }

    let results = join_all(tasks).await;

    let mut success = 0;
    let mut failed = 0;

    for result in results {
        match result {
            Ok(Ok(())) => success += 1,
            Ok(Err(e)) => {
                if !config.output.quiet {
                    output::print_error(&format!("{}", e));
                }
                failed += 1;
            }
            Err(e) => {
                if !config.output.quiet {
                    output::print_error(&format!("Task failed: {}.", e));
                }
                failed += 1;
            }
        }
    }

    if !config.output.quiet {
        println!();
        output::print_info(&format!("Done: {} succeeded, {} failed.", success, failed));
    }

    if failed > 0 {
        bail!("{} package(s) failed to install", failed);
    }

    Ok(())
}

fn is_semver_constraint(version: &str) -> bool {
    version.starts_with('^')
        || version.starts_with('~')
        || version.starts_with('>')
        || version.starts_with('<')
        || version.starts_with('=')
}

async fn find_matching_release(
    client: &GithubClient,
    owner: &str,
    repo: &str,
    constraint: &VersionConstraint,
) -> Result<Release> {
    let releases = client.get_releases(owner, repo).await?;

    let mut matching: Vec<Release> = releases
        .into_iter()
        .filter(|r| {
            parse_tag_version(&r.tag_name)
                .map(|v| constraint.matches(&v))
                .unwrap_or(false)
        })
        .collect();

    if matching.is_empty() {
        bail!(
            "No release matching '{}' found for {}/{}.",
            constraint_display(constraint),
            owner,
            repo
        );
    }

    matching.sort_by(|a, b| {
        let va =
            parse_tag_version(&a.tag_name).unwrap_or_else(|_| Version::parse("0.0.0").unwrap());
        let vb =
            parse_tag_version(&b.tag_name).unwrap_or_else(|_| Version::parse("0.0.0").unwrap());
        vb.cmp(&va)
    });

    Ok(matching.into_iter().next().unwrap())
}

fn constraint_display(constraint: &VersionConstraint) -> String {
    match constraint {
        VersionConstraint::Exact(v) => format!("= {}", v),
        VersionConstraint::Range(r) => r.to_string(),
    }
}
