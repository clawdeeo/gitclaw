# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-04-21

### Added
- Configuration file support with multiple config sources (env, project-local, XDG, legacy)
- `--dry-run` flag for install command to preview changes without downloading
- `completions` command for generating shell completions (bash, zsh, fish, powershell, elvish)
- Config options: `install_dir`, `show_progress`, `prefer_strip`, `verify_checksums`, `color`, `quiet`, `verbose`

### Changed
- Wired config values throughout codebase (no longer ignored)

## [0.1.0] - 2026-04-21

### Added
- Initial release of gitclaw
- Install binaries directly from GitHub releases
- Support for tar.gz, tar.bz2, tar.xz, and zip archives
- Automatic OS/architecture detection (Linux, macOS, Windows)
- Package registry for tracking installed packages
- List, update, and uninstall commands
- Search command for browsing releases
- Automatic binary discovery and extraction
- Progress bars for downloads
- GitHub Actions CI/CD with reusable workflows
- Cross-platform builds (Linux, macOS, Windows)
- 84+ tests with 80%+ coverage
- `CONTRIBUTING.md`, `CHANGELOG.md`, `AGENTS.md` documentation
