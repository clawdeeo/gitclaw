use tempfile::TempDir;

use gitclaw::cache;
use gitclaw::config::Config;

fn make_config() -> (Config, TempDir) {
    let dir = TempDir::new().unwrap();
    let config = Config {
        install_dir: dir.path().to_path_buf(),
        ..Config::default()
    };
    (config, dir)
}

#[test]
fn test_cache_key_format() {
    let key = cache::cache_key("BurntSushi", "ripgrep", "13.0.0", "ripgrep.tar.gz");
    assert_eq!(key, "BurntSushi_ripgrep_13.0.0_ripgrep.tar.gz");
}

#[test]
fn test_cache_dir_uses_config() {
    let (config, _dir) = make_config();
    let cache_dir = cache::cache_dir(&config);
    assert!(cache_dir.ends_with("cache"));
    assert!(cache_dir.starts_with(config.install_dir));
}

#[test]
fn test_cache_path_constructs_correctly() {
    let (config, _dir) = make_config();
    let path = cache::cache_path(&config, "test_key");
    assert!(path.ends_with("test_key"));
    assert_eq!(path.parent().unwrap(), cache::cache_dir(&config));
}

#[test]
fn test_file_hash_deterministic() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_file");
    std::fs::write(&file, b"hello world").unwrap();

    let hash1 = cache::file_hash(&file).unwrap();
    let hash2 = cache::file_hash(&file).unwrap();
    assert_eq!(hash1, hash2);
    assert!(!hash1.is_empty());
}

#[test]
fn test_file_hash_different_content() {
    let dir = TempDir::new().unwrap();
    let file_a = dir.path().join("a");
    let file_b = dir.path().join("b");
    std::fs::write(&file_a, b"content a").unwrap();
    std::fs::write(&file_b, b"content b").unwrap();

    let hash_a = cache::file_hash(&file_a).unwrap();
    let hash_b = cache::file_hash(&file_b).unwrap();
    assert_ne!(hash_a, hash_b);
}

#[test]
fn test_file_hash_nonexistent() {
    let result = cache::file_hash(std::path::Path::new("/nonexistent/file"));
    assert!(result.is_err());
}

#[test]
fn test_get_cached_miss() {
    let (config, _dir) = make_config();
    let result = cache::get_cached(&config, "nonexistent_key", None);
    assert!(result.is_none());
}

#[test]
fn test_get_cached_hit() {
    let (config, _dir) = make_config();
    let key = "test_asset.tar.gz";
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    let cached_path = cache_dir.join(key);
    std::fs::write(&cached_path, b"cached content").unwrap();

    let result = cache::get_cached(&config, key, None);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), cached_path);
}

#[test]
fn test_get_cached_hash_mismatch() {
    let (config, _dir) = make_config();
    let key = "test_asset.tar.gz";
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    let cached_path = cache_dir.join(key);
    std::fs::write(&cached_path, b"cached content").unwrap();

    let result = cache::get_cached(&config, key, Some("wrong_hash"));
    assert!(result.is_none());
}

#[test]
fn test_get_cached_hash_match() {
    let (config, _dir) = make_config();
    let key = "test_asset.tar.gz";
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    let cached_path = cache_dir.join(key);
    std::fs::write(&cached_path, b"cached content").unwrap();

    let hash = cache::file_hash(&cached_path).unwrap();
    let result = cache::get_cached(&config, key, Some(&hash));
    assert!(result.is_some());
}

#[test]
fn test_store_creates_cache_dir_and_file() {
    let (config, _dir) = make_config();
    let source_dir = TempDir::new().unwrap();
    let source_file = source_dir.path().join("asset.tar.gz");
    std::fs::write(&source_file, b"downloaded content").unwrap();

    let result = cache::store(&config, "owner_repo_1.0.0_asset.tar.gz", &source_file);
    assert!(result.is_ok());

    let stored = result.unwrap();
    assert!(stored.exists());
    let content = std::fs::read_to_string(&stored).unwrap();
    assert_eq!(content, "downloaded content");
}

#[test]
fn test_store_overwrites_existing() {
    let (config, _dir) = make_config();
    let source_dir = TempDir::new().unwrap();
    let source_file = source_dir.path().join("asset.tar.gz");

    std::fs::write(&source_file, b"version 1").unwrap();
    cache::store(&config, "test_key", &source_file).unwrap();

    std::fs::write(&source_file, b"version 2").unwrap();
    let stored = cache::store(&config, "test_key", &source_file).unwrap();
    let content = std::fs::read_to_string(&stored).unwrap();
    assert_eq!(content, "version 2");
}

#[test]
fn test_clean_removes_files() {
    let (config, _dir) = make_config();
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("file_a"), b"a").unwrap();
    std::fs::write(cache_dir.join("file_b"), b"bb").unwrap();

    let count = cache::clean(&config).unwrap();
    assert_eq!(count, 2);
    assert!(!cache_dir.join("file_a").exists());
    assert!(!cache_dir.join("file_b").exists());
}

#[test]
fn test_clean_empty_dir() {
    let (config, _dir) = make_config();
    let count = cache::clean(&config).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_clean_preserves_subdirs() {
    let (config, _dir) = make_config();
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("file_a"), b"a").unwrap();
    std::fs::create_dir(cache_dir.join("subdir")).unwrap();

    let count = cache::clean(&config).unwrap();
    assert_eq!(count, 1);
    assert!(cache_dir.join("subdir").exists());
    assert!(!cache_dir.join("file_a").exists());
}

#[test]
fn test_size_empty() {
    let (config, _dir) = make_config();
    let size = cache::size(&config).unwrap();
    assert_eq!(size, 0);
}

#[test]
fn test_size_with_files() {
    let (config, _dir) = make_config();
    let cache_dir = cache::cache_dir(&config);
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("small"), b"12345").unwrap();
    std::fs::write(cache_dir.join("large"), b"1234567890").unwrap();

    let size = cache::size(&config).unwrap();
    assert_eq!(size, 15);
}

#[test]
fn test_full_cache_roundtrip() {
    let (config, _dir) = make_config();

    // Simulate download
    let source_dir = TempDir::new().unwrap();
    let source = source_dir.path().join("ripgrep-13.0.0.tar.gz");
    std::fs::write(&source, b"ripgrep binary content").unwrap();

    // Store in cache
    let key = cache::cache_key("BurntSushi", "ripgrep", "13.0.0", "ripgrep-13.0.0.tar.gz");
    let cached = cache::store(&config, &key, &source).unwrap();
    let hash = cache::file_hash(&cached).unwrap();

    // Cache hit with matching hash
    let result = cache::get_cached(&config, &key, Some(&hash));
    assert!(result.is_some());

    // Cache miss with wrong hash
    let result = cache::get_cached(&config, &key, Some("wrong_hash"));
    assert!(result.is_none());

    // Cache hit without hash check
    let result = cache::get_cached(&config, &key, None);
    assert!(result.is_some());

    // Verify size
    let size = cache::size(&config).unwrap();
    assert!(size > 0);

    // Clean and verify
    let removed = cache::clean(&config).unwrap();
    assert_eq!(removed, 1);
    assert_eq!(cache::size(&config).unwrap(), 0);
}