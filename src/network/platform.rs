use thiserror::Error;

use crate::core::constants::{
    SCORE_CHECKSUM_PENALTY, SCORE_KNOWN_EXTENSION, SCORE_LINUX_PARTIAL, SCORE_PLATFORM_MATCH,
    SCORE_SHELL_SCRIPT,
};

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Unsupported architecture: {0}")]
    UnsupportedArch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_64,
    Aarch64,
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::X86_64 => write!(f, "x86_64"),
            Arch::Aarch64 => write!(f, "aarch64"),
        }
    }
}

impl Arch {
    pub fn aliases(&self) -> &[&'static str] {
        match self {
            Arch::X86_64 => &[
                "linux-x86_64",
                "linux-amd64",
                "linux-x64",
                "x86_64-unknown-linux-gnu",
                "x86_64-unknown-linux-musl",
                "x86_64",
                "amd64",
                "x64",
            ],
            Arch::Aarch64 => &[
                "linux-aarch64",
                "linux-arm64",
                "aarch64-unknown-linux-gnu",
                "aarch64-unknown-linux-musl",
                "aarch64",
                "arm64",
            ],
        }
    }
}

pub fn detect_arch() -> Result<Arch, PlatformError> {
    match std::env::consts::ARCH {
        "x86_64" => Ok(Arch::X86_64),
        "aarch64" | "arm64" => Ok(Arch::Aarch64),
        other => Err(PlatformError::UnsupportedArch(other.to_string())),
    }
}

pub fn current_platform() -> Result<Arch, PlatformError> {
    detect_arch()
}

pub fn score_asset(name: &str, arch: Arch) -> i32 {
    let lower = name.to_lowercase();
    let mut score = 0;

    for alias in arch.aliases() {
        if lower.contains(alias) {
            score += SCORE_PLATFORM_MATCH;
            break;
        }
    }

    if score == 0 && lower.contains("linux") {
        score += SCORE_LINUX_PARTIAL;
    }

    if score >= SCORE_LINUX_PARTIAL {
        if lower.ends_with(".tar.gz")
            || lower.ends_with(".tgz")
            || lower.ends_with(".tar.xz")
            || lower.ends_with(".tar.bz2")
            || lower.ends_with(".zip")
            || lower.ends_with(".appimage")
            || lower.ends_with(".deb")
            || lower.ends_with(".rpm")
            || lower.ends_with(".tar")
        {
            score += SCORE_KNOWN_EXTENSION;
        }

        if lower.ends_with(".sh") {
            score += SCORE_SHELL_SCRIPT;
        }
    }

    if crate::core::checksum::is_checksum_asset(&lower) {
        score += SCORE_CHECKSUM_PENALTY;
    }

    score
}

pub fn find_best_asset<'a>(assets: &[&'a str], arch: Arch) -> Option<&'a str> {
    assets
        .iter()
        .map(|&n| (n, score_asset(n, arch)))
        .filter(|(_, s)| *s > 0)
        .max_by_key(|(_, s)| *s)
        .map(|(n, _)| n)
}
