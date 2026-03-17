---
name: tui-workspaces
description: Implement workspace and space selection screens with async data loading in clickup-tui. Use this when building the first TUI data screens.
---

# TUI Workspace & Space Screens

## Context

This skill creates the first real data-driven TUI screens — workspace selection and space listing — along with the async data loading system and keyboard input routing.

## Crates

- `clickup-tui`

## Prerequisites

- Skill `tui-scaffold` completed — app skeleton, event loop, layout framework exist

## Deliverables

- `crates/clickup-tui/src/data.rs` — Async loader functions that send `DataPayload` events
- `crates/clickup-tui/src/ui/workspace_select.rs` — Selectable workspace list
- `crates/clickup-tui/src/ui/space_list.rs` — Selectable space list with private indicators
- `crates/clickup-tui/src/input.rs` — Key routing per screen

## Implementation

### data.rs

Async loading pattern — never block the main thread:

```rust
pub fn load_workspaces(client: Arc<ClickUpClient>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        match client.get_workspaces().await {
            Ok(workspaces) => { tx.send(AppEvent::Data(DataPayload::Workspaces(workspaces))).await.ok(); }
            Err(e) => { tx.send(AppEvent::Data(DataPayload::Error(e.to_string()))).await.ok(); }
        }
    });
}
```

- One function per data load: `load_workspaces`, `load_spaces`, `load_tasks`, etc.
- All functions take `Arc<ClickUpClient>` and `mpsc::Sender<AppEvent>`
- Results sent as `DataPayload` variants through the event channel
- Errors sent as `DataPayload::Error(String)` — displayed in the status bar

### input.rs

Dispatch key events based on current `Screen`:

```rust
pub fn handle_key(app: &mut App, key: KeyEvent, /* ... */) {
    match app.screen {
        Screen::WorkspaceSelect => handle_workspace_keys(app, key),
        Screen::SpaceList => handle_space_keys(app, key),
        // ...
    }
}
```

Navigation keys (consistent across all list screens):
- `j` / `↓`: select next item (wrap to top)
- `k` / `↑`: select previous item (wrap to bottom)
- `Enter`: drill into selected item (push breadcrumb, change screen, load data)
- `Esc`: go back one screen (pop breadcrumb)
- `q` / `Ctrl+C`: quit
- `r`: refresh current data

### ui/workspace_select.rs

- Render a `ratatui::widgets::List` of workspace names
- Highlight selected item with inverted colors
- Show member count next to each workspace name
- On `Enter`: store selected workspace, push breadcrumb, switch to `SpaceList`, load spaces

### ui/space_list.rs

- Render a `ratatui::widgets::List` of space names
- Show lock icon (🔒) next to private spaces
- Show space color as a colored dot or prefix
- On `Enter`: store selected space, push breadcrumb, switch to `ListView`, load lists/tasks

### Loading state

- Show a spinner or "Loading..." in the status bar while data is being fetched
- Disable navigation while loading to prevent double-requests

## Acceptance

- Launch TUI → workspaces load automatically → list appears
- `j`/`k` navigate the workspace list with wrapping
- `Enter` on a workspace → spaces load → space list appears
- `Esc` from space list → back to workspace list (no re-fetch)
- `r` refreshes current data
- Loading state is visible during API calls
- Private spaces show a lock indicator
