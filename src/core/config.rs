use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::constants::{
    CONFIG_FILE, ENV_VAR_CONFIG, GITCLAW_DIR, LEGACY_HOME_CONFIG_FILE, LOCAL_CONFIG_FILE,
    XDG_CONFIG_SUBDIR,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Auto,
    Never,
    Always,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadConfig {
    #[serde(default = "default_true")]
    pub show_progress: bool,

    #[serde(default = "default_true")]
    pub prefer_strip: bool,

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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default)]
    pub color: ColorMode,

    #[serde(default)]
    pub quiet: bool,

    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_install_dir")]
    pub install_dir: PathBuf,
    pub github_token: Option<String>,

    #[serde(default)]
    pub download: DownloadConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            github_token: None,
            download: DownloadConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_install_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(GITCLAW_DIR)
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        if let Some(legacy) = Self::load_from_legacy()? {
            config.merge(legacy);
        }

        if let Some(xdg) = Self::load_from_xdg()? {
            config.merge(xdg);
        }

        if let Some(local) = Self::load_from_local()? {
            config.merge(local);
        }

        if let Some(env) = Self::load_from_env()? {
            config.merge(env);
        }

        Ok(config)
    }

    pub fn load_from_env() -> Result<Option<Self>> {
        if let Ok(path) = env::var(ENV_VAR_CONFIG) {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from GITCLAW_CONFIG: {}", path))?;

            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from GITCLAW_CONFIG: {}", path))?;

            return Ok(Some(config));
        }
        Ok(None)
    }

    pub fn load_from_local() -> Result<Option<Self>> {
        let path = PathBuf::from(LOCAL_CONFIG_FILE);

        if path.exists() {
            let content =
                fs::read_to_string(&path).with_context(|| "Failed to read project-local config")?;

            let config: Config =
                toml::from_str(&content).with_context(|| "Failed to parse project-local config")?;

            return Ok(Some(config));
        }

        Ok(None)
    }

    pub fn load_from_xdg() -> Result<Option<Self>> {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join(XDG_CONFIG_SUBDIR).join(CONFIG_FILE);

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

    pub fn load_from_legacy() -> Result<Option<Self>> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(LEGACY_HOME_CONFIG_FILE);

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

    pub fn merge(&mut self, other: Config) {
        if let Some(token) = other.github_token {
            self.github_token = Some(token);
        }

        self.install_dir = other.install_dir;
        self.download.show_progress = other.download.show_progress;
        self.download.prefer_strip = other.download.prefer_strip;
        self.download.verify_checksums = other.download.verify_checksums;
        self.output.color = other.output.color;
        self.output.quiet = other.output.quiet;
        self.output.verbose = other.output.verbose;
    }
}
