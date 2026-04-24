use thiserror::Error;

use crate::core::constants::{
    ARCH_AARCH64, ARCH_AMD64, ARCH_ARM64, ARCH_X64, ARCH_X86_64, EXT_APPIMAGE, EXT_DEB, EXT_RPM,
    EXT_SH, EXT_TAR, EXT_TAR_BZ2, EXT_TAR_GZ, EXT_TAR_XZ, EXT_TGZ, EXT_ZIP, PLATFORM_AARCH64_GNU,
    PLATFORM_AARCH64_MUSL, PLATFORM_LINUX, PLATFORM_LINUX_AARCH64, PLATFORM_LINUX_AMD64,
    PLATFORM_LINUX_ARM64, PLATFORM_LINUX_X64, PLATFORM_LINUX_X86_64, PLATFORM_X86_64_GNU,
    PLATFORM_X86_64_MUSL, SCORE_CHECKSUM_PENALTY, SCORE_KNOWN_EXTENSION, SCORE_LINUX_PARTIAL,
    SCORE_PLATFORM_MATCH, SCORE_SHELL_SCRIPT,
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
            Arch::X86_64 => write!(f, "{}", ARCH_X86_64),
            Arch::Aarch64 => write!(f, "{}", ARCH_AARCH64),
        }
    }
}

impl Arch {
    pub fn aliases(&self) -> &[&'static str] {
        match self {
            Arch::X86_64 => &[
                PLATFORM_LINUX_X86_64,
                PLATFORM_LINUX_AMD64,
                PLATFORM_LINUX_X64,
                PLATFORM_X86_64_GNU,
                PLATFORM_X86_64_MUSL,
                ARCH_X86_64,
                ARCH_AMD64,
                ARCH_X64,
            ],
            Arch::Aarch64 => &[
                PLATFORM_LINUX_AARCH64,
                PLATFORM_LINUX_ARM64,
                PLATFORM_AARCH64_GNU,
                PLATFORM_AARCH64_MUSL,
                ARCH_AARCH64,
                ARCH_ARM64,
            ],
        }
    }
}

pub fn detect_arch() -> Result<Arch, PlatformError> {
    match std::env::consts::ARCH {
        ARCH_X86_64 => Ok(Arch::X86_64),
        ARCH_AARCH64 | ARCH_ARM64 => Ok(Arch::Aarch64),
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

    if score == 0 && lower.contains(PLATFORM_LINUX) {
        score += SCORE_LINUX_PARTIAL;
    }

    if score >= SCORE_LINUX_PARTIAL {
        if lower.ends_with(EXT_TAR_GZ)
            || lower.ends_with(EXT_TGZ)
            || lower.ends_with(EXT_TAR_XZ)
            || lower.ends_with(EXT_TAR_BZ2)
            || lower.ends_with(EXT_ZIP)
            || lower.ends_with(EXT_APPIMAGE)
            || lower.ends_with(EXT_DEB)
            || lower.ends_with(EXT_RPM)
            || lower.ends_with(EXT_TAR)
        {
            score += SCORE_KNOWN_EXTENSION;
        }

        if lower.ends_with(EXT_SH) {
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
