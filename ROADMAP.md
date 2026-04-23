# ROADMAP

Planned features and improvements toward gitclaw 1.0.0.

## 0.4.0 — Dependency Management

**Semver range support**

Install versions matching constraints:

```bash
gitclaw install user/repo ">=1.0.0"
gitclaw install user/repo "^1.2.3"
```

**Lockfile support**

Reproducible installs via `gitclaw.lock`:

```bash
gitclaw lock
gitclaw install --locked
```

**Package aliases**

Short names for frequently used packages:

```bash
gitclaw alias rg BurntSushi/ripgrep
gitclaw install rg
```

## 0.5.0 — User Experience

**Asset caching**

Cache downloaded archives to `~/.gitclaw/cache/` — skip re-download if hash matches.

```bash
gitclaw cache clean
gitclaw cache size
```

**Outdated check**

```bash
gitclaw list --outdated
```

Compares installed version against latest GitHub release.

**Local installs**

Project-scoped installation isolated from the global registry:

```bash
gitclaw install --local user/repo
```

## 0.6.0 — Advanced Features

**Release channels**

```bash
gitclaw install user/repo --channel nightly
gitclaw install user/repo --channel beta
```

**Export / import**

Share package lists between machines:

```bash
gitclaw export > deps.toml
gitclaw import deps.toml
```

## 0.7.0 — Platform Integration

**Package manager awareness**

Warn when a package is already available via a system package manager (apt, etc.).

## 1.0.0 — Stability

- All 0.x features stable and documented
- Stable registry format (no breaking changes without migration)
- Stable config format
- Full test coverage across all modules
- Published to crates.io

## Rejected Ideas

| Idea | Reason |
|------|--------|
| Package signing | Checksums are sufficient for the use case |
| Auto-update on launch | Too noisy; explicit is better |
| GUI application | Out of scope; TUI is sufficient |
| Telemetry | Privacy concerns |
| Web dashboard | Out of scope for a CLI tool |

*Last updated: 2026-04-23*
