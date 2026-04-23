use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use walkdir::WalkDir;

use crate::core::config::Config;
use crate::core::constants::{
    APP_NAME, DIR_EXTRACTED, REPO_NAME, REPO_OWNER, TEMP_DIR_SELF_UPDATE,
};
use crate::core::extract::extract_archive;
use crate::network::github::{find_matching_asset, GithubClient, Platform};
use crate::output;

fn current_executable() -> Result<PathBuf> {
    env::current_exe().context("Failed to get current executable path.")
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub async fn check_for_update(config: &Config) -> Result<()> {
    let client = GithubClient::new(config.github_token.clone())?;
    let release = client.get_release(REPO_OWNER, REPO_NAME, "latest").await?;

    let current = current_version();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    output::print_header("Self Update");
    output::print_kv("Current version", &current);
    output::print_kv("Latest version", &latest);
    println!();

    if latest == current {
        output::print_success(&format!("{} is up to date.", APP_NAME));
    } else {
        output::print_info(&format!(
            "Update available: {} -> {}.",
            current.dimmed(),
            latest.green().bold()
        ));
        output::print_info(&format!("Run '{} self' to install.", APP_NAME));
    }

    Ok(())
}

pub async fn perform_update(config: &Config) -> Result<()> {
    let client = GithubClient::new(config.github_token.clone())?;
    let release = client.get_release(REPO_OWNER, REPO_NAME, "latest").await?;

    let current = current_version();
    let latest = release.tag_name.trim_start_matches('v').to_string();

    if latest == current {
        output::print_success(&format!(
            "{} is already at the latest version ({}).",
            APP_NAME, current
        ));
        return Ok(());
    }

    output::print_header("Self Update");
    output::print_info(&format!(
        "Updating: {} -> {}.",
        current.dimmed(),
        latest.green().bold()
    ));

    let platform = Platform::current()?;
    let asset = find_matching_asset(&release, platform)
        .map_err(|_| anyhow!("No suitable asset found for platform: {}.", platform))?;

    if !config.output.quiet {
        output::print_info(&format!("Downloading {}.", asset.name.dimmed()));
    }

    let temp_dir = std::env::temp_dir().join(TEMP_DIR_SELF_UPDATE);
    std::fs::create_dir_all(&temp_dir)?;
    let download_path = temp_dir.join(&asset.name);

    client
        .download_asset(asset, &download_path, config.download.show_progress)
        .await?;

    let current_exe = current_executable()?;

    if asset.name.ends_with(".tar.gz")
        || asset.name.ends_with(".zip")
        || asset.name.ends_with(".tar.xz")
    {
        let extract_dir = temp_dir.join(DIR_EXTRACTED);
        output::print_info("Extracting.");
        extract_archive(&download_path, &extract_dir, true)?;
        let new_binary = find_binary(&extract_dir, REPO_NAME)?;
        replace_binary(&new_binary, &current_exe)?;
    } else {
        replace_binary(&download_path, &current_exe)?;
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    output::print_success(&format!(
        "{} updated successfully to {}.",
        APP_NAME,
        latest.green().bold()
    ));
    Ok(())
}

fn find_binary(dir: &std::path::Path, name: &str) -> Result<PathBuf> {
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

    bail!("Binary '{}' not found in extracted archive.", name)
}

fn replace_binary(new: &std::path::Path, current: &std::path::Path) -> Result<()> {
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
            bail!("Failed to install new binary: {}.", e)
        }
    }
}
