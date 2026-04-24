use std::fs;
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256, Sha512};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha512,
    Md5,
}

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

pub fn is_checksum_asset(name: &str) -> bool {
    let lower = name.to_lowercase();

    lower.ends_with(".sha256")
        || lower.ends_with(".sha512")
        || lower.ends_with(".sha")
        || lower.ends_with(".sig")
        || lower.ends_with(".asc")
        || lower.ends_with(".md5")
        || lower.ends_with(".checksum")
        || lower.contains("checksum")
        || lower.contains("sha256sum")
        || lower.contains("sha512sum")
}

pub fn find_checksum_file(
    asset_name: &str,
    assets: &[crate::network::github::Asset],
) -> Option<(ChecksumAlgorithm, String)> {
    let suffixes: [(&str, ChecksumAlgorithm); 3] = [
        (".sha256", ChecksumAlgorithm::Sha256),
        (".sha512", ChecksumAlgorithm::Sha512),
        (".md5", ChecksumAlgorithm::Md5),
    ];

    for asset in assets {
        for (suffix, algo) in &suffixes {
            if asset.name == format!("{}{}", asset_name, suffix) {
                return Some((*algo, asset.browser_download_url.clone()));
            }
        }
    }

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

pub fn parse_checksum_file(content: &str, target_filename: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 2 {
            let hash = parts[0];
            let filename = parts[1].trim_start_matches('*');

            if filename == target_filename {
                return Some(hash.to_string());
            }
        }
    }

    None
}
