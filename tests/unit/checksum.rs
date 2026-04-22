//! Tests for checksum verification module

use std::io::Write;
use tempfile::TempDir;

/// Test detecting checksum file types
#[test]
fn test_is_checksum_file() {
    assert_eq!(
        gitclaw::checksum::is_checksum_file("app.tar.gz.sha256"),
        Some(gitclaw::checksum::ChecksumAlgorithm::Sha256)
    );
    assert_eq!(
        gitclaw::checksum::is_checksum_file("app.sha512"),
        Some(gitclaw::checksum::ChecksumAlgorithm::Sha512)
    );
    assert_eq!(
        gitclaw::checksum::is_checksum_file("app.md5"),
        Some(gitclaw::checksum::ChecksumAlgorithm::Md5)
    );
    assert_eq!(gitclaw::checksum::is_checksum_file("app.tar.gz"), None);
}

/// Test calculating SHA256 checksum
#[test]
fn test_calculate_checksum_sha256() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"hello world").unwrap();
    drop(file);

    let hash = gitclaw::checksum::calculate_checksum(
        &file_path,
        gitclaw::checksum::ChecksumAlgorithm::Sha256,
    )
    .unwrap();

    // Known SHA256 hash for "hello world"
    assert_eq!(
        hash,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

/// Test parsing checksum file
#[test]
fn test_parse_checksum_file() {
    let content = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9  test.txt\n";
    let result = gitclaw::checksum::parse_checksum_file(content, "test.txt");
    assert_eq!(
        result,
        Some("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string())
    );
}

/// Test parsing checksum file with binary marker
#[test]
fn test_parse_checksum_file_binary_marker() {
    let content = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9 *test.txt\n";
    let result = gitclaw::checksum::parse_checksum_file(content, "test.txt");
    assert_eq!(
        result,
        Some("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string())
    );
}

/// Test parsing checksum file with no match
#[test]
fn test_parse_checksum_file_no_match() {
    let content = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9  other.txt\n";
    let result = gitclaw::checksum::parse_checksum_file(content, "test.txt");
    assert_eq!(result, None);
}

/// Test parsing checksum file with comments
#[test]
fn test_parse_checksum_file_with_comments() {
    let content = "# This is a comment\nb94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9  test.txt\n";
    let result = gitclaw::checksum::parse_checksum_file(content, "test.txt");
    assert_eq!(
        result,
        Some("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".to_string())
    );
}

/// Test verifying file with correct checksum
#[test]
fn test_verify_file_success() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"hello world").unwrap();
    drop(file);

    let result = gitclaw::checksum::verify_file(
        &file_path,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        gitclaw::checksum::ChecksumAlgorithm::Sha256,
    );
    assert!(result.is_ok());
}

/// Test verifying file with wrong checksum
#[test]
fn test_verify_file_failure() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(b"hello world").unwrap();
    drop(file);

    let result = gitclaw::checksum::verify_file(
        &file_path,
        "wronghash",
        gitclaw::checksum::ChecksumAlgorithm::Sha256,
    );
    assert!(result.is_err());
}
