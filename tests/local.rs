use tempfile::TempDir;

use gitclaw::config::Config;
use gitclaw::registry::Registry;
use gitclaw::util;

#[test]
fn test_local_install_dir_structure() {
    let dir = TempDir::new().unwrap();
    let local_dir = dir.path().join(".gitclaw");
    let config = Config {
        install_dir: local_dir.clone(),
        ..Config::default()
    };

    assert!(config.install_dir.ends_with(".gitclaw"));
    let bin = util::bin_dir_from(&config.install_dir);
    assert!(bin.ends_with("bin"));
    assert!(bin.starts_with(local_dir.to_str().unwrap()));
}

#[test]
fn test_local_registry_path() {
    let dir = TempDir::new().unwrap();
    let local_dir = dir.path().join(".gitclaw");
    let config = Config {
        install_dir: local_dir.clone(),
        ..Config::default()
    };

    let reg_path = util::registry_path_from(&config.install_dir);
    assert!(reg_path.ends_with("registry.toml"));
    assert!(reg_path.starts_with(local_dir.to_str().unwrap()));
}

#[test]
fn test_local_registry_isolation() {
    let local_dir = TempDir::new().unwrap();
    let local_config = Config {
        install_dir: local_dir.path().join(".gitclaw"),
        ..Config::default()
    };

    let local_reg_path = util::registry_path_from(&local_config.install_dir);
    let global_dir = TempDir::new().unwrap();
    let global_config = Config {
        install_dir: global_dir.path().to_path_buf(),
        ..Config::default()
    };
    let global_reg_path = util::registry_path_from(&global_config.install_dir);

    assert_ne!(local_reg_path, global_reg_path);
}

#[test]
fn test_local_registry_load_save() {
    let dir = TempDir::new().unwrap();
    let local_dir = dir.path().join(".gitclaw");
    let config = Config {
        install_dir: local_dir.clone(),
        ..Config::default()
    };

    let reg_path = util::registry_path_from(&config.install_dir);
    std::fs::create_dir_all(reg_path.parent().unwrap()).unwrap();

    let mut reg = Registry::load_from(&reg_path).unwrap();
    reg.add(gitclaw::registry::InstalledPackage {
        name: "sharkdp/bat".to_string(),
        owner: "sharkdp".to_string(),
        repo: "bat".to_string(),
        version: "0.24.0".to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: local_dir.join("bin").join("bat"),
        install_dir: local_dir.join("packages").join("sharkdp").join("bat"),
        asset_name: "bat-v0.24.0-x86_64-linux.tar.gz".to_string(),
        identifier: "bat".to_string(),
        channel: None,
    });
    reg.save().unwrap();

    let loaded = Registry::load_from(&reg_path).unwrap();
    assert!(loaded.is_installed("sharkdp/bat"));
    assert_eq!(loaded.packages.len(), 1);
}

#[test]
fn test_local_cache_dir_isolation() {
    let dir = TempDir::new().unwrap();
    let local_dir = dir.path().join(".gitclaw");
    let config = Config {
        install_dir: local_dir.clone(),
        ..Config::default()
    };

    let cache_dir = gitclaw::cache::cache_dir(&config);
    assert!(cache_dir.starts_with(local_dir.to_str().unwrap()));
    assert!(cache_dir.ends_with("cache"));
}
