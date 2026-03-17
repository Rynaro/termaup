---
name: cli-data
description: Implement space, list, and task browsing commands with rich output for clickup-cli. Use this when creating data browsing commands in the CLI crate.
---

# CLI Data Commands

## Context

This skill implements all the data browsing commands — workspace listing, space listing, list listing, and task viewing — with three output formats (table, JSON, markdown) and rich formatting.

## Crates

- `clickup-cli`

## Prerequisites

- Skill `cli-auth` completed — `output.rs` and `client_factory.rs` exist

## Deliverables

- `crates/clickup-cli/src/commands/workspaces.rs` — `workspace list`
- `crates/clickup-cli/src/commands/spaces.rs` — `space list`, `space get`
- `crates/clickup-cli/src/commands/lists.rs` — `list list`, `list get`
- `crates/clickup-cli/src/commands/tasks.rs` — `task list`, `task get`, `task view`
- Enhanced `crates/clickup-cli/src/output.rs` — table/JSON/markdown formatters, status/priority/date helpers

## Implementation

### Command structure

All commands follow: `clickup <resource> <action> [options]`

```
clickup workspace list [--format table|json]
clickup space list [--workspace ID] [--format table|json]
clickup space get SPACE_ID [--format table|json]
clickup list list --space SPACE_ID [--folder FOLDER_ID] [--format table|json]
clickup list get LIST_ID [--format table|json]
clickup task list --list LIST_ID [--format table|json]
clickup task get TASK_ID [--format table|json|markdown]
clickup task view TASK_ID  # Rich markdown view
```

### Output format flag

```rust
#[derive(ValueEnum, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Markdown,
}
```

Add `--format` as a global option or per-command option.

### Table output (default)

Use `comfy-table` with:
- Bordered style
- Colored headers (using `owo-colors`)
- Status column: colored to match ClickUp status color
- Priority column: colored indicator (🔴 Urgent, 🟠 High, 🟡 Normal, 🔵 Low)
- Date columns: convert millisecond timestamps to human-readable (e.g., "Mar 16, 2026") using `chrono`

### JSON output

Pretty-print with `serde_json::to_string_pretty` (2-space indent).

### Markdown output

Use `termimad` to render task markdown descriptions in the terminal.

### `task view` (rich view)

Full-width task detail display:
1. Header: task name, status badge, priority badge
2. Metadata: assignees, dates, tags, list/folder/space breadcrumb
3. Description: rendered markdown (via `termimad`)
4. URL: clickable link to ClickUp web UI

## Acceptance

- All commands produce correct output in all three formats
- Tables have colored headers, status indicators, and formatted dates
- JSON output is valid and pretty-printed
- `task view` renders markdown description beautifully in the terminal
- Commands without required IDs show helpful error messages
- `--workspace` flag defaults to `Config.default_workspace_id` when omitted
