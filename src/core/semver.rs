use anyhow::{bail, Result};
use semver::{Version, VersionReq};

pub enum VersionConstraint {
    Exact(Version),
    Range(VersionReq),
}

impl VersionConstraint {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();

        if trimmed.starts_with('^')
            || trimmed.starts_with('~')
            || trimmed.starts_with('>')
            || trimmed.starts_with('<')
            || trimmed.starts_with('=')
        {
            let req = VersionReq::parse(trimmed)
                .map_err(|e| anyhow::anyhow!("Invalid semver range '{}': {}.", trimmed, e))?;
            return Ok(VersionConstraint::Range(req));
        }

        if let Ok(v) = Version::parse(trimmed) {
            return Ok(VersionConstraint::Exact(v));
        }

        if let Ok(req) = VersionReq::parse(trimmed) {
            return Ok(VersionConstraint::Range(req));
        }

        bail!("Cannot parse '{}' as a version or semver range.", trimmed)
    }

    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Exact(v) => v == version,
            VersionConstraint::Range(req) => req.matches(version),
        }
    }
}

pub fn strip_v_prefix(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

pub fn parse_tag_version(tag: &str) -> Result<Version> {
    let raw = strip_v_prefix(tag);
    Version::parse(raw)
        .map_err(|e| anyhow::anyhow!("Cannot parse version from tag '{}': {}.", tag, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_version() {
        let c = VersionConstraint::parse("1.2.3").unwrap();
        assert!(c.matches(&Version::parse("1.2.3").unwrap()));
        assert!(!c.matches(&Version::parse("1.2.4").unwrap()));
    }

    #[test]
    fn test_caret_range() {
        let c = VersionConstraint::parse("^1.2.3").unwrap();
        assert!(c.matches(&Version::parse("1.2.3").unwrap()));
        assert!(c.matches(&Version::parse("1.2.9").unwrap()));
        assert!(c.matches(&Version::parse("1.3.0").unwrap()));
        assert!(!c.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_tilde_range() {
        let c = VersionConstraint::parse("~1.2.3").unwrap();
        assert!(c.matches(&Version::parse("1.2.3").unwrap()));
        assert!(c.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!c.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_gte_range() {
        let c = VersionConstraint::parse(">=1.0.0").unwrap();
        assert!(c.matches(&Version::parse("1.0.0").unwrap()));
        assert!(c.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!c.matches(&Version::parse("0.9.0").unwrap()));
    }

    #[test]
    fn test_strip_v() {
        assert_eq!(strip_v_prefix("v1.2.3"), "1.2.3");
        assert_eq!(strip_v_prefix("1.2.3"), "1.2.3");
    }

    #[test]
    fn test_parse_tag() {
        let v = parse_tag_version("v1.2.3").unwrap();
        assert_eq!(v, Version::parse("1.2.3").unwrap());

        let v = parse_tag_version("1.2.3").unwrap();
        assert_eq!(v, Version::parse("1.2.3").unwrap());
    }

    #[test]
    fn test_invalid_constraint() {
        assert!(VersionConstraint::parse("not-a-version").is_err());
    }
}
