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
