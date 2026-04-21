//! Configuration file support for gitclaw
//!
//! Supports multiple config locations in order of precedence:
//! 1. $GITCLAW_CONFIG environment variable
//! 2. ./.gitclaw.toml (project-local)
//! 3. ~/.config/gitclaw/config.toml (XDG)
//! 4. ~/.gitclaw.toml (legacy)

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Download preferences
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadConfig {
    /// Show progress bars during download
    #[serde(default = "default_true")]
    pub show_progress: bool,

    /// Strip directory components when extracting
    #[serde(default = "default_true")]
    pub prefer_strip: bool,

    /// Verify checksums when available
    #[serde(default = "default_true")]
    pub verify_checksums: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            show_progress: true,
            prefer_strip: true,
            verify_checksums: true,
        }
    }
}

/// Output preferences
#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    /// Color output: "auto", "always", "never"
    #[serde(default = "default_color")]
    pub color: String,

    /// Suppress non-error output
    #[serde(default)]
    pub quiet: bool,

    /// Verbose output
    #[serde(default)]
    pub verbose: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            color: "auto".to_string(),
            quiet: false,
            verbose: false,
        }
    }
}

/// Gitclaw configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    /// Installation directory
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,

    /// GitHub API token (optional)
    pub github_token: Option<String>,

    /// Download preferences
    #[serde(default)]
    pub download: DownloadConfig,

    /// Output preferences
    #[serde(default)]
    pub output: OutputConfig,
}

fn default_true() -> bool {
    true
}

fn default_color() -> String {
    "auto".to_string()
}

fn default_install_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".gitclaw")
        .join("bin")
}

impl Config {
    /// Load configuration from all sources, merging with precedence:
    /// CLI args > env var > project-local > XDG > legacy > defaults
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Load from legacy location (~/.gitclaw.toml) - lowest precedence
        if let Some(legacy) = Self::load_from_legacy()? {
            config.merge(legacy);
        }

        // Load from XDG location (~/.config/gitclaw/config.toml)
        if let Some(xdg) = Self::load_from_xdg()? {
            config.merge(xdg);
        }

        // Load from project-local (./.gitclaw.toml)
        if let Some(local) = Self::load_from_local()? {
            config.merge(local);
        }

        // Load from environment variable
        if let Some(env) = Self::load_from_env()? {
            config.merge(env);
        }

        Ok(config)
    }

    /// Load from $GITCLAW_CONFIG environment variable
    pub fn load_from_env() -> Result<Option<Self>> {
        if let Ok(path) = env::var("GITCLAW_CONFIG") {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from GITCLAW_CONFIG: {}", path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from GITCLAW_CONFIG: {}", path))?;
            return Ok(Some(config));
        }
        Ok(None)
    }

    /// Load from ./.gitclaw.toml (project-local)
    pub fn load_from_local() -> Result<Option<Self>> {
        let path = PathBuf::from(".gitclaw.toml");
        if path.exists() {
            let content =
                fs::read_to_string(&path).with_context(|| "Failed to read project-local config")?;
            let config: Config =
                toml::from_str(&content).with_context(|| "Failed to parse project-local config")?;
            return Ok(Some(config));
        }
        Ok(None)
    }

    /// Load from XDG config location (~/.config/gitclaw/config.toml)
    pub fn load_from_xdg() -> Result<Option<Self>> {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("gitclaw").join("config.toml");
            if path.exists() {
                let content =
                    fs::read_to_string(&path).with_context(|| "Failed to read XDG config")?;
                let config: Config =
                    toml::from_str(&content).with_context(|| "Failed to parse XDG config")?;
                return Ok(Some(config));
            }
        }
        Ok(None)
    }

    /// Load from legacy location (~/.gitclaw.toml)
    pub fn load_from_legacy() -> Result<Option<Self>> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".gitclaw.toml");
            if path.exists() {
                let content =
                    fs::read_to_string(&path).with_context(|| "Failed to read legacy config")?;
                let config: Config =
                    toml::from_str(&content).with_context(|| "Failed to parse legacy config")?;
                return Ok(Some(config));
            }
        }
        Ok(None)
    }

    /// Merge another config into this one, with other taking precedence
    pub fn merge(&mut self, other: Config) {
        if let Some(token) = other.github_token {
            self.github_token = Some(token);
        }
        if other.install_dir != default_install_dir() {
            self.install_dir = other.install_dir;
        }

        // Merge download preferences
        if !other.download.show_progress {
            self.download.show_progress = other.download.show_progress;
        }
        if !other.download.prefer_strip {
            self.download.prefer_strip = other.download.prefer_strip;
        }
        if !other.download.verify_checksums {
            self.download.verify_checksums = other.download.verify_checksums;
        }

        // Merge output preferences
        if other.output.color != "auto" {
            self.output.color = other.output.color;
        }
        if other.output.quiet {
            self.output.quiet = other.output.quiet;
        }
        if other.output.verbose {
            self.output.verbose = other.output.verbose;
        }
    }

    /// Get the GitHub token, if configured
    #[allow(dead_code)]
    pub fn github_token(&self) -> Option<&str> {
        self.github_token.as_deref()
    }

    /// Get the installation directory
    #[allow(dead_code)]
    pub fn install_dir(&self) -> &PathBuf {
        &self.install_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.download.show_progress);
        assert!(config.download.prefer_strip);
        assert!(config.download.verify_checksums);
        assert_eq!(config.output.color, "auto");
        assert!(!config.output.quiet);
        assert!(!config.output.verbose);
    }

    #[test]
    fn test_load_from_local() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(".gitclaw.toml");

        fs::write(
            &config_path,
            r#"
install_dir = "/custom/bin"
github_token = "test-token"

[download]
show_progress = false
"#,
        )
        .unwrap();

        // Change to temp directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp).unwrap();

        let config = Config::load_from_local().unwrap().unwrap();
        assert_eq!(config.install_dir, PathBuf::from("/custom/bin"));
        assert_eq!(config.github_token, Some("test-token".to_string()));
        assert!(!config.download.show_progress);

        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_load_from_legacy() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        let config_path = home.join(".gitclaw.toml");

        fs::write(
            &config_path,
            r#"
install_dir = "/legacy/bin"
"#,
        )
        .unwrap();

        // Mock home directory - use USERPROFILE on Windows, HOME on Unix
        #[cfg(windows)]
        env::set_var("USERPROFILE", home);
        #[cfg(not(windows))]
        env::set_var("HOME", home);

        let config = Config::load_from_legacy().unwrap().unwrap();
        assert_eq!(config.install_dir, PathBuf::from("/legacy/bin"));

        #[cfg(windows)]
        env::remove_var("USERPROFILE");
        #[cfg(not(windows))]
        env::remove_var("HOME");
    }

    #[test]
    fn test_load_from_env() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("env-config.toml");

        fs::write(
            &config_path,
            r#"
install_dir = "/env/bin"
github_token = "env-token"
"#,
        )
        .unwrap();

        env::set_var("GITCLAW_CONFIG", &config_path);

        let config = Config::load_from_env().unwrap().unwrap();
        assert_eq!(config.install_dir, PathBuf::from("/env/bin"));
        assert_eq!(config.github_token, Some("env-token".to_string()));

        env::remove_var("GITCLAW_CONFIG");
    }
}
