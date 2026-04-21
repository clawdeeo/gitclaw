use anyhow::{anyhow, bail, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::extract::extract_archive;
use crate::github::{find_matching_asset, parse_package, Asset, GithubClient, Platform, Release};
use crate::registry::{bin_dir, InstalledPackage, Registry};

pub async fn handle_install(package: &str, force: bool) -> Result<()> {
    let (owner, repo, version) = parse_package(package)?;
    let key = format!("{}/{}", owner, repo);

    let mut reg = Registry::load()?;
    if !force && reg.is_installed(&key) {
        let pkg = reg.packages.get(&key).unwrap();
        println!(
            "{} already installed ({}). Use --force to reinstall.",
            key, pkg.version
        );
        return Ok(());
    }

    let client = GithubClient::new(None)?;
    let release = match &version {
        Some(v) => client.get_release(&owner, &repo, v).await?,
        None => client.get_release(&owner, &repo, "latest").await?,
    };

    let asset = select_best_asset(&release)?;
    println!("Release: {}", release.tag_name);
    println!("Asset:   {}", asset.name);

    // Download to a temporary location
    let temp_dir = std::env::temp_dir().join(format!("gitclaw-{}-{}", owner, repo));
    std::fs::create_dir_all(&temp_dir)?;
    let download_path = temp_dir.join(&asset.name);
    client.download_asset(asset, &download_path).await?;

    let install_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("No home directory"))?
        .join(".gitclaw")
        .join("packages")
        .join(&key);
    fs::create_dir_all(&install_dir)?;

    extract_archive(&download_path, &install_dir)?;
    let binary = find_binary(&install_dir, &repo)?;

    let pkg = InstalledPackage {
        name: key.clone(),
        owner: owner.clone(),
        repo: repo.clone(),
        version: release.tag_name.clone(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: binary.clone(),
        install_dir: install_dir.clone(),
        asset_name: asset.name.clone(),
    };
    reg.add(pkg);
    reg.save()?;

    create_symlink(&binary, &repo)?;

    println!("Installed {} -> {}", key, binary.display());
    println!("   Run: ~/.gitclaw/bin/{} (add to $PATH)", repo);
    Ok(())
}

pub async fn handle_update(package: Option<&str>) -> Result<()> {
    match package {
        Some(p) => update_one(p).await,
        None => update_all().await,
    }
}

async fn update_one(package: &str) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;
    let key = format!("{}/{}", owner, repo);
    let reg = Registry::load()?;
    if !reg.is_installed(&key) {
        bail!("{} not installed. Use 'gitclaw install' first.", key);
    }
    let installed = reg.packages.get(&key).unwrap();
    println!("Checking {} (current: {})...", key, installed.version);

    let client = GithubClient::new(None)?;
    let latest = client.get_release(&owner, &repo, "latest").await?;

    if latest.tag_name == installed.version {
        println!("{} is up to date ({})", key, installed.version);
        return Ok(());
    }
    println!(
        "Update available: {} -> {}",
        installed.version, latest.tag_name
    );
    crate::registry::uninstall(package)?;
    handle_install(package, false).await
}

async fn update_all() -> Result<()> {
    let reg = Registry::load()?;
    if reg.packages.is_empty() {
        println!("No packages installed.");
        return Ok(());
    }
    let names: Vec<String> = reg.packages.keys().cloned().collect();
    let mut updated = 0u32;
    let mut current = 0u32;
    for name in &names {
        match update_one(name).await {
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
    println!("\nDone: {} updated, {} current", updated, current);
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

fn create_symlink(binary: &Path, name: &str) -> Result<()> {
    let dir = bin_dir()?;
    fs::create_dir_all(&dir)?;
    let link = dir.join(name);
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
