# Contributing to termaup

Thank you for your interest in contributing! This guide will help you get started.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable toolchain)
- [Docker](https://docs.docker.com/get-docker/) (optional, for containerized builds)
- [just](https://github.com/casey/just) (optional, for task running)

### Getting Started

```sh
# Clone the repository
git clone https://github.com/YOUR_USER/termaup.git
cd termaup

# Option 1: Using bin/ scripts (Docker, no Rust required)
bin/setup              # Build Docker image
bin/dev test           # Run tests
bin/dev check          # Full CI check (fmt + lint + test)

# Option 2: Using cargo directly
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Project Structure

```
termaup/
├── crates/
│   ├── clickup-api/     # Library: async ClickUp API client
│   │   └── src/
│   │       ├── client.rs       # HTTP client with rate limiting
│   │       ├── error.rs        # Error types (thiserror)
│   │       ├── config.rs       # Configuration management
│   │       ├── auth.rs         # Token storage (keyring + file)
│   │       ├── models/         # Serde structs for API responses
│   │       ├── endpoints/      # Typed API endpoint methods
│   │       ├── rate_limiter.rs # Rate limit tracking
│   │       └── pagination.rs   # Automatic pagination
│   ├── clickup-cli/     # Binary: CLI application
│   │   └── src/
│   │       ├── main.rs         # Clap app setup
│   │       ├── commands/       # Command handlers
│   │       ├── output.rs       # Output formatting
│   │       └── client_factory.rs # Client creation helper
│   └── clickup-tui/     # Binary: TUI application
│       └── src/
│           ├── main.rs         # Terminal lifecycle
│           ├── app.rs          # App state machine
│           ├── event.rs        # Event system
│           ├── input.rs        # Key/mouse handling
│           ├── theme.rs        # Color theme
│           ├── data.rs         # Async data loaders
│           ├── ui/             # Screen renderers
│           └── widgets/        # Reusable widgets (markdown)
├── Dockerfile
├── docker-compose.yml
├── justfile
└── .github/workflows/
    ├── ci.yml           # CI checks
    └── release.yml      # Release automation
```

## Code Style

- **Formatting:** Enforced by `rustfmt` (edition 2024, max_width = 100)
- **Linting:** `cargo clippy -- -D warnings` — zero warnings policy
- **Error handling:**
  - Library code (`clickup-api`): use `thiserror` and the crate's `Result<T>` alias
  - Binary code: use `anyhow::Result` with `.context()` for descriptive errors
- **No `unwrap()`** outside of test code
- **Doc comments:** All public items must have `///` doc comments
- **Imports:** Group as std → external → crate-internal, separated by blank lines

## Testing

```sh
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p clickup-api

# Run a specific test
cargo test -p clickup-api test_get_task

# Run with output
cargo test --workspace -- --nocapture
```

### Testing guidelines

- Use `#[tokio::test]` for async tests
- Use `wiremock` for HTTP mocking — never call the real ClickUp API
- Test names should be descriptive: `test_<what>_<condition>_<expected>`
- Fixture JSON files go in `tests/fixtures/` directories

## Pull Request Process

1. **Fork** the repository and create a feature branch
2. **Write tests** for new functionality
3. **Ensure all checks pass:**
   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   cargo build --workspace --release
   ```
4. **Use conventional commits:** `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
5. **Open a PR** with a clear description of the changes

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add task comment support
fix: handle rate limit 429 responses correctly
refactor: extract pagination into helper module
test: add workspace deserialization tests
docs: update CLI reference in README
chore: bump ratatui to 0.29
```

## Reporting Issues

Use the [GitHub issue templates](.github/ISSUE_TEMPLATE/) for:
- 🐛 **Bug reports** — Include reproduction steps and version info
- 💡 **Feature requests** — Describe the use case and proposed solution
