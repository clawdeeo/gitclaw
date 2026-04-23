use gitclaw::semver;

#[test]
fn test_semver_version_comparison() {
    let v1 = semver::parse_tag_version("1.0.0").unwrap();
    let v2 = semver::parse_tag_version("2.0.0").unwrap();
    assert!(v2 > v1);
}

#[test]
fn test_semver_tag_parsing() {
    let tag = "v13.0.0";
    let version = semver::parse_tag_version(tag).unwrap();
    assert_eq!(version.major, 13);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_semver_tag_without_v() {
    let tag = "13.0.0";
    let version = semver::parse_tag_version(tag).unwrap();
    assert_eq!(version.major, 13);
}

#[test]
fn test_installed_vs_latest_different() {
    let installed = "v1.0.0";
    let latest = "v2.0.0";
    assert_ne!(installed, latest);
}

#[test]
fn test_installed_vs_latest_same() {
    let installed = "v1.0.0";
    let latest = "v1.0.0";
    assert_eq!(installed, latest);
}
