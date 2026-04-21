#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
}

pub fn gitclaw_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".gitclaw"))
}

pub fn bin_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join("bin"))
}

pub fn cache_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join("cache"))
}

pub fn downloads_dir() -> Result<PathBuf> {
    Ok(cache_dir()?.join("downloads"))
}

pub fn packages_dir() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join("packages"))
}

pub fn registry_path() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join("registry.toml"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(gitclaw_dir()?.join("config.toml"))
}

pub fn ensure_dirs() -> Result<()> {
    std::fs::create_dir_all(bin_dir()?)?;
    std::fs::create_dir_all(downloads_dir()?)?;
    std::fs::create_dir_all(packages_dir()?)?;
    Ok(())
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
    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    format!("{:.1} {}", value, UNITS[exp])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512.0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }
}
