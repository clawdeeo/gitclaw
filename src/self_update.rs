use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::path::PathBuf;

use crate::config::Config;
use crate::extract::extract_archive;
use crate::github::{find_matching_asset, GithubClient, Platform};

const REPO_OWNER: &str = "clawdeeo";
const REPO_NAME: &str = "gitclaw";

/// Get the current executable path
fn current_executable() -> Result<PathBuf> {
    env::current_exe().context("Failed to get current executable path")
}

/// Get the current version from Cargo.toml
fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check for updates without installing
pub async fn check_for_update(config: &Config) -> Result<()> {
    let client = GithubClient::new(config.github_token.clone())?;
    let release = client.get_release(REPO_OWNER, REPO_NAME, "latest").await?;

    let current = current_version();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    println!("Current version: {}", current);
    println!("Latest version:  {}", latest);

    if latest == current {
        println!("gitclaw is up to date!");
    } else {
        println!("Update available: {} -> {}", current, latest);
        println!("Run 'gitclaw self-update' to install");
    }

    Ok(())
}

/// Perform self-update
pub async fn perform_update(config: &Config) -> Result<()> {
    let client = GithubClient::new(config.github_token.clone())?;
    let release = client.get_release(REPO_OWNER, REPO_NAME, "latest").await?;

    let current = current_version();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    if latest == current {
        println!("gitclaw is already at the latest version ({})", current);
        return Ok(());
    }

    println!("Updating gitclaw: {} -> {}", current, latest);

    // Find matching asset for current platform
    let platform = Platform::current()?;
    let asset = find_matching_asset(&release, platform)
        .map_err(|_| anyhow!("No suitable asset found for platform: {}", platform))?;

    if !config.output.quiet {
        println!("Downloading: {}", asset.name);
    }

    // Download to temp location
    let temp_dir = std::env::temp_dir().join("gitclaw-self-update");
    std::fs::create_dir_all(&temp_dir)?;
    let download_path = temp_dir.join(&asset.name);

    client
        .download_asset(asset, &download_path, config.download.show_progress)
        .await?;

    // Get current executable path
    let current_exe = current_executable()?;

    // Handle based on archive type vs direct binary
    if asset.name.ends_with(".tar.gz")
        || asset.name.ends_with(".zip")
        || asset.name.ends_with(".tar.xz")
    {
        let extract_dir = temp_dir.join("extracted");
        extract_archive(&download_path, &extract_dir, true)?;
        let new_binary = find_binary(&extract_dir, REPO_NAME)?;
        replace_binary(&new_binary, &current_exe)?;
    } else {
        replace_binary(&download_path, &current_exe)?;
    }
    let _ = std::fs::remove_dir_all(&temp_dir);
    println!("gitclaw updated successfully to {}", latest);
    Ok(())
}

/// Find binary in extracted directory
fn find_binary(dir: &std::path::Path, name: &str) -> Result<PathBuf> {
    use walkdir::WalkDir;

    for entry in WalkDir::new(dir).max_depth(2) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        if file_name == name {
            return Ok(entry.path().to_path_buf());
        }
    }
    bail!("Binary '{}' not found in extracted archive", name)
}

/// Replace current binary with new one
#[cfg(unix)]
fn replace_binary(new: &std::path::Path, current: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // On Unix: write to temp file, then rename (atomic)
    let backup = current.with_extension("backup");
    std::fs::rename(current, &backup)?;
    match std::fs::copy(new, current) {
        Ok(_) => {
            let mut perms = std::fs::metadata(current)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(current, perms)?;
            let _ = std::fs::remove_file(&backup);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::rename(&backup, current);
            bail!("Failed to install new binary: {}", e)
        }
    }
}

#[cfg(windows)]
fn replace_binary(new: &std::path::Path, current: &std::path::Path) -> Result<()> {
    // On Windows: rename current (in-use files can't be overwritten)
    let backup = current.with_extension("exe.backup");
    std::fs::rename(current, &backup)?;
    match std::fs::copy(new, current) {
        Ok(_) => {
            let _ = std::fs::remove_file(&backup);
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::rename(&backup, current);
            bail!("Failed to install new binary: {}", e)
        }
    }
}
