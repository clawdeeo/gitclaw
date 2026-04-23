#[test]
fn test_parse_package_simple() {
    let (owner, repo, version) = gitclaw::github::parse_package("BurntSushi/ripgrep").unwrap();
    assert_eq!(owner, "BurntSushi");
    assert_eq!(repo, "ripgrep");
    assert!(version.is_none());
}

#[test]
fn test_parse_package_with_version() {
    let (owner, repo, version) =
        gitclaw::github::parse_package("BurntSushi/ripgrep@13.0.0").unwrap();
    assert_eq!(owner, "BurntSushi");
    assert_eq!(repo, "ripgrep");
    assert_eq!(version, Some("13.0.0".to_string()));
}

#[test]
fn test_parse_package_invalid_no_owner() {
    assert!(gitclaw::github::parse_package("ripgrep").is_err());
}

#[test]
fn test_parse_package_empty() {
    assert!(gitclaw::github::parse_package("").is_err());
}

#[test]
fn test_parse_package_multiple_slashes() {
    assert!(gitclaw::github::parse_package("user/repo/extra").is_err());
}

#[test]
fn test_parse_package_empty_owner() {
    assert!(gitclaw::github::parse_package("/repo").is_err());
}

#[test]
fn test_parse_package_empty_repo() {
    assert!(gitclaw::github::parse_package("user/").is_err());
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

#[test]
fn test_find_matching_asset_empty_assets() {
    use gitclaw::github::{find_matching_asset, Platform, Release};

    let release = Release {
        tag_name: "v1.0.0".to_string(),
        name: Some("Release".to_string()),
        body: None,
        assets: vec![],
    };

    assert!(find_matching_asset(&release, Platform::LinuxX86_64).is_err());
}

#[test]
fn test_find_matching_asset_only_checksums() {
    use gitclaw::github::{find_matching_asset, Asset, Platform, Release};

    let release = Release {
        tag_name: "v1.0.0".to_string(),
        name: Some("Release".to_string()),
        body: None,
        assets: vec![
            Asset {
                id: 1,
                name: "app.sha256".to_string(),
                browser_download_url: "https://example.com/sha256".to_string(),
                size: 100,
            },
            Asset {
                id: 2,
                name: "checksums.txt".to_string(),
                browser_download_url: "https://example.com/checksums".to_string(),
                size: 200,
            },
        ],
    };

    assert!(find_matching_asset(&release, Platform::LinuxX86_64).is_err());
}

#[test]
fn test_platform_current() {
    let _ = gitclaw::github::Platform::current();
}

#[test]
fn test_github_error_display() {
    use gitclaw::github::GithubError;

    let err = GithubError::ApiError {
        status: 404,
        message: "Not Found".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("404"));
    assert!(msg.contains("Not Found"));
}

#[test]
fn test_github_error_release_not_found() {
    use gitclaw::github::GithubError;

    let err = GithubError::ReleaseNotFound {
        owner: "user".to_string(),
        repo: "repo".to_string(),
        version: "v1.0.0".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("user/repo@v1.0.0"));
}

#[test]
fn test_github_error_no_matching_asset() {
    use gitclaw::github::GithubError;

    let err = GithubError::NoMatchingAsset {
        platform: "linux-x86_64".to_string(),
        release: "v1.0.0".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("linux-x86_64"));
    assert!(msg.contains("v1.0.0"));
}

#[test]
#[ignore = "requires network access"]
fn test_github_client_creation() {
    let client = gitclaw::github::GithubClient::new(None);
    assert!(client.is_ok());
}

#[test]
#[ignore = "requires network access"]
fn test_github_client_with_token() {
    let client = gitclaw::github::GithubClient::new(Some("fake-token".to_string()));
    assert!(client.is_ok());
}
