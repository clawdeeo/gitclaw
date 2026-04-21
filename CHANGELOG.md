# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD pipeline for Linux, macOS, and Windows
- Code coverage reporting with cargo-tarpaulin
- `CONTRIBUTING.md` with contribution guidelines
- `CHANGELOG.md` to track project changes
- `rust-toolchain.toml` for consistent Rust toolchain

### Changed
- Improved error handling in `main.rs` (replaced `unwrap()` with `?`)
- Updated README with CI badge

## [0.1.0] - 2024-04-21

### Added
- Initial release of gitclaw
- Install binaries directly from GitHub releases
- Support for tar.gz, tar.bz2, tar.xz, and zip archives
- Automatic OS/architecture detection
- Package registry for tracking installed packages
- List, update, and uninstall commands
- Search command for browsing releases
- Automatic binary discovery and extraction
- Progress bars for downloads
