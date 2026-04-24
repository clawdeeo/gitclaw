mod fixtures;

use fixtures::{BAT_REPO, BAT_VERSION, FD_OWNER, FD_REPO, FD_VERSION, OWNER, REPO, VERSION};
use tempfile::TempDir;

use gitclaw::config::Config;
use gitclaw::export::{ExportEntry, ExportFile};
use gitclaw::registry::{InstalledPackage, Registry};
use gitclaw::util::registry_path_from;

use std::path::PathBuf;

fn make_config() -> (Config, TempDir) {
    let dir = TempDir::new().unwrap();
    let config = Config {
        install_dir: dir.path().to_path_buf(),
        ..Config::default()
    };
    (config, dir)
}

fn sample_pkg(name: &str, owner: &str, repo: &str, version: &str) -> InstalledPackage {
    InstalledPackage {
        name: name.to_string(),
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: version.to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        binary_path: PathBuf::from("/tmp/bin"),
        install_dir: PathBuf::from("/tmp/install"),
        asset_name: format!("{}-{}.tar.gz", repo, version),
        identifier: repo.to_string(),
        channel: None,
    }
}

#[test]
fn test_export_entry_serialization() {
    let entry = ExportEntry {
        owner: OWNER.to_string(),
        repo: REPO.to_string(),
        version: VERSION.to_string(),
    };

    let export = ExportFile {
        packages: vec![entry],
    };

    let toml_str = export.to_toml().unwrap();
    assert!(toml_str.contains(OWNER));
    assert!(toml_str.contains(REPO));
    assert!(toml_str.contains(VERSION));
}

#[test]
fn test_export_entry_deserialization() {
    let toml_str = r#"
[[package]]
owner = "sharkdp"
repo = "fd"
version = "8.7.0"
"#;

    let export = ExportFile::from_toml(toml_str).unwrap();
    assert_eq!(export.packages.len(), 1);
    assert_eq!(export.packages[0].owner, FD_OWNER);
    assert_eq!(export.packages[0].repo, FD_REPO);
    assert_eq!(export.packages[0].version, FD_VERSION);
}

#[test]
fn test_export_roundtrip() {
    let export = ExportFile {
        packages: vec![
            ExportEntry {
                owner: OWNER.to_string(),
                repo: REPO.to_string(),
                version: VERSION.to_string(),
            },
            ExportEntry {
                owner: FD_OWNER.to_string(),
                repo: FD_REPO.to_string(),
                version: FD_VERSION.to_string(),
            },
        ],
    };

    let toml_str = export.to_toml().unwrap();
    let reloaded = ExportFile::from_toml(&toml_str).unwrap();
    assert_eq!(reloaded.packages.len(), 2);
    assert_eq!(reloaded.packages[0], export.packages[0]);
    assert_eq!(reloaded.packages[1], export.packages[1]);
}

#[test]
fn test_export_from_registry_sorted() {
    let mut reg = Registry::default();
    reg.add(sample_pkg(
        &format!("{}/{}", FD_OWNER, FD_REPO),
        FD_OWNER,
        FD_REPO,
        FD_VERSION,
    ));

    reg.add(sample_pkg(
        &format!("{}/{}", OWNER, REPO),
        OWNER,
        REPO,
        VERSION,
    ));

    reg.add(sample_pkg(
        &format!("{}/{}", FD_OWNER, BAT_REPO),
        FD_OWNER,
        BAT_REPO,
        BAT_VERSION,
    ));

    let export = ExportFile::from_registry(&reg);
    assert_eq!(export.packages[0].owner, OWNER);
    assert_eq!(export.packages[1].repo, BAT_REPO);
    assert_eq!(export.packages[2].repo, FD_REPO);
}

#[test]
fn test_export_from_empty_registry() {
    let reg = Registry::default();
    let export = ExportFile::from_registry(&reg);
    assert!(export.packages.is_empty());
}

#[test]
fn test_export_file_write_and_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("deps.toml");

    let export = ExportFile {
        packages: vec![ExportEntry {
            owner: OWNER.to_string(),
            repo: REPO.to_string(),
            version: VERSION.to_string(),
        }],
    };

    let toml_str = export.to_toml().unwrap();
    std::fs::write(&path, &toml_str).unwrap();

    let loaded = ExportFile::from_file(&path).unwrap();
    assert_eq!(loaded.packages.len(), 1);
    assert_eq!(loaded.packages[0].repo, REPO);
}

#[test]
fn test_export_from_registry_with_config() {
    let (config, _dir) = make_config();
    let reg_path = registry_path_from(&config.install_dir);
    std::fs::create_dir_all(reg_path.parent().unwrap()).unwrap();

    let mut reg = Registry::load_from(&reg_path).unwrap();
    reg.add(sample_pkg(
        &format!("{}/{}", OWNER, REPO),
        OWNER,
        REPO,
        VERSION,
    ));
    reg.save().unwrap();

    let loaded = Registry::load_from(&reg_path).unwrap();
    let export = ExportFile::from_registry(&loaded);
    assert_eq!(export.packages.len(), 1);
}

#[test]
fn test_export_toml_format() {
    let export = ExportFile {
        packages: vec![
            ExportEntry {
                owner: FD_OWNER.to_string(),
                repo: BAT_REPO.to_string(),
                version: BAT_VERSION.to_string(),
            },
            ExportEntry {
                owner: FD_OWNER.to_string(),
                repo: FD_REPO.to_string(),
                version: FD_VERSION.to_string(),
            },
        ],
    };

    let toml_str = export.to_toml().unwrap();
    assert!(toml_str.contains("[[package]]"));
    assert!(toml_str.contains(&format!("owner = \"{}\"", FD_OWNER)));
    assert!(toml_str.contains(&format!("repo = \"{}\"", BAT_REPO)));
    assert!(toml_str.contains(&format!("repo = \"{}\"", FD_REPO)));
}
