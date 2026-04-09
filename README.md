<div align="center">

# 🚀 termaup

**ClickUp in your terminal — a fast CLI and TUI client built in Rust.**

[![CI](https://github.com/Rynaro/termaup/actions/workflows/ci.yml/badge.svg)](https://github.com/Rynaro/termaup/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Status: WIP](https://img.shields.io/badge/Status-Work%20in%20Progress-orange)

> ⚠️ **This project is under active development.** Some features are still being
> built and APIs may change. A public roadmap will be shared soon — stay tuned!

</div>

---

termaup brings the power of [ClickUp](https://clickup.com) to your terminal.
Navigate workspaces, browse tasks, and read rich markdown descriptions without
ever leaving the command line.

Whether you prefer quick one-off commands or an interactive full-screen
experience, termaup has you covered with both a **CLI** and a **TUI**.

## ✨ Features

| | Feature | Status |
|---|---------|--------|
| 🖥️ | **CLI** — Command-line access to workspaces, spaces, lists, and tasks | ✅ Available |
| 🎨 | **TUI** — Interactive terminal UI with keyboard navigation | ✅ Available |
| 📝 | **Rich Markdown** — Task descriptions rendered in the terminal | ✅ Available |
| 🔍 | **Search & Filter** — Instantly filter items by name | ✅ Available |
| 🔐 | **Secure Auth** — Tokens stored in your OS keychain (file fallback) | ✅ Available |
| 🎯 | **Multiple Output Formats** — Table, JSON, and markdown for CLI | ✅ Available |
| ⚡ | **Async & Fast** — tokio runtime, connection pooling, rate-limit handling | ✅ Available |
| 🐳 | **Docker Ready** — Run without installing Rust | ✅ Available |
| 🏷️ | **Task Management** — Checklists, custom fields, time tracking | ✅ Available |
| 📋 | **Homebrew Formula** | 🔜 Coming soon |
| 📦 | **Pre-built Binaries** | 🔜 Coming soon |

## 📦 Installation

### From source (requires [Rust](https://rustup.rs/))

```sh
# Install the CLI
cargo install --path crates/clickup-cli

# Install the TUI
cargo install --path crates/clickup-tui
```

### Docker

No Rust toolchain needed — everything runs in a container:

```sh
# First-time setup
bin/setup

# Run CLI commands
bin/clickup auth login
bin/clickup space list
bin/clickup task view TASK_ID

# Launch the TUI
bin/termaup
```

Or directly with Docker Compose:

```sh
docker compose run --rm clickup auth login
docker compose run --rm clickup-tui
```

## 🚀 Quick Start

1. **Get a ClickUp API token** from your [ClickUp App settings](https://app.clickup.com/settings/apps).

2. **Authenticate:**

   ```sh
   clickup auth login
   # You will be prompted to enter your token securely
   ```

3. **Explore your workspace:**

   ```sh
   clickup workspace list
   clickup space list
   clickup task list --list LIST_ID
   clickup task view TASK_ID
   ```

4. **Or launch the TUI** for an interactive experience:

   ```sh
   clickup-tui
   ```

## 📖 CLI Reference

### Authentication

| Command | Description |
|---------|-------------|
| `clickup auth login` | Authenticate with ClickUp (prompts for token) |
| `clickup auth status` | Show current authentication status |
| `clickup auth logout` | Remove stored credentials |
| `clickup auth switch` | Switch default workspace |

### Browsing

| Command | Description |
|---------|-------------|
| `clickup workspace list` | List all workspaces |
| `clickup space list` | List spaces in a workspace |
| `clickup space get SPACE_ID` | Show space details |
| `clickup list list --space SPACE_ID` | List all lists in a space |
| `clickup list list --folder FOLDER_ID` | List lists in a folder |
| `clickup list get LIST_ID` | Show list details |
| `clickup task list --list LIST_ID` | List tasks in a list |
| `clickup task get TASK_ID` | Show task details |
| `clickup task view TASK_ID` | Rich task view with markdown rendering |

### Global Flags

| Flag | Description |
|------|-------------|
| `--format table\|json\|markdown` | Output format (default: table) |
| `--workspace ID` | Override default workspace |

## ⌨️ TUI Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `Enter` | Open selected item |
| `Esc` | Go back |
| `/` | Search / filter |
| `r` | Refresh data |
| `?` | Toggle help |
| `q` / `Ctrl+C` | Quit |

**Navigation:** Workspaces → Spaces → Lists → Tasks → Task Detail

## ⚙️ Configuration

Config lives at `~/.config/clickup-rs/config.toml`:

```toml
default_workspace_id = "your_workspace_id"
```

| Environment Variable | Description |
|----------------------|-------------|
| `CLICKUP_TOKEN` | Override the stored API token |
| `CLICKUP_LOG` | Set log level (`error`, `warn`, `info`, `debug`, `trace`) |

```sh
# Debug mode
CLICKUP_LOG=debug clickup task list --list LIST_ID

# TUI logs (written to file since TUI owns the terminal)
tail -f ~/.config/clickup-rs/tui.log
```

## 🛠️ Development

Convenience scripts are available in `bin/` for Docker-based workflows:

| Script | Description |
|--------|-------------|
| `bin/setup` | Build Docker image and prepare config directory |
| `bin/clickup` | Run any CLI command via Docker |
| `bin/termaup` | Launch the TUI via Docker |
| `bin/dev <cmd>` | Dev helper: `build`, `test`, `lint`, `fmt`, `check`, `release`, `clean` |
| `bin/test` | Run the full test suite |

Or if you have Rust installed locally:

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## 🏗️ Architecture

termaup is organized as a Cargo workspace with three crates:

```
termaup/
├── crates/
│   ├── clickup-api   # Library — async API client, models, rate limiting, pagination
│   ├── clickup-cli   # Binary — clap-based CLI with rich output formatting
│   └── clickup-tui   # Binary — ratatui-based TUI with async data loading
├── bin/              # Docker convenience scripts
├── docs/             # Additional documentation
└── Cargo.toml        # Workspace root
```

**Dependency flow:** both `clickup-cli` and `clickup-tui` depend on `clickup-api`.
The two binaries are independent of each other.

## ⚠️ Known ClickUp API Limitations

The ClickUp API v2 has some quirks that affect what termaup can do:

- **Thread replies cannot be edited or deleted.** The ClickUp API returns
  `401 "Oauth token not found"` when attempting `PUT` or `DELETE` on threaded
  reply comment IDs. Only top-level task comments support these operations.
  This is a ClickUp API limitation, not an authentication issue.

For the full catalog of API quirks and the workarounds termaup uses, see
[`docs/clickup-api-quirks.md`](docs/clickup-api-quirks.md).

## 🗺️ Roadmap

A detailed public roadmap is coming soon! In the meantime, here's a glimpse of
what's planned:

- Pre-built binaries and Homebrew distribution
- Task creation and editing from the terminal
- OAuth2 authentication flow
- Notification support
- Offline caching

Watch or star this repo to get notified when the roadmap drops. 🌟

## 🤝 Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for
development setup, code style guidelines, and PR conventions.

## 📄 License

[MIT](LICENSE) © termaup contributors
