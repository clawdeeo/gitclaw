use std::collections::HashMap;
use std::fs;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::core::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasMap {
    #[serde(flatten)]
    pub aliases: HashMap<String, String>,
}

const ALIASES_FILE: &str = "aliases.toml";

impl AliasMap {
    pub fn load(config: &Config) -> Result<Self> {
        let path = config.install_dir.join(ALIASES_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).with_context(|| "Failed to read aliases file")?;
        toml::from_str(&content).with_context(|| "Failed to parse aliases file")
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        let path = config.install_dir.join(ALIASES_FILE);
        let content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize aliases")?;
        fs::write(&path, content).with_context(|| "Failed to write aliases file")
    }

    pub fn resolve(&self, name: &str) -> Option<&str> {
        self.aliases.get(name).map(|s| s.as_str())
    }

    pub fn add(&mut self, alias: &str, target: &str, _config: &Config) -> Result<()> {
        if alias.contains('/') {
            bail!(
                "Alias '{}' cannot contain '/'. Use a short name without slashes.",
                alias
            );
        }

        if target.contains('/') && target.split('/').count() != 2 {
            bail!("Target '{}' must be in owner/repo format.", target);
        }

        if let Some(existing) = self.aliases.get(alias) {
            if existing == target {
                bail!("Alias '{}' already points to '{}'.", alias, target);
            }
        }

        self.aliases.insert(alias.to_string(), target.to_string());
        Ok(())
    }

    pub fn check_clash(&self, name: &str, config: &Config) -> Option<String> {
        let registry_path = crate::core::util::registry_path_from(&config.install_dir);
        if let Ok(reg) = crate::core::registry::Registry::load_from(&registry_path) {
            for pkg in reg.packages.values() {
                if pkg.repo == name || pkg.identifier == name {
                    return Some(format!("{}/{}", pkg.owner, pkg.repo));
                }
            }
        }
        None
    }

    pub fn remove(&mut self, alias: &str) -> bool {
        self.aliases.remove(alias).is_some()
    }

    pub fn list(&self) -> Vec<(&String, &String)> {
        let mut entries: Vec<_> = self.aliases.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        entries
    }
}

pub fn handle_alias_add(alias: &str, target: &str, config: &Config) -> Result<()> {
    let mut aliases = AliasMap::load(config)?;

    if let Some(clash) = aliases.check_clash(alias, config) {
        crate::output::print_warn(&format!(
            "Warning: alias '{}' matches installed package '{}'.",
            alias, clash
        ));
    }

    aliases.add(alias, target, config)?;
    aliases.save(config)?;
    crate::output::print_success(&format!("Alias '{}' -> '{}' added.", alias, target));
    Ok(())
}

pub fn handle_alias_remove(alias: &str, config: &Config) -> Result<()> {
    let mut aliases = AliasMap::load(config)?;
    if !aliases.remove(alias) {
        bail!("Alias '{}' not found.", alias);
    }
    aliases.save(config)?;
    crate::output::print_success(&format!("Alias '{}' removed.", alias));
    Ok(())
}

pub fn handle_alias_list(config: &Config) -> Result<()> {
    let aliases = AliasMap::load(config)?;
    let entries = aliases.list();

    if entries.is_empty() {
        crate::output::print_info("No aliases configured.");
        crate::output::print_info("Use 'gitclaw alias add <name> <owner/repo>' to create one.");
        return Ok(());
    }

    println!("{}", format!("{:<20} {}", "Alias", "Target").bold());

    for (alias, target) in &entries {
        println!("{:<20} {}", alias.cyan(), target);
    }

    println!();
    crate::output::print_info(&format!("{} alias(es) configured.", entries.len()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_add() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            install_dir: dir.path().to_path_buf(),
            ..Config::default()
        };
        let mut aliases = AliasMap::default();
        aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
        assert_eq!(aliases.resolve("rg"), Some("BurntSushi/ripgrep"));
    }

    #[test]
    fn test_alias_add_slash_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            install_dir: dir.path().to_path_buf(),
            ..Config::default()
        };
        let mut aliases = AliasMap::default();
        assert!(aliases
            .add("owner/repo", "BurntSushi/ripgrep", &config)
            .is_err());
    }

    #[test]
    fn test_alias_remove() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            install_dir: dir.path().to_path_buf(),
            ..Config::default()
        };
        let mut aliases = AliasMap::default();
        aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
        assert!(aliases.remove("rg"));
        assert!(!aliases.remove("rg"));
        assert_eq!(aliases.resolve("rg"), None);
    }

    #[test]
    fn test_alias_resolve_missing() {
        let aliases = AliasMap::default();
        assert_eq!(aliases.resolve("nonexistent"), None);
    }

    #[test]
    fn test_alias_list_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            install_dir: dir.path().to_path_buf(),
            ..Config::default()
        };
        let mut aliases = AliasMap::default();
        aliases.add("fd", "sharkdp/fd", &config).unwrap();
        aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
        aliases.add("bat", "sharkdp/bat", &config).unwrap();

        let list = aliases.list();
        assert_eq!(list[0].0.as_str(), "bat");
        assert_eq!(list[1].0.as_str(), "fd");
        assert_eq!(list[2].0.as_str(), "rg");
    }

    #[test]
    fn test_alias_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            install_dir: dir.path().to_path_buf(),
            ..Config::default()
        };

        let mut aliases = AliasMap::default();
        aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
        aliases.add("fd", "sharkdp/fd", &config).unwrap();
        aliases.save(&config).unwrap();

        let loaded = AliasMap::load(&config).unwrap();
        assert_eq!(loaded.resolve("rg"), Some("BurntSushi/ripgrep"));
        assert_eq!(loaded.resolve("fd"), Some("sharkdp/fd"));
    }
}
