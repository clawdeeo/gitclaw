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
