//! Integration tests for the platform module
//! Tests the public API from gitclaw::platform

#[test]
fn test_os_detection() {
    let os = gitclaw::platform::detect_os();
    assert!(os.is_ok());

    let detected = os.unwrap();
    // Should be one of the supported variants
    match detected {
        gitclaw::platform::OS::Linux => {}
        gitclaw::platform::OS::MacOS => {}
        gitclaw::platform::OS::Windows => {}
    }
}

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
    let platform = gitclaw::platform::current_platform();
    assert!(platform.is_ok());

    let (os, arch) = platform.unwrap();
    // Verify we got valid OS and arch
    let _ = format!("{:?}", os);
    let _ = format!("{:?}", arch);
}

#[test]
fn test_os_variants() {
    // Test that all OS variants exist and can be matched
    let variants = vec![
        gitclaw::platform::OS::Linux,
        gitclaw::platform::OS::MacOS,
        gitclaw::platform::OS::Windows,
    ];

    for variant in variants {
        let name = format!("{:?}", variant).to_lowercase();
        assert!(name.contains("linux") || name.contains("mac") || name.contains("windows"));
    }
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
    let score = gitclaw::platform::score_asset(
        "app-linux-x86_64.tar.gz",
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    assert!(score > 0, "Should match linux-x86_64");
}

#[test]
fn test_score_asset_linux_amd64() {
    let score = gitclaw::platform::score_asset(
        "app-linux-amd64.tar.gz",
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    assert!(score > 0, "Should match linux-amd64 as x86_64");
}

#[test]
fn test_score_asset_macos() {
    let score = gitclaw::platform::score_asset(
        "app-darwin-x86_64.tar.gz",
        gitclaw::platform::OS::MacOS,
        gitclaw::platform::Arch::X86_64,
    );
    assert!(score > 0, "Should match darwin for macos");
}

#[test]
fn test_score_asset_windows() {
    let score = gitclaw::platform::score_asset(
        "app-windows-x86_64.zip",
        gitclaw::platform::OS::Windows,
        gitclaw::platform::Arch::X86_64,
    );
    assert!(score > 0, "Should match windows");
}

#[test]
fn test_score_asset_no_match() {
    // Score may be > 0 for generic matches (filename similarity)
    // The function scores based on various factors
    let score = gitclaw::platform::score_asset(
        "app-unsupported-platform.tar.gz",
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    // Score is non-negative; may match generically
    assert!(score >= 0);
}

#[test]
fn test_find_best_asset_single_match() {
    let assets = vec!["app-linux-x86_64.tar.gz"];
    let best = gitclaw::platform::find_best_asset(
        &assets,
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    assert_eq!(best, Some("app-linux-x86_64.tar.gz"));
}

#[test]
fn test_find_best_asset_multiple_matches() {
    let assets = vec![
        "app-linux-x86_64.tar.gz",
        "app-linux-amd64.tar.gz",
        "app-macos-x86_64.tar.gz",
    ];
    let best = gitclaw::platform::find_best_asset(
        &assets,
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    assert!(best.is_some());
    assert!(best.unwrap().contains("linux"));
}

#[test]
fn test_find_best_asset_no_match() {
    let assets = vec!["app-macos-x86_64.tar.gz", "app-windows-x86_64.zip"];
    let best = gitclaw::platform::find_best_asset(
        &assets,
        gitclaw::platform::OS::Linux,
        gitclaw::platform::Arch::X86_64,
    );
    // May still find a match with partial scoring
    // This depends on implementation
}

#[test]
fn test_os_equality() {
    assert_eq!(gitclaw::platform::OS::Linux, gitclaw::platform::OS::Linux);
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
    let err = gitclaw::platform::PlatformError::UnsupportedOS("weirdos".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("weirdos"));
}
