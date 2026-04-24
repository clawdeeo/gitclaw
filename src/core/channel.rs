use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::network::github::Release;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Stable => write!(f, "stable"),
            Channel::Beta => write!(f, "beta"),
            Channel::Nightly => write!(f, "nightly"),
        }
    }
}

impl FromStr for Channel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stable" => Ok(Channel::Stable),
            "beta" => Ok(Channel::Beta),
            "nightly" => Ok(Channel::Nightly),
            other => bail!(
                "Unknown channel: '{}'. Use stable, beta, or nightly.",
                other
            ),
        }
    }
}

impl Channel {
    pub fn default_patterns(&self) -> Vec<String> {
        match self {
            Channel::Stable => vec!["!*-*".to_string()],
            Channel::Beta => vec!["*-beta".to_string(), "*-rc*".to_string()],
            Channel::Nightly => vec!["*-nightly".to_string(), "*-dev".to_string()],
        }
    }

    pub fn patterns_with_overrides(
        &self,
        overrides: Option<&HashMap<String, Vec<String>>>,
    ) -> Vec<String> {
        if let Some(over) = overrides {
            if let Some(patterns) = over.get(&self.to_string()) {
                return patterns.clone();
            }
        }

        self.default_patterns()
    }
}

pub fn matches_channel(tag: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    for pattern in patterns {
        if let Some(exclude) = pattern.strip_prefix('!') {
            if glob_match(tag, exclude) {
                return false;
            }
        } else if glob_match(tag, pattern) {
            return true;
        }
    }

    let all_exclusions = patterns.iter().all(|p| p.starts_with('!'));
    if all_exclusions {
        return true;
    }

    false
}

fn glob_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');

    match (starts_with_wildcard, ends_with_wildcard) {
        (true, true) => {
            let inner = &pattern[1..pattern.len() - 1];
            text.contains(inner)
        }
        (true, false) => {
            let suffix = &pattern[1..];
            text.ends_with(suffix)
        }
        (false, true) => {
            let prefix = &pattern[..pattern.len() - 1];
            text.starts_with(prefix)
        }
        (false, false) => text == pattern,
    }
}

pub fn filter_releases(
    releases: &[Release],
    channel: Channel,
    overrides: Option<&HashMap<String, Vec<String>>>,
) -> Vec<Release> {
    let patterns = channel.patterns_with_overrides(overrides);

    releases
        .iter()
        .filter(|r| matches_channel(&r.tag_name, &patterns))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_from_str() {
        assert_eq!("stable".parse::<Channel>().unwrap(), Channel::Stable);
        assert_eq!("beta".parse::<Channel>().unwrap(), Channel::Beta);
        assert_eq!("nightly".parse::<Channel>().unwrap(), Channel::Nightly);
        assert_eq!("STABLE".parse::<Channel>().unwrap(), Channel::Stable);
        assert!("unknown".parse::<Channel>().is_err());
    }

    #[test]
    fn test_channel_display() {
        assert_eq!(Channel::Stable.to_string(), "stable");
        assert_eq!(Channel::Beta.to_string(), "beta");
        assert_eq!(Channel::Nightly.to_string(), "nightly");
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("1.0.0-beta", "1.0.0-beta"));
        assert!(!glob_match("1.0.0-beta", "2.0.0-beta"));
    }

    #[test]
    fn test_glob_match_wildcard_prefix() {
        assert!(glob_match("v1.0.0-nightly", "*-nightly"));
        assert!(glob_match("v2.0.0-rc1", "*-rc*"));
        assert!(!glob_match("v1.0.0", "*-nightly"));
    }

    #[test]
    fn test_glob_match_wildcard_suffix() {
        assert!(glob_match("v1.0.0-rc1", "v1.0.0-*"));
        assert!(!glob_match("v2.0.0-beta", "v1.0.0-*"));
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("anything", "*"));
    }

    #[test]
    fn test_matches_channel_beta() {
        let patterns = Channel::Beta.default_patterns();
        assert!(matches_channel("v1.0.0-beta", &patterns));
        assert!(matches_channel("v1.0.0-rc1", &patterns));
        assert!(matches_channel("v1.0.0-rc2", &patterns));
        assert!(!matches_channel("v1.0.0", &patterns));
        assert!(!matches_channel("v1.0.0-nightly", &patterns));
    }

    #[test]
    fn test_matches_channel_nightly() {
        let patterns = Channel::Nightly.default_patterns();
        assert!(matches_channel("v1.0.0-nightly", &patterns));
        assert!(matches_channel("v1.0.0-dev", &patterns));
        assert!(!matches_channel("v1.0.0", &patterns));
        assert!(!matches_channel("v1.0.0-beta", &patterns));
    }

    #[test]
    fn test_matches_channel_stable_exclusion() {
        let patterns = Channel::Stable.default_patterns();
        assert!(matches_channel("v1.0.0", &patterns));
        assert!(!matches_channel("v1.0.0-beta", &patterns));
        assert!(!matches_channel("v1.0.0-nightly", &patterns));
        assert!(!matches_channel("v1.0.0-rc1", &patterns));
    }

    #[test]
    fn test_patterns_with_overrides() {
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
    fn test_filter_releases() {
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
        ];

        let stable = filter_releases(&releases, Channel::Stable, None);
        assert_eq!(stable.len(), 1);
        assert_eq!(stable[0].tag_name, "v1.0.0");

        let beta = filter_releases(&releases, Channel::Beta, None);
        assert_eq!(beta.len(), 1);
        assert_eq!(beta[0].tag_name, "v1.1.0-beta");

        let nightly = filter_releases(&releases, Channel::Nightly, None);
        assert_eq!(nightly.len(), 1);
        assert_eq!(nightly[0].tag_name, "v1.0.1-nightly");
    }
}
