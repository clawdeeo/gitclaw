use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use walkdir::WalkDir;

use crate::core::constants::{
    CONFIG_FILE, DIR_BIN, DIR_CACHE, DIR_DOWNLOADS, DIR_PACKAGES, EXEC_PERMISSION_BITS,
    GITCLAW_DIR, REGISTRY_FILE, WALK_MAX_DEPTH,
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
    env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|dir| dir.join(binary).exists()))
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

pub fn package_key(owner: &str, repo: &str) -> String {
    format!("{}/{}", owner, repo)
}

pub fn find_binary(dir: &Path, repo_name: &str) -> Result<PathBuf> {
    for entry in WalkDir::new(dir).max_depth(WALK_MAX_DEPTH) {
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
            .map(|m| m.permissions().mode() & EXEC_PERMISSION_BITS != 0)
            .unwrap_or(false)
        {
            return Ok(entry.path().to_path_buf());
        }
    }

    for entry in WalkDir::new(dir).max_depth(WALK_MAX_DEPTH) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        if fs::metadata(entry.path())
            .map(|m| m.permissions().mode() & EXEC_PERMISSION_BITS != 0)
            .unwrap_or(false)
        {
            return Ok(entry.path().to_path_buf());
        }
    }

    bail!("No binary found in {}", dir.display())
}
