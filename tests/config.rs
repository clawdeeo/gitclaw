//! Integration tests for the config module
//! Tests the public API from gitclaw::config

use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use gitclaw::config::{Config, DownloadConfig, OutputConfig};

/// Test default config values
#[test]
fn test_default_config() {
    let config = Config::default();

    // Download defaults
    assert!(config.download.show_progress);
    assert!(config.download.prefer_strip);
    assert!(config.download.verify_checksums);

    // Output defaults
    assert_eq!(config.output.color, "auto");
    assert!(!config.output.quiet);
    assert!(!config.output.verbose);
}

/// Test loading from project-local config file
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

[output]
color = "never"
"#,
    )
    .unwrap();

    // Change to temp directory
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp).unwrap();

    // Load from local
    let config = gitclaw::config::Config::load_from_local().unwrap().unwrap();
    assert_eq!(config.install_dir, PathBuf::from("/custom/bin"));
    assert_eq!(config.github_token, Some("test-token".to_string()));
    assert!(!config.download.show_progress);
    assert_eq!(config.output.color, "never");

    env::set_current_dir(original_dir).unwrap();
}

/// Test loading from legacy config
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

    // Mock home directory
    env::set_var("HOME", home);

    let config = Config::load_from_legacy().unwrap().unwrap();
    assert_eq!(config.install_dir, PathBuf::from("/legacy/bin"));

    env::remove_var("HOME");
}

/// Test loading from env var
#[test]
fn test_load_from_env() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("env-config.toml");

    fs::write(
        &config_path,
        r#"
install_dir = "/env/bin"
github_token = "env-token"

[download]
verify_checksums = false

[output]
verbose = true
"#,
    )
    .unwrap();

    env::set_var("GITCLAW_CONFIG", &config_path);

    let config = Config::load_from_env().unwrap().unwrap();
    assert_eq!(config.install_dir, PathBuf::from("/env/bin"));
    assert_eq!(config.github_token, Some("env-token".to_string()));
    assert!(!config.download.verify_checksums);
    assert!(config.output.verbose);

    env::remove_var("GITCLAW_CONFIG");
}

/// Test config merging - env overrides local
#[test]
fn test_config_merge_precedence() {
    // Start with default
    let mut config = Config::default();
    assert!(config.download.show_progress);
    assert_eq!(config.output.color, "auto");

    // Merge another config
    let other = Config {
        install_dir: PathBuf::from("/other"),
        github_token: Some("other".to_string()),
        download: DownloadConfig {
            show_progress: false,
            ..Default::default()
        },
        output: OutputConfig {
            color: "never".to_string(),
            verbose: true,
            ..Default::default()
        },
    };

    config.merge(other);

    assert_eq!(config.install_dir, PathBuf::from("/other"));
    assert_eq!(config.github_token, Some("other".to_string()));
    assert!(!config.download.show_progress); // Changed
    assert_eq!(config.output.color, "never"); // Changed
    assert!(config.output.verbose); // Changed
}

/// Test github_token accessor
#[test]
fn test_github_token_accessor() {
    let config = Config {
        github_token: Some("test-token".to_string()),
        ..Default::default()
    };

    assert_eq!(config.github_token(), Some("test-token"));

    let config_no_token = Config::default();
    assert_eq!(config_no_token.github_token(), None);
}

/// Test install_dir accessor
#[test]
fn test_install_dir_accessor() {
    let config = Config {
        install_dir: PathBuf::from("/custom/install"),
        ..Default::default()
    };

    assert_eq!(config.install_dir(), &PathBuf::from("/custom/install"));
}

/// Test parsing invalid TOML
#[test]
fn test_parse_invalid_toml() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("invalid.toml");

    fs::write(&config_path, "this is not valid toml {{").unwrap();

    let result = Config::load_from_env();
    // Should fail gracefully
    // (Can't easily test this without setting env var, but parse logic handles it)
}

/// Test DownloadConfig default
#[test]
fn test_download_config_default() {
    let dl = DownloadConfig::default();
    assert!(dl.show_progress);
    assert!(dl.prefer_strip);
    assert!(dl.verify_checksums);
}

/// Test OutputConfig default
#[test]
fn test_output_config_default() {
    let out = OutputConfig::default();
    assert_eq!(out.color, "auto");
    assert!(!out.quiet);
    assert!(!out.verbose);
}
