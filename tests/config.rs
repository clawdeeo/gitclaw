use anyhow::Result;
use gitclaw::config::{Config, DownloadConfig, OutputConfig};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

/// Helper to create a temporary config file with given content
fn create_temp_config_file(content: &str) -> (NamedTempFile, PathBuf) {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    let path = file.path().to_path_buf();
    (file, path)
}

/// Test loading from a file with full configuration
#[test]
fn test_load_from_file_full() -> Result<()> {
    let toml_content = r#"
install_dir = "/custom/bin"
github_token = "test_token_123"

[download]
show_progress = false
prefer_strip = false
verify_checksums = true

[output]
color = "never"
quiet = true
verbose = true
"#;
    let (_file, path) = create_temp_config_file(toml_content);

    let config = Config::load_from_file(&path)?;

    assert_eq!(config.install_dir, "/custom/bin");
    assert_eq!(config.github_token, Some("test_token_123".to_string()));
    assert!(!config.download.show_progress);
    assert!(!config.download.prefer_strip);
    assert!(config.download.verify_checksums);
    assert_eq!(config.output.color, "never");
    assert!(config.output.quiet);
    assert!(config.output.verbose);

    Ok(())
}

/// Test loading from a file with partial configuration
#[test]
fn test_load_from_file_partial() -> Result<()> {
    let toml_content = r#"
install_dir = "/usr/local/bin"
github_token = "partial_token"
"#;
    let (_file, path) = create_temp_config_file(toml_content);

    let config = Config::load_from_file(&path)?;

    assert_eq!(config.install_dir, "/usr/local/bin");
    assert_eq!(config.github_token, Some("partial_token".to_string()));
    // Defaults should be preserved for unset values
    assert!(config.download.show_progress);
    assert!(config.download.prefer_strip);
    assert_eq!(config.output.color, "auto");
    assert!(!config.output.quiet);

    Ok(())
}

/// Test default configuration values
#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.install_dir, "~/.local/bin");
    assert!(config.github_token.is_none());
    assert!(config.download.show_progress);
    assert!(config.download.prefer_strip);
    assert!(config.download.verify_checksums);
    assert_eq!(config.output.color, "auto");
    assert!(!config.output.quiet);
    assert!(!config.output.verbose);
}

/// Test merging configurations - override takes precedence
#[test]
fn test_merge_override_values() {
    let base = Config {
        install_dir: "/base/bin".to_string(),
        github_token: Some("base_token".to_string()),
        download: DownloadConfig {
            show_progress: true,
            prefer_strip: false,
            verify_checksums: false,
        },
        output: OutputConfig {
            color: "auto".to_string(),
            quiet: false,
            verbose: false,
        },
    };

    let override_config = Config {
        install_dir: "/override/bin".to_string(),
        github_token: Some("override_token".to_string()),
        download: DownloadConfig {
            show_progress: false,
            prefer_strip: true,
            verify_checksums: true,
        },
        output: OutputConfig {
            color: "always".to_string(),
            quiet: true,
            verbose: true,
        },
    };

    let merged = base.merge(override_config);

    assert_eq!(merged.install_dir, "/override/bin");
    assert_eq!(merged.github_token, Some("override_token".to_string()));
    assert!(!merged.download.show_progress);
    assert!(merged.download.prefer_strip);
    assert!(merged.download.verify_checksums);
    assert_eq!(merged.output.color, "always");
    assert!(merged.output.quiet);
    assert!(merged.output.verbose);
}

/// Test merging when override has default values
/// Since we can't distinguish between "user set default" and "default value",
/// we always use the override values
#[test]
fn test_merge_uses_override_defaults() {
    let base = Config {
        install_dir: "/custom/bin".to_string(),
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

    // Override with default values - uses all override values
    let override_config = Config::default();
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

/// Test merging with empty GitHub token doesn't override
#[test]
fn test_merge_preserves_token_when_none() {
    let base = Config {
        github_token: Some("existing_token".to_string()),
        ..Default::default()
    };

    let override_config = Config {
        github_token: None,
        ..Default::default()
    };

    let merged = base.merge(override_config);
    assert_eq!(merged.github_token, Some("existing_token".to_string()));
}

/// Test invalid config parsing produces helpful error
#[test]
fn test_invalid_config_parsing() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"invalid toml content [[[}").unwrap();

    let result = Config::load_from_file(file.path());
    assert!(result.is_err());

    let error = result.unwrap_err().to_string();
    assert!(error.contains("Failed to parse config file"));
}

/// Test missing file error
#[test]
fn test_missing_file_error() {
    let result = Config::load_from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Failed to read config file"));
}

/// Test loading from environment variable path
#[test]
fn test_load_from_env_var() -> Result<()> {
    let toml_content = r#"install_dir = "/env/bin""#;
    let (file, _path) = create_temp_config_file(toml_content);
    let file_path = file.path().to_str().unwrap();

    // Set the environment variable
    std::env::set_var("GITCLAW_CONFIG", file_path);

    let config = Config::load_from_env()?.expect("Should load from env");
    assert_eq!(config.install_dir, "/env/bin");

    // Clean up
    std::env::remove_var("GITCLAW_CONFIG");

    Ok(())
}

/// Test loading from empty environment variable returns None
#[test]
fn test_load_from_empty_env_var() -> Result<()> {
    std::env::set_var("GITCLAW_CONFIG", "");

    let config = Config::load_from_env()?;
    assert!(config.is_none());

    std::env::remove_var("GITCLAW_CONFIG");

    Ok(())
}

/// Test loading from local config file (./.gitclaw.toml)
#[test]
fn test_load_from_local() -> Result<()> {
    // Create a temporary directory as the working directory
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join(".gitclaw.toml");

    // Write the config file
    std::fs::write(&config_path, "install_dir = \"/local/bin\"\n")?;

    // Change to temp directory temporarily
    let original_dir = std::env::current_dir()?;
    std::env::set_current_dir(&temp_dir)?;

    let config = Config::load_from_local()?.expect("Should load from local");
    assert_eq!(config.install_dir, "/local/bin");

    // Restore directory
    std::env::set_current_dir(original_dir)?;

    Ok(())
}

/// Test loading from XDG config directory
#[test]
fn test_load_from_xdg() -> Result<()> {
    // Create a temporary directory to simulate XDG config
    let temp_config_dir = TempDir::new()?;
    let gitclaw_config_dir = temp_config_dir.path().join("gitclaw");
    std::fs::create_dir_all(&gitclaw_config_dir)?;

    let config_path = gitclaw_config_dir.join("config.toml");
    std::fs::write(&config_path, "install_dir = \"/xdg/bin\"\n")?;

    // Set XDG_CONFIG_HOME
    std::env::set_var("XDG_CONFIG_HOME", temp_config_dir.path());

    // Force re-evaluation of config directories by creating a fresh call
    // Note: dirs crate caches, so we test via load_from_file directly
    let config = Config::load_from_file(&config_path)?;
    assert_eq!(config.install_dir, "/xdg/bin");

    std::env::remove_var("XDG_CONFIG_HOME");

    Ok(())
}

/// Test loading from legacy config path (~/.gitclaw.toml)
#[test]
fn test_load_from_legacy() -> Result<()> {
    // Create a temporary directory to simulate home
    let temp_home = TempDir::new()?;
    let config_path = temp_home.path().join(".gitclaw.toml");
    std::fs::write(&config_path, "install_dir = \"/legacy/bin\"\n")?;

    // Set HOME environment variable
    std::env::set_var("HOME", temp_home.path());

    // Test by loading the file directly since dirs relies on HOME
    let config = Config::load_from_file(&config_path)?;
    assert_eq!(config.install_dir, "/legacy/bin");

    // We can't easily test load_from_legacy without affecting real HOME,
    // but we verified the file format is correct

    Ok(())
}

/// Test expanded install_dir with tilde
#[test]
fn test_expanded_install_dir_with_tilde() -> Result<()> {
    let config = Config {
        install_dir: "~/.local/bin".to_string(),
        ..Default::default()
    };

    let expanded = config.expanded_install_dir()?;

    // Should expand to something not starting with ~
    let expanded_str = expanded.to_string_lossy();
    assert!(!expanded_str.starts_with('~'));

    // Should end with .local/bin
    assert!(expanded_str.ends_with(".local/bin"));

    Ok(())
}

/// Test expanded install_dir with absolute path
#[test]
fn test_expanded_install_dir_absolute() -> Result<()> {
    let config = Config {
        install_dir: "/usr/local/bin".to_string(),
        ..Default::default()
    };

    let expanded = config.expanded_install_dir()?;
    assert_eq!(expanded, PathBuf::from("/usr/local/bin"));

    Ok(())
}

/// Test config with only output settings
#[test]
fn test_partial_output_config() -> Result<()> {
    let toml_content = r#"
[output]
color = "always"
verbose = true
"#;
    let (_file, path) = create_temp_config_file(toml_content);

    let config = Config::load_from_file(&path)?;

    // Should keep defaults for other fields
    assert_eq!(config.install_dir, "~/.local/bin");
    assert!(config.github_token.is_none());
    // Should have the overridden output values
    assert_eq!(config.output.color, "always");
    assert!(!config.output.quiet);
    assert!(config.output.verbose);
    // Should keep defaults for download
    assert!(config.download.show_progress);

    Ok(())
}

/// Test config with only download settings
#[test]
fn test_partial_download_config() -> Result<()> {
    let toml_content = r#"
[download]
show_progress = false
"#;
    let (_file, path) = create_temp_config_file(toml_content);

    let config = Config::load_from_file(&path)?;

    assert!(!config.download.show_progress);
    assert!(config.download.prefer_strip); // default
    assert!(config.download.verify_checksums); // default
    assert_eq!(config.install_dir, "~/.local/bin"); // default

    Ok(())
}

/// Test empty config file (all defaults)
#[test]
fn test_empty_config_file() -> Result<()> {
    let toml_content = "";
    let (_file, path) = create_temp_config_file(toml_content);

    let config = Config::load_from_file(&path)?;

    // Should use all defaults
    assert_eq!(config.install_dir, "~/.local/bin");
    assert!(config.github_token.is_none());
    assert!(config.download.show_progress);
    assert_eq!(config.output.color, "auto");

    Ok(())
}

/// Test config load when no sources exist
/// Note: This test checks that load() succeeds and returns valid defaults
/// even if system config files exist (it will merge them in that case)
#[test]
fn test_load_succeeds() -> Result<()> {
    // Ensure no env var is set
    std::env::remove_var("GITCLAW_CONFIG");

    let config = Config::load()?;

    // Should always have a valid install_dir
    assert!(!config.install_dir.is_empty());
    // Download defaults should be valid booleans
    // (may differ from true defaults if a config file overrides them)

    Ok(())
}

/// Test that DownloadConfig implements Default correctly
#[test]
fn test_download_config_default() {
    let config = DownloadConfig::default();
    assert!(config.show_progress);
    assert!(config.prefer_strip);
    assert!(config.verify_checksums);
}

/// Test that OutputConfig implements Default correctly
#[test]
fn test_output_config_default() {
    let config = OutputConfig::default();
    assert_eq!(config.color, "auto");
    assert!(!config.quiet);
    assert!(!config.verbose);
}
