use std::io::Write;
use std::path::Path;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::core::config::Config;
use crate::core::constants::GITHUB_API_BASE;
use crate::output;

const GITHUB_API: &str = GITHUB_API_BASE;

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
            arch => bail!("Unsupported architecture: {}.", arch),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GithubClient {
    pub(crate) client: Client,
    token: Option<String>,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent(concat!("gitclaw/", env!("CARGO_PKG_VERSION")))
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

    pub async fn get_releases(
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
        bail!("Expected user/repo or user/repo@version, got '{}'.", input);
    }

    Ok((parts[0].to_string(), parts[1].to_string(), version))
}

pub async fn search_releases(package: &str, limit: usize, config: &Config) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;

    let client = GithubClient::new(config.github_token.clone())?;

    let per_page = limit.min(100);
    let url = format!(
        "{}/repos/{}/{}/releases?per_page={}",
        GITHUB_API, owner, repo, per_page
    );
    let resp = client.add_auth(client.client.get(&url)).send().await?;

    if !resp.status().is_success() {
        bail!("GitHub API error: {}.", resp.status());
    }

    let has_next = resp
        .headers()
        .get("link")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("rel=\"next\""))
        .unwrap_or(false);

    let releases: Vec<Release> = resp.json().await?;

    if releases.is_empty() {
        output::print_info("No releases found.");
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "{:<20} {:<42} {:<8} {}",
            "Tag", "Name", "Assets", "Total Size"
        )
        .bold()
    );

    for r in releases.iter().take(limit) {
        let name = r.name.as_deref().unwrap_or("").to_string();
        let name_display = if name.len() > 40 {
            format!("{}...", &name[..37])
        } else {
            name
        };

        let asset_count = r.assets.len();
        let total_size: u64 = r.assets.iter().map(|a| a.size).sum();

        println!(
            "{:<20} {:<42} {:<8} {}",
            r.tag_name.green().bold(),
            name_display.dimmed(),
            asset_count.to_string().cyan(),
            format_size(total_size).cyan()
        );
    }

    println!();

    if has_next {
        output::print_info(&format!(
            "{} releases shown but more exist. Use --limit to show more.",
            releases.len()
        ));
    } else {
        output::print_info(&format!("{} release(s) found.", releases.len()));
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
