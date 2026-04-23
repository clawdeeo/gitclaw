# AGENTS.md

Development guide for gitclaw agents and contributors.

## Architecture

- **Flat `src/` structure** — all modules are `.rs` files at the top level, no nested folders
- **Two binaries, one source** — `gitclaw` and `gcw` both compile from `src/main.rs`
- **`src/lib.rs`** re-exports all modules publicly; integration tests consume the crate as a library
- **Integration tests in `tests/`** — one file per module, flat structure, no subdirectories
- **In-module unit tests** live in `#[cfg(test)]` blocks at the bottom of each `.rs` file

## Source Modules

| File | Responsibility |
|------|----------------|
| `main.rs` | Entry point, CLI dispatch, `run_package()` |
| `lib.rs` | Module re-exports |
| `cli.rs` | Clap CLI definition |
| `banner.rs` | `print_success` (`[EXEC]`), `print_info` (cyan), `print_kv`, `print_header` |
| `config.rs` | `Config`, `DownloadConfig`, `OutputConfig`, config loading/merging |
| `checksum.rs` | `ChecksumAlgorithm`, `verify_file`, `calculate_checksum`, `parse_checksum_file` |
| `extract.rs` | `ArchiveType`, `extract_archive`, `detect_archive_type` |
| `github.rs` | `GithubClient`, `Release`, `Asset`, `Platform`, `GithubError`, `parse_package`, `find_matching_asset` |
| `install.rs` | `handle_install`, `handle_update`, `handle_install_multiple` |
| `platform.rs` | `Arch`, `PlatformError`, `detect_arch`, `current_platform`, `score_asset`, `find_best_asset` |
| `registry.rs` | `InstalledPackage`, `Registry`, `list_installed`, `uninstall` |
| `self_update.rs` | `check_for_update`, `perform_update` |
| `util.rs` | Path helpers, `format_bytes` |

## Test Structure

Integration tests live in `tests/` (flat, no subdirectories):

```
tests/
  checksum.rs
  config.rs
  extract.rs
  github.rs
  platform.rs
  registry.rs
```

Each file covers the corresponding module's public API. Tests that validate error paths are kept in the same file as the feature they test.

## Workflow

1. Pull `main` before starting
2. Branch prefix: `fix/`, `feat/`, `docs/`, `chore/`
3. Commit format: `type: description` (lowercase, imperative)
4. Verify locally before pushing:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. Open PR → squash merge into `main`

## Code Style

- No comments or docstrings anywhere in `.rs` files — code should be self-explanatory
- Use `?` for error propagation; no `unwrap()` in production code
- Opening brace on same line
- Blank line after multi-line blocks
- `#[serde(default)]` on any new optional struct fields for backward compatibility
- New `InstalledPackage` fields must use `#[serde(default)]` to stay compatible with existing registry TOML

## Output Conventions

- Success messages: `[EXEC]` prefix, bold green — via `banner::print_success`
- Info messages: `[INFO]` prefix, cyan — via `banner::print_info`
- Key-value pairs: via `banner::print_kv`
- No horizontal separator lines
- `NO_COLOR` env var disables all color output

## CI Flow

Verify → Test → Build

*Last updated: 2026-04-23*
