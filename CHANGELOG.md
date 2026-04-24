# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0] - 2026-04-23

### Added
- Install channel persisted in registry: `--channel` stored per package
- `update` respects stored channel when checking for newer releases
- Optional `channel` field on `InstalledPackage` (backward compatible)
- New tests in `tests/channel_persist.rs`

## [0.6.0] - 2026-04-23

### Added
- Release channels: `install --channel nightly|beta|stable`
- `search --channel <name>` filters releases by channel
- Channel pattern overrides in `.gitclaw.toml` under `[channels]`
- `export` outputs installed packages as TOML to stdout
- `export -o deps.toml` writes to file
- `import deps.toml` installs all packages from a TOML file
- Import skips already-installed packages unless `--force` is set
- Export output is deterministic: sorted by owner then repo

## [0.5.0] - 2026-04-23

### Added
- Asset caching: downloaded archives cached to `~/.gitclaw/cache/`, reused on subsequent installs
- `cache clean` and `cache size` commands
- `list --outdated` compares installed versions against latest GitHub releases
- Local installs: `install --local user/repo` installs to `./.gitclaw/`
- `uninstall --local` for local project directory

## [0.4.0] - 2026-04-23

### Added
- Semver range support: `install user/repo "^1.2.3"`
- Lockfile: `lock` generates `gitclaw.lock`, `install --locked` reproduces exact versions
- Package aliases: `alias add rg BurntSushi/ripgrep` then `install rg`
- `alias list` and `alias remove` commands

## [0.3.2] - 2026-04-23

### Added
- `.deb` archive extraction with ar format parsing
- `zstd` decompression for `.tar.zst` archives
- GNU and BSD ar header size field support
- `gcw` short alias binary
- 74 new integration tests

### Changed
- `identifier` field set to repo name at install time
- `uninstall` accepts short name (repo or identifier) in addition to `owner/repo`
- `list` shows an `Identifier` column
- `#[serde(default)]` applied to `identifier` field for backward compatibility

### Fixed
- Platform asset matching now correctly selects platform-specific assets
- Rust target triple aliases added for ripgrep and similar projects
- Symlinks in `bin/` now use absolute paths

## [0.3.0] - 2026-04-21

### Added
- Self-update command (`self-update`, `self-update --check`)
- Checksum verification for downloaded assets (`--verify` flag)
- Parallel install support (`install pkg1 pkg2 pkg3`)

## [0.2.0] - 2026-04-21

### Added
- Configuration file support with multiple config sources
- `--dry-run` flag for install command
- `completions` command for shell completions
- Config options: `install_dir`, `show_progress`, `prefer_strip`, `verify_checksums`, `color`, `quiet`, `verbose`

## [0.1.0] - 2026-04-21

### Added
- Initial release
- Install binaries from GitHub releases
- Archive support: tar.gz, tar.bz2, tar.xz, zip
- Automatic architecture detection (Linux x86_64, aarch64)
- Package registry for tracking installed packages
- List, update, uninstall, and search commands
- Progress bars for downloads
- GitHub Actions CI/CD