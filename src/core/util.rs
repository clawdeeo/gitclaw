use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::core::constants::{
    CONFIG_FILE, DIR_BIN, DIR_CACHE, DIR_DOWNLOADS, DIR_PACKAGES, GITCLAW_DIR, REGISTRY_FILE,
};

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory."))
}

pub fn gitclaw_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(GITCLAW_DIR))
}

pub fn bin_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join(DIR_BIN))
}

pub fn bin_dir_from(base: &Path) -> PathBuf {
    base.join(DIR_BIN)
}

pub fn cache_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join(DIR_CACHE))
}

pub fn downloads_dir() -> Result<PathBuf> {
    Ok(cache_dir()?.join(DIR_DOWNLOADS))
}

pub fn packages_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join(DIR_PACKAGES))
}

pub fn registry_path() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join(REGISTRY_FILE))
}

pub fn registry_path_from(base: &Path) -> PathBuf {
    base.join(REGISTRY_FILE)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join(CONFIG_FILE))
}

pub fn is_in_path(binary: &str) -> bool {
    env::var("PATH")
        .map(|path| {
            path.split(':')
                .any(|dir| PathBuf::from(dir).join(binary).exists())
        })
        .unwrap_or(false)
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let exponent = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exponent as i32);
    format!("{:.1} {}", value, UNITS[exponent])
}
