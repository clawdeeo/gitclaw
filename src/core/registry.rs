use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::core::config::Config;
use crate::core::constants::{APP_NAME_SHORT, RELEASE_TAG_LATEST};
use crate::core::util::registry_path_from;
use crate::network::github::{parse_package, GithubClient};
use crate::output;

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
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    #[serde(default)]
    pub packages: HashMap<String, InstalledPackage>,
    #[serde(skip)]
    path: PathBuf,
}

impl Registry {
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                packages: HashMap::new(),
                path: path.clone(),
            });
        }
        let s = fs::read_to_string(path).context("Read registry")?;
        let mut reg: Registry = toml::from_str(&s).context("Parse registry")?;
        reg.path = path.clone();
        Ok(reg)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self).context("Serialize registry")?;
        fs::write(&self.path, s).context("Write registry")?;
        debug!("Registry saved to {:?}", self.path);
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

pub fn list_installed(verbose: bool, install_dir: &Path) -> Result<()> {
    let registry_path = registry_path_from(install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if reg.packages.is_empty() {
        output::print_info("No packages installed.");
        output::print_info(&format!(
            "Use '{} install user/repo' to get started.",
            APP_NAME_SHORT
        ));
        return Ok(());
    }

    if verbose {
        let mut pkgs: Vec<_> = reg.packages.values().collect();
        pkgs.sort_by_key(|p| &p.name);

        for pkg in pkgs {
            output::print_kv("Package", &pkg.name);
            output::print_kv("Identifier", &pkg.identifier);
            output::print_kv("Version", &pkg.version);
            output::print_kv("Binary", &pkg.binary_path.display().to_string());

            output::print_kv(
                "Installed",
                &pkg.installed_at[..10.min(pkg.installed_at.len())],
            );
        }
    } else {
        println!(
            "{}",
            format!(
                "{:<25} {:<20} {:<15} {:<30} {}",
                "Package", "Identifier", "Version", "Path", "Date"
            )
            .bold()
        );

        let mut pkgs: Vec<_> = reg.packages.values().collect();
        pkgs.sort_by_key(|p| &p.name);

        for pkg in pkgs {
            let date = &pkg.installed_at[..10.min(pkg.installed_at.len())];

            let path_short = pkg
                .binary_path
                .display()
                .to_string()
                .replace(&dirs::home_dir().unwrap().display().to_string(), "~")
                .replace("/packages/", "/p/");

            let path_display = if path_short.len() > 28 {
                format!("{}...", &path_short[..25])
            } else {
                path_short
            };

            println!(
                "{:<25} {:<20} {:<15} {:<30} {}",
                pkg.name.dimmed(),
                pkg.identifier.cyan(),
                pkg.version,
                path_display,
                date
            );
        }
    }

    println!();
    output::print_info(&format!("{} package(s) installed.", reg.packages.len()));
    Ok(())
}

pub async fn list_outdated(install_dir: &Path, token: Option<&str>) -> Result<()> {
    let registry_path = registry_path_from(install_dir);
    let reg = Registry::load_from(&registry_path)?;

    if reg.packages.is_empty() {
        output::print_info("No packages installed.");
        return Ok(());
    }

    let client = GithubClient::new(token.map(|s| s.to_string()))?;
    let mut outdated = Vec::new();

    for pkg in reg.packages.values() {
        let latest = match client
            .get_release(&pkg.owner, &pkg.repo, RELEASE_TAG_LATEST)
            .await
        {
            Ok(r) => r.tag_name,
            Err(_) => continue,
        };

        if latest != pkg.version {
            outdated.push((pkg.name.clone(), pkg.version.clone(), latest));
        }
    }

    if outdated.is_empty() {
        output::print_success("All packages are up to date.");
        return Ok(());
    }

    println!(
        "{}",
        format!("{:<30} {:<20} {}", "Package", "Installed", "Latest").bold()
    );

    for (name, installed, latest) in &outdated {
        println!(
            "{:<30} {:<20} {}",
            name.dimmed(),
            installed,
            latest.green().bold()
        );
    }

    println!();
    output::print_info(&format!("{} package(s) outdated.", outdated.len()));
    Ok(())
}

pub fn uninstall(package: &str, install_dir: &Path, config: &Config) -> Result<()> {
    let registry_path = registry_path_from(install_dir);
    let mut reg = Registry::load_from(&registry_path)?;

    let key = if package.contains('/') {
        let (owner, repo, _) = parse_package(package)?;
        format!("{}/{}", owner, repo)
    } else {
        let resolved = if let Some(alias_target) =
            crate::core::alias::AliasMap::load(config)?.resolve(package)
        {
            output::print_info(&format!("Alias '{}' -> '{}'.", package, alias_target));
            alias_target.to_string()
        } else {
            package.to_string()
        };

        let matches: Vec<_> = reg
            .packages
            .values()
            .filter(|p| p.identifier == resolved || p.repo == resolved)
            .map(|p| p.name.clone())
            .collect();

        match matches.len() {
            0 => anyhow::bail!("Package '{}' not installed.", package),
            1 => matches.into_iter().next().unwrap(),
            _ => anyhow::bail!(
                "Multiple packages match '{}'. Use full name (owner/repo).",
                package
            ),
        }
    };

    let pkg = reg
        .remove(&key)
        .ok_or_else(|| anyhow!("{} not installed.", key))?;

    if pkg.install_dir.exists() {
        fs::remove_dir_all(&pkg.install_dir).context("Remove install dir")?;
    }
    let link = install_dir.join("bin").join(&pkg.repo);
    if link.exists() || link.is_symlink() {
        fs::remove_file(&link).context("Remove symlink")?;
    }
    reg.save()?;
    output::print_success(&format!("Uninstalled {}.", key.cyan().bold()));
    Ok(())
}
