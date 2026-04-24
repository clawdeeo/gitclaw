mod fixtures;

use fixtures::{FD_OWNER, FD_REPO, OWNER, REPO};
use std::path::PathBuf;

use gitclaw::core::channel::Channel;
use gitclaw::registry::{InstalledPackage, Registry};
use tempfile::TempDir;

fn make_pkg_with_channel(
    name: &str,
    owner: &str,
    repo: &str,
    version: &str,
    channel: Option<Channel>,
) -> InstalledPackage {
    InstalledPackage {
        name: name.to_string(),
        owner: owner.to_string(),
        repo: repo.to_string(),
        version: version.to_string(),
        installed_at: "2026-01-01T00:00:00Z".to_string(),
        binary_path: PathBuf::from("/tmp/test"),
        install_dir: PathBuf::from("/tmp/test"),
        asset_name: format!("{}.tar.gz", repo),
        identifier: repo.to_string(),
        channel,
    }
}

#[test]
fn test_installed_package_with_channel() {
    let pkg = make_pkg_with_channel(
        "user/repo",
        "user",
        "repo",
        "1.0.0-nightly",
        Some(Channel::Nightly),
    );

    assert_eq!(pkg.channel, Some(Channel::Nightly));
}

#[test]
fn test_installed_package_without_channel() {
    let pkg = make_pkg_with_channel("user/repo", "user", "repo", "1.0.0", None);
    assert_eq!(pkg.channel, None);
}

#[test]
fn test_registry_save_load_with_channel() {
    let dir = TempDir::new().unwrap();
    let reg_path = dir.path().join("registry.toml");
    std::fs::create_dir_all(dir.path()).unwrap();
    let mut reg = Registry::load_from(&reg_path).unwrap();

    reg.add(make_pkg_with_channel(
        &format!("{}/{}", OWNER, REPO),
        OWNER,
        REPO,
        "14.0.0-nightly",
        Some(Channel::Nightly),
    ));

    reg.add(make_pkg_with_channel(
        &format!("{}/{}", FD_OWNER, FD_REPO),
        FD_OWNER,
        FD_REPO,
        "8.7.0",
        None,
    ));

    reg.save().unwrap();

    let loaded = Registry::load_from(&reg_path).unwrap();
    let rg = loaded.packages.get(&format!("{}/{}", OWNER, REPO)).unwrap();
    assert_eq!(rg.channel, Some(Channel::Nightly));

    let fd = loaded
        .packages
        .get(&format!("{}/{}", FD_OWNER, FD_REPO))
        .unwrap();
    assert_eq!(fd.channel, None);
}

#[test]
fn test_registry_backward_compat_no_channel_field() {
    let dir = TempDir::new().unwrap();
    let reg_path = dir.path().join("registry.toml");

    let toml_str = r#"
[packages."user/repo"]
name = "user/repo"
owner = "user"
repo = "repo"
version = "1.0.0"
installed_at = "2026-01-01T00:00:00Z"
binary_path = "/tmp/test"
install_dir = "/tmp/test"
asset_name = "repo.tar.gz"
identifier = "repo"
"#;

    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(&reg_path, toml_str).unwrap();

    let reg = Registry::load_from(&reg_path).unwrap();
    let pkg = reg.packages.get("user/repo").unwrap();
    assert_eq!(pkg.channel, None);
}
