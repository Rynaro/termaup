# F24 — Pre-built Binaries & Release Pipeline

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 6 — Distribution & Infrastructure
> **Complexity:** 5/12 | **Confidence:** 90%

---

## Problem Statement

Users currently must compile termaup from source with a full Rust toolchain. This is a high barrier to entry for non-Rust developers and makes termaup impractical for quick evaluation. A release pipeline with pre-built binaries is essential for adoption and is a prerequisite for Homebrew distribution (F25).

## Approach

Create a GitHub Actions workflow triggered on `v*` git tags that cross-compiles both binaries (`clickup` for CLI, `termaup` for TUI) across 5 target platforms, generates SHA256 checksums, extracts a changelog from git history or a `CHANGELOG.md`, and publishes everything as a GitHub Release.

Use the `cross` tool for Linux cross-compilation (musl targets) and native GitHub Actions runners for macOS and Windows. Each artifact is a compressed tarball (`.tar.gz` for Unix, `.zip` for Windows) containing the binary and a `README.md`.

### Rejected Alternatives

1. **cargo-dist** — Opinionated release tool that generates installers and manages the workflow automatically. While convenient, it adds significant complexity, generates boilerplate that's hard to customize, and is another moving dependency. Rejected for maintainability — a hand-crafted workflow is simpler for this project's scale.

2. **Nix flake builds** — Reproducible builds via Nix. Adds Nix as a build dependency, which has a steep learning curve. Rejected — can be added later as an optional distribution channel.

3. **Single-binary approach** — Combine CLI + TUI into one binary with subcommands. This conflicts with the existing workspace design where `clickup-cli` and `clickup-tui` are separate crates with independent dependency trees. Rejected to preserve crate separation.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| GitHub Actions release workflow | Nix/Snap/Flatpak packaging | AUR package |
| 5 target platforms (macOS x86_64 + aarch64, Linux x86_64 + aarch64 musl, Windows x86_64) | Windows ARM | Docker release images |
| SHA256 checksum generation | Code signing / notarization | Automatic CHANGELOG generation |
| GitHub Release creation with assets | MSI / .deb / .rpm installers | GitHub Release notes from conventional commits |
| Archive naming convention | | |

## Stories

### S-1: Release CI Workflow

**As a** maintainer, **I want** a GitHub Actions workflow that builds release binaries when a version tag is pushed, **so that** I don't have to manually build and upload artifacts for each release.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create workflow file | `.github/workflows/release.yml` | Trigger on `push: tags: ['v*']` |
| 2 | Define build matrix | `.github/workflows/release.yml` | Matrix strategy with 5 targets: `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc` |
| 3 | Configure runners | `.github/workflows/release.yml` | macOS: `macos-latest` (aarch64 native). Linux: `ubuntu-latest` with `cross`. Windows: `windows-latest` |
| 4 | Build step | `.github/workflows/release.yml` | `cargo build --release --target ${{ matrix.target }}` for native targets, `cross build --release --target ${{ matrix.target }}` for cross-compiled |
| 5 | Install cross | `.github/workflows/release.yml` | `cargo install cross` step for Linux cross-compilation jobs |
| 6 | Rename binaries | `.github/workflows/release.yml` | Copy to `clickup-{target}` and `termaup-{target}` naming |
| 7 | Upload artifacts | `.github/workflows/release.yml` | Use `actions/upload-artifact@v4` to persist binaries between jobs |

#### Acceptance Criteria

- **GIVEN** a push of tag `v0.2.0`, **WHEN** the workflow runs, **THEN** it builds both `clickup` and `termaup` binaries for all 5 targets.
- **GIVEN** a push to `main` branch (no tag), **WHEN** CI runs, **THEN** the release workflow does NOT trigger.
- **GIVEN** a build failure on one target, **WHEN** the matrix runs, **THEN** the other targets continue (fail-fast: false) but the release step does not proceed.

#### Agent Hints

- **Class:** builder
- **Context:** The workspace has two binary crates: `crates/clickup-cli` (produces `clickup` binary) and `crates/clickup-tui` (produces `termaup` binary). Check `Cargo.toml` in each for the `[[bin]]` section or package name. For macOS aarch64, `macos-14` or `macos-latest` provides native ARM runners.
- **Gates:**
  - [ ] P0: Workflow YAML is valid
  - [ ] P1: Matrix covers all 5 targets
  - [ ] P2: `cargo fmt` + `cargo clippy` are not affected (no Rust changes)

---

### S-2: Cross-Compilation Matrix

**As a** maintainer, **I want** each target to produce correct binaries with proper naming and compression, **so that** users can download the right binary for their platform.

**Timebox:** ≤1d | **Risk:** Medium (cross-compilation edge cases) | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Define archive step | `.github/workflows/release.yml` | For Unix: `tar czf clickup-{version}-{target}.tar.gz clickup termaup`. For Windows: `Compress-Archive` for `.zip` |
| 2 | Handle binary extensions | `.github/workflows/release.yml` | Windows binaries have `.exe` extension |
| 3 | Add musl build deps | `.github/workflows/release.yml` | Install `musl-tools` on Linux runners for `x86_64-unknown-linux-musl` |
| 4 | Set Rust toolchain targets | `.github/workflows/release.yml` | `rustup target add ${{ matrix.target }}` for native builds |
| 5 | Test binary execution | `.github/workflows/release.yml` | For native targets: run `./clickup --version` and `./termaup --version` as smoke test |
| 6 | Archive naming convention | `.github/workflows/release.yml` | `termaup-{version}-{target}.tar.gz` (e.g., `termaup-v0.2.0-x86_64-apple-darwin.tar.gz`) |

#### Acceptance Criteria

- **GIVEN** the Linux musl target, **WHEN** built, **THEN** the binary is statically linked and runs on any Linux distribution.
- **GIVEN** each target, **WHEN** the build completes, **THEN** the archive contains both `clickup` and `termaup` binaries.
- **GIVEN** the Windows target, **WHEN** the build completes, **THEN** the archive is `.zip` format and contains `clickup.exe` and `termaup.exe`.
- **GIVEN** native runner targets, **WHEN** the binary is built, **THEN** `--version` outputs the correct version.

#### Agent Hints

- **Class:** builder
- **Context:** Use matrix `include` to associate each target with its runner, build tool (`cargo` vs `cross`), archive format, and binary suffix. The version can be extracted from the tag: `echo ${GITHUB_REF#refs/tags/}`.
- **Gates:**
  - [ ] P0: Workflow YAML is valid
  - [ ] P1: All 5 targets produce archives
  - [ ] P2: Naming convention is consistent

---

### S-3: Release Automation with Checksums

**As a** user downloading pre-built binaries, **I want** SHA256 checksums published alongside the release, **so that** I can verify the integrity of downloaded files.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Download all artifacts | `.github/workflows/release.yml` | New job `release` that depends on all build jobs, uses `actions/download-artifact@v4` |
| 2 | Generate checksums | `.github/workflows/release.yml` | `shasum -a 256 *.tar.gz *.zip > SHA256SUMS` |
| 3 | Create GitHub Release | `.github/workflows/release.yml` | Use `softprops/action-gh-release@v2` with all archives + `SHA256SUMS` as assets |
| 4 | Release body template | `.github/workflows/release.yml` | Include installation instructions, checksum verification command, and link to CHANGELOG |
| 5 | Mark as pre-release | `.github/workflows/release.yml` | Tags matching `v*-rc*` or `v*-beta*` are marked as pre-release |

#### Acceptance Criteria

- **GIVEN** all build targets succeed, **WHEN** the release job runs, **THEN** a GitHub Release is created with the tag name as title.
- **GIVEN** a release is created, **WHEN** assets are inspected, **THEN** it contains 5 archives (4 `.tar.gz` + 1 `.zip`) plus `SHA256SUMS`.
- **GIVEN** a downloaded archive, **WHEN** verified against `SHA256SUMS`, **THEN** `shasum -c SHA256SUMS` succeeds.
- **GIVEN** tag `v0.2.0-rc1`, **WHEN** the release is created, **THEN** it is marked as pre-release.
- **GIVEN** tag `v0.2.0`, **WHEN** the release is created, **THEN** it is NOT marked as pre-release.

#### Agent Hints

- **Class:** builder
- **Context:** The `softprops/action-gh-release` action supports `files` glob patterns and `prerelease` boolean. Use `${{ contains(github.ref, '-rc') || contains(github.ref, '-beta') }}` for pre-release detection.
- **Gates:**
  - [ ] P0: Workflow YAML is valid
  - [ ] P1: Release is created with all assets
  - [ ] P2: SHA256SUMS file is correct and verifiable

---

## Execution Sequence

```
S-1 (CI workflow) → S-2 (cross-compilation) → S-3 (release + checksums)
```

Linear chain — each story extends the workflow file.

## Assumptions

1. **GitHub Actions macOS runners** provide both x86_64 and aarch64 natively. Risk if wrong: may need `cross` for macOS aarch64 as well. Mitigation: `macos-14` runners are aarch64-native.
2. **`cross` handles aarch64-unknown-linux-musl** correctly. Risk if wrong: may need a custom Docker image for cross. Mitigation: test the cross build locally before relying on CI.
3. **Both binaries can be built in the same cargo invocation** per target. Risk if wrong: may need separate build steps. Mitigation: `cargo build --release --target T` builds all workspace members.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Single workflow file, but complex matrix with 5 targets |
| Ambiguity | 1 | Well-established pattern, many open-source examples |
| Dependencies | 1 | External tooling (cross, GitHub Actions) is stable |
| Risk | 1 | No Rust code changes; CI-only, easily iterable |

**Total: 5/12** → Lightweight processing
