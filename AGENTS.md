# AGENTS.md

Development guide for gitclaw contributors and agents.

## Architecture

- Nested `src/` structure: `core/`, `network/`, `cli/`, `output/`
- Two binaries, one source: `gitclaw` and `gcw` both compile from `src/main.rs`
- `src/lib.rs` re-exports all modules publicly; integration tests consume the crate as a library
- Integration tests in `tests/` (flat, no subdirectories, one file per module)
- No in-module unit tests, all tests live in `tests/`

## Source Modules

| File | Responsibility |
|------|----------------|
| `main.rs` | Entry point, CLI dispatch, `run_package()` |
| `lib.rs` | Module re-exports |
| `cli/mod.rs` | Clap CLI definition |
| `output/mod.rs` | `print_success` (`[EXEC]`), `print_info` (cyan), `print_warn` (yellow), `print_error` (red), `print_kv`, `print_header` |
| `core/config.rs` | `Config`, `DownloadConfig`, `OutputConfig`, config loading/merging |
| `core/checksum.rs` | `ChecksumAlgorithm`, `verify_file`, `calculate_checksum`, `parse_checksum_file` |
| `core/extract.rs` | `ArchiveType`, `extract_archive`, `detect_archive_type` |
| `core/install.rs` | `handle_install`, `handle_update`, `handle_install_multiple` |
| `core/registry.rs` | `InstalledPackage`, `Registry`, `list_installed`, `uninstall` |
| `core/updater.rs` | `check_for_update`, `perform_update` |
| `core/util.rs` | Path helpers, `format_bytes` |
| `network/github.rs` | `GithubClient`, `Release`, `Asset`, `Platform`, `GithubError`, `parse_package`, `find_matching_asset` |
| `network/platform.rs` | `Arch`, `PlatformError`, `detect_arch`, `current_platform`, `score_asset`, `find_best_asset` |

## Output Conventions

- Success: `[EXEC]` prefix, bold green, via `output::print_success`
- Info: `[INFO]` prefix, cyan, via `output::print_info`
- Warning: `[WARN]` prefix, bold yellow, via `output::print_warn`
- Error: `[ERR]` prefix, bold red, stderr, via `output::print_error`
- Key-value pairs: via `output::print_kv`
- No horizontal separator lines
- `NO_COLOR` env var disables all color output

## Code Style

- No comments or docstrings anywhere in `.rs` files
- Use `?` for error propagation, no `unwrap()` in production code
- Opening brace on same line
- Blank line after multi-line blocks
- `#[serde(default)]` on any new optional struct fields for backward compatibility
- Imports sorted alphabetically within groups (stdlib / external / crate-internal), blank line between groups
- All tests go in `tests/`, no `#[cfg(test)]` blocks in source files

## Spec-Driven Development

1. Create `.specs/feature-name.md` from TEMPLATE.md before coding
2. Small fixes: skip spec, write a clear PR description
3. Features: full spec with acceptance criteria
4. Review spec with user before implementation
5. Checkpoints tied to deliverables, not percentages
6. Specs are temporary planning artifacts, delete after merge
7. Post-mortem lessons go to AGENTS.md, not the spec

## CI Flow

Verify, Test, Build

## Post-Mortems

After rework or significant issues, document: what went wrong, root cause, prevention. Add to this file so it persists.

*Last updated: 2026-04-24*