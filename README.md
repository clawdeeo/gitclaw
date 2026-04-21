# gitclaw

[![Main](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml/badge.svg)](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml)

Install software directly from GitHub releases.

## Overview

gitclaw is a CLI tool that automates the discovery, download, and installation of binaries from GitHub releases. It handles OS/architecture detection, asset matching, extraction, and PATH management.

## Installation

```bash
cargo install --path .
```

Or download a pre-built release (coming soon).

## Usage

### Install a package

```bash
# Install latest version
gitclaw install BurntSushi/ripgrep

# Install specific version
gitclaw install BurntSushi/ripgrep@13.0.0

# Force reinstall
gitclaw install BurntSushi/ripgrep --force
```

### List installed packages

```bash
gitclaw list
gitclaw list --verbose
```

### Update packages

```bash
# Update a specific package
gitclaw update BurntSushi/ripgrep

# Update all packages
gitclaw update
```

### Uninstall a package

```bash
gitclaw uninstall BurntSushi/ripgrep
```

### Search for releases

```bash
gitclaw search sharkdp/fd --limit 5
```

## Configuration

gitclaw supports configuration files in TOML format. Configs are merged in this order of precedence (higher overrides lower):

1. `$GITCLAW_CONFIG` environment variable (highest priority)
2. `./.gitclaw.toml` (project-local)
3. `~/.config/gitclaw/config.toml` (XDG config)
4. `~/.gitclaw.toml` (legacy)
5. Defaults (lowest priority)

### Example config file

```toml
# Installation directory (default: ~/.gitclaw/bin)
install_dir = "~/bin"

# Optional: GitHub token for higher rate limits or private repos
github_token = "ghp_xxxxxxxxxxxx"

[download]
show_progress = true      # Show download progress bars (default: true)
prefer_strip = true       # Strip directory components when extracting (default: true)
verify_checksums = true   # Verify checksums when available (default: true)

[output]
color = "auto"            # Color output: auto, always, never (default: auto)
quiet = false             # Suppress non-error output (default: false)
verbose = false           # Enable verbose output (default: false)
```

### GitHub Token

A GitHub token is **optional** and only required for:
- **Private repositories** — accessing releases in private repos
- **Higher rate limits** — unauthenticated requests are limited to 60/hour, authenticated gets 5,000/hour

Set it via config file (see above), environment variable, or CLI flag:

```bash
export GITHUB_TOKEN=your_token_here
gitclaw --token YOUR_TOKEN install user/repo
```

## How It Works

1. **Parse**: Extract owner/repo/version from input
2. **Fetch**: Query GitHub API for release metadata
3. **Match**: Find asset matching current OS/architecture
4. **Download**: Stream asset to temp location with progress bar
5. **Extract**: Handle tar.gz, zip, tar.bz2, tar.xz, or plain binaries
6. **Discover**: Find executable binary in extracted contents
7. **Install**: Copy binary to the configured install directory
8. **Register**: Track installation in the registry

## Directory Structure

```
~/.gitclaw/
├── bin/                    # Installed binaries (add to PATH)
├── packages/               # Extracted package contents
└── registry.toml          # Installation database
```

## Supported Platforms

| OS | Architecture | Asset Keywords |
|----|-------------|----------------|
| Linux | x86_64 | `linux-x86_64`, `linux-amd64` |
| Linux | aarch64 | `linux-aarch64`, `linux-arm64` |
| macOS | x86_64 | `darwin-x86_64`, `darwin-amd64` |
| macOS | aarch64 | `darwin-aarch64`, `darwin-arm64` |
| Windows | x86_64 | `windows-x86_64`, `windows-amd64` |

## Architecture

```
src/
├── main.rs          # CLI entry point
├── cli.rs           # Clap CLI definitions
├── config.rs        # Configuration file support
├── github.rs        # GitHub API client
├── install.rs       # Install/update logic
├── extract/         # Archive extraction (flat module files)
├── platform.rs      # OS/arch detection
├── registry.rs      # Package tracking
└── util.rs          # Utilities
```

## Development

```bash
# Build
cargo build --release

# Test
cargo test

# Run
cargo run -- install BurntSushi/ripgrep
```

## License

MIT
