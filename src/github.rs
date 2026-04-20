use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub prerelease: bool,
    pub draft: bool,
    pub published_at: Option<String>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    pub content_type: String,
}

impl GitHubClient {
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

    pub async fn get_releases(&self, owner: &str, repo: &str) -> Result<Vec<Release>> {
        let url = format!("{}/repos/{}/{}/releases", GITHUB_API, owner, repo);
        debug!("GET {}", url);
        let resp = self.add_auth(self.client.get(&url)).send().await?;
        if !resp.status().is_success() {
            bail!("GitHub API error ({}): {}", resp.status(), resp.text().await.unwrap_or_default());
        }
        Ok(resp.json().await?)
    }

    pub async fn get_release_by_tag(&self, owner: &str, repo: &str, tag: &str) -> Result<Release> {
        let url = format!("{}/repos/{}/{}/releases/tags/{}", GITHUB_API, owner, repo, tag);
        let resp = self.add_auth(self.client.get(&url)).send().await?;
        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }
        warn!("Tag endpoint failed, searching all releases for {}", tag);
        let releases = self.get_releases(owner, repo).await?;
        let tags = [tag.to_string(), format!("v{}", tag), tag.trim_start_matches('v').to_string()];
        for t in &tags {
            if let Some(r) = releases.iter().find(|r| &r.tag_name == t) {
                return Ok(r.clone());
            }
        }
        bail!("Release {} not found for {}/{}", tag, owner, repo)
    }

    pub async fn get_latest_release(&self, owner: &str, repo: &str) -> Result<Release> {
        let url = format!("{}/repos/{}/{}/releases/latest", GITHUB_API, owner, repo);
        let resp = self.add_auth(self.client.get(&url)).send().await?;
        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }
        warn!("Latest endpoint failed, using fallback");
        let releases = self.get_releases(owner, repo).await?;
        releases.into_iter()
            .find(|r| !r.draft && !r.prerelease)
            .ok_or_else(|| anyhow!("No releases found for {}/{}", owner, repo))
    }

    pub async fn download_asset(&self, asset: &Asset, on_progress: impl Fn(u64, u64)) -> Result<Vec<u8>> {
        let resp = self.add_auth(self.client.get(&asset.browser_download_url)).send().await?;
        if !resp.status().is_success() {
            bail!("Download failed: HTTP {}", resp.status());
        }
        let total = resp.content_length().unwrap_or(asset.size);
        let mut downloaded: u64 = 0;
        let mut buf = Vec::with_capacity(total as usize);
        use futures::StreamExt;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total);
        }
        Ok(buf)
    }
}

pub fn parse_package(input: &str) -> Result<(String, String, Option<String>)> {
    let s = input.trim_start_matches("https://github.com/").trim_start_matches("github.com/");
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

pub async fn search_releases(package: &str, limit: usize) -> Result<()> {
    let (owner, repo, _) = parse_package(package)?;
    println!("Releases for {}/{}:\n", owner, repo);
    let client = GitHubClient::new(None)?;
    let releases = client.get_releases(&owner, &repo).await?;
    if releases.is_empty() {
        println!("No releases found.");
        return Ok(());
    }
    for r in releases.iter().take(limit) {
        let badge = if r.prerelease { " [pre-release]" } else if r.draft { " [draft]" } else { "" };
        println!("  {}{} - {}", r.tag_name, badge, r.name.as_deref().unwrap_or("-"));
        println!("    Published: {}", r.published_at.as_deref().unwrap_or("unknown"));
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
    if b < KB { format!("{:.0} B", b) }
    else if b < KB * KB { format!("{:.1} KB", b / KB) }
    else { format!("{:.1} MB", b / KB / KB) }
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
    fn test_parse_package_invalid() {
        assert!(parse_package("invalid").is_err());
    }
}
