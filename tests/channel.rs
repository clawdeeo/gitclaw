use gitclaw::channel::{filter_releases, matches_channel, Channel};
use gitclaw::network::github::Release;

#[test]
fn test_channel_stable_matches_plain_tag() {
    let patterns = Channel::Stable.default_patterns();
    assert!(matches_channel("v1.0.0", &patterns));
    assert!(matches_channel("1.0.0", &patterns));
}

#[test]
fn test_channel_stable_excludes_prerelease() {
    let patterns = Channel::Stable.default_patterns();
    assert!(!matches_channel("v1.0.0-beta", &patterns));
    assert!(!matches_channel("v1.0.0-nightly", &patterns));
    assert!(!matches_channel("v1.0.0-rc1", &patterns));
}

#[test]
fn test_channel_beta_matches() {
    let patterns = Channel::Beta.default_patterns();
    assert!(matches_channel("v1.0.0-beta", &patterns));
    assert!(matches_channel("v1.0.0-rc1", &patterns));
    assert!(matches_channel("v1.0.0-rc2", &patterns));
}

#[test]
fn test_channel_beta_excludes_stable_and_nightly() {
    let patterns = Channel::Beta.default_patterns();
    assert!(!matches_channel("v1.0.0", &patterns));
    assert!(!matches_channel("v1.0.0-nightly", &patterns));
}

#[test]
fn test_channel_nightly_matches() {
    let patterns = Channel::Nightly.default_patterns();
    assert!(matches_channel("v1.0.0-nightly", &patterns));
    assert!(matches_channel("v1.0.0-dev", &patterns));
}

#[test]
fn test_channel_nightly_excludes_stable_and_beta() {
    let patterns = Channel::Nightly.default_patterns();
    assert!(!matches_channel("v1.0.0", &patterns));
    assert!(!matches_channel("v1.0.0-beta", &patterns));
}

#[test]
fn test_filter_releases_by_channel() {
    let releases = vec![
        Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![],
        },
        Release {
            tag_name: "v1.1.0-beta".to_string(),
            name: None,
            body: None,
            assets: vec![],
        },
        Release {
            tag_name: "v1.0.1-nightly".to_string(),
            name: None,
            body: None,
            assets: vec![],
        },
        Release {
            tag_name: "v2.0.0-rc1".to_string(),
            name: None,
            body: None,
            assets: vec![],
        },
    ];

    let stable = filter_releases(&releases, Channel::Stable, None);
    assert_eq!(stable.len(), 1);
    assert_eq!(stable[0].tag_name, "v1.0.0");

    let beta = filter_releases(&releases, Channel::Beta, None);
    assert_eq!(beta.len(), 2);

    let nightly = filter_releases(&releases, Channel::Nightly, None);
    assert_eq!(nightly.len(), 1);
    assert_eq!(nightly[0].tag_name, "v1.0.1-nightly");
}

#[test]
fn test_channel_parse() {
    let ch: Channel = "stable".parse().unwrap();
    assert_eq!(ch, Channel::Stable);

    let ch: Channel = "beta".parse().unwrap();
    assert_eq!(ch, Channel::Beta);

    let ch: Channel = "nightly".parse().unwrap();
    assert_eq!(ch, Channel::Nightly);

    assert!("unknown".parse::<Channel>().is_err());
}

#[test]
fn test_channel_display() {
    assert_eq!(Channel::Stable.to_string(), "stable");
    assert_eq!(Channel::Beta.to_string(), "beta");
    assert_eq!(Channel::Nightly.to_string(), "nightly");
}

#[test]
fn test_channel_patterns_with_overrides() {
    use std::collections::HashMap;

    let mut overrides = HashMap::new();
    overrides.insert(
        "nightly".to_string(),
        vec!["*-canary".to_string(), "*-edge".to_string()],
    );

    let patterns = Channel::Nightly.patterns_with_overrides(Some(&overrides));
    assert_eq!(patterns, vec!["*-canary", "*-edge"]);

    let patterns = Channel::Beta.patterns_with_overrides(Some(&overrides));
    assert_eq!(patterns, Channel::Beta.default_patterns());
}

#[test]
fn test_glob_exact_match() {
    let patterns = vec!["1.0.0-beta".to_string()];
    assert!(matches_channel("1.0.0-beta", &patterns));
    assert!(!matches_channel("2.0.0-beta", &patterns));
}

#[test]
fn test_glob_wildcard_prefix() {
    let patterns = vec!["*-nightly".to_string(), "*-rc*".to_string()];
    assert!(matches_channel("v1.0.0-nightly", &patterns));
    assert!(matches_channel("v2.0.0-rc1", &patterns));
    assert!(!matches_channel("v1.0.0", &patterns));
}

#[test]
fn test_glob_wildcard_suffix() {
    let patterns = vec!["v1.0.0-*".to_string()];
    assert!(matches_channel("v1.0.0-rc1", &patterns));
    assert!(!matches_channel("v2.0.0-beta", &patterns));
}

#[test]
fn test_glob_star_matches_anything() {
    let patterns = vec!["*".to_string()];
    assert!(matches_channel("anything", &patterns));
}

#[test]
fn test_glob_contains_match() {
    let patterns = vec!["*rc*".to_string()];
    assert!(matches_channel("v1.0.0-rc1", &patterns));
    assert!(!matches_channel("v1.0.0", &patterns));
}

#[test]
fn test_empty_patterns_match_nothing() {
    let patterns: Vec<String> = vec![];
    assert!(!matches_channel("v1.0.0", &patterns));
}
