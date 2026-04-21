use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256, Sha512};
use std::fs;
use std::io::Read;
use std::path::Path;

/// Supported checksum algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha512,
    Md5,
}

/// Checksum file information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ChecksumFile {
    pub algorithm: ChecksumAlgorithm,
    pub filename: String,
    pub expected_hash: String,
}

/// Detect if a filename is a checksum file
pub fn is_checksum_file(filename: &str) -> Option<ChecksumAlgorithm> {
    let lower = filename.to_lowercase();
    if lower.ends_with(".sha256") || lower.contains(".sha256.") {
        Some(ChecksumAlgorithm::Sha256)
    } else if lower.ends_with(".sha512") || lower.contains(".sha512.") {
        Some(ChecksumAlgorithm::Sha512)
    } else if lower.ends_with(".md5") || lower.contains(".md5.") {
        Some(ChecksumAlgorithm::Md5)
    } else {
        None
    }
}

/// Find checksum file for a given asset in release assets
pub fn find_checksum_file(
    asset_name: &str,
    assets: &[crate::github::Asset],
) -> Option<(ChecksumAlgorithm, String)> {
    // Try exact match patterns first
    let patterns = vec![
        format!("{}.sha256", asset_name),
        format!("{}.sha512", asset_name),
        format!("{}.md5", asset_name),
    ];

    for asset in assets {
        for pattern in &patterns {
            if asset.name == *pattern {
                let algo = is_checksum_file(&asset.name)?;
                return Some((algo, asset.browser_download_url.clone()));
            }
        }
    }

    // Try generic checksum files (checksums.txt, SHA256SUMS, etc.)
    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if name_lower.contains("checksum") || name_lower.contains("sha256sum") {
            return Some((
                ChecksumAlgorithm::Sha256,
                asset.browser_download_url.clone(),
            ));
        }
    }

    None
}

/// Verify file against expected checksum
pub fn verify_file(file_path: &Path, expected: &str, algo: ChecksumAlgorithm) -> Result<()> {
    let calculated = calculate_checksum(file_path, algo)?;
    let expected_clean = expected.trim().to_lowercase();
    let calculated_clean = calculated.to_lowercase();

    if expected_clean != calculated_clean {
        bail!(
            "Checksum mismatch:\n  Expected:   {}\n  Calculated: {}",
            expected_clean,
            calculated_clean
        );
    }

    Ok(())
}

/// Calculate checksum of a file
pub fn calculate_checksum(file_path: &Path, algo: ChecksumAlgorithm) -> Result<String> {
    let mut file = fs::File::open(file_path).context("Failed to open file for checksum")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .context("Failed to read file")?;

    let hash = match algo {
        ChecksumAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        ChecksumAlgorithm::Md5 => {
            let hash = md5::compute(&buffer);
            format!("{:x}", hash)
        }
    };

    Ok(hash)
}

/// Parse checksum from checksum file content
pub fn parse_checksum_file(content: &str, target_filename: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle "HASH  filename" format (sha256sum style)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let hash = parts[0];
            let filename = parts[1].trim_start_matches('*'); // Remove binary marker
            if filename == target_filename {
                return Some(hash.to_string());
            }
        }
    }
    None
}
