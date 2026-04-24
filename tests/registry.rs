mod fixtures;

use fixtures::{OWNER, PACKAGE, REPO, VERSION};
use std::path::PathBuf;

#[test]
fn test_installed_package_struct() {
    let pkg = gitclaw::registry::InstalledPackage {
        name: PACKAGE.to_string(),
        owner: OWNER.to_string(),
        repo: REPO.to_string(),
        version: VERSION.to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: PathBuf::from("/home/user/.gitclaw/bin/rg"),
        install_dir: PathBuf::from(format!("/home/user/.gitclaw/packages/{}/{}", OWNER, REPO)),
        asset_name: format!("{}-{}-x86_64-unknown-linux-musl.tar.gz", REPO, VERSION),
        identifier: REPO.to_string(),
        channel: None,
    };

    assert_eq!(pkg.name, PACKAGE);
    assert_eq!(pkg.owner, OWNER);
    assert_eq!(pkg.repo, REPO);
    assert_eq!(pkg.version, VERSION);
    assert_eq!(pkg.identifier, REPO);

    assert_eq!(
        pkg.asset_name,
        format!("{}-{}-x86_64-unknown-linux-musl.tar.gz", REPO, VERSION)
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
        binary_path: PathBuf::from("/home/user/.gitclaw/bin/package"),
        install_dir: PathBuf::from("/home/user/.gitclaw/packages/test/package"),
        asset_name: "package-1.0.0.tar.gz".to_string(),
        identifier: "package".to_string(),
        channel: None,
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
            binary_path: PathBuf::from("/home/user/.gitclaw/bin/pkg1"),
            install_dir: PathBuf::from("/home/user/.gitclaw/packages/user1/pkg1"),
            asset_name: "pkg1.tar.gz".to_string(),
            identifier: "pkg1".to_string(),
            channel: None,
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
            binary_path: PathBuf::from("/home/user/.gitclaw/bin/pkg2"),
            install_dir: PathBuf::from("/home/user/.gitclaw/packages/user2/pkg2"),
            asset_name: "pkg2.tar.gz".to_string(),
            identifier: "pkg2".to_string(),
            channel: None,
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
        binary_path: PathBuf::from("/home/user/.gitclaw/bin/pkg"),
        install_dir: PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
        asset_name: "pkg.tar.gz".to_string(),
        identifier: "pkg".to_string(),
        channel: None,
    };

    registry.packages.insert(pkg.name.clone(), pkg);

    let retrieved = registry.packages.get("test/pkg");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().version, "1.0.0");

    assert!(!registry.packages.contains_key("nonexistent/pkg"));
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
            binary_path: PathBuf::from("/home/user/.gitclaw/bin/package"),
            install_dir: PathBuf::from("/home/user/.gitclaw/packages/test/package"),
            asset_name: "package.tar.gz".to_string(),
            identifier: "package".to_string(),
            channel: None,
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
            binary_path: PathBuf::from("/home/user/.gitclaw/bin/pkg"),
            install_dir: PathBuf::from("/home/user/.gitclaw/packages/test/pkg"),
            asset_name: "pkg.tar.gz".to_string(),
            identifier: "pkg".to_string(),
            channel: None,
        },
    );

    let serialized = toml::to_string(&registry).unwrap();
    assert!(serialized.contains("test/pkg"));
    assert!(serialized.contains("1.0.0"));

    let deserialized: gitclaw::registry::Registry = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.packages.len(), 1);
    assert!(deserialized.packages.contains_key("test/pkg"));
}

#[test]
fn test_serialize_deserialize_without_identifier() {
    let toml_str = r#"
[packages."legacy/pkg"]
name = "legacy/pkg"
owner = "legacy"
repo = "pkg"
version = "1.0.0"
installed_at = "2024-01-01T00:00:00Z"
binary_path = "/home/user/.gitclaw/bin/pkg"
install_dir = "/home/user/.gitclaw/packages/legacy/pkg"
asset_name = "pkg.tar.gz"
"#;

    let registry: gitclaw::registry::Registry = toml::from_str(toml_str).unwrap();
    assert_eq!(registry.packages.len(), 1);
    let pkg = registry.packages.get("legacy/pkg").unwrap();
    assert_eq!(pkg.identifier, "");
}

#[test]
fn test_registry_is_not_installed() {
    let registry = gitclaw::registry::Registry::default();
    assert!(!registry.is_installed("nonexistent/package"));
}

#[test]
fn test_registry_crud() {
    let mut reg = gitclaw::registry::Registry::default();

    let pkg = gitclaw::registry::InstalledPackage {
        name: "user/repo".to_string(),
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: "v1.0.0".to_string(),
        installed_at: "2024-01-01".to_string(),
        binary_path: PathBuf::from("/tmp/binary"),
        install_dir: PathBuf::from("/tmp/install"),
        asset_name: "tool.tar.gz".to_string(),
        identifier: "repo".to_string(),
        channel: None,
    };

    assert!(!reg.is_installed("user/repo"));
    reg.add(pkg);
    assert!(reg.is_installed("user/repo"));

    let removed = reg.remove("user/repo");
    assert!(removed.is_some());
    assert!(!reg.is_installed("user/repo"));
}
