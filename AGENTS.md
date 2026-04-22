# AGENTS.md

Quick guide for gitclaw development.

## Architecture
- Flat `src/` structure — modules are `.rs` files, no nested folders
- Tests in `tests/unit/` — public API only
- Target: 80%+ coverage

## Workflow
1. Pull main first
2. Branch: `fix/`, `feat/`, `docs/`, `chore/`
3. Commit: `type: description` (lowercase)
4. Local verify before push:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. PR → squash merge

## Code Style
- No comments unless complex
- `?` for errors, no `unwrap()` in production
- Opening brace on same line
- Blank line after multi-line blocks

## CI Flow
Verify → Test → Build

*Last updated: 2026-04-22*
