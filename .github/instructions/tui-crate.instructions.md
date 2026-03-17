---
applyTo: "crates/clickup-tui/**/*.rs"
---

# clickup-tui — Coding Rules

## Context

The `clickup-tui` binary crate is the terminal user interface for clickup-rs.

## Constraints

- Use `ratatui` 0.29 for rendering and `crossterm` 0.28 for terminal control
- The app uses an event-driven architecture with `tokio::sync::mpsc` channels
- NEVER block the main render thread — all API calls happen via `tokio::spawn`
- The main loop: poll events → update state → render frame
- Each screen is a separate module in `src/ui/` with a `render(app: &App, frame: &mut Frame, area: Rect)` function
- Screen-specific key handling goes in `src/input.rs`, dispatched by current `Screen` enum variant

## Patterns

### App state

- `App` struct holds ALL state — screens don't have their own state
- `Screen` enum: `Loading`, `WorkspaceSelect`, `SpaceList`, `ListView`, `TaskDetail`
- Selection state: `selected_index: usize` with wrapping navigation
- Breadcrumb: `Vec<String>` pushed/popped on screen transitions
- Data caching: loaded data stays in `App` so back-navigation doesn't re-fetch

### Navigation keys

| Key | Action |
|-----|--------|
| `j` / `↓` | Select next item |
| `k` / `↑` | Select previous item |
| `Enter` | Drill into selected item |
| `Esc` | Go back one screen |
| `q` / `Ctrl+C` | Quit |
| `/` | Open search filter |
| `?` | Toggle help overlay |
| `r` | Refresh current data |

### Layout

- Top bar (1 row): breadcrumb path
- Main area: screen content
- Bottom bar (1 row): key hints for current screen

### Terminal management

- Enter alternate screen and enable raw mode on start
- Restore terminal on quit AND on panic (install a panic hook)
- TUI logging goes to file (`~/.config/clickup-rs/tui.log`), never to stdout/stderr
