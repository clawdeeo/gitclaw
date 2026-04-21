# gitclaw

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

### GitHub Token

Set a GitHub token to avoid rate limits:

```bash
export GITHUB_TOKEN=your_token_here
```

Or pass it via CLI:

```bash
gitclaw --token YOUR_TOKEN install user/repo
```

## How It Works

1. **Parse**: Extract owner/repo/version from input
2. **Fetch**: Query GitHub API for release metadata
3. **Match**: Find asset matching current OS/architecture
4. **Download**: Stream asset to temp location with progress bar
5. **Extract**: Handle tar.gz, zip, tar.bz2, tar.xz, or plain binaries
6. **Discover**: Find executable binary in extracted contents
7. **Install**: Copy binary to `~/.gitclaw/bin/`
8. **Register**: Track installation in `~/.gitclaw/registry.toml`

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
├── github.rs        # GitHub API client
├── install.rs       # Install/update logic
├── extract/mod.rs   # Archive extraction
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
