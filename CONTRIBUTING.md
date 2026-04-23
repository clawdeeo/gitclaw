# Contributing to gitclaw

## Development Setup

```bash
git clone https://github.com/clawdeeo/gitclaw.git
cd gitclaw
cargo build
cargo test
```

## Code Style

- No comments or docstrings in `.rs` files
- Use `?` for error propagation; no `unwrap()` in production code
- Run `cargo fmt` before committing
- All clippy warnings must be resolved: `cargo clippy -- -D warnings`

## Testing

- New features require integration tests in `tests/<module>.rs`
- In-module unit tests go in `#[cfg(test)]` blocks at the bottom of the source file
- New `InstalledPackage` fields must use `#[serde(default)]` for registry compatibility

## Submitting Changes

1. Branch from `main` with a prefix: `fix/`, `feat/`, `docs/`, `chore/`
2. Verify before pushing:
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   ```
3. Open a PR — squash merge into `main`

## Commit Message Convention

```
feat: add short description
fix: correct something
docs: update readme
chore: bump dependency
```

Lowercase, imperative, no period at end.
