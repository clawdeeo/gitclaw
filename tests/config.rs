use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use gitclaw::config::{Config, DownloadConfig, OutputConfig};

#[test]
fn default_config_values() {
    let config = Config::default();

    assert!(config.download.show_progress);
    assert!(config.download.prefer_strip);
    assert!(config.download.verify_checksums);
    assert_eq!(config.output.color, "auto");
    assert!(!config.output.quiet);
    assert!(!config.output.verbose);
}

#[test]
fn load_from_local_config() {
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

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(&temp).unwrap();

    let config = Config::load_from_local().unwrap().unwrap();
    assert_eq!(config.install_dir, PathBuf::from("/custom/bin"));
    assert_eq!(config.github_token, Some("test-token".to_string()));
    assert!(!config.download.show_progress);
    assert_eq!(config.output.color, "never");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn parse_legacy_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join(".gitclaw.toml");

    fs::write(
        &config_path,
        r#"
install_dir = "/legacy/bin"
"#,
    )
    .unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    let config: Config = toml::from_str(&content).unwrap();
    assert_eq!(config.install_dir, PathBuf::from("/legacy/bin"));
}

#[test]
fn load_from_env_var() {
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

#[test]
fn config_merge_precedence() {
    let mut config = Config::default();
    assert!(config.download.show_progress);
    assert_eq!(config.output.color, "auto");

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
    assert!(!config.download.show_progress);
    assert_eq!(config.output.color, "never");
    assert!(config.output.verbose);
}

#[test]
fn github_token_field() {
    let config = Config {
        github_token: Some("test-token".to_string()),
        ..Default::default()
    };
    assert_eq!(config.github_token.as_deref(), Some("test-token"));

    let config_no_token = Config::default();
    assert_eq!(config_no_token.github_token.as_deref(), None);
}

#[test]
fn install_dir_field() {
    let config = Config {
        install_dir: PathBuf::from("/custom/install"),
        ..Default::default()
    };
    assert_eq!(config.install_dir, PathBuf::from("/custom/install"));
}

#[test]
fn download_config_default() {
    let dl = DownloadConfig::default();
    assert!(dl.show_progress);
    assert!(dl.prefer_strip);
    assert!(dl.verify_checksums);
}

#[test]
fn output_config_default() {
    let out = OutputConfig::default();
    assert_eq!(out.color, "auto");
    assert!(!out.quiet);
    assert!(!out.verbose);
}
