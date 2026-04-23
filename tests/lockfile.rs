use std::path::PathBuf;

use tempfile::TempDir;

use gitclaw::lockfile::Lockfile;
use gitclaw::registry::{InstalledPackage, Registry};

fn make_pkg(name: &str, owner: &str, repo: &str, version: &str, asset: &str) -> InstalledPackage {
    InstalledPackage {
        name: name.to_string(),
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: version.to_string(),
        installed_at: "2026-01-01T00:00:00Z".to_string(),
        binary_path: PathBuf::from("/tmp/test"),
        install_dir: PathBuf::from("/tmp/test"),
        asset_name: asset.to_string(),
        identifier: repo.to_string(),
    }
}

#[test]
fn test_lockfile_from_registry() {
    let mut reg = Registry::default();
    reg.add(make_pkg(
        "BurntSushi/ripgrep",
        "BurntSushi",
        "ripgrep",
        "v14.1.0",
        "ripgrep-14.tar.gz",
    ));
    reg.add(make_pkg(
        "sharkdp/fd",
        "sharkdp",
        "fd",
        "v10.2.0",
        "fd-10.tar.gz",
    ));

    let lockfile = Lockfile::from_registry(&reg);
    assert_eq!(lockfile.packages.len(), 2);

    let rg = lockfile
        .packages
        .iter()
        .find(|p| p.repo == "ripgrep")
        .unwrap();
    assert_eq!(rg.owner, "BurntSushi");
    assert_eq!(rg.version, "v14.1.0");
    assert_eq!(rg.asset, "ripgrep-14.tar.gz");
}

#[test]
fn test_lockfile_roundtrip() {
    let dir = TempDir::new().unwrap();

    let lockfile = Lockfile {
        packages: vec![gitclaw::lockfile::LockEntry {
            owner: "BurntSushi".to_string(),
            repo: "ripgrep".to_string(),
            version: "v14.1.0".to_string(),
            asset: "ripgrep-14.tar.gz".to_string(),
        }],
    };

    lockfile.save(dir.path()).unwrap();
    let loaded = Lockfile::load(dir.path()).unwrap();

    assert_eq!(loaded.packages.len(), 1);
    assert_eq!(loaded.packages[0].owner, "BurntSushi");
    assert_eq!(loaded.packages[0].repo, "ripgrep");
    assert_eq!(loaded.packages[0].version, "v14.1.0");
}

#[test]
fn test_lockfile_toml_format() {
    let lockfile = Lockfile {
        packages: vec![gitclaw::lockfile::LockEntry {
            owner: "sharkdp".to_string(),
            repo: "fd".to_string(),
            version: "v10.2.0".to_string(),
            asset: "fd-10.tar.gz".to_string(),
        }],
    };

    let toml_str = toml::to_string_pretty(&lockfile).unwrap();
    assert!(toml_str.contains("[[package]]"));
    assert!(toml_str.contains("owner = \"sharkdp\""));
    assert!(toml_str.contains("repo = \"fd\""));
}

#[test]
fn test_lockfile_empty_registry() {
    let reg = Registry::default();
    let lockfile = Lockfile::from_registry(&reg);
    assert!(lockfile.packages.is_empty());
}

#[test]
fn test_lockfile_is_present() {
    let dir = TempDir::new().unwrap();
    assert!(!Lockfile::is_present(dir.path()));

    let lockfile = Lockfile::default();
    lockfile.save(dir.path()).unwrap();
    assert!(Lockfile::is_present(dir.path()));
}
