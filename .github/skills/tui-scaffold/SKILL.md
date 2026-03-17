---
name: tui-scaffold
description: Build the TUI app skeleton with event loop, state machine, and layout framework for clickup-tui. Use this when scaffolding the TUI crate.
---

# TUI Scaffold

## Context

This skill creates the foundational TUI architecture — the app state machine, event system, terminal management, and layout framework. All subsequent TUI skills build on this scaffold.

## Crates

- `clickup-tui`

## Prerequisites

- Skill `api-endpoints` completed — `ClickUpClient` endpoint methods exist

## Deliverables

- `crates/clickup-tui/src/app.rs` — `App` struct with `Screen` enum, selection state, breadcrumbs
- `crates/clickup-tui/src/event.rs` — `EventHandler` with crossterm polling + tokio mpsc channels
- `crates/clickup-tui/src/main.rs` — Terminal setup/teardown, main loop, panic hook
- `crates/clickup-tui/src/ui/mod.rs` — Main render dispatcher
- `crates/clickup-tui/src/ui/layout.rs` — Standard 3-row layout
- Placeholder screens for all views

## Implementation

### app.rs

```rust
pub enum Screen {
    Loading,
    WorkspaceSelect,
    SpaceList,
    ListView,
    TaskDetail,
}

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub selected_index: usize,
    pub breadcrumbs: Vec<String>,
    // Data caches (populated by later skills)
    pub workspaces: Vec<Workspace>,
    pub spaces: Vec<Space>,
    pub tasks: Vec<Task>,
    pub current_task: Option<Task>,
}
```

- `App` holds ALL state — screens never have their own state
- Selection wraps around (up from 0 goes to last, down from last goes to 0)
- Breadcrumbs pushed on drill-down, popped on back navigation

### event.rs

```rust
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Data(DataPayload),  // For async API responses
}
```

- Use `tokio::sync::mpsc` channel for cross-task communication
- Poll crossterm events with a 250ms tick rate
- Forward key events and data payloads through the channel

### main.rs

1. Initialize tracing to file (`~/.config/clickup-rs/tui.log`)
2. Create `ClickUpClient` from stored token
3. Enter alternate screen + enable raw mode
4. Install panic hook that restores terminal before printing panic info
5. Main loop: `poll event → update state → render frame`
6. On exit: restore terminal (leave alternate screen + disable raw mode)

### ui/layout.rs

Standard 3-row layout:
- Row 0 (1 line): breadcrumb bar showing navigation path
- Row 1 (flexible): main content area (delegated to screen-specific renderer)
- Row 2 (1 line): key hints bar showing available keys for current screen

### Placeholder screens

Each screen shows centered text: "Screen Name — Coming Soon" with key hint "q: quit | ?: help"

## Acceptance

- TUI launches and shows a placeholder screen
- `q` and `Ctrl+C` quit cleanly
- Terminal is fully restored after quit (no raw mode artifacts)
- Terminal is restored on panic (panic hook works)
- Breadcrumb bar renders at top
- Key hints bar renders at bottom
- Logging writes to file, not stdout/stderr
