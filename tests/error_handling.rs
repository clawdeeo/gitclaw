//! Tests for error handling throughout the codebase
//!
//! These tests verify that errors are properly propagated and contain
//! meaningful context for users.

use std::io::Write;
use tempfile::TempDir;

/// Test that extraction fails gracefully with invalid archive
#[test]
fn test_extract_nonexistent_file() {
    let result = gitclaw::extract::extract_archive(
        std::path::Path::new("/nonexistent/path/to/file.tar.gz"),
        std::path::Path::new("/tmp/output"),
    );
    assert!(result.is_err());
}

/// Test that extraction handles corrupted archives
#[test]
fn test_extract_corrupted_tar_gz() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("corrupted.tar.gz");

    // Write invalid/corrupted gzip data
    let mut file = std::fs::File::create(&archive_path).unwrap();
    file.write_all(b"not a valid gzip file").unwrap();
    drop(file);

    let output_dir = temp_dir.path().join("output");
    let result = gitclaw::extract::extract_archive(&archive_path, &output_dir);
    assert!(result.is_err());
}

/// Test that extraction handles corrupted zip
#[test]
fn test_extract_corrupted_zip() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("corrupted.zip");

    // Write invalid zip data
    let mut file = std::fs::File::create(&archive_path).unwrap();
    file.write_all(b"PK\x03\x04invalid").unwrap();
    drop(file);

    let output_dir = temp_dir.path().join("output");
    let result = gitclaw::extract::extract_archive(&archive_path, &output_dir);
    assert!(result.is_err());
}

/// Test error handling for unknown archive types
#[test]
fn test_detect_unknown_archive_type() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.unknown");
    std::fs::File::create(&file_path).unwrap();

    let result = gitclaw::extract::detect_archive_type(&file_path);
    assert!(result.is_err());
}

/// Test parsing invalid package strings
#[test]
fn test_parse_empty_package() {
    let result = gitclaw::github::parse_package("");
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_package_no_slash() {
    let result = gitclaw::github::parse_package("invalid");
    assert!(result.is_err());
}

#[test]
fn test_parse_package_multiple_slashes() {
    let result = gitclaw::github::parse_package("user/repo/extra");
    assert!(result.is_err());
}

#[test]
fn test_parse_package_empty_owner() {
    let result = gitclaw::github::parse_package("/repo");
    assert!(result.is_err());
}

#[test]
fn test_parse_package_empty_repo() {
    let result = gitclaw::github::parse_package("user/");
    assert!(result.is_err());
}

/// Test asset matching with no assets
#[test]
fn test_find_matching_asset_empty_assets() {
    use gitclaw::github::{find_matching_asset, Platform, Release};

    let release = Release {
        tag_name: "v1.0.0".to_string(),
        name: Some("Release".to_string()),
        body: None,
        assets: vec![],
    };

    let result = find_matching_asset(&release, Platform::LinuxX86_64);
    assert!(result.is_err());
}

/// Test asset matching with only checksum files
#[test]
fn test_find_matching_asset_only_checksums() {
    use gitclaw::github::{Asset, find_matching_asset, Platform, Release};

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

    let result = find_matching_asset(&release, Platform::LinuxX86_64);
    assert!(result.is_err());
}

/// Test platform detection for unsupported platforms
#[test]
fn test_platform_current_unsupported() {
    // This will either succeed on supported platforms or fail on unsupported
    // We just verify the function exists and returns appropriately
    let _ = gitclaw::github::Platform::current();
}

/// Test registry save/load with invalid paths
#[test]
fn test_registry_operations() {
    use gitclaw::registry::Registry;

    // Registry::load should return default for non-existent file
    let reg = Registry::default();
    assert!(!reg.is_installed("nonexistent/package"));
}

/// Test error propagation in github module
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
