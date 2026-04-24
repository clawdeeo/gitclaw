use tempfile::TempDir;

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
fn test_alias_add_and_resolve() {
    let (config, _dir) = make_config();

    let mut aliases = gitclaw::alias::AliasMap::default();
    aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
    aliases.save(&config).unwrap();

    let loaded = gitclaw::alias::AliasMap::load(&config).unwrap();
    assert_eq!(loaded.resolve("rg"), Some("BurntSushi/ripgrep"));
    assert_eq!(loaded.resolve("fd"), None);
}

#[test]
fn test_alias_add_multiple() {
    let (config, _dir) = make_config();

    let mut aliases = gitclaw::alias::AliasMap::default();
    aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
    aliases.add("fd", "sharkdp/fd", &config).unwrap();
    aliases.add("bat", "sharkdp/bat", &config).unwrap();
    aliases.save(&config).unwrap();

    let loaded = gitclaw::alias::AliasMap::load(&config).unwrap();
    assert_eq!(loaded.resolve("rg"), Some("BurntSushi/ripgrep"));
    assert_eq!(loaded.resolve("fd"), Some("sharkdp/fd"));
    assert_eq!(loaded.resolve("bat"), Some("sharkdp/bat"));
}

#[test]
fn test_alias_remove() {
    let (config, _dir) = make_config();

    let mut aliases = gitclaw::alias::AliasMap::default();
    aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
    aliases.save(&config).unwrap();

    let mut loaded = gitclaw::alias::AliasMap::load(&config).unwrap();
    assert!(loaded.remove("rg"));
    assert!(!loaded.remove("nonexistent"));
    loaded.save(&config).unwrap();

    let reloaded = gitclaw::alias::AliasMap::load(&config).unwrap();
    assert_eq!(reloaded.resolve("rg"), None);
}

#[test]
fn test_alias_slash_rejected() {
    let (config, _dir) = make_config();
    let mut aliases = gitclaw::alias::AliasMap::default();
    assert!(aliases
        .add("owner/repo", "BurntSushi/ripgrep", &config)
        .is_err());
}

#[test]
fn test_alias_resolve_missing() {
    let aliases = gitclaw::alias::AliasMap::default();
    assert_eq!(aliases.resolve("nonexistent"), None);
}

#[test]
fn test_alias_list_sorted() {
    let (config, _dir) = make_config();
    let mut aliases = gitclaw::alias::AliasMap::default();
    aliases.add("fd", "sharkdp/fd", &config).unwrap();
    aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
    aliases.add("bat", "sharkdp/bat", &config).unwrap();

    let list = aliases.list();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].0.as_str(), "bat");
    assert_eq!(list[1].0.as_str(), "fd");
    assert_eq!(list[2].0.as_str(), "rg");
}

#[test]
fn test_alias_roundtrip() {
    let (config, _dir) = make_config();

    let mut aliases = gitclaw::alias::AliasMap::default();
    aliases.add("rg", "BurntSushi/ripgrep", &config).unwrap();
    aliases.add("fd", "sharkdp/fd", &config).unwrap();
    aliases.save(&config).unwrap();

    let loaded = gitclaw::alias::AliasMap::load(&config).unwrap();
    assert_eq!(loaded.resolve("rg"), Some("BurntSushi/ripgrep"));
    assert_eq!(loaded.resolve("fd"), Some("sharkdp/fd"));
}
