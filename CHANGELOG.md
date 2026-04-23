# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `identifier` field on `InstalledPackage` — set to the repo name at install time
- `uninstall` now accepts short name (repo or identifier) in addition to `owner/repo`
- `list` shows an `Identifier` column (cyan) in both normal and verbose modes
- 74 new integration tests across `tests/checksum.rs`, `tests/config.rs`, `tests/extract.rs`, `tests/github.rs`, `tests/platform.rs`, `tests/registry.rs`
- `gcw` short alias binary

### Changed
- Success prefix changed from `[OK]` to `[EXEC]` (bold green)
- Info prefix is now cyan
- Removed all horizontal separator lines from output
- `tests/unit/` folder dissolved — all integration tests now live directly under `tests/`
- `assert_cmd` moved to `[dev-dependencies]`
- Removed unused dependencies: `base64`, `base64ct`, `indexmap`, `url`, `time`

### Fixed
- `gcw list` crash when registry TOML was written before `identifier` field existed — `#[serde(default)]` applied

## [0.3.1] - 2026-04-22

### Fixed
- Platform asset matching now correctly selects platform-specific assets
- Added Rust target triple aliases for ripgrep and similar Rust projects
- Symlinks in bin/ now use absolute paths

## [0.3.0] - 2026-04-21

### Added
- Self-update command (`gitclaw self-update`, `gitclaw self-update --check`)
- Checksum verification for downloaded assets (`--verify` flag)
- Parallel install support (`gitclaw install pkg1 pkg2 pkg3`)

### Changed
- Install command now accepts multiple package arguments

## [0.2.0] - 2026-04-21

### Added
- Configuration file support with multiple config sources (env, project-local, XDG, legacy)
- `--dry-run` flag for install command
- `completions` command for shell completions (bash, zsh, fish, powershell, elvish)
- Config options: `install_dir`, `show_progress`, `prefer_strip`, `verify_checksums`, `color`, `quiet`, `verbose`

### Changed
- Config values are now wired throughout the codebase

## [0.1.0] - 2026-04-21

### Added
- Initial release
- Install binaries from GitHub releases
- Archive support: tar.gz, tar.bz2, tar.xz, zip
- Automatic architecture detection (Linux x86_64, aarch64)
- Package registry for tracking installed packages
- List, update, and uninstall commands
- Search command for browsing releases
- Progress bars for downloads
- GitHub Actions CI/CD
