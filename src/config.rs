use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

/// Download configuration settings
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct DownloadConfig {
    /// Show progress bars during download
    #[serde(default = "default_true")]
    pub show_progress: bool,
    /// Prefer stripping binaries
    #[serde(default = "default_true")]
    pub prefer_strip: bool,
    /// Verify checksums after download
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

/// Output configuration settings
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct OutputConfig {
    /// Color mode: "auto", "always", or "never"
    #[serde(default = "default_color")]
    pub color: String,
    /// Suppress non-essential output
    #[serde(default)]
    pub quiet: bool,
    /// Enable verbose output
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

/// Main configuration struct for gitclaw
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Installation directory for binaries
    #[serde(default = "default_install_dir")]
    pub install_dir: String,
    /// GitHub personal access token (optional)
    pub github_token: Option<String>,
    /// Download preferences
    #[serde(default)]
    pub download: DownloadConfig,
    /// Output preferences
    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_dir: "~/.local/bin".to_string(),
            github_token: None,
            download: DownloadConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_color() -> String {
    "auto".to_string()
}

fn default_install_dir() -> String {
    "~/.local/bin".to_string()
}

impl Config {
    /// Create a new empty configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from all supported sources, merging with precedence:
    /// 1. $GITCLAW_CONFIG env var (highest)
    /// 2. ./.gitclaw.toml (project-local)
    /// 3. ~/.config/gitclaw/config.toml (XDG)
    /// 4. ~/.gitclaw.toml (legacy)
    pub fn load() -> Result<Self> {
        let mut config = Self::new();

        // Load from lowest to highest precedence
        if let Some(cfg) = Self::load_from_legacy()? {
            config = config.merge(cfg);
        }

        if let Some(cfg) = Self::load_from_xdg()? {
            config = config.merge(cfg);
        }

        if let Some(cfg) = Self::load_from_local()? {
            config = config.merge(cfg);
        }

        if let Some(cfg) = Self::load_from_env()? {
            config = config.merge(cfg);
        }

        Ok(config)
    }

    /// Load configuration from $GITCLAW_CONFIG environment variable
    pub fn load_from_env() -> Result<Option<Self>> {
        if let Ok(path) = std::env::var("GITCLAW_CONFIG") {
            if !path.is_empty() {
                return Self::load_from_file(&path).map(Some);
            }
        }
        Ok(None)
    }

    /// Load configuration from ./.gitclaw.toml (project-local)
    pub fn load_from_local() -> Result<Option<Self>> {
        let path = PathBuf::from(".").join(".gitclaw.toml");
        if path.exists() {
            return Self::load_from_file(&path).map(Some);
        }
        Ok(None)
    }

    /// Load configuration from XDG config directory
    /// (~/.config/gitclaw/config.toml)
    pub fn load_from_xdg() -> Result<Option<Self>> {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("gitclaw").join("config.toml");
            if path.exists() {
                return Self::load_from_file(&path).map(Some);
            }
        }
        Ok(None)
    }

    /// Load configuration from legacy location
    /// (~/.gitclaw.toml)
    pub fn load_from_legacy() -> Result<Option<Self>> {
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".gitclaw.toml");
            if path.exists() {
                return Self::load_from_file(&path).map(Some);
            }
        }
        Ok(None)
    }

    /// Load configuration from a specific file path
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Merge another config into this one, with `other` taking precedence.
    /// All values in `other` override values in `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            install_dir: other.install_dir,
            github_token: other.github_token.or(self.github_token),
            download: DownloadConfig {
                show_progress: other.download.show_progress,
                prefer_strip: other.download.prefer_strip,
                verify_checksums: other.download.verify_checksums,
            },
            output: OutputConfig {
                color: other.output.color,
                quiet: other.output.quiet || self.output.quiet,
                verbose: other.output.verbose || self.output.verbose,
            },
        }
    }

    /// Expand the install_dir path, handling `~` prefix
    #[allow(dead_code)]
    pub fn expanded_install_dir(&self) -> Result<PathBuf> {
        if self.install_dir.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                let remainder = &self.install_dir[2..];
                return Ok(home.join(remainder));
            }
        }
        Ok(PathBuf::from(&self.install_dir))
    }

    /// Get effective color setting based on output config
    #[allow(dead_code)]
    pub fn color_enabled(&self) -> bool {
        match self.output.color.as_str() {
            "always" => true,
            "never" => false,
            _ => {
                // "auto" - check if stdout is a TTY
                atty::is(atty::Stream::Stdout)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::new();
        assert_eq!(config.install_dir, "~/.local/bin");
        assert_eq!(config.github_token, None);
        assert!(config.download.show_progress);
        assert!(config.download.prefer_strip);
        assert!(config.download.verify_checksums);
        assert_eq!(config.output.color, "auto");
        assert!(!config.output.quiet);
        assert!(!config.output.verbose);
    }

    #[test]
    fn test_load_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        let toml_content = r#"
install_dir = "/custom/bin"
github_token = "test_token_123"

[download]
show_progress = false
prefer_strip = false

[output]
color = "never"
quiet = true
"#;
        file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load_from_file(file.path()).unwrap();
        assert_eq!(config.install_dir, "/custom/bin");
        assert_eq!(config.github_token, Some("test_token_123".to_string()));
        assert!(!config.download.show_progress);
        assert!(!config.download.prefer_strip);
        assert_eq!(config.output.color, "never");
        assert!(config.output.quiet);
    }

    #[test]
    fn test_merge_configs() {
        let base = Config::new();
        let override_config = Config {
            install_dir: "/usr/local/bin".to_string(),
            github_token: Some("token".to_string()),
            download: DownloadConfig {
                show_progress: false,
                prefer_strip: true,
                verify_checksums: true,
            },
            output: OutputConfig {
                color: "always".to_string(),
                quiet: true,
                verbose: false,
            },
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.install_dir, "/usr/local/bin");
        assert_eq!(merged.github_token, Some("token".to_string()));
        assert!(!merged.download.show_progress);
        assert!(merged.output.quiet);
        assert_eq!(merged.output.color, "always");
    }

    #[test]
    fn test_merge_preserves_unset_values() {
        let base = Config {
            install_dir: "/base/bin".to_string(),
            github_token: Some("base_token".to_string()),
            download: DownloadConfig {
                show_progress: false,
                prefer_strip: false,
                verify_checksums: false,
            },
            output: OutputConfig {
                color: "never".to_string(),
                quiet: true,
                verbose: true,
            },
        };

        // Override with defaults - uses all override values
        let override_config = Config::new();
        let merged = base.merge(override_config);

        // All values come from override (the defaults)
        assert_eq!(merged.install_dir, "~/.local/bin");
        assert_eq!(merged.github_token, Some("base_token".to_string())); // preserved since None in override
        assert!(merged.download.show_progress);
        assert!(merged.download.prefer_strip);
        assert!(merged.download.verify_checksums);
        assert_eq!(merged.output.color, "auto");
        // quiet/verbose are OR'd together, so should still be true
        assert!(merged.output.quiet);
        assert!(merged.output.verbose);
    }

    #[test]
    fn test_expand_install_dir() {
        let config = Config {
            install_dir: "~/.local/bin".to_string(),
            ..Default::default()
        };

        let expanded = config.expanded_install_dir().unwrap();
        if let Some(home) = dirs::home_dir() {
            assert_eq!(expanded, home.join(".local/bin"));
        }
    }

    #[test]
    fn test_invalid_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"invalid toml content [[[}").unwrap();

        let result = Config::load_from_file(file.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Failed to parse config file"));
    }

    #[test]
    fn test_missing_file_error() {
        let result = Config::load_from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }
}
