use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::registry::Registry;
use crate::core::util::registry_path_from;
use crate::output;

const LOCKFILE_NAME: &str = "gitclaw.lock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub owner: String,
    pub repo: String,
    pub version: String,
    pub asset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    #[serde(rename = "package")]
    pub packages: Vec<LockEntry>,
}

impl Lockfile {
    pub fn from_registry(registry: &Registry) -> Self {
        let packages = registry
            .packages
            .values()
            .map(|p| LockEntry {
                owner: p.owner.clone(),
                repo: p.repo.clone(),
                version: p.version.clone(),
                asset: p.asset_name.clone(),
            })
            .collect();

        Lockfile { packages }
    }

    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(LOCKFILE_NAME);
        let content = fs::read_to_string(&path).with_context(|| "Failed to read lockfile")?;
        toml::from_str(&content).with_context(|| "Failed to parse lockfile")
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join(LOCKFILE_NAME);
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize lockfile")?;
        fs::write(&path, content).with_context(|| "Failed to write lockfile")
    }

    pub fn is_present(dir: &Path) -> bool {
        dir.join(LOCKFILE_NAME).exists()
    }
}

pub fn generate_lockfile(install_dir: &Path, project_dir: &Path) -> Result<()> {
    let registry_path = registry_path_from(install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if reg.packages.is_empty() {
        output::print_info("No packages installed. Nothing to lock.");
        return Ok(());
    }

    let lockfile = Lockfile::from_registry(&reg);
    lockfile.save(project_dir)?;

    output::print_success(&format!(
        "Lockfile written with {} package(s).",
        lockfile.packages.len()
    ));
    Ok(())
}

pub async fn install_locked(config: &crate::core::config::Config) -> Result<()> {
    let project_dir = std::env::current_dir()?;

    if !Lockfile::is_present(&project_dir) {
        anyhow::bail!(
            "No gitclaw.lock found in {}. Run 'gitclaw lock' first.",
            project_dir.display()
        );
    }

    let lockfile = Lockfile::load(&project_dir)?;

    if lockfile.packages.is_empty() {
        output::print_info("Lockfile is empty. Nothing to install.");
        return Ok(());
    }

    output::print_info(&format!(
        "Installing {} package(s) from lockfile.",
        lockfile.packages.len()
    ));

    for entry in &lockfile.packages {
        let package_spec = format!("{}/{}@{}", entry.owner, entry.repo, entry.version);
        crate::core::install::handle_install(&package_spec, false, false, false, config, None)
            .await?;
    }

    output::print_success("All locked packages installed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::core::registry::InstalledPackage;

    fn make_installed_package(
        name: &str,
        owner: &str,
        repo: &str,
        version: &str,
        asset: &str,
    ) -> InstalledPackage {
        InstalledPackage {
            name: name.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            version: version.to_string(),
            installed_at: "2026-01-01T00:00:00Z".to_string(),
            binary_path: PathBuf::from("/tmp/test"),
            install_dir: PathBuf::from("/tmp/test"),
            asset_name: asset.to_string(),
            identifier: repo.to_string(),
            channel: None,
        }
    }

    #[test]
    fn test_lockfile_from_registry() {
        let mut reg = Registry::default();
        reg.add(make_installed_package(
            "BurntSushi/ripgrep",
            "BurntSushi",
            "ripgrep",
            "v14.1.0",
            "ripgrep-14.tar.gz",
        ));
        reg.add(make_installed_package(
            "sharkdp/fd",
            "sharkdp",
            "fd",
            "v10.2.0",
            "fd-10.tar.gz",
        ));

        let lockfile = Lockfile::from_registry(&reg);
        assert_eq!(lockfile.packages.len(), 2);

        let rg = lockfile
            .packages
            .iter()
            .find(|p| p.repo == "ripgrep")
            .unwrap();
        assert_eq!(rg.owner, "BurntSushi");
        assert_eq!(rg.version, "v14.1.0");
        assert_eq!(rg.asset, "ripgrep-14.tar.gz");
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let dir = tempfile::tempdir().unwrap();

        let lockfile = Lockfile {
            packages: vec![LockEntry {
                owner: "BurntSushi".to_string(),
                repo: "ripgrep".to_string(),
                version: "v14.1.0".to_string(),
                asset: "ripgrep-14.tar.gz".to_string(),
            }],
        };

        lockfile.save(dir.path()).unwrap();
        let loaded = Lockfile::load(dir.path()).unwrap();

        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.packages[0].owner, "BurntSushi");
        assert_eq!(loaded.packages[0].repo, "ripgrep");
        assert_eq!(loaded.packages[0].version, "v14.1.0");
    }

    #[test]
    fn test_lockfile_toml_format() {
        let lockfile = Lockfile {
            packages: vec![LockEntry {
                owner: "sharkdp".to_string(),
                repo: "fd".to_string(),
                version: "v10.2.0".to_string(),
                asset: "fd-10.tar.gz".to_string(),
            }],
        };

        let toml_str = toml::to_string_pretty(&lockfile).unwrap();
        assert!(toml_str.contains("[[package]]"));
        assert!(toml_str.contains("owner = \"sharkdp\""));
        assert!(toml_str.contains("repo = \"fd\""));
    }

    #[test]
    fn test_lockfile_empty_registry() {
        let reg = Registry::default();
        let lockfile = Lockfile::from_registry(&reg);
        assert!(lockfile.packages.is_empty());
    }

    #[test]
    fn test_is_present() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!Lockfile::is_present(dir.path()));

        let lockfile = Lockfile::default();
        lockfile.save(dir.path()).unwrap();
        assert!(Lockfile::is_present(dir.path()));
    }
}
