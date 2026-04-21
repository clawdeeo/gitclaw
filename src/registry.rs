use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub owner: String,
    pub repo: String,
    pub version: String,
    pub installed_at: String,
    pub binary_path: PathBuf,
    pub install_dir: PathBuf,
    pub asset_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub packages: HashMap<String, InstalledPackage>,
}

impl Registry {
    fn path() -> Result<PathBuf> {
        Ok(dirs::home_dir()
            .ok_or_else(|| anyhow!("No home directory"))?
            .join(".gitclaw")
            .join("registry.toml"))
    }

    pub fn load() -> Result<Self> {
        let p = Self::path()?;
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = fs::read_to_string(&p).context("Read registry")?;
        toml::from_str(&s).context("Parse registry")
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::path()?;
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self).context("Serialize registry")?;
        fs::write(&p, s).context("Write registry")?;
        debug!("Registry saved to {:?}", p);
        Ok(())
    }

    pub fn is_installed(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    pub fn add(&mut self, pkg: InstalledPackage) {
        self.packages.insert(pkg.name.clone(), pkg);
    }

    pub fn remove(&mut self, name: &str) -> Option<InstalledPackage> {
        self.packages.remove(name)
    }
}

pub fn gitclaw_home() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("No home directory"))?
        .join(".gitclaw"))
}

pub fn bin_dir() -> Result<PathBuf> {
    Ok(gitclaw_home()?.join("bin"))
}

pub fn list_installed(verbose: bool) -> Result<()> {
    let reg = Registry::load()?;
    if reg.packages.is_empty() {
        println!("No packages installed. Use 'gitclaw install user/repo' to get started.");
        return Ok(());
    }
    if verbose {
        for pkg in reg.packages.values() {
            println!("{}", pkg.name);
            println!("  Version:   {}", pkg.version);
            println!("  Binary:    {}", pkg.binary_path.display());
            println!("  Installed: {}", pkg.installed_at);
            println!();
        }
    } else {
        println!("{:<30} {:<15} DATE", "PACKAGE", "VERSION");
        println!("{}", "-".repeat(60));
        let mut pkgs: Vec<_> = reg.packages.values().collect();
        pkgs.sort_by_key(|p| &p.name);
        for pkg in pkgs {
            let date = &pkg.installed_at[..10.min(pkg.installed_at.len())];
            println!("{:<30} {:<15} {}", pkg.name, pkg.version, date);
        }
    }
    println!("\n{} package(s)", reg.packages.len());
    Ok(())
}

pub fn uninstall(package: &str) -> Result<()> {
    let (owner, repo, _) = crate::github::parse_package(package)?;
    let key = format!("{}/{}", owner, repo);
    let mut reg = Registry::load()?;
    let pkg = reg
        .remove(&key)
        .ok_or_else(|| anyhow!("{} not installed", key))?;

    if pkg.install_dir.exists() {
        fs::remove_dir_all(&pkg.install_dir).context("Remove install dir")?;
    }
    let link = bin_dir()?.join(&repo);
    if link.exists() || link.is_symlink() {
        fs::remove_file(&link).context("Remove symlink")?;
    }
    reg.save()?;
    println!("Uninstalled {}", key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_crud() {
        let mut reg = Registry::default();
        let pkg = InstalledPackage {
            name: "user/repo".into(),
            owner: "user".into(),
            repo: "repo".into(),
            version: "v1.0.0".into(),
            installed_at: "2024-01-01".into(),
            binary_path: PathBuf::from("/tmp/binary"),
            install_dir: PathBuf::from("/tmp/install"),
            asset_name: "tool.tar.gz".into(),
        };
        assert!(!reg.is_installed("user/repo"));
        reg.add(pkg);
        assert!(reg.is_installed("user/repo"));
        let removed = reg.remove("user/repo");
        assert!(removed.is_some());
        assert!(!reg.is_installed("user/repo"));
    }
}
