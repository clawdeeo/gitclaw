use semver::Version;

use gitclaw::semver::{parse_tag_version, strip_v_prefix, VersionConstraint};

#[test]
fn test_semver_exact_version() {
    let constraint = VersionConstraint::parse("1.2.3").unwrap();
    assert!(constraint.matches(&Version::parse("1.2.3").unwrap()));
    assert!(!constraint.matches(&Version::parse("1.2.4").unwrap()));
}

#[test]
fn test_semver_caret_range() {
    let constraint = VersionConstraint::parse("^1.2.3").unwrap();
    assert!(constraint.matches(&Version::parse("1.2.3").unwrap()));
    assert!(constraint.matches(&Version::parse("1.2.9").unwrap()));
    assert!(constraint.matches(&Version::parse("1.3.0").unwrap()));
    assert!(!constraint.matches(&Version::parse("2.0.0").unwrap()));
}

#[test]
fn test_semver_tilde_range() {
    let constraint = VersionConstraint::parse("~1.2.3").unwrap();
    assert!(constraint.matches(&Version::parse("1.2.3").unwrap()));
    assert!(constraint.matches(&Version::parse("1.2.9").unwrap()));
    assert!(!constraint.matches(&Version::parse("1.3.0").unwrap()));
}

#[test]
fn test_semver_gte_range() {
    let constraint = VersionConstraint::parse(">=1.0.0").unwrap();
    assert!(constraint.matches(&Version::parse("1.0.0").unwrap()));
    assert!(constraint.matches(&Version::parse("2.0.0").unwrap()));
    assert!(!constraint.matches(&Version::parse("0.9.0").unwrap()));
}

#[test]
fn test_semver_lt_range() {
    let constraint = VersionConstraint::parse("<2.0.0").unwrap();
    assert!(constraint.matches(&Version::parse("1.9.9").unwrap()));
    assert!(!constraint.matches(&Version::parse("2.0.0").unwrap()));
}

#[test]
fn test_semver_lte_range() {
    let constraint = VersionConstraint::parse("<=2.0.0").unwrap();
    assert!(constraint.matches(&Version::parse("2.0.0").unwrap()));
    assert!(constraint.matches(&Version::parse("1.9.9").unwrap()));
    assert!(!constraint.matches(&Version::parse("2.0.1").unwrap()));
}

#[test]
fn test_semver_strip_v_prefix() {
    assert_eq!(strip_v_prefix("v1.2.3"), "1.2.3");
    assert_eq!(strip_v_prefix("1.2.3"), "1.2.3");
}

#[test]
fn test_semver_parse_tag_version() {
    let v = parse_tag_version("v1.2.3").unwrap();
    assert_eq!(v, Version::parse("1.2.3").unwrap());

    let v = parse_tag_version("1.2.3").unwrap();
    assert_eq!(v, Version::parse("1.2.3").unwrap());
}

#[test]
fn test_semver_invalid_constraint() {
    assert!(VersionConstraint::parse("not-a-version").is_err());
}
