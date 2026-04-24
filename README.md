# gitclaw

[![CI](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml/badge.svg)](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml)

Install software from GitHub releases.

Also available as `gcw`.

## Install

```bash
cargo install --path .
```

## Usage

```bash
gcw install sharkdp/bat
gcw list
```

See the [full command reference](#commands) below.

## Commands

| Command | Description |
|---------|-------------|
| `install <owner/repo>` | Install a package from GitHub releases |
| `install <owner/repo> "^14"` | Install with a semver range |
| `install --locked` | Install exact versions from the lockfile |
| `install --local <owner/repo>` | Install to the project-local `.gitclaw/` directory |
| `install --channel nightly <owner/repo>` | Install from a specific release channel |
| `list` | List installed packages |
| `list --outdated` | Show packages with newer versions available |
| `list --verbose` | Show detailed package information |
| `update [package]` | Update a package (or all if none specified) |
| `uninstall <package>` | Uninstall a package |
| `search <owner/repo>` | Browse available releases |
| `lock` | Generate a lockfile from installed packages |
| `alias add <name> <owner/repo>` | Create a short name for a package |
| `alias remove <name>` | Remove an alias |
| `alias list` | List all aliases |
| `cache size` | Show total cache size on disk |
| `cache clean` | Remove all cached archives |
| `export [-o file]` | Export installed packages as TOML |
| `import <file>` | Install packages from a TOML file |
| `self-update` | Update gitclaw to the latest version |
| `self-update --check` | Check for updates without installing |
| `completions <shell>` | Generate shell completions |
| `platform` | Show platform information |
| `run <package> [args...]` | Run an installed package |

## Configuration

Config files (TOML) are merged in order of precedence:

1. `$GITCLAW_CONFIG` environment variable
2. `./.gitclaw.toml`
3. `~/.config/gitclaw/config.toml`
4. `~/.gitclaw.toml`

```toml
install_dir = "~/bin"
github_token = "ghp_xxx"

[download]
show_progress = true
verify_checksums = true

[output]
quiet = false
verbose = false
```

## How It Works

1. Query the GitHub API for release metadata
2. Match the asset to the current OS and architecture
3. Download with a progress bar
4. Extract the archive (tar.gz, zip, tar.bz2, tar.xz, tar.zst, .deb, or plain binary)
5. Install the binary to `~/.gitclaw/bin/`

## Supported Platforms

| OS | x86_64 | aarch64 |
|----|--------|---------|
| Linux | yes | yes |

## Development

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT -- Copyright (c) 2026 Francesco Sardone (Airscript)
