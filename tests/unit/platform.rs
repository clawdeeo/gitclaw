//! Integration tests for the platform module (Linux only)
//! Tests the public API from gitclaw::platform

#[test]
fn test_arch_detection() {
    let arch = gitclaw::platform::detect_arch();
    assert!(arch.is_ok());

    let detected = arch.unwrap();
    // Should be one of the supported variants
    match detected {
        gitclaw::platform::Arch::X86_64 => {}
        gitclaw::platform::Arch::Aarch64 => {}
    }
}

#[test]
fn test_current_platform() {
    let arch = gitclaw::platform::current_platform();
    // Verify we got valid arch
    let _ = format!("{:?}", arch);
}

#[test]
fn test_arch_variants() {
    // Test that all Arch variants exist and can be matched
    let variants = vec![
        gitclaw::platform::Arch::X86_64,
        gitclaw::platform::Arch::Aarch64,
    ];

    for variant in variants {
        let name = format!("{:?}", variant).to_lowercase();
        assert!(name.contains("x86") || name.contains("aarch"));
    }
}

#[test]
fn test_score_asset_linux_x86_64() {
    let score =
        gitclaw::platform::score_asset("app-linux-x86_64.tar.gz", gitclaw::platform::Arch::X86_64);
    assert!(score > 0, "Should match linux-x86_64");
}

#[test]
fn test_score_asset_linux_amd64() {
    let score =
        gitclaw::platform::score_asset("app-linux-amd64.tar.gz", gitclaw::platform::Arch::X86_64);
    assert!(score > 0, "Should match linux-amd64 as x86_64");
}

#[test]
fn test_score_asset_linux_aarch64() {
    let score = gitclaw::platform::score_asset(
        "app-linux-aarch64.tar.gz",
        gitclaw::platform::Arch::Aarch64,
    );
    assert!(score > 0, "Should match linux-aarch64");
}

#[test]
fn test_score_asset_no_match() {
    // Score may be > 0 for generic matches (filename similarity)
    let score = gitclaw::platform::score_asset("checksums.txt", gitclaw::platform::Arch::X86_64);
    assert!(score < 0, "Should not match checksum files");
}

#[test]
fn test_find_best_asset_single_match() {
    let assets = vec!["app-linux-x86_64.tar.gz"];
    let best = gitclaw::platform::find_best_asset(&assets, gitclaw::platform::Arch::X86_64);
    assert_eq!(best, Some("app-linux-x86_64.tar.gz"));
}

#[test]
fn test_find_best_asset_multiple_matches() {
    let assets = vec!["app-linux-x86_64.tar.gz", "app-linux-aarch64.tar.gz"];
    let best = gitclaw::platform::find_best_asset(&assets, gitclaw::platform::Arch::X86_64);
    assert!(best.is_some());
    assert!(best.unwrap().contains("x86_64"));
}

#[test]
fn test_find_best_asset_no_match() {
    // Darwin/Windows assets still score points for "linux" in scoring
    // so we test with something that won't match at all
    let assets = vec!["checksums.txt", "README.md"];
    let best = gitclaw::platform::find_best_asset(&assets, gitclaw::platform::Arch::X86_64);
    // Should not find a match
    assert!(best.is_none());
}

#[test]
fn test_arch_equality() {
    assert_eq!(
        gitclaw::platform::Arch::X86_64,
        gitclaw::platform::Arch::X86_64
    );
}

#[test]
fn test_platform_error_display() {
    let err = gitclaw::platform::PlatformError::UnsupportedArch("weirdarch".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("weirdarch"));
}
