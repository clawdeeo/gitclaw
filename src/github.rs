use crate::banner;
use crate::config::Config;
use anyhow::{bail, Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, warn};

const GITHUB_API: &str = "https://api.github.com";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX86_64,
    LinuxAarch64,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::LinuxX86_64 => write!(f, "linux-x86_64"),
            Platform::LinuxAarch64 => write!(f, "linux-aarch64"),
        }
    }
}

impl Platform {
    fn aliases(&self) -> &[&'static str] {
        match self {
            Platform::LinuxX86_64 => &[
                "linux-x86_64",
                "linux-amd64",
                "linux-x64",
                "x86_64-unknown-linux-gnu",
                "x86_64-unknown-linux-musl",
            ],
            Platform::LinuxAarch64 => &[
                "linux-aarch64",
                "linux-arm64",
                "aarch64-unknown-linux-gnu",
                "aarch64-unknown-linux-musl",
            ],
        }
    }

    #[allow(dead_code)]
    fn extensions(&self) -> &[&'static str] {
        &[".tar.gz", ".tar.xz", ".tar.bz2", ".tgz", ".zip"]
    }

    pub fn current() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Platform::LinuxX86_64),
            "aarch64" | "arm64" => Ok(Platform::LinuxAarch64),
            arch => bail!("Unsupported architecture: {}", arch),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GithubClient {
    client: Client,
    token: Option<String>,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("gitclaw/0.1.0")
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client, token })
    }

    fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref t) = self.token {
            req.bearer_auth(t)
        } else {
            req
        }
    }

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

    async fn get_release_by_tag(
        &self,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> std::result::Result<Release, GithubError> {
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

        warn!("Tag endpoint failed, searching all releases for {}", tag);
        match self.get_releases(owner, repo).await {
            Ok(releases) => {
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

    pub async fn download_asset(
        &self,
        asset: &Asset,
        path: &Path,
        show_progress: bool,
    ) -> std::result::Result<(), GithubError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

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
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .map_err(|e| GithubError::ParseError(e.to_string()))?
                    .progress_chars("█▓░"),
            );

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

    pub async fn download_text(&self, url: &str) -> std::result::Result<String, GithubError> {
        let resp = self.add_auth(self.client.get(url)).send().await?;
        if !resp.status().is_success() {
            return Err(GithubError::DownloadError(format!(
                "HTTP {}",
                resp.status()
            )));
        }
        Ok(resp.text().await?)
    }
}

pub fn find_matching_asset(
    release: &Release,
    platform: Platform,
) -> std::result::Result<&Asset, GithubError> {
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

    let aliases = platform.aliases();
    let mut best: Option<(i32, &Asset)> = None;

    for asset in candidates {
        let name_lower = asset.name.to_lowercase();
        let mut score = 0;

        for alias in aliases {
            if name_lower.contains(alias) {
                score += 10;
                break;
            }
        }

        if score == 0 && name_lower.contains("linux") {
            score += 5;
        }

        if score >= 5 {
            if name_lower.ends_with(".tar.gz")
                || name_lower.ends_with(".tgz")
                || name_lower.ends_with(".tar.xz")
                || name_lower.ends_with(".tar.bz2")
                || name_lower.ends_with(".zip")
                || name_lower.ends_with(".appimage")
                || name_lower.ends_with(".deb")
                || name_lower.ends_with(".rpm")
                || name_lower.ends_with(".tar")
            {
                score += 5;
            }
            if name_lower.ends_with(".sh") {
                score += 2;
            }
        }

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

pub async fn search_releases(package: &str, limit: usize, config: &Config) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;
    banner::print_header(&format!("Releases for {}/{}", owner.cyan(), repo.cyan()));

    let client = GithubClient::new(config.github_token.clone())?;

    let url = format!("{}/repos/{}/{}/releases", GITHUB_API, owner, repo);
    let resp = client.client.get(&url).send().await?;

    if !resp.status().is_success() {
        bail!("GitHub API error: {}", resp.status());
    }

    let releases: Vec<Release> = resp.json().await?;

    if releases.is_empty() {
        banner::print_info("No releases found.");
        return Ok(());
    }

    for r in releases.iter().take(limit) {
        println!(
            "{} {}",
            r.tag_name.green().bold(),
            r.name.as_deref().unwrap_or("").dimmed()
        );
        for a in &r.assets {
            println!(
                "  {} {}",
                a.name.dimmed(),
                format!("({})", format_size(a.size)).cyan()
            );
        }
        println!();
    }

    if releases.len() > limit {
        banner::print_info(&format!("... and {} more releases", releases.len() - limit));
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
                    name: "app-linux-aarch64.tar.gz".to_string(),
                    browser_download_url: "https://example.com/aarch64".to_string(),
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

        let asset = find_matching_asset(&release, Platform::LinuxAarch64).unwrap();
        assert_eq!(asset.name, "app-linux-aarch64.tar.gz");
    }

    #[test]
    fn test_find_matching_asset_generic_linux() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![Asset {
                id: 1,
                name: "app-linux.tar.gz".to_string(),
                browser_download_url: "https://example.com/linux".to_string(),
                size: 1000,
            }],
        };

        let asset = find_matching_asset(&release, Platform::LinuxX86_64).unwrap();
        assert_eq!(asset.name, "app-linux.tar.gz");

        let asset = find_matching_asset(&release, Platform::LinuxAarch64).unwrap();
        assert_eq!(asset.name, "app-linux.tar.gz");
    }

    #[test]
    fn test_find_matching_asset_deb_rpm() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![
                Asset {
                    id: 1,
                    name: "app_1.0.0_linux_amd64.deb".to_string(),
                    browser_download_url: "https://example.com/deb".to_string(),
                    size: 1000,
                },
                Asset {
                    id: 2,
                    name: "app-1.0.0-linux-x86_64.rpm".to_string(),
                    browser_download_url: "https://example.com/rpm".to_string(),
                    size: 1000,
                },
            ],
        };

        let asset = find_matching_asset(&release, Platform::LinuxX86_64).unwrap();
        assert!(asset.name.ends_with(".deb") || asset.name.ends_with(".rpm"));
    }

    #[test]
    fn test_find_matching_asset_shell_script() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![Asset {
                id: 1,
                name: "install-linux-x86_64.sh".to_string(),
                browser_download_url: "https://example.com/sh".to_string(),
                size: 1000,
            }],
        };

        let asset = find_matching_asset(&release, Platform::LinuxX86_64).unwrap();
        assert_eq!(asset.name, "install-linux-x86_64.sh");
    }

    #[test]
    fn test_find_matching_asset_prefers_specific_arch() {
        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: None,
            assets: vec![
                Asset {
                    id: 1,
                    name: "app-linux.tar.gz".to_string(),
                    browser_download_url: "https://example.com/generic".to_string(),
                    size: 1000,
                },
                Asset {
                    id: 2,
                    name: "app-linux-x86_64.tar.gz".to_string(),
                    browser_download_url: "https://example.com/specific".to_string(),
                    size: 1000,
                },
            ],
        };

        let asset = find_matching_asset(&release, Platform::LinuxX86_64).unwrap();
        assert_eq!(asset.name, "app-linux-x86_64.tar.gz");

        let asset = find_matching_asset(&release, Platform::LinuxAarch64).unwrap();
        assert_eq!(asset.name, "app-linux.tar.gz");
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
