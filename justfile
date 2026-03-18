# termaup — justfile for common development tasks

# Build all workspace crates
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Run clippy lints (zero warnings)
lint:
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Full pre-commit check: format, lint, test
check: fmt-check lint test

# Build release binaries
release:
    cargo build --workspace --release

# Build Docker image
docker:
    docker compose build

# Install the CLI binary
install-cli:
    cargo install --path crates/clickup-cli

# Install the TUI binary
install-tui:
    cargo install --path crates/clickup-tui

# Install both binaries
install: install-cli install-tui

# Run the CLI (pass args after --)
cli *ARGS:
    cargo run -p clickup-cli -- {{ARGS}}

# Run the TUI
tui:
    cargo run -p clickup-tui

# Clean build artifacts
clean:
    cargo clean
