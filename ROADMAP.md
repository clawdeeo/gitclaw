# ROADMAP - Gitclaw Evolution

> Planned features and improvements for gitclaw.
> Completed features are removed from this document.

---

## 0.4.0 - Dependency Management

### Semver Range Support
Install versions matching constraints.

**Syntax:**
- `gitclaw install user/repo ">=1.0.0"`
- `gitclaw install user/repo "^1.2.3"`
- `gitclaw install user/repo "~1.2"`

### Lockfile Support
Reproducible installs via `gitclaw.lock`.

**Commands:**
- `gitclaw lock` — generate lockfile from current installs
- `gitclaw install --locked` — install from lockfile

**Format:** TOML with exact versions and hashes.

### Package Aliases
Short names for frequently used packages.

**Commands:**
- `gitclaw alias rg BurntSushi/ripgrep`
- `gitclaw install rg` → installs ripgrep

---

## 0.5.0 - User Experience

### TUI Mode
Interactive terminal UI with `ratatui`.

**Features:**
- `gitclaw --tui` or default when TTY detected
- Browse releases visually
- Preview assets before install
- Filter/search interactively

### Asset Caching
Cache downloaded archives.

**Location:** `~/.gitclaw/cache/`

**Commands:**
- Skip re-download if hash matches
- `gitclaw cache clean` — clear cache
- `gitclaw cache size` — show cache usage

### Outdated Check
Show packages with available updates.

**Commands:**
- `gitclaw list --outdated`
- Compares installed vs latest GitHub release

### Local Installs
Project-local package installation.

**Commands:**
- `gitclaw install --local user/repo`
- Installs to `./.gitclaw/bin/`
- Ignores global registry

---

## 0.6.0 - Advanced Features

### Release Channels
Install from specific channels.

**Channels:**
- `stable` — default
- `nightly` — latest pre-release
- `beta` — tagged beta versions

**Commands:**
- `gitclaw install user/repo --channel nightly`

### Export/Import
Share package lists between machines.

**Commands:**
- `gitclaw export > deps.toml`
- `gitclaw import deps.toml`

---

## 0.7.0 - Platform Integration

### Homebrew Integration
Detect and integrate with Homebrew on macOS.

**Features:**
- Warn if package already in Homebrew
- Option to prefer Homebrew for certain packages

### Windows PATH Management
Automatically add to Windows PATH.

**Features:**
- Modify system or user PATH
- Registry edits on Windows

---

## Future - Extensibility

### Plugin System
Extend gitclaw with custom plugins.

**Hooks:**
- `pre-install` — custom validation
- `post-install` — setup, symlinks
- `pre-uninstall` — cleanup

**Custom extractors:** Support non-standard archive formats.

---

## Rejected Ideas

| Idea | Reason |
|------|--------|
| Package signing | Too complex, checksums sufficient |
| Auto-detect updates | Too noisy, explicit is better |
| GUI application | Out of scope, TUI is sufficient |
| Telemetry | Privacy concerns, not worth it |
| Web Dashboard | Out of scope for CLI tool |

---

*Last updated: 2026-04-21*
*Contributors: clawdeeo, airscript*
