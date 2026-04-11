# F25 — Homebrew Formula

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 6 — Distribution & Infrastructure
> **Complexity:** 3/12 | **Confidence:** 92%

---

## Problem Statement

macOS users expect to install terminal tools via `brew install`. Without a Homebrew formula, termaup requires either building from source or manually downloading release binaries and placing them on `$PATH`. This friction reduces adoption among the primary target audience — macOS power users.

## Approach

Create a Homebrew tap repository (`rynaro/homebrew-tap`) containing a formula that downloads pre-built binaries from GitHub Releases. The formula installs both `clickup` (CLI) and `termaup` (TUI) binaries. An additional CI step in the release workflow (F24) automatically updates the formula with the new version and SHA256 checksums after each release.

### Rejected Alternatives

1. **Submit to homebrew-core** — Requires significant adoption metrics and ongoing maintenance burden with Homebrew's review process. Rejected for now; can be pursued once termaup reaches sufficient popularity.

2. **Build from source in formula** — `brew install` would invoke `cargo build`, requiring a Rust toolchain on the user's machine. Defeats the purpose of pre-built binaries. Rejected for user experience.

3. **Separate formula per binary** — One formula for `clickup`, another for `termaup`. Adds unnecessary complexity; they're always released together and share a version. Rejected for simplicity.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Homebrew tap repository (`rynaro/homebrew-tap`) | homebrew-core submission | Linux Homebrew support (Linuxbrew) |
| Formula for macOS (x86_64 + aarch64) | Windows package manager (Scoop/Chocolatey) | Shell completion installation via formula |
| CI auto-update step in release workflow | Formula tests with `brew test` | Cask formula (if TUI needs special terminal handling) |
| `brew install rynaro/tap/termaup` command | | |

## Stories

### S-1: Homebrew Formula

**As a** macOS user, **I want** to run `brew install rynaro/tap/termaup` to install both CLI and TUI binaries, **so that** I can start using termaup immediately without compiling from source.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** F24 (release pipeline must exist)

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create tap repository | `rynaro/homebrew-tap` (GitHub) | New repo with `Formula/` directory |
| 2 | Create formula file | `Formula/termaup.rb` (in tap repo) | Ruby formula class `Termaup < Formula` |
| 3 | Define architecture blocks | `Formula/termaup.rb` | `on_macos do; on_intel do; url "...x86_64-apple-darwin.tar.gz"; sha256 "..."; end; on_arm do; url "...aarch64-apple-darwin.tar.gz"; sha256 "..."; end; end` |
| 4 | Install both binaries | `Formula/termaup.rb` | `bin.install "clickup"` and `bin.install "termaup"` in the `install` method |
| 5 | Add test block | `Formula/termaup.rb` | `test do; assert_match version.to_s, shell_output("#{bin}/clickup --version"); end` |
| 6 | Add description and metadata | `Formula/termaup.rb` | `desc`, `homepage`, `license`, `head` fields |

#### Acceptance Criteria

- **GIVEN** the tap is added (`brew tap rynaro/tap`), **WHEN** `brew install rynaro/tap/termaup` is run on macOS aarch64, **THEN** both `clickup` and `termaup` binaries are installed and available on `$PATH`.
- **GIVEN** macOS x86_64, **WHEN** installed via the formula, **THEN** the correct x86_64 binary is downloaded.
- **GIVEN** the formula is installed, **WHEN** `clickup --version` is run, **THEN** it outputs the installed version.
- **GIVEN** the formula is installed, **WHEN** `termaup --version` is run, **THEN** it outputs the installed version.
- **GIVEN** the formula, **WHEN** `brew audit --strict Formula/termaup.rb` is run, **THEN** it passes with no errors.

#### Agent Hints

- **Class:** builder
- **Context:** The Homebrew formula should reference GitHub Release URLs: `https://github.com/rynaro/termaup/releases/download/v{VERSION}/termaup-v{VERSION}-{target}.tar.gz`. SHA256 values come from the `SHA256SUMS` file in each release. The formula lives in a SEPARATE repository (`rynaro/homebrew-tap`), not in the termaup repo.
- **Gates:**
  - [ ] P0: Formula is valid Ruby syntax
  - [ ] P1: `brew audit --strict` passes
  - [ ] P2: Both binaries are installed correctly

---

### S-2: CI Auto-Update Formula

**As a** maintainer, **I want** the release workflow to automatically update the Homebrew formula when a new version is released, **so that** Homebrew users always get the latest version without manual intervention.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1, F24-S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add update-homebrew job | `.github/workflows/release.yml` | New job `update-homebrew` that runs after `release` job |
| 2 | Extract checksums | `.github/workflows/release.yml` | Parse `SHA256SUMS` for macOS x86_64 and aarch64 archives |
| 3 | Clone tap repo | `.github/workflows/release.yml` | `git clone https://x-access-token:${{ secrets.HOMEBREW_TAP_TOKEN }}@github.com/rynaro/homebrew-tap.git` |
| 4 | Update formula via sed | `.github/workflows/release.yml` | Replace version, SHA256 values, and URLs in `Formula/termaup.rb` |
| 5 | Commit and push | `.github/workflows/release.yml` | Commit with message `chore: bump termaup to {version}` and push |
| 6 | Add repository secret | GitHub Settings | `HOMEBREW_TAP_TOKEN` — a PAT with repo scope for the tap repo |

#### Acceptance Criteria

- **GIVEN** a new GitHub Release `v0.3.0` is published, **WHEN** the `update-homebrew` job runs, **THEN** `Formula/termaup.rb` in `rynaro/homebrew-tap` is updated with the new version and checksums.
- **GIVEN** the updated formula, **WHEN** a user runs `brew upgrade termaup`, **THEN** they get the new version.
- **GIVEN** the `HOMEBREW_TAP_TOKEN` secret is missing, **WHEN** the job runs, **THEN** it fails with a clear error (not a silent hang).
- **GIVEN** a pre-release tag (`v0.3.0-rc1`), **WHEN** the release workflow runs, **THEN** the Homebrew formula is NOT updated (only stable releases).

#### Agent Hints

- **Class:** builder
- **Context:** Use `sed` or a small script to replace version and SHA256 strings in the formula. The formula has predictable structure — use sed with version-anchored patterns. Alternatively, use `envsubst` with a template. The job should be conditioned on `if: ${{ !contains(github.ref, '-rc') && !contains(github.ref, '-beta') }}`.
- **Gates:**
  - [ ] P0: Workflow YAML is valid
  - [ ] P1: Formula is correctly updated with new version + checksums
  - [ ] P2: Pre-release exclusion works

---

## Execution Sequence

```
F24 (release pipeline) → S-1 (formula) → S-2 (auto-update)
```

S-1 can be developed in parallel with F24 using placeholder URLs, but cannot be tested end-to-end until F24 ships.

## Assumptions

1. **The tap repository will be `rynaro/homebrew-tap`**. Risk if wrong: update all URLs and references. Low risk — this is a naming decision.
2. **GitHub Actions can push to external repos** using a PAT secret. Risk if wrong: none — this is a standard GitHub Actions pattern.
3. **macOS is the only Homebrew platform needed initially**. Risk if wrong: can add Linux support later with `on_linux` blocks.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | Single formula file + one CI job |
| Ambiguity | 1 | Homebrew formula conventions are well-documented |
| Dependencies | 1 | Depends on F24 but the interface is clear (release URLs + checksums) |
| Risk | 0 | No Rust code changes; formula is declarative and easily tested |

**Total: 3/12** → Lightweight processing
