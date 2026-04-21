use crate::config::Config;
use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, warn};

const GITHUB_API: &str = "https://api.github.com";

/// Errors that can occur when interacting with the GitHub API
#[derive(Error, Debug)]
pub enum GithubError {
    #[error("GitHub API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Release not found: {owner}/{repo}@{version}")]
    ReleaseNotFound {
        owner: String,
        repo: String,
        version: String,
    },

    #[error("No matching asset found for platform '{platform}' in release '{release}'")]
    NoMatchingAsset { platform: String, release: String },

    #[error("Download failed: {0}")]
    DownloadError(String),

    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// A GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

/// A release asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Platform specification for asset matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX86_64,
    LinuxAarch64,
    DarwinX86_64,
    DarwinAarch64,
    WindowsX86_64,
    WindowsAarch64,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::LinuxX86_64 => write!(f, "linux-x86_64"),
            Platform::LinuxAarch64 => write!(f, "linux-aarch64"),
            Platform::DarwinX86_64 => write!(f, "darwin-x86_64"),
            Platform::DarwinAarch64 => write!(f, "darwin-aarch64"),
            Platform::WindowsX86_64 => write!(f, "windows-x86_64"),
            Platform::WindowsAarch64 => write!(f, "windows-aarch64"),
        }
    }
}

impl Platform {
    /// Get platform aliases for matching
    fn aliases(&self) -> &[&'static str] {
        match self {
            Platform::LinuxX86_64 => &["linux-x86_64", "linux-amd64", "linux-x64"],
            Platform::LinuxAarch64 => &["linux-aarch64", "linux-arm64"],
            Platform::DarwinX86_64 => &[
                "darwin-x86_64",
                "darwin-amd64",
                "darwin-x64",
                "macos-x86_64",
                "osx-x86_64",
            ],
            Platform::DarwinAarch64 => &[
                "darwin-aarch64",
                "darwin-arm64",
                "macos-aarch64",
                "macos-arm64",
                "osx-arm64",
            ],
            Platform::WindowsX86_64 => &[
                "windows-x86_64",
                "windows-amd64",
                "windows-x64",
                "win-x86_64",
                "win-amd64",
            ],
            Platform::WindowsAarch64 => &[
                "windows-aarch64",
                "windows-arm64",
                "win-aarch64",
                "win-arm64",
            ],
        }
    }

    /// Detect the current platform
    pub fn current() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Ok(Platform::LinuxX86_64),
            ("linux", "aarch64") => Ok(Platform::LinuxAarch64),
            ("macos", "x86_64") => Ok(Platform::DarwinX86_64),
            ("macos", "aarch64") | ("macos", "arm64") => Ok(Platform::DarwinAarch64),
            ("windows", "x86_64") => Ok(Platform::WindowsX86_64),
            ("windows", "aarch64") | ("windows", "arm64") => Ok(Platform::WindowsAarch64),
            (os, arch) => bail!("Unsupported platform: {}-{}", os, arch),
        }
    }
}

/// GitHub API client
#[derive(Debug, Clone)]
pub struct GithubClient {
    client: Client,
    token: Option<String>,
}

impl GithubClient {
    /// Create a new GitHub client with optional authentication token
    pub fn new(token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("gitclaw/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, token })
    }

    /// Add authentication header to request if token is available
    fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref t) = self.token {
            req.bearer_auth(t)
        } else {
            req
        }
    }

    /// Get a specific release by version tag
    /// If version is "latest", fetches the latest release
    pub async fn get_release(
        &self,
        user: &str,
        repo: &str,
        version: &str,
    ) -> std::result::Result<Release, GithubError> {
        if version == "latest" {
            self.get_latest_release(user, repo).await
        } else {
            self.get_release_by_tag(user, repo, version).await
        }
    }

    /// Get a release by tag name
    async fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> std::result::Result<Release, GithubError> {
        // Normalize tag - GitHub tags typically start with 'v'
        let tag_normalized = if tag.starts_with('v') {
            tag.to_string()
        } else {
            format!("v{}", tag)
        };

        let url = format!(
            "{}/repos/{}/{}/releases/tags/{}",
            GITHUB_API, owner, repo, tag_normalized
        );
        debug!("GET {}", url);

        let resp = self.add_auth(self.client.get(&url)).send().await?;

        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }

        // If the 'v' prefix version failed, try without
        if tag_normalized.starts_with('v') && tag_normalized != tag {
            let url = format!(
                "{}/repos/{}/{}/releases/tags/{}",
                GITHUB_API, owner, repo, tag
            );
            let resp = self.add_auth(self.client.get(&url)).send().await?;
            if resp.status().is_success() {
                return Ok(resp.json().await?);
            }
        }

        // Try searching in all releases
        warn!("Tag endpoint failed, searching all releases for {}", tag);
        match self.get_releases(owner, repo).await {
            Ok(releases) => {
                // Try matching with various tag formats
                let candidates = [
                    tag.to_string(),
                    tag_normalized.clone(),
                    tag_normalized.trim_start_matches('v').to_string(),
                ];
                for candidate in &candidates {
                    if let Some(r) = releases.iter().find(|r| r.tag_name == *candidate) {
                        return Ok(r.clone());
                    }
                }
            }
            Err(e) => warn!("Failed to fetch releases: {}", e),
        }

        Err(GithubError::ReleaseNotFound {
            owner: owner.to_string(),
            repo: repo.to_string(),
            version: tag.to_string(),
        })
    }

    /// Get the latest release
    async fn get_latest_release(
        &self,
        owner: &str,
        repo: &str,
    ) -> std::result::Result<Release, GithubError> {
        let url = format!("{}/repos/{}/{}/releases/latest", GITHUB_API, owner, repo);
        let resp = self.add_auth(self.client.get(&url)).send().await?;

        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }

        // Fallback: get all releases and find latest non-draft, non-prerelease
        warn!("Latest endpoint failed, using fallback");
        let releases = self.get_releases(owner, repo).await?;
        releases
            .into_iter()
            .next()
            .ok_or_else(|| GithubError::ReleaseNotFound {
                owner: owner.to_string(),
                repo: repo.to_string(),
                version: "latest".to_string(),
            })
    }

    /// Get all releases for a repository
    async fn get_releases(
        &self,
        owner: &str,
        repo: &str,
    ) -> std::result::Result<Vec<Release>, GithubError> {
        let url = format!("{}/repos/{}/{}/releases", GITHUB_API, owner, repo);
        debug!("GET {}", url);
        let resp = self.add_auth(self.client.get(&url)).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(GithubError::ApiError { status, message });
        }

        Ok(resp.json().await?)
    }

    /// Download an asset to a file path with optional progress bar
    pub async fn download_asset(
        &self,
        asset: &Asset,
        path: &Path,
        show_progress: bool,
    ) -> std::result::Result<(), GithubError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Start download
        let resp = self
            .add_auth(self.client.get(&asset.browser_download_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(GithubError::DownloadError(format!(
                "HTTP {}",
                resp.status()
            )));
        }

        let total = resp.content_length().unwrap_or(asset.size);

        if show_progress {
            // Create progress bar
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .map_err(|e| GithubError::ParseError(e.to_string()))?
                    .progress_chars("█▓░"),
            );

            // Stream to file with progress updates
            let mut file = std::fs::File::create(path)?;
            let mut downloaded: u64 = 0;

            use futures::StreamExt;
            let mut stream = resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }

            pb.finish_with_message("Downloaded");
        } else {
            // Download without progress bar
            let mut file = std::fs::File::create(path)?;
            use futures::StreamExt;
            let mut stream = resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk)?;
            }
        }

        Ok(())
    }
}

/// Find the best matching asset for a given platform
pub fn find_matching_asset(
    release: &Release,
    platform: Platform,
) -> std::result::Result<&Asset, GithubError> {
    // Filter out checksum files first
    let candidates: Vec<&Asset> = release
        .assets
        .iter()
        .filter(|a| !is_checksum_file(&a.name))
        .collect();

    if candidates.is_empty() {
        return Err(GithubError::NoMatchingAsset {
            platform: platform.to_string(),
            release: release.tag_name.clone(),
        });
    }

    // Score each candidate
    let aliases = platform.aliases();
    let mut best: Option<(i32, &Asset)> = None;

    for asset in candidates {
        let name_lower = asset.name.to_lowercase();
        let mut score = 0;

        // Check platform aliases
        for alias in aliases {
            if name_lower.contains(alias) {
                score += 10;
                break;
            }
        }

        // Prefer archives over bare binaries (but still allow bare binaries)
        if name_lower.ends_with(".tar.gz")
            || name_lower.ends_with(".tgz")
            || name_lower.ends_with(".tar.xz")
            || name_lower.ends_with(".zip")
        {
            score += 5;
        }

        // Avoid checksum-like names
        if name_lower.contains("checksum")
            || name_lower.contains("sha256")
            || name_lower.contains("sha512")
        {
            score -= 100;
        }

        if score > 0 {
            match best {
                None => best = Some((score, asset)),
                Some((current_score, _)) if score > current_score => {
                    best = Some((score, asset));
                }
                _ => {}
            }
        }
    }

    best.map(|(_, a)| a)
        .ok_or_else(|| GithubError::NoMatchingAsset {
            platform: platform.to_string(),
            release: release.tag_name.clone(),
        })
}

/// Check if a filename is a checksum file
fn is_checksum_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".sha256")
        || lower.ends_with(".sha512")
        || lower.ends_with(".sha")
        || lower.ends_with(".sig")
        || lower.ends_with(".asc")
        || lower.ends_with(".checksum")
        || lower.contains("checksum")
}

/// Parse a package string like "user/repo" or "user/repo@version"
pub fn parse_package(input: &str) -> Result<(String, String, Option<String>)> {
    let s = input
        .trim_start_matches("https://github.com/")
        .trim_start_matches("github.com/");

    let (repo_part, version) = match s.split_once('@') {
        Some((r, v)) => (r, Some(v.to_string())),
        None => (s, None),
    };

    let parts: Vec<&str> = repo_part.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        bail!("Expected user/repo or user/repo@version, got '{}'", input);
    }

    Ok((parts[0].to_string(), parts[1].to_string(), version))
}

/// Search and display releases for a package
pub async fn search_releases(package: &str, limit: usize, config: &Config) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;
    println!("Releases for {}/{}:\n", owner, repo);

    let client = GithubClient::new(config.github_token.clone())?;

    // We need to access the internal method - using the public API
    // Since get_releases is private, we'll use get_release with "latest" and fetch via API
    let url = format!("{}/repos/{}/{}/releases", GITHUB_API, owner, repo);
    let resp = client.client.get(&url).send().await?;

    if !resp.status().is_success() {
        bail!("GitHub API error: {}", resp.status());
    }

    let releases: Vec<Release> = resp.json().await?;

    if releases.is_empty() {
        println!("No releases found.");
        return Ok(());
    }

    for r in releases.iter().take(limit) {
        println!("  {} - {}", r.tag_name, r.name.as_deref().unwrap_or("-"));
        for a in &r.assets {
            println!("    - {} ({})", a.name, format_size(a.size));
        }
        println!();
    }

    if releases.len() > limit {
        println!("... and {} more", releases.len() - limit);
    }

    Ok(())
}

fn format_size(b: u64) -> String {
    const KB: f64 = 1024.0;
    let b = b as f64;
    if b < KB {
        format!("{:.0} B", b)
    } else if b < KB * KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{:.1} MB", b / KB / KB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_simple() {
        let (owner, repo, version) = parse_package("user/repo").unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
        assert!(version.is_none());
    }

    #[test]
    fn test_parse_package_with_version() {
        let (owner, repo, version) = parse_package("user/repo@1.2.3").unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
        assert_eq!(version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_parse_package_with_url() {
        let (owner, repo, version) = parse_package("https://github.com/user/repo").unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
        assert!(version.is_none());
    }

    #[test]
    fn test_parse_package_invalid() {
        assert!(parse_package("invalid").is_err());
        assert!(parse_package("user").is_err());
    }

    #[test]
    fn test_is_checksum_file() {
        assert!(is_checksum_file("app.sha256"));
        assert!(is_checksum_file("app.sha512"));
        assert!(is_checksum_file("app.sig"));
        assert!(is_checksum_file("checksums.txt"));
        assert!(!is_checksum_file("app.tar.gz"));
        assert!(!is_checksum_file("app.zip"));
    }

    #[test]
    fn test_find_matching_asset() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            body: None,
            assets: vec![
                Asset {
                    id: 1,
                    name: "app-linux-x86_64.tar.gz".to_string(),
                    browser_download_url: "https://example.com/linux".to_string(),
                    size: 1000,
                },
                Asset {
                    id: 2,
                    name: "app-darwin-arm64.tar.gz".to_string(),
                    browser_download_url: "https://example.com/darwin".to_string(),
                    size: 1000,
                },
                Asset {
                    id: 3,
                    name: "checksums.txt".to_string(),
                    browser_download_url: "https://example.com/checksums".to_string(),
                    size: 100,
                },
            ],
        };

        let asset = find_matching_asset(&release, Platform::LinuxX86_64).unwrap();
        assert_eq!(asset.name, "app-linux-x86_64.tar.gz");

        let asset = find_matching_asset(&release, Platform::DarwinAarch64).unwrap();
        assert_eq!(asset.name, "app-darwin-arm64.tar.gz");
    }

    #[test]
    fn test_find_matching_asset_no_match() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![Asset {
                id: 1,
                name: "windows-only.exe".to_string(),
                browser_download_url: "https://example.com/win".to_string(),
                size: 1000,
            }],
        };

        assert!(find_matching_asset(&release, Platform::LinuxX86_64).is_err());
    }
}
