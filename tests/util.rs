#[test]
fn test_format_bytes() {
    assert_eq!(gitclaw::util::format_bytes(0), "0 B");
    assert_eq!(gitclaw::util::format_bytes(512), "512.0 B");
    assert_eq!(gitclaw::util::format_bytes(1024), "1.0 KB");
    assert_eq!(gitclaw::util::format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(gitclaw::util::format_bytes(1024 * 1024 * 1024), "1.0 GB");
}
