use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::checksum::{find_checksum_file, verify_file};
use crate::config::Config;
use crate::extract::extract_archive;
use crate::github::{find_matching_asset, parse_package, Asset, GithubClient, Platform, Release};
use crate::registry::{InstalledPackage, Registry};
use crate::util::{bin_dir_from, registry_path_from};

pub async fn handle_install(
    package: &str,
    force: bool,
    dry_run: bool,
    verify: bool,
    config: &Config,
) -> Result<()> {
    let (owner, repo, version) = parse_package(package)?;
    let key = format!("{}/{}", owner, repo);

    let registry_path = registry_path_from(&config.install_dir);
    let mut reg = Registry::load_from(&registry_path)?;
    if !force && reg.is_installed(&key) {
        let pkg = reg.packages.get(&key).unwrap();
        println!(
            "{} already installed ({}). Use --force to reinstall.",
            key, pkg.version
        );
        return Ok(());
    }

    let client = GithubClient::new(config.github_token.clone())?;
    let release = match &version {
        Some(v) => client.get_release(&owner, &repo, v).await?,
        None => client.get_release(&owner, &repo, "latest").await?,
    };

    let asset = select_best_asset(&release)?;

    let pkg_install_dir = config.install_dir.join("packages").join(&key);
    let bin_dir = bin_dir_from(&config.install_dir);

    if dry_run {
        println!("[DRY RUN] Would install {}:", key);
        println!("  Release:      {}", release.tag_name);
        println!("  Asset:        {}", asset.name);
        println!("  Install dir:  {}", pkg_install_dir.display());
        println!("  Binary:       {}/{}", pkg_install_dir.display(), repo);
        println!("  Symlink:      {}/{}", bin_dir.display(), repo);
        return Ok(());
    }

    if !config.output.quiet {
        println!("Release: {}", release.tag_name);
        println!("Asset:   {}", asset.name);
    }

    // Download to a temporary location
    let temp_dir = std::env::temp_dir().join(format!("gitclaw-{}-{}", owner, repo));
    std::fs::create_dir_all(&temp_dir)?;
    let download_path = temp_dir.join(&asset.name);
    client
        .download_asset(asset, &download_path, config.download.show_progress)
        .await?;

    // Verify checksum if requested or configured
    if verify || config.download.verify_checksums {
        if let Some((algo, checksum_url)) = find_checksum_file(&asset.name, &release.assets) {
            let checksum_data = client.download_text(&checksum_url).await?;
            if let Some(expected) =
                crate::checksum::parse_checksum_file(&checksum_data, &asset.name)
            {
                if !config.output.quiet {
                    println!("Verifying checksum...");
                }
                verify_file(&download_path, &expected, algo)?;
                if !config.output.quiet {
                    println!("Checksum verified");
                }
            }
        } else if verify {
            bail!("Checksum verification requested but no checksum file found");
        }
    }

    let pkg_install_dir = config.install_dir.join("packages").join(&key);
    fs::create_dir_all(&pkg_install_dir)?;

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
    };
    reg.add(pkg);
    reg.save()?;

    let bin_dir = bin_dir_from(&config.install_dir);
    create_symlink(&binary, &repo, &bin_dir)?;

    if !config.output.quiet {
        println!("Installed {} -> {}", key, binary.display());
        println!("   Run: {}/{} (add to $PATH)", bin_dir.display(), repo);
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
        bail!("{} not installed. Use 'gitclaw install' first.", key);
    }
    let installed = reg.packages.get(&key).unwrap();
    if !config.output.quiet {
        println!("Checking {} (current: {})...", key, installed.version);
    }

    let client = GithubClient::new(config.github_token.clone())?;
    let latest = client.get_release(&owner, &repo, "latest").await?;

    if latest.tag_name == installed.version {
        if !config.output.quiet {
            println!("{} is up to date ({})", key, installed.version);
        }
        return Ok(());
    }
    if !config.output.quiet {
        println!(
            "Update available: {} -> {}",
            installed.version, latest.tag_name
        );
    }
    crate::registry::uninstall(package, &config.install_dir)?;
    handle_install(package, false, false, false, config).await
}

async fn update_all(config: &Config) -> Result<()> {
    let registry_path = registry_path_from(&config.install_dir);
    let reg = Registry::load_from(&registry_path)?;
    if reg.packages.is_empty() {
        if !config.output.quiet {
            println!("No packages installed.");
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
                    warn!("Update {} failed: {}", name, e);
                }
            }
        }
    }
    if !config.output.quiet {
        println!("\nDone: {} updated, {} current", updated, current);
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
            warn!("No platform match; using sole asset");
            Ok(&release.assets[0])
        }
        Err(e) => bail!("No suitable asset for {}: {:?}", platform, e),
    }
}

fn find_binary(dir: &Path, repo_name: &str) -> Result<PathBuf> {
    use walkdir::WalkDir;
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(entry.path())
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                return Ok(entry.path().to_path_buf());
            }
        }
        #[cfg(windows)]
        {
            return Ok(entry.path().to_path_buf());
        }
    }
    for entry in WalkDir::new(dir).max_depth(3) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(entry.path())
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                return Ok(entry.path().to_path_buf());
            }
        }
        #[cfg(windows)]
        {
            if entry
                .path()
                .extension()
                .map(|e| e == "exe")
                .unwrap_or(false)
            {
                return Ok(entry.path().to_path_buf());
            }
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
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(binary, &link)?;
    }
    #[cfg(windows)]
    {
        fs::copy(binary, &link)?;
    }
    Ok(())
}

/// Install multiple packages in parallel
pub async fn handle_install_multiple(
    packages: &[String],
    force: bool,
    dry_run: bool,
    verify: bool,
    config: &Config,
) -> Result<()> {
    use futures::future::join_all;

    let total = packages.len();
    if !config.output.quiet {
        println!("Installing {} packages...", total);
    }

    let mut tasks = Vec::new();
    for pkg in packages {
        let pkg = pkg.clone();
        let config = config.clone();
        let task =
            tokio::spawn(
                async move { handle_install(&pkg, force, dry_run, verify, &config).await },
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
                    eprintln!("Error: {}", e);
                }
                failed += 1;
            }
            Err(e) => {
                if !config.output.quiet {
                    eprintln!("Task failed: {}", e);
                }
                failed += 1;
            }
        }
    }

    if !config.output.quiet {
        println!("\nDone: {} succeeded, {} failed", success, failed);
    }

    if failed > 0 {
        bail!("{} package(s) failed to install", failed);
    }

    Ok(())
}
