---
applyTo: "crates/clickup-tui/**/*.rs"
---

# TUI Crate Instructions

You are working in the `clickup-tui` binary crate. This is the terminal user interface for clickup-rs.

## Rules
- Use `ratatui` 0.29 for rendering and `crossterm` 0.28 for terminal control
- The app uses an event-driven architecture with `tokio::sync::mpsc` channels
- NEVER block the main render thread — all API calls happen via `tokio::spawn`
- The main loop: poll events → update state → render frame
- Each screen is a separate module in `src/ui/` with a `render(app: &App, frame: &mut Frame, area: Rect)` function
- Screen-specific key handling goes in `src/input.rs`, dispatched by current `Screen` enum variant

## App State
- `App` struct holds ALL state — screens don't have their own state
- `Screen` enum: Loading, WorkspaceSelect, SpaceList, ListView, TaskDetail
- Selection state: `selected_index: usize` with wrapping navigation
- Breadcrumb: `Vec<String>` pushed/popped on screen transitions
- Data caching: loaded data stays in App so back-navigation doesn't re-fetch

## Navigation
- `j` / `↓`: select next item
- `k` / `↑`: select previous item
- `Enter`: drill into selected item
- `Esc`: go back one screen
- `q` or `Ctrl+C`: quit
- `/`: open search filter
- `?`: toggle help overlay
- `r`: refresh current data

## Layout
- Top bar (1 row): breadcrumb path
- Main area: screen content
- Bottom bar (1 row): key hints for current screen

## Markdown Rendering
- Parse markdown using `pulldown-cmark`
- Convert pulldown-cmark events to `Vec<ratatui::text::Line>` with styled `Span`s
- Headers: Bold + color (H1=Blue, H2=Cyan, H3=White)
- Bold/Italic: corresponding `Style` modifiers
- Code spans: gray background
- Code blocks: bordered block
- Lists: indented with "•" or numbered
- Links: underlined blue
- Horizontal rules: line of "─" characters

## Terminal Management
- Enter alternate screen and enable raw mode on start
- Restore terminal on quit AND on panic (install a panic hook)
- TUI logging goes to file (`~/.config/clickup-rs/tui.log`), never to stdout/stderr
