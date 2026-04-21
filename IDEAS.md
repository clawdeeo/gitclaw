# IDEAS.md - Gitclaw Evolution Roadmap

> Brainstorming future features and improvements for gitclaw.
> This is a living document — ideas may be implemented, modified, or discarded.

---

## Core Features

### 1. Configuration File Support
**Description:** User configuration via `.gitclaw.toml` or `gitclaw.json`.

**Use cases:**
- Set default install directory
- Store GitHub token
- Preferred asset formats
- Per-project config (`.gitclawrc` in repo root)

**Example:**
```toml
# ~/.gitclaw.toml
install_dir = "~/.local/bin"
github_token = "ghp_..."

[preferences]
prefer_strip = true
show_progress = true
```

---

### 2. Self-Update
**Description:** Update gitclaw itself without manual reinstall.

**Commands:**
- `gitclaw self-update` — check and install latest release
- `gitclaw self-update --check` — only check for updates
- Optional: auto-check on startup (configurable)

---

### 3. Dry Run Mode
**Description:** Preview what would happen without downloading.

**Commands:**
- `gitclaw install --dry-run user/repo`
- Shows: which asset selected, where it would install, etc.

**Benefit:** CI-friendly, prevents surprises.

---

### 4. Checksum Verification
**Description:** Verify downloaded assets against published checksums.

**Features:**
- Auto-detect `.sha256`, `.sha512`, `.md5` files in release
- Warn if checksum doesn't match
- `gitclaw install --verify user/repo`

**Security:** Prevents corrupted or malicious downloads.

---

### 5. Parallel Installs
**Description:** Install multiple packages concurrently.

**Commands:**
- `gitclaw install pkg1 pkg2 pkg3`
- Downloads and extracts in parallel
- Shared progress bar

**Benefit:** Much faster bulk operations.

---

### 6. Semver Range Support
**Description:** Install versions matching constraints.

**Syntax:**
- `gitclaw install user/repo ">=1.0.0"`
- `gitclaw install user/repo "^1.2.3"`
- `gitclaw install user/repo "~1.2"`

**Benefit:** Flexible version management without exact tags.

---

### 7. Lockfile Support
**Description:** Reproducible installs via `gitclaw.lock`.

**Commands:**
- `gitclaw lock` — generate lockfile from current installs
- `gitclaw install --locked` — install from lockfile

**Format:** TOML with exact versions and hashes.

---

### 8. Shell Completions
**Description:** Tab completion for shells.

**Supported:** Bash, Zsh, Fish

**Features:**
- Complete package names from registry
- Complete flags and subcommands
- `gitclaw completions bash > /etc/bash_completion.d/gitclaw`

---

## UX Improvements

### 9. TUI Mode
**Description:** Interactive terminal UI with `ratatui`.

**Commands:**
- `gitclaw --tui` or default when TTY detected
- Browse releases visually
- Preview assets before install
- Filter/search interactively

---

### 10. Asset Caching
**Description:** Cache downloaded archives.

**Location:** `~/.gitclaw/cache/`

**Commands:**
- Skip re-download if hash matches
- `gitclaw cache clean` — clear cache
- `gitclaw cache size` — show cache usage

---

### 11. Local Installs
**Description:** Project-local package installation.

**Commands:**
- `gitclaw install --local user/repo`
- Installs to `./.gitclaw/bin/`
- Ignores global registry

**Use case:** CI/CD, reproducible environments.

---

### 12. Package Aliases
**Description:** Short names for frequently used packages.

**Commands:**
- `gitclaw alias rg BurntSushi/ripgrep`
- `gitclaw install rg` → installs ripgrep
- Store in config file

---

### 13. Outdated Check
**Description:** Show packages with available updates.

**Commands:**
- `gitclaw list --outdated`
- Compares installed vs latest GitHub release
- Shows current → latest version

---

## Advanced Features

### 14. Plugin System
**Description:** Extend gitclaw with custom plugins.

**Hooks:**
- `pre-install` — custom validation
- `post-install` — setup, symlinks
- `pre-uninstall` — cleanup

**Custom extractors:** Support non-standard archive formats.

---

### 15. Release Channels
**Description:** Install from specific channels.

**Channels:**
- `stable` — default
- `nightly` — latest pre-release
- `beta` — tagged beta versions

**Commands:**
- `gitclaw install user/repo --channel nightly`

---

### 16. Export/Import
**Description:** Share package lists between machines.

**Commands:**
- `gitclaw export > deps.toml`
- `gitclaw import deps.toml`

**Format:** TOML with package specs.

---

## Platform-Specific

### 17. Homebrew Integration
**Description:** Detect and integrate with Homebrew on macOS.

**Features:**
- Warn if package already in Homebrew
- Option to prefer Homebrew for certain packages

---

### 18. Windows PATH Management
**Description:** Automatically add to Windows PATH.

**Features:**
- Modify system or user PATH
- Registry edits on Windows

---

## Nice to Have

### 19. Telemetry (Opt-in)
**Description:** Anonymous usage statistics.

**Data:**
- Popular packages
- Install success/failure rates
- Platform distribution

**Privacy:** Opt-in only, no personal data.

---

### 20. Web Dashboard
**Description:** Browser-based package browser.

**Features:**
- Search GitHub releases
- View package popularity
- One-click install (generates shell command)

---

## Rejected Ideas

> Ideas considered but rejected, with reasoning.

| Idea | Reason |
|------|--------|
| Package signing | Too complex, checksums sufficient |
| Auto-detect updates | Too noisy, explicit is better |
| GUI application | Out of scope, TUI is sufficient |

---

## Implementation Priority

### Phase 1 (v0.2.0) ✅
1. Config file support ✅
2. Shell completions ✅
3. Dry run mode ✅

### Phase 2 (v0.3.0)
4. Self-update
5. Checksum verification
6. Parallel installs

### Phase 3 (v0.4.0)
7. Semver ranges
8. Lockfile support
9. TUI mode

### Phase 4 (Future)
10. Plugin system
11. Release channels
12. Advanced caching

---

*Last updated: 2026-04-21*
*Contributors: clawdeeo, airscript*
