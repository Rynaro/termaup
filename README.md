# 🚀 termaup

[![CI](https://github.com/YOUR_USER/termaup/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USER/termaup/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**A blazing-fast CLI and TUI client for [ClickUp](https://clickup.com), built in Rust.**

termaup brings the full power of ClickUp to your terminal — navigate workspaces, browse tasks, and view rich markdown descriptions without ever leaving the command line.

## ✨ Features

- 🖥️ **CLI** — Fast command-line access to ClickUp workspaces, spaces, lists, and tasks
- 🎨 **TUI** — Beautiful interactive terminal UI with keyboard navigation and real-time filtering
- 📝 **Rich Markdown** — Task descriptions rendered with full markdown formatting in the terminal
- 🔍 **Search & Filter** — Instantly filter workspaces, spaces, and tasks by name
- 🔐 **Secure Auth** — API tokens stored in your OS keychain (with file fallback)
- 🎯 **Multiple Output Formats** — Table, JSON, and markdown output for CLI commands
- ⚡ **Async & Fast** — Built on tokio with connection pooling and rate limit handling
- 🐳 **Docker Ready** — Run in containers with config volume mounting

## 📸 Screenshots

<!-- TODO: Add GIF recordings of the TUI and CLI in action -->
<!-- Use a tool like `vhs` (https://github.com/charmbracelet/vhs) to record terminal sessions -->

## 📦 Installation

### From source

```sh
# Install the CLI
cargo install --path crates/clickup-cli

# Install the TUI
cargo install --path crates/clickup-tui
```

### Docker

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

### Binary releases

Download pre-built binaries from the [GitHub Releases](https://github.com/YOUR_USER/termaup/releases) page.

### Homebrew (coming soon)

```sh
# brew install termaup
```

## 🚀 Quick Start

1. **Authenticate** with your ClickUp API token ([get one here](https://app.clickup.com/settings/apps)):

   ```sh
   clickup auth login --token pk_YOUR_TOKEN
   ```

2. **List your spaces:**

   ```sh
   clickup space list
   ```

3. **Browse tasks in a list:**

   ```sh
   clickup task list --list LIST_ID
   ```

4. **View a task with rich markdown:**

   ```sh
   clickup task view TASK_ID
   ```

5. **Launch the TUI** for interactive browsing:

   ```sh
   clickup-tui
   ```

## 📖 CLI Reference

### Authentication

| Command | Description |
|---------|-------------|
| `clickup auth login [--token TOKEN]` | Authenticate with ClickUp (prompts for token if not provided) |
| `clickup auth status` | Show current authentication status |
| `clickup auth logout` | Remove stored credentials |
| `clickup auth switch [--workspace ID]` | Switch default workspace |

### Browsing

| Command | Description |
|---------|-------------|
| `clickup workspace list` | List all workspaces |
| `clickup space list [--workspace ID]` | List spaces in a workspace |
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
| `↑` / `k` | Move up / scroll up |
| `↓` / `j` | Move down / scroll down |
| `Enter` | Select / open item |
| `Esc` | Go back one screen |
| `/` | Search / filter current list |
| `r` | Refresh current data |
| `?` | Toggle help overlay |
| `q` / `Ctrl+C` | Quit |

**TUI Navigation flow:** Workspaces → Spaces → Tasks → Task Detail

## ⚙️ Configuration

### Config file

Located at `~/.config/clickup-rs/config.toml`:

```toml
default_workspace_id = "your_workspace_id"
api_base_url = "https://api.clickup.com/api/v2"  # optional override
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `CLICKUP_TOKEN` | Override stored API token |
| `CLICKUP_LOG` | Log level filter (`error`, `warn`, `info`, `debug`, `trace`) |

### Debugging

```sh
# Verbose CLI output
CLICKUP_LOG=debug clickup task list --list LIST_ID

# Maximum verbosity
CLICKUP_LOG=trace clickup space list

# TUI logs (written to file since TUI uses the terminal)
tail -f ~/.config/clickup-rs/tui.log
```

## 🛠️ `bin/` Scripts

Convenience scripts that work without installing Rust (Docker only):

| Script | Description |
|--------|-------------|
| `bin/setup` | Build Docker image and prepare config directory |
| `bin/clickup` | Run any CLI command via Docker |
| `bin/termaup` | Launch the TUI via Docker |
| `bin/dev <cmd>` | Development helper (`build`, `test`, `lint`, `fmt`, `check`, `release`, `clean`) |
| `bin/docker-build` | Rebuild the Docker image |
| `bin/test` | Run the test suite |

## 🏗️ Architecture

termaup is organized as a Cargo workspace with three crates:

| Crate | Type | Description |
|-------|------|-------------|
| `clickup-api` | Library | Pure async ClickUp API v2 client with typed endpoints, rate limiting, and pagination |
| `clickup-cli` | Binary | clap-based CLI with table, JSON, and markdown output |
| `clickup-tui` | Binary | ratatui-based TUI with async data loading and markdown rendering |

Dependency flow: `clickup-cli` and `clickup-tui` both depend on `clickup-api`. The two binaries never depend on each other.

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and PR guidelines.

## 📄 License

MIT — see [LICENSE](LICENSE) for details.
