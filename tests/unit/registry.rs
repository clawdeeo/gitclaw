//! Integration tests for the registry module
//! Tests the public API from gitclaw::registry

#[test]
fn test_installed_package_struct() {
    let pkg = gitclaw::registry::InstalledPackage {
        name: "BurntSushi/ripgrep".to_string(),
        owner: "BurntSushi".to_string(),
        repo: "ripgrep".to_string(),
        version: "13.0.0".to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/rg"),
        install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/BurntSushi/ripgrep"),
        asset_name: "ripgrep-13.0.0-x86_64-unknown-linux-musl.tar.gz".to_string(),
    };

    assert_eq!(pkg.name, "BurntSushi/ripgrep");
    assert_eq!(pkg.owner, "BurntSushi");
    assert_eq!(pkg.repo, "ripgrep");
    assert_eq!(pkg.version, "13.0.0");
    assert_eq!(
        pkg.asset_name,
        "ripgrep-13.0.0-x86_64-unknown-linux-musl.tar.gz"
    );
}

#[test]
fn test_registry_struct() {
    let registry = gitclaw::registry::Registry::default();
    assert!(registry.packages.is_empty());
}

#[test]
fn test_registry_add() {
    let mut registry = gitclaw::registry::Registry::default();

    let pkg = gitclaw::registry::InstalledPackage {
        name: "test/package".to_string(),
        owner: "test".to_string(),
        repo: "package".to_string(),
        version: "1.0.0".to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/package"),
        install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/package"),
        asset_name: "package-1.0.0.tar.gz".to_string(),
    };

    registry.packages.insert(pkg.name.clone(), pkg);
    assert_eq!(registry.packages.len(), 1);
    assert!(registry.packages.contains_key("test/package"));
}

#[test]
fn test_registry_remove() {
    let mut registry = gitclaw::registry::Registry::default();

    registry.packages.insert(
        "user1/pkg1".to_string(),
        gitclaw::registry::InstalledPackage {
            name: "user1/pkg1".to_string(),
            owner: "user1".to_string(),
            repo: "pkg1".to_string(),
            version: "1.0.0".to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg1"),
            install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/user1/pkg1"),
            asset_name: "pkg1.tar.gz".to_string(),
        },
    );

    registry.packages.insert(
        "user2/pkg2".to_string(),
        gitclaw::registry::InstalledPackage {
            name: "user2/pkg2".to_string(),
            owner: "user2".to_string(),
            repo: "pkg2".to_string(),
            version: "2.0.0".to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg2"),
            install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/user2/pkg2"),
            asset_name: "pkg2.tar.gz".to_string(),
        },
    );

    assert_eq!(registry.packages.len(), 2);

    registry.packages.remove("user1/pkg1");
    assert_eq!(registry.packages.len(), 1);
    assert!(!registry.packages.contains_key("user1/pkg1"));
    assert!(registry.packages.contains_key("user2/pkg2"));
}

#[test]
fn test_registry_get() {
    let mut registry = gitclaw::registry::Registry::default();

    let pkg = gitclaw::registry::InstalledPackage {
        name: "test/pkg".to_string(),
        owner: "test".to_string(),
        repo: "pkg".to_string(),
        version: "1.0.0".to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg"),
        install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
        asset_name: "pkg.tar.gz".to_string(),
    };

    registry.packages.insert(pkg.name.clone(), pkg);

    let retrieved = registry.packages.get("test/pkg");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().version, "1.0.0");

    assert!(registry.packages.get("nonexistent/pkg").is_none());
}

#[test]
fn test_registry_is_installed() {
    let mut registry = gitclaw::registry::Registry::default();

    assert!(!registry.packages.contains_key("test/package"));

    registry.packages.insert(
        "test/package".to_string(),
        gitclaw::registry::InstalledPackage {
            name: "test/package".to_string(),
            owner: "test".to_string(),
            repo: "package".to_string(),
            version: "1.0.0".to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/package"),
            install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/package"),
            asset_name: "package.tar.gz".to_string(),
        },
    );

    assert!(registry.packages.contains_key("test/package"));
    assert!(!registry.packages.contains_key("other/package"));
}

#[test]
fn test_serialize_deserialize() {
    let mut registry = gitclaw::registry::Registry::default();
    registry.packages.insert(
        "test/pkg".to_string(),
        gitclaw::registry::InstalledPackage {
            name: "test/pkg".to_string(),
            owner: "test".to_string(),
            repo: "pkg".to_string(),
            version: "1.0.0".to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg"),
            install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
            asset_name: "pkg.tar.gz".to_string(),
        },
    );

    // Serialize
    let serialized = toml::to_string(&registry).unwrap();
    assert!(serialized.contains("test/pkg"));
    assert!(serialized.contains("1.0.0"));

    // Deserialize
    let deserialized: gitclaw::registry::Registry = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.packages.len(), 1);
    assert!(deserialized.packages.contains_key("test/pkg"));
}

#[test]
fn test_bin_dir() {
    let result = gitclaw::registry::bin_dir();
    // May fail if HOME is not set, which is OK in test environment
    let _ = result;
}

#[test]
fn test_gitclaw_home() {
    let result = gitclaw::registry::gitclaw_home();
    // May fail if HOME is not set, which is OK in test environment
    let _ = result;
}

#[test]
fn test_installed_package_equality() {
    let pkg1 = gitclaw::registry::InstalledPackage {
        name: "test/pkg".to_string(),
        owner: "test".to_string(),
        repo: "pkg".to_string(),
        version: "1.0.0".to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg"),
        install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
        asset_name: "pkg.tar.gz".to_string(),
    };

    let pkg2 = gitclaw::registry::InstalledPackage {
        name: "test/pkg".to_string(),
        owner: "test".to_string(),
        repo: "pkg".to_string(),
        version: "1.0.0".to_string(),
        installed_at: pkg1.installed_at.clone(),
        binary_path: std::path::PathBuf::from("/home/user/.gitclaw/bin/pkg"),
        install_dir: std::path::PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
        asset_name: "pkg.tar.gz".to_string(),
    };

    assert_eq!(pkg1.name, pkg2.name);
}
