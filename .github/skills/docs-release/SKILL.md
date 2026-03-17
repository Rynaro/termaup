---
name: docs-release
description: Create documentation, issue templates, release workflows, and test suites for clickup-rs. Use this when preparing for release or writing project documentation.
---

# Documentation & Release

## Context

This skill creates all the documentation, testing infrastructure, and release automation needed to ship clickup-rs as a public project.

## Crates

- All (workspace-wide)

## Prerequisites

- Skill `cli-data` completed — CLI is functional
- Skill `tui-polish` completed — TUI is functional

## Deliverables

- `README.md` — Comprehensive project documentation
- `CONTRIBUTING.md` — Contributor guidelines
- `.github/ISSUE_TEMPLATE/bug_report.md` — Bug report template
- `.github/ISSUE_TEMPLATE/feature_request.md` — Feature request template
- `.github/workflows/release.yml` — Cross-compile + GitHub Release workflow
- Integration test suite with JSON fixtures
- Snapshot tests for CLI output (using `insta`)
- TUI render tests with `ratatui::backend::TestBackend`
- `justfile` — Common development recipes

## Implementation

### README.md

Structure:
1. Project banner/logo + badges (CI, crate version, license)
2. One-paragraph description
3. Screenshots/GIFs of CLI and TUI in action
4. Installation (cargo install, GitHub releases, Docker)
5. Quick start (login, list tasks, open TUI)
6. CLI command reference (all commands with examples)
7. TUI keybindings table
8. Configuration reference
9. Architecture overview (crate diagram)
10. Contributing link
11. License

### Release workflow

See [release matrix](./release-matrix.md) for platform targets and artifact format.

### justfile

```just
default:
    just --list

check:
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings

test:
    cargo test --workspace

build:
    cargo build --workspace --release

ci: check test build
```

### Test suites

- **API integration tests**: `crates/clickup-api/tests/` — wiremock-based tests for each endpoint
- **CLI snapshot tests**: `crates/clickup-cli/tests/` — `insta` snapshots for formatted output
- **TUI render tests**: `crates/clickup-tui/tests/` — `TestBackend` rendering verification
- All test fixtures in `tests/fixtures/` relative to each crate

## Acceptance

- README is complete with installation, usage, keybindings, and architecture sections
- CONTRIBUTING.md covers: setup, coding standards, PR process, testing
- Issue templates render correctly on GitHub
- Release workflow produces binaries for all targets in the release matrix
- All snapshot tests pass and produce readable inline snapshots
- `just ci` runs all quality checks in order
