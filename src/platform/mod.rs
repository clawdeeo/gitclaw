use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Unsupported OS: {0}")]
    UnsupportedOS(String),
    #[error("Unsupported arch: {0}")]
    UnsupportedArch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OS { Linux, MacOS, Windows }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch { X86_64, Aarch64 }

impl std::fmt::Display for OS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { OS::Linux => write!(f, "linux"), OS::MacOS => write!(f, "macos"), OS::Windows => write!(f, "windows") }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Arch::X86_64 => write!(f, "x86_64"), Arch::Aarch64 => write!(f, "aarch64") }
    }
}

impl OS {
    pub fn aliases(&self) -> &[&'static str] {
        match self { OS::Linux => &["linux", "Linux", "LINUX"], OS::MacOS => &["darwin", "macos", "osx", "Darwin", "MacOS"], OS::Windows => &["windows", "win", "Windows", "WIN"] }
    }
    pub fn extensions(&self) -> &[&'static str] {
        match self { OS::Linux => &[".tar.gz", ".tar.xz", ".tar.bz2", ".tgz", ".zip"], OS::MacOS => &[".tar.gz", ".tgz", ".zip", ".dmg"], OS::Windows => &[".zip", ".exe", ".msi"] }
    }
}

impl Arch {
    pub fn aliases(&self) -> &[&'static str] {
        match self { Arch::X86_64 => &["x86_64", "amd64", "x64", "X86_64", "AMD64"], Arch::Aarch64 => &["aarch64", "arm64", "ARM64", "AARCH64"] }
    }
}

pub fn detect_os() -> Result<OS, PlatformError> {
    match std::env::consts::OS { "linux" => Ok(OS::Linux), "macos" => Ok(OS::MacOS), "windows" => Ok(OS::Windows), other => Err(PlatformError::UnsupportedOS(other.to_string())) }
}

pub fn detect_arch() -> Result<Arch, PlatformError> {
    match std::env::consts::ARCH { "x86_64" => Ok(Arch::X86_64), "aarch64" | "arm64" => Ok(Arch::Aarch64), other => Err(PlatformError::UnsupportedArch(other.to_string())) }
}

pub fn current_platform() -> Result<(OS, Arch), PlatformError> { Ok((detect_os()?, detect_arch()?)) }

pub fn score_asset(name: &str, os: OS, arch: Arch) -> i32 {
    let lower = name.to_lowercase(); let mut score = 0;
    for alias in os.aliases() { if lower.contains(&alias.to_lowercase()) { score += 10; break; } }
    for alias in arch.aliases() { if lower.contains(&alias.to_lowercase()) { score += 10; break; } }
    for ext in os.extensions() {
        if lower.ends_with(ext) { score += if matches!(os, OS::Linux | OS::MacOS) && ext.starts_with(".tar") { 5 } else if matches!(os, OS::Windows) && *ext == ".zip" { 5 } else { 2 }; break; }
    }
    if lower.contains("checksum") || lower.contains("sha256") || lower.contains(".asc") || lower.contains(".sig") { score -= 50; }
    score
}

pub fn find_best_asset<'a>(assets: &[&'a str], os: OS, arch: Arch) -> Option<&'a str> {
    assets.iter().map(|&n| (n, score_asset(n, os, arch))).filter(|(_, s)| *s > 0).max_by_key(|(_, s)| *s).map(|(n, _)| n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_detect_platform() { let _ = current_platform().unwrap(); }
    #[test] fn test_score_linux_x86_64() {
        assert!(score_asset("tool-linux-x86_64.tar.gz", OS::Linux, Arch::X86_64) > 0);
        assert!(score_asset("tool-linux-amd64.tar.gz", OS::Linux, Arch::X86_64) > 0);
        assert!(score_asset("checksums.txt", OS::Linux, Arch::X86_64) < 0);
    }
    #[test] fn test_find_best_asset() {
        let assets = vec!["app-darwin-arm64.tar.gz", "app-linux-x86_64.tar.gz", "app-windows-x86_64.zip", "checksums.txt"];
        let refs: Vec<&str> = assets.iter().map(|s| *s).collect();
        assert_eq!(find_best_asset(&refs, OS::Linux, Arch::X86_64), Some("app-linux-x86_64.tar.gz"));
        assert_eq!(find_best_asset(&refs, OS::MacOS, Arch::Aarch64), Some("app-darwin-arm64.tar.gz"));
    }
}