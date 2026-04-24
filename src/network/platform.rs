use thiserror::Error;

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
            Arch::X86_64 => &["x86_64", "amd64", "x64"],
            Arch::Aarch64 => &["aarch64", "arm64"],
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

pub fn current_platform() -> Arch {
    detect_arch().expect("Linux x86_64 or aarch64 required")
}

pub fn score_asset(name: &str, arch: Arch) -> i32 {
    let lower = name.to_lowercase();
    let mut score = 0;

    if lower.contains("linux") {
        score += 10;
    }

    for alias in arch.aliases() {
        if lower.contains(alias) {
            score += 10;
            break;
        }
    }

    if lower.ends_with(".tar.gz") || lower.ends_with(".tar.xz") || lower.ends_with(".tgz") {
        score += 5;
    }

    if lower.contains("checksum")
        || lower.contains("sha256")
        || lower.contains(".asc")
        || lower.contains(".sig")
        || lower.contains(".sha")
    {
        score -= 50;
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
