---
name: foundation
description: Scaffold the Cargo workspace, Docker, and CI/CD pipeline for clickup-rs. Use this when setting up project infrastructure, creating the workspace root, or configuring build tooling.
---

# Foundation

## Context

This skill bootstraps the clickup-rs project from an empty repository into a compiling Cargo workspace with three member crates, containerization, and continuous integration. It is the first skill executed and has no prerequisites.

## Crates

- Workspace root (all crates)
- `crates/clickup-api` (library)
- `crates/clickup-cli` (binary)
- `crates/clickup-tui` (binary)

## Prerequisites

- None — this is the first skill

## Deliverables

- `Cargo.toml` — Workspace root with `[workspace.dependencies]` defining all shared dependencies
- `crates/clickup-api/Cargo.toml` — Library crate with `[dependencies]` referencing workspace deps
- `crates/clickup-api/src/lib.rs` — Empty library root
- `crates/clickup-cli/Cargo.toml` — Binary crate depending on `clickup-api`
- `crates/clickup-cli/src/main.rs` — Minimal main with tokio runtime
- `crates/clickup-tui/Cargo.toml` — Binary crate depending on `clickup-api`
- `crates/clickup-tui/src/main.rs` — Minimal main with tokio runtime
- `Dockerfile` — Multi-stage build following [Docker guide](./docker-guide.md)
- `docker-compose.yml` — Development compose config
- `.github/workflows/ci.yml` — CI pipeline following [CI guide](./ci-guide.md)
- `rustfmt.toml` — `max_width = 100`, `edition = "2024"`
- `clippy.toml` — Workspace-level clippy config
- `.gitignore` — Rust + IDE patterns
- `LICENSE` — MIT license
- `README.md` — Project stub with badges

## Implementation

1. Create root `Cargo.toml` with `[workspace]` members pointing to `crates/*`
2. Define ALL shared dependencies in `[workspace.dependencies]` — see AGENTS.md §Dependency Stack for versions
3. Create each member crate with `cargo init` or manual `Cargo.toml` + `src/`
4. Member crates reference shared deps as `dependency.workspace = true`
5. Create `Dockerfile` following the [Docker guide](./docker-guide.md)
6. Create `docker-compose.yml` following the [Docker guide](./docker-guide.md)
7. Create `.github/workflows/ci.yml` following the [CI guide](./ci-guide.md)
8. Create tooling configs (`rustfmt.toml`, `clippy.toml`, `.gitignore`)

## Acceptance

- `cargo build --workspace` succeeds with zero warnings
- `cargo clippy --workspace -- -D warnings` passes
- `cargo fmt --all -- --check` passes
- `docker build .` succeeds
- CI workflow is valid YAML
