# Contributing to gitclaw

Thank you for your interest in contributing to gitclaw! This document provides guidelines for contributing.

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/clawdeeo/gitclaw.git
   cd gitclaw
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Code Style

- Run `cargo fmt` before committing to ensure consistent formatting
- Run `cargo clippy` to check for common issues
- Address all clippy warnings before submitting a PR

## Testing

- All new features should include unit tests
- Integration tests are in the `tests/` directory
- Aim for 80%+ code coverage

## Submitting Changes

1. Create a new branch for your feature/fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and commit with clear messages:
   ```bash
   git commit -m "feat: add new feature"
   ```

3. Push to your fork and open a Pull Request

## Commit Message Convention

We follow conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

## Code of Conduct

- Be respectful and constructive
- Focus on the issue, not the person
- Help others learn and grow
