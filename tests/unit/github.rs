//! Integration tests for the github module
//! Tests the public API from gitclaw::github

#[test]
fn test_parse_package_simple() {
    let result = gitclaw::github::parse_package("BurntSushi/ripgrep");
    assert!(result.is_ok());

    let (owner, repo, version) = result.unwrap();
    assert_eq!(owner, "BurntSushi");
    assert_eq!(repo, "ripgrep");
    assert!(version.is_none());
}

#[test]
fn test_parse_package_with_version() {
    let result = gitclaw::github::parse_package("BurntSushi/ripgrep@13.0.0");
    assert!(result.is_ok());

    let (owner, repo, version) = result.unwrap();
    assert_eq!(owner, "BurntSushi");
    assert_eq!(repo, "ripgrep");
    assert_eq!(version, Some("13.0.0".to_string()));
}

#[test]
fn test_parse_package_invalid_no_owner() {
    let result = gitclaw::github::parse_package("ripgrep");
    assert!(result.is_err());
}

#[test]
fn test_parse_package_empty() {
    let result = gitclaw::github::parse_package("");
    assert!(result.is_err());
}

#[test]
fn test_release_struct_creation() {
    use gitclaw::github::{Asset, Release};

    let release = Release {
        tag_name: "v1.0.0".to_string(),
        name: Some("Version 1.0.0".to_string()),
        body: Some("Release notes".to_string()),
        assets: vec![Asset {
            id: 12345,
            name: "app-linux-x86_64.tar.gz".to_string(),
            browser_download_url: "https://example.com/asset.tar.gz".to_string(),
            size: 1024,
        }],
    };

    assert_eq!(release.tag_name, "v1.0.0");
    assert_eq!(release.assets.len(), 1);
    assert_eq!(release.assets[0].name, "app-linux-x86_64.tar.gz");
}

#[test]
fn test_asset_struct_creation() {
    use gitclaw::github::Asset;

    let asset = Asset {
        id: 12345,
        name: "app-linux-x86_64.tar.gz".to_string(),
        browser_download_url: "https://github.com/user/repo/releases/download/v1.0.0/asset.tar.gz"
            .to_string(),
        size: 1048576,
    };

    assert_eq!(asset.id, 12345);
    assert_eq!(asset.size, 1048576);
    assert!(asset.browser_download_url.contains("github.com"));
}

// Note: These tests require network access and a valid GitHub token
// They're marked with #[ignore] by default

#[test]
#[ignore = "requires network access"]
fn test_github_client_creation() {
    let client = gitclaw::github::GithubClient::new(None);
    assert!(client.is_ok());
}

#[test]
#[ignore = "requires network access"]
fn test_github_client_with_token() {
    // This would require a real token
    let client = gitclaw::github::GithubClient::new(Some("fake-token".to_string()));
    assert!(client.is_ok());
}
