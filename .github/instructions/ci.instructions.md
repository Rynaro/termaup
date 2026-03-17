---
applyTo: ".github/workflows/**"
---

# CI/CD Instructions

## GitHub Actions Workflows
- Use `actions/checkout@v4`
- Use `dtolnay/rust-toolchain@stable` with components: rustfmt, clippy
- Cache cargo registry and target directory with `actions/cache@v4`
- CI job must run: fmt check → clippy → test → build (in that order, fail fast)
- Release workflow triggers on `v*` tags
- Cross-compilation uses `cross` for multi-platform builds
- Release artifacts: tar.gz for Linux/macOS, zip for Windows
- Generate SHA256 checksums for all artifacts
