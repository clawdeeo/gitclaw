use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::core::config::Config;
use crate::core::constants::DIR_CACHE;
use crate::core::util;
use crate::output;

pub fn cache_dir(config: &Config) -> PathBuf {
    config.install_dir.join(DIR_CACHE)
}

pub fn cache_key(owner: &str, repo: &str, version: &str, filename: &str) -> String {
    format!("{}_{}_{}_{}", owner, repo, version, filename)
}

pub fn cache_path(config: &Config, key: &str) -> PathBuf {
    cache_dir(config).join(key)
}

pub fn file_hash(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| format!("Read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn get_cached(config: &Config, key: &str, expected_hash: Option<&str>) -> Option<PathBuf> {
    let path = cache_path(config, key);

    if !path.exists() {
        return None;
    }

    if let Some(expected) = expected_hash {
        match file_hash(&path) {
            Ok(actual) if actual == expected => Some(path),
            _ => None,
        }
    } else {
        Some(path)
    }
}

pub fn store(config: &Config, key: &str, source: &Path) -> Result<PathBuf> {
    let dir = cache_dir(config);
    fs::create_dir_all(&dir)?;
    let dest = dir.join(key);
    fs::copy(source, &dest).with_context(|| format!("Copy to cache {}", dest.display()))?;
    Ok(dest)
}

pub fn clean(config: &Config) -> Result<u64> {
    let dir = cache_dir(config);

    if !dir.exists() {
        return Ok(0);
    }

    let mut count = 0u64;
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
            count += 1;
        }
    }

    Ok(count)
}

pub fn size(config: &Config) -> Result<u64> {
    let dir = cache_dir(config);

    if !dir.exists() {
        return Ok(0);
    }

    let mut total = 0u64;

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            total += entry.metadata()?.len();
        }
    }

    Ok(total)
}

pub fn handle_cache_clean(config: &Config) -> Result<()> {
    let removed = clean(config)?;

    if removed == 0 {
        output::print_info("Cache is already empty.");
    } else {
        output::print_success(&format!("Removed {} cached file(s).", removed));
    }

    Ok(())
}

pub fn handle_cache_size(config: &Config) -> Result<()> {
    let bytes = size(config)?;
    output::print_info(&format!("Cache size: {}.", util::format_bytes(bytes)));
    Ok(())
}
