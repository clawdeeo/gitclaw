# AGENTS.md - Gitclaw Development Guide

> **The Claw is the Law** — This document guides development workflows for the gitclaw project.

---

## Repository Structure

```
gitclaw/
├── .github/workflows/     # CI/CD reusable workflows
├── src/                   # Source code (flat structure)
├── tests/                 # Integration tests
├── Cargo.toml            # Dependencies
├── README.md             # User documentation
└── AGENTS.md             # This file
```

---

## Architecture Principles

### Module Organization (Flat Structure)
All modules are `.rs` files in `src/`, no nested folders:

```rust
// src/main.rs
mod cli;
mod extract;
mod github;
mod install;
mod platform;
mod registry;
mod util;
```

**Why flat?** Simpler imports, easier navigation, scales well for CLI size.

---

## Testing Strategy

### Test Location
**Integration tests ONLY** — Place all tests in `tests/` folder, not inline:

```
tests/
├── extract.rs      # Tests src/extract.rs public API
├── github.rs       # Tests src/github.rs public API
├── platform.rs     # Tests src/platform.rs public API
├── registry.rs     # Tests src/registry.rs public API
└── error_handling.rs  # Cross-module error tests
```

### Test Requirements
- Target **80%+ coverage**
- Test public API only (don't test internals)
- Use descriptive test names: `test_<function>_<scenario>`

---

## CI/CD Structure

### Reusable Workflows
Split CI into focused workflows with dependency chain:

**Flow:** `Verify → Test → Build`

| Workflow | Jobs | Purpose |
|----------|------|---------|
| `verify.yml` | `format`, `clippy` | Code quality gates |
| `test.yml` | `linux`, `macos`, `windows` | Cross-platform tests |
| `build.yml` | `linux`, `macos`, `windows` | Release binaries |
| `main.yml` | Orchestrator | Chains workflows with `needs:` |

### Platform Strategy
- **Linux**: Use `rust:1.75-slim` container (faster, lighter)
- **macOS/Windows**: Use `dtolnay/rust-toolchain` (no containers available)

---

## Code Style

### Minimal Comments
Only comment when:
- Logic is complex/non-obvious
- External dependencies behave unexpectedly
- Workarounds exist for known issues

**Avoid**: Obvious comments, "what" comments, redundant docstrings on private items.

### Error Handling
- Use `thiserror` for custom error types
- Propagate with `?` operator
- Add context with `.context()` from `anyhow`
- **NO `unwrap()` in production code**

---

## Git Workflow

### Branch Naming
- `feature/description` — new features
- `fix/description` — bug fixes
- `refactor/description` — code restructuring
- `docs/description` — documentation

### Commits
Follow conventional commits:
```
<type>: <description>

<body explaining why, not what>
```

Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`

### Pulling Before Branching
**Always pull from main before creating a new branch.**

If you forget and main has moved forward:

```bash
# Save your changes
git checkout your-branch
git rebase main
# Resolve any conflicts, then:
git push --force-with-lease origin your-branch
```

**Why:** Keeps history linear and avoids merge commits in PRs.

---

## Dependencies

### Adding Dependencies
1. Prefer crates.io over git repos
2. Pin major versions: `crate = "1"` not `"1.2.3"`
3. Group related deps (e.g., `tokio = { features = ["full"] }`)

### Updating Dependencies
- Run `cargo update` periodically
- Check for security advisories with `cargo audit`

---

## Documentation

### README.md
Keep updated with:
- Installation instructions
- Basic usage examples
- Platform support matrix
- Configuration options

### Code Documentation
- Public API: Doc comments (`///`)
- Complex logic: Inline comments (`//`)
- Modules: Module-level docs (`//!`)

---

## Performance Guidelines

### CI Optimization
- Use containers for Linux jobs (cached, faster)
- Parallelize independent jobs
- Cache `target/` between runs

### Binary Size
- Enable LTO in release: `lto = true`
- Strip symbols: `strip = true`
- Use `cargo bloat` to analyze

---

## Security

### Secrets
- Use `GITHUB_TOKEN` for API access
- Never commit `.env` files
- Store tokens in repository secrets, not code

### Dependencies
- Run `cargo audit` before releases
- Pin transitive deps if vulnerable

---

## Release Process

1. Update `CHANGELOG.md`
2. Bump version in `Cargo.toml`
3. Tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. CI builds release binaries automatically
6. Create GitHub release with artifacts

---

## Common Patterns

### Adding a New Command
1. Add variant to `cli.rs` (`Commands` enum)
2. Implement handler in `src/<module>.rs`
3. Wire in `main.rs` match statement
4. Add tests in `tests/<module>.rs`

### Error Propagation
```rust
// Good: Context + specific error
let file = fs::read(&path)
    .context(format!("Failed to read {}", path.display()))?;

// Bad: Generic error
let file = fs::read(&path)?;  // Less helpful
```

---

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Actions Docs](https://docs.github.com/en/actions)

---

*Last updated: 2026-04-21*
