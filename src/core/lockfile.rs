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
