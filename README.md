# gitclaw

[![Main](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml/badge.svg)](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml)

Install software from GitHub releases.

Also available as `gcw` — a shorter alias.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Install latest version
gitclaw install BurntSushi/ripgrep

# Install specific version
gitclaw install BurntSushi/ripgrep@13.0.0

# List, update, uninstall
gitclaw list
gitclaw update BurntSushi/ripgrep
gitclaw uninstall BurntSushi/ripgrep

# Shell completions
gitclaw completions bash > ~/.local/share/bash-completion/completions/gitclaw
```

## Configuration

Config files (TOML) are merged in order of precedence:
1. `$GITCLAW_CONFIG` env var
2. `./.gitclaw.toml`
3. `~/.config/gitclaw/config.toml`
4. `~/.gitclaw.toml`

```toml
install_dir = "~/bin"
github_token = "ghp_xxx"  # Optional: higher rate limits, private repos

[download]
show_progress = true
verify_checksums = true

[output]
quiet = false
verbose = false
```

## How It Works

1. Query GitHub API for release metadata
2. Match asset to current OS/architecture
3. Download with progress bar
4. Extract: supports tar.gz, zip, tar.bz2, tar.xz, tar.zst, .deb, and plain binaries
5. Install binary to `~/.gitclaw/bin/`

## Supported Platforms

| OS | x86_64 | aarch64 |
|----|--------|---------|
| Linux | yes | yes |
| macOS | yes | yes |
| Windows | yes | yes |

## Development

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- install owner/repo
```

See [AGENTS.md](AGENTS.md) for contribution guidelines.

## License

MIT License - Copyright (c) 2026 Francesco Sardone (Airscript)
