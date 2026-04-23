use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::config::Config;
use crate::core::registry::Registry;
use crate::core::util::registry_path_from;
use crate::output;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportEntry {
    pub owner: String,
    pub repo: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportFile {
    #[serde(rename = "package")]
    pub packages: Vec<ExportEntry>,
}

impl ExportFile {
    pub fn from_registry(registry: &Registry) -> Self {
        let mut entries: Vec<ExportEntry> = registry
            .packages
            .values()
            .map(|p| ExportEntry {
                owner: p.owner.clone(),
                repo: p.repo.clone(),
                version: p.version.clone(),
            })
            .collect();

        entries.sort_by(|a, b| a.owner.cmp(&b.owner).then_with(|| a.repo.cmp(&b.repo)));

        ExportFile { packages: entries }
    }

    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Serialize export file")
    }

    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("Parse export file")
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Read export file {}", path.display()))?;
        Self::from_toml(&content)
    }
}

pub fn handle_export(config: &Config, output_path: Option<&str>) -> Result<()> {
    let registry_path = registry_path_from(&config.install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if reg.packages.is_empty() {
        output::print_info("No packages installed. Nothing to export.");
        return Ok(());
    }

    let export = ExportFile::from_registry(&reg);
    let toml_str = export.to_toml()?;

    match output_path {
        Some(path) => {
            fs::write(path, &toml_str).with_context(|| format!("Write to {}", path))?;
            output::print_success(&format!(
                "Exported {} package(s) to {}.",
                export.packages.len(),
                path
            ));
        }
        None => {
            println!("{}", toml_str);
        }
    }

    Ok(())
}

pub async fn handle_import(config: &Config, file: &str, force: bool) -> Result<()> {
    let path = Path::new(file);
    let export = ExportFile::from_file(path)?;

    if export.packages.is_empty() {
        output::print_info("No packages found in import file.");
        return Ok(());
    }

    output::print_info(&format!(
        "Importing {} package(s) from {}.",
        export.packages.len(),
        file
    ));

    for entry in &export.packages {
        let spec = format!("{}/{}@{}", entry.owner, entry.repo, entry.version);

        if !force {
            let registry_path = registry_path_from(&config.install_dir);
            let reg = Registry::load_from(&registry_path)?;
            let key = format!("{}/{}", entry.owner, entry.repo);

            if reg.is_installed(&key) {
                output::print_info(&format!("{} already installed. Skipping.", key));
                continue;
            }
        }

        crate::core::install::handle_install(&spec, force, false, false, config, None).await?;
    }

    Ok(())
}
