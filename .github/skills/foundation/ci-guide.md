# CI/CD Guide — clickup-rs

## GitHub Actions CI Pipeline (`.github/workflows/ci.yml`)

### Triggers
- Push to `main` branch
- Pull requests targeting `main`

### Job: `check`

Steps in order (fail fast):

1. **Checkout** — `actions/checkout@v4`
2. **Rust toolchain** — `dtolnay/rust-toolchain@stable` with components: `rustfmt`, `clippy`
3. **Cache** — `actions/cache@v4` caching `~/.cargo/registry`, `~/.cargo/git`, `target/`
4. **Format check** — `cargo fmt --all -- --check`
5. **Clippy** — `cargo clippy --workspace -- -D warnings`
6. **Test** — `cargo test --workspace`
7. **Build** — `cargo build --workspace --release`

### Key conventions
- Use `RUSTFLAGS: -D warnings` to treat all warnings as errors
- Cache key should include `Cargo.lock` hash for invalidation
- Run on `ubuntu-latest`

## Release Workflow (`.github/workflows/release.yml`)

### Triggers
- Push tags matching `v*`

### Build matrix
- See the [release matrix](../docs-release/release-matrix.md) for platform targets

### Steps
1. Cross-compile using `cross` for each platform target
2. Package: `tar.gz` for Linux/macOS, `zip` for Windows
3. Generate SHA256 checksums for all artifacts
4. Create GitHub Release with all artifacts attached
