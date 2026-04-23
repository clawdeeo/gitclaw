use gitclaw::semver::VersionConstraint;

#[test]
fn test_semver_exact_version() {
    let c = VersionConstraint::parse("1.2.3").unwrap();
    assert!(c.matches(&semver::Version::parse("1.2.3").unwrap()));
    assert!(!c.matches(&semver::Version::parse("1.2.4").unwrap()));
}

#[test]
fn test_semver_caret_range() {
    let c = VersionConstraint::parse("^1.2.3").unwrap();
    assert!(c.matches(&semver::Version::parse("1.2.3").unwrap()));
    assert!(c.matches(&semver::Version::parse("1.2.9").unwrap()));
    assert!(c.matches(&semver::Version::parse("1.3.0").unwrap()));
    assert!(!c.matches(&semver::Version::parse("2.0.0").unwrap()));
}

#[test]
fn test_semver_tilde_range() {
    let c = VersionConstraint::parse("~1.2.3").unwrap();
    assert!(c.matches(&semver::Version::parse("1.2.3").unwrap()));
    assert!(c.matches(&semver::Version::parse("1.2.9").unwrap()));
    assert!(!c.matches(&semver::Version::parse("1.3.0").unwrap()));
}

#[test]
fn test_semver_gte_range() {
    let c = VersionConstraint::parse(">=1.0.0").unwrap();
    assert!(c.matches(&semver::Version::parse("1.0.0").unwrap()));
    assert!(c.matches(&semver::Version::parse("2.0.0").unwrap()));
    assert!(!c.matches(&semver::Version::parse("0.9.0").unwrap()));
}

#[test]
fn test_semver_strip_v_prefix() {
    assert_eq!(gitclaw::semver::strip_v_prefix("v1.2.3"), "1.2.3");
    assert_eq!(gitclaw::semver::strip_v_prefix("1.2.3"), "1.2.3");
}

#[test]
fn test_semver_parse_tag_version() {
    let v = gitclaw::semver::parse_tag_version("v1.2.3").unwrap();
    assert_eq!(v, semver::Version::parse("1.2.3").unwrap());

    let v = gitclaw::semver::parse_tag_version("1.2.3").unwrap();
    assert_eq!(v, semver::Version::parse("1.2.3").unwrap());
}

#[test]
fn test_semver_invalid_constraint() {
    assert!(VersionConstraint::parse("not-a-version").is_err());
}

#[test]
fn test_semver_lt_range() {
    let c = VersionConstraint::parse("<2.0.0").unwrap();
    assert!(c.matches(&semver::Version::parse("1.9.9").unwrap()));
    assert!(!c.matches(&semver::Version::parse("2.0.0").unwrap()));
}

#[test]
fn test_semver_lte_range() {
    let c = VersionConstraint::parse("<=2.0.0").unwrap();
    assert!(c.matches(&semver::Version::parse("2.0.0").unwrap()));
    assert!(c.matches(&semver::Version::parse("1.0.0").unwrap()));
    assert!(!c.matches(&semver::Version::parse("2.0.1").unwrap()));
}
