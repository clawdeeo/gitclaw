# gitclaw

[![CI](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml/badge.svg)](https://github.com/clawdeeo/gitclaw/actions/workflows/main.yml)

Install software from GitHub releases.  
Also available as `gcw`.

![GIF](assets/gcw.gif)

## Install

```bash
cargo install --path .
```

## Usage

```bash
gcw install sharkdp/bat
gcw list
```

## Commands

| Command     | Description                                  |
| ----------- | -------------------------------------------- |
| alias       | Manage package aliases.                      |
| cache       | Manage the asset cache.                      |
| completions | Generate shell completions.                  |
| export      | Export installed packages to TOML.           |
| import      | Install packages from a TOML file.           |
| install     | Install packages from GitHub releases.       |
| list        | List installed packages.                     |
| lock        | Generate a lockfile from installed packages. |
| platform    | Show platform information.                   |
| run         | Run an installed package.                    |
| search      | Search for releases on GitHub.               |
| self        | Update gitclaw to the latest version.        |
| uninstall   | Uninstall a package.                         |
| update      | Update installed packages.                   |

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

| OS    | x86_64 | aarch64 |
| ----- | ------ | ------- |
| Linux | yes    | yes     |

## Development

```bash
cargo fmt
cargo clippy
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under MIT, Copyright (c) 2026 Francesco Sardone (Airscript)
