---
name: tui-tasks
description: Implement list view, task detail, and markdown rendering in clickup-tui. Use this when building task display screens or the markdown rendering widget.
---

# TUI List & Task Screens

## Context

This skill creates the list view screen (showing folders, lists, and tasks in a navigable table) and the task detail screen (showing full task metadata with a scrollable rendered markdown description). It also implements the markdown-to-ratatui converter.

## Crates

- `clickup-tui`

## Prerequisites

- Skill `tui-workspaces` completed — data loading, input routing, and basic screens exist

## Deliverables

- `crates/clickup-tui/src/ui/list_view.rs` — Folder/list hierarchy + task table
- `crates/clickup-tui/src/ui/task_detail.rs` — Full task view with metadata + scrollable markdown + comments
- `crates/clickup-tui/src/widgets/markdown.rs` — `pulldown-cmark` → ratatui `Line`/`Span` converter

## Implementation

### ui/list_view.rs

Two-panel layout:
- Left panel (30%): folder/list tree — folders as expandable groups, lists as selectable items
- Right panel (70%): task table for the selected list

Task table columns:
| Column | Width | Content |
|--------|-------|---------|
| Status | 12 | Colored status text matching ClickUp status color |
| Priority | 3 | Colored indicator (🔴🟠🟡🔵) |
| Name | flex | Task name (truncated to fit) |
| Assignee | 15 | First assignee initials or "—" |
| Due | 12 | Formatted date or "—" |

- `Enter` on a task: switch to `TaskDetail`, load full task with markdown description
- Load tasks for the selected list using pagination (all pages)

### ui/task_detail.rs

Vertical layout:
1. **Header block** (bordered): Task name, status badge, priority badge
2. **Metadata block**: Assignees, dates (created, due, closed), tags, list/folder/space path, URL
3. **Description block** (scrollable): Rendered markdown using `widgets/markdown.rs`
4. **Comments section**: List of comments with author and timestamp

- Scrolling: `j`/`↓` scroll down, `k`/`↑` scroll up, `g` go to top, `G` go to bottom
- `Esc`: go back to list view

### widgets/markdown.rs

Convert markdown to ratatui styled text. See [markdown rendering spec](./markdown-rendering.md) for the complete mapping rules.

```rust
pub fn markdown_to_lines(md: &str) -> Vec<Line<'static>>
```

Use `pulldown-cmark::Parser` to iterate over markdown events and build `Vec<Line>` with styled `Span`s.

## Acceptance

- Space → list view shows folder/list hierarchy on the left, tasks on the right
- Task table shows colored status, priority indicators, and formatted dates
- `Enter` on a task shows full task detail with rendered markdown description
- Markdown headings, bold, italic, code, lists, and links render correctly
- Task description scrolls smoothly with `j`/`k`
- `Esc` returns to list view without data loss
- Comments display with author name and timestamp
