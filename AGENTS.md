# AGENTS.md

Development guide for gitclaw agents and contributors.

## Architecture

- **Nested `src/` structure** — modules are organized under `src/core/`, `src/network/`, `src/cli/`, and `src/output/`
- **Two binaries, one source** — `gitclaw` and `gcw` both compile from `src/main.rs`
- **`src/lib.rs`** re-exports all modules publicly; integration tests consume the crate as a library
- **Integration tests in `tests/`** — one file per module, flat structure, no subdirectories
- **No in-module unit tests** — all tests live in `tests/`

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
  util.rs
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
- Imports sorted alphabetically within groups (stdlib / external / crate-internal), blank line between groups; run `cargo fmt` before committing
- All tests go in `tests/` — no `#[cfg(test)]` blocks in source files

## Output Conventions

- Success messages: `[EXEC]` prefix, bold green — via `output::print_success`
- Info messages: `[INFO]` prefix, cyan — via `output::print_info`
- Warning messages: `[WARN]` prefix, bold yellow — via `output::print_warn`
- Error messages: `[ERR]` prefix, bold red, stderr — via `output::print_error`
- Key-value pairs: via `output::print_kv`
- No horizontal separator lines
- `NO_COLOR` env var disables all color output

## CI Flow

Verify → Test → Build

## Spec-Driven Development

1. Create `.specs/feature-name.md` from TEMPLATE.md before coding
2. Small fixes: skip spec, write a clear PR description
3. Features: full spec with acceptance criteria
4. Review spec with user before implementation
5. Checkpoints tied to deliverables, not percentages
6. Keep specs in git — archive after merge
7. Post-mortem lessons go to AGENTS.md, not the spec

## PR Discipline

- Always branch: `feat/`, `fix/`, `docs/`, `chore/`
- Never push to main directly
- Squash merge after review
- Include tests and documentation updates

## Definition of Done

- [ ] Code complete
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] Run all three locally before pushing
- [ ] Documentation updated (CHANGELOG, README if needed)
- [ ] Manual verification done
- [ ] PR opened and reviewed

## Post-Mortems

After rework or significant issues:
- What went wrong
- Root cause
- Prevention
- Add to AGENTS.md so it persists

*Last updated: 2026-04-23*

