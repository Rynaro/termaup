# termaup (clickup-rs) — Product Roadmap

> **SPECTRA v4.2.0** — Master roadmap artifact
> **Generated:** 2026-04-10
> **Status:** Living document — updated as features ship

---

## Executive Summary

termaup is a fast CLI and TUI client for ClickUp, written in Rust. The project currently provides **read-only browsing** with full comment CRUD as the only write capability. This roadmap charts the path from "browse & comment" to a **complete terminal-native ClickUp experience** that can replace the web UI for power users.

---

## Current State Assessment

### What's Built

#### API Crate (`clickup-api`) — 13 endpoints, 27 model structs

| Category | Endpoints | Methods |
|----------|-----------|---------|
| Authentication | `GET /user` | `get_authenticated_user()` |
| Workspaces | `GET /team` | `get_workspaces()` |
| Spaces | `GET /team/{id}/space`, `GET /space/{id}` | `get_spaces()`, `get_space()` |
| Folders | `GET /space/{id}/folder` | `get_folders()` |
| Lists | `GET /folder/{id}/list`, `GET /space/{id}/list`, `GET /list/{id}` | `get_lists_in_folder()`, `get_folderless_lists()`, `get_list()` |
| Tasks | `GET /list/{id}/task` (paginated+filtered), `GET /task/{id}` | `get_tasks()`, `get_tasks_with_filters()`, `get_tasks_page()`, `get_task()` |
| Comments | Full CRUD + threading | `get_task_comments()`, `get_comment_replies()`, `create_task_comment()`, `create_comment_reply()`, `update_comment()`, `delete_comment()` |

**Infrastructure:** Rate limiter, auto-pagination, defensive serde deserialization (13 known API quirks handled), OS keyring token storage, config system.

#### CLI Crate (`clickup-cli`) — 5 command groups, 11 subcommands

| Command | Subcommands | Status |
|---------|-------------|--------|
| `auth` | login, status, logout, switch | ✅ Integrated |
| `workspace` | list | ✅ Integrated |
| `space` | list, get | ✅ Integrated |
| `list` | list, get | ✅ Integrated |
| `task` | list, get, view | ✅ Integrated |
| `comment` | list, create, reply, edit, delete | ⚠️ Code exists, NOT wired in main.rs |
| `logs` | list, show, export, clean | ✅ Integrated |

**Output:** 3 formats (table, JSON, markdown), colored status/priority, date formatting, attachment type icons.

#### TUI Crate (`clickup-tui`) — 6 screens, full feature set

| Screen | Features |
|--------|----------|
| WorkspaceSelect | List workspaces, color-coded, member count |
| SpaceList | List spaces, private indicator, status count |
| SpaceContent | Expandable folder tree, folderless lists, task counts |
| TaskList | 3 view modes (List/VisionSections/VisionBoard), pagination, filters, search, Me Mode |
| TaskDetail | Full metadata, markdown description, scroll, custom fields |
| Comment Sidebar | Full CRUD, threading, @mention picker, multi-line compose |

**Infrastructure:** Event-driven architecture, async data loading, mouse support, help overlay, filter persistence, error handling with auto-dismiss, theme system (Cozy/Sober).

### Existing Feature Plans (In Progress)

| Plan | Status | Scope |
|------|--------|-------|
| `2025-07-25-mention-users` | Implemented | @mention support in comments |
| `2026-04-09-comment-compose-keystrokes` | Planned | Enter/Alt+Enter remap for comment compose |
| `2026-04-10-custom-field-display` | Planned | Human-readable custom field values |

---

## Gap Analysis: ClickUp API v2 Coverage

### Endpoints NOT Implemented

| Category | Missing Endpoints | Priority |
|----------|-------------------|----------|
| **Task CRUD** | `POST /list/{id}/task`, `PUT /task/{id}`, `DELETE /task/{id}` | 🔴 Critical |
| **Custom Fields** | `GET /list/{id}/field`, `POST /task/{id}/field/{field_id}` | 🔴 Critical |
| **Time Tracking** | `GET /task/{id}/time`, `POST /task/{id}/time` | 🟠 High |
| **Tags** | `GET /space/{id}/tag`, `POST /task/{id}/tag/{name}`, `DELETE /task/{id}/tag/{name}` | 🟠 High |
| **Checklists** | `POST /task/{id}/checklist`, `PUT /checklist/{id}`, `DELETE /checklist/{id}`, checklist items CRUD | 🟠 High |
| **Dependencies** | `POST /task/{id}/dependency`, `DELETE /task/{id}/dependency` | 🟡 Medium |
| **Linked Tasks** | `POST /task/{id}/link/{links_to}`, `DELETE /task/{id}/link/{links_to}` | 🟡 Medium |
| **Attachments** | `POST /task/{id}/attachment` (multipart upload) | 🟡 Medium |
| **Space CRUD** | `POST /team/{id}/space`, `PUT /space/{id}`, `DELETE /space/{id}` | 🟡 Medium |
| **Folder CRUD** | `POST /space/{id}/folder`, `PUT /folder/{id}`, `DELETE /folder/{id}` | 🟡 Medium |
| **List CRUD** | `POST /folder/{id}/list`, `POST /space/{id}/list`, `PUT /list/{id}`, `DELETE /list/{id}` | 🟡 Medium |
| **Views** | `GET /team/{id}/view`, `GET /space/{id}/view`, `GET /list/{id}/view` | 🔵 Low |
| **Goals** | Full CRUD: `GET/POST/PUT/DELETE /goal/{id}`, key results | 🔵 Low |
| **Docs** | ClickUp Docs API (v2 limited, v3 upcoming) | 🔵 Low |
| **Webhooks** | `POST /team/{id}/webhook`, `DELETE /webhook/{id}` | 🔵 Low |
| **Templates** | `POST /list/{id}/taskTemplate/{template_id}` | 🔵 Low |
| **Members** | `GET /list/{id}/member`, `GET /task/{id}/member` | 🔵 Low |
| **Bulk Operations** | `GET /team/{id}/task` (filtered cross-list search) | 🟡 Medium |

### TUI Features NOT Implemented

| Feature | Priority | Complexity |
|---------|----------|------------|
| Subtask drill-down navigation | 🔴 Critical | Medium |
| Task creation dialog | 🔴 Critical | High |
| Quick status change (inline) | 🔴 Critical | Medium |
| Task field editing (priority, assignees, due date) | 🔴 Critical | High |
| Task description editing | 🟠 High | High |
| Checklist item toggling | 🟠 High | Medium |
| Time tracking log/display | 🟠 High | Medium |
| Tag add/remove | 🟠 High | Low |
| Custom field value editing | 🟠 High | High |
| Clipboard copy (task URL, ID) | 🟠 High | Low |
| Dependency graph visualization | 🟡 Medium | High |
| Attachment preview/download | 🟡 Medium | Medium |
| Quick-jump between lists/spaces | 🟡 Medium | Medium |
| Bulk task selection & operations | 🟡 Medium | High |
| Keyboard shortcut customization | 🔵 Low | Medium |
| Offline caching | 🔵 Low | High |
| Notification center | 🔵 Low | High |

### CLI Features NOT Implemented

| Feature | Priority |
|---------|----------|
| Wire existing comment commands into main.rs | 🔴 Critical |
| `task create` | 🔴 Critical |
| `task update` (status, priority, assignees, due date) | 🔴 Critical |
| `task delete` | 🔴 Critical |
| `task search` (cross-list) | 🟠 High |
| Custom field set/update commands | 🟠 High |
| Time tracking commands | 🟠 High |
| Tag management commands | 🟠 High |
| Checklist management commands | 🟡 Medium |
| Shell completions (bash, zsh, fish) | 🟡 Medium |
| `folder` CRUD commands | 🟡 Medium |
| `space` CRUD commands | 🔵 Low |
| Export commands (CSV, JSON bulk) | 🔵 Low |

---

## Roadmap Phases

### Phase 1: Core Task Lifecycle 🔴

**Goal:** Transform termaup from read-only browser to a tool where users can manage tasks without leaving the terminal.

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F01 | [Wire CLI Comment Commands](#f01) | CLI | — |
| F02 | [Task Creation](#f02) | API + CLI + TUI | — |
| F03 | [Quick Status Changes](#f03) | API + CLI + TUI | — |
| F04 | [Subtask Navigation](#f04) | TUI | — |
| F05 | [Task Deletion](#f05) | API + CLI + TUI | — |

### Phase 2: Task Editing 🔴

**Goal:** Full task field editing across CLI and TUI.

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F06 | [Task Field Editing](#f06) | API + CLI + TUI | F03 |
| F07 | [Task Description Editing](#f07) | API + TUI | F06 |
| F08 | [Assignee Management](#f08) | API + CLI + TUI | F06 |

### Phase 3: Task Enrichment 🟠

**Goal:** Manage all task-attached data (custom fields, checklists, tags, time tracking).

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F09 | [Custom Field Editing](#f09) | API + CLI + TUI | F06 |
| F10 | [Checklist Management](#f10) | API + CLI + TUI | — |
| F11 | [Tag Management](#f11) | API + CLI + TUI | — |
| F12 | [Time Tracking](#f12) | API + CLI + TUI | — |
| F13 | [Dependency & Link Management](#f13) | API + CLI + TUI | — |

### Phase 4: Navigation & Content 🟠

**Goal:** Richer content access and navigation capabilities.

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F14 | [Deep Subtask Navigation](#f14) | TUI | F04 |
| F15 | [Attachment Handling](#f15) | API + CLI + TUI | — |
| F16 | [Clipboard Support](#f16) | TUI | — |
| F17 | [Cross-List Task Search](#f17) | API + CLI + TUI | — |
| F18 | [Quick-Jump Navigation](#f18) | TUI | — |

### Phase 5: Workspace Management 🟡

**Goal:** Full CRUD for organizational hierarchy (spaces, folders, lists).

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F19 | [Space CRUD](#f19) | API + CLI | — |
| F20 | [Folder CRUD](#f20) | API + CLI + TUI | — |
| F21 | [List CRUD](#f21) | API + CLI + TUI | — |
| F22 | [Bulk Task Operations](#f22) | API + CLI + TUI | F17 |

### Phase 6: Distribution & Infrastructure 🟡

**Goal:** Make termaup easy to install and configure for all users.

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F23 | [OAuth2 Authentication](#f23) | API + CLI | — |
| F24 | [Pre-built Binaries & Release Pipeline](#f24) | CI/CD | — |
| F25 | [Homebrew Formula](#f25) | Distribution | F24 |
| F26 | [Shell Completions](#f26) | CLI | — |

### Phase 7: Advanced Features 🔵

**Goal:** Power-user features and long-term vision.

| ID | Feature | Scope | Depends On |
|----|---------|-------|------------|
| F27 | [Offline Caching](#f27) | API + TUI | — |
| F28 | [Keyboard Shortcut Customization](#f28) | TUI | — |
| F29 | [Goal Tracking](#f29) | API + CLI + TUI | — |
| F30 | [View Management](#f30) | API + CLI + TUI | — |
| F31 | [ClickUp Docs Browsing](#f31) | API + TUI | — |

---

## Feature Specifications

> Each feature has a dedicated plan file at `.spectra/plans/features/F{NN}-{slug}.md`.
> The following are summaries; see individual files for full SPECTRA specs.

<a id="f01"></a>
### F01 — Wire CLI Comment Commands

**Phase:** 1 | **Priority:** 🔴 Critical | **Complexity:** 2/12
**Scope:** CLI only

The comment commands (`list`, `create`, `reply`, `edit`, `delete`) are fully implemented in `crates/clickup-cli/src/commands/comments.rs` but NOT registered in `main.rs` or `commands/mod.rs`. This is a wiring-only change.

**Stories:** 1 (wire + test)
**Files:** `main.rs`, `commands/mod.rs`

---

<a id="f02"></a>
### F02 — Task Creation

**Phase:** 1 | **Priority:** 🔴 Critical | **Complexity:** 7/12
**Scope:** API + CLI + TUI

Add ability to create tasks from both CLI and TUI. Requires new API endpoint (`POST /list/{id}/task`), new model (`CreateTaskRequest`), CLI command (`task create`), and TUI creation dialog.

**Key decisions:**
- TUI: Modal dialog with fields (name, description, status, priority, assignees, due date)
- CLI: `clickup task create --list LIST_ID --name "Title" [--description "..."] [--status "..."] [--priority N] [--assignee USER_ID] [--due-date "YYYY-MM-DD"]`
- Minimum viable: name only (everything else optional)

**Stories:** 4 (API model + endpoint, CLI command, TUI dialog, tests)

---

<a id="f03"></a>
### F03 — Quick Status Changes

**Phase:** 1 | **Priority:** 🔴 Critical | **Complexity:** 5/12
**Scope:** API + CLI + TUI

Change a task's status inline without opening a full edit dialog. Requires `PUT /task/{id}` endpoint. TUI: press `s` on task list or detail → picker shows available statuses → select → update. CLI: `clickup task status TASK_ID --status "done"`.

**Stories:** 3 (API endpoint, CLI command, TUI picker)

---

<a id="f04"></a>
### F04 — Subtask Navigation

**Phase:** 1 | **Priority:** 🔴 Critical | **Complexity:** 5/12
**Scope:** TUI

Currently subtasks are displayed as a list on the TaskDetail screen but cannot be drilled into. This feature adds the ability to press Enter on a subtask to navigate to its TaskDetail, with breadcrumb updates and back-navigation via Esc.

**Stories:** 2 (navigation logic, breadcrumb updates)

---

<a id="f05"></a>
### F05 — Task Deletion

**Phase:** 1 | **Priority:** 🔴 Critical | **Complexity:** 4/12
**Scope:** API + CLI + TUI

Delete tasks with confirmation. API: `DELETE /task/{id}`. CLI: `clickup task delete TASK_ID [--yes]`. TUI: `D` key on task detail → confirmation modal → delete → navigate back.

**Stories:** 3 (API endpoint, CLI command, TUI confirmation + action)

---

<a id="f06"></a>
### F06 — Task Field Editing

**Phase:** 2 | **Priority:** 🔴 Critical | **Complexity:** 8/12
**Scope:** API + CLI + TUI

Edit task fields: status, priority, due date, start date, time estimate. Uses `PUT /task/{id}` with partial body. TUI: press `e` on task detail → edit mode with field-by-field navigation. CLI: `clickup task update TASK_ID --status "done" --priority 2 --due-date "2026-05-01"`.

**Key complexity:** The `UpdateTaskRequest` model must handle partial updates (only send changed fields). Use `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`.

**Stories:** 5 (API model + endpoint, CLI command, TUI edit mode, field pickers, tests)

---

<a id="f07"></a>
### F07 — Task Description Editing

**Phase:** 2 | **Priority:** 🟠 High | **Complexity:** 7/12
**Scope:** API + TUI

Edit task description via external editor (`$EDITOR`). TUI: press `E` on task detail → opens `$EDITOR` with current markdown description → on save, sends `PUT /task/{id}` with updated `markdown_description`. Requires terminal teardown/restore around editor invocation.

**Stories:** 3 (editor integration, API update, terminal management)

---

<a id="f08"></a>
### F08 — Assignee Management

**Phase:** 2 | **Priority:** 🔴 Critical | **Complexity:** 6/12
**Scope:** API + CLI + TUI

Add/remove assignees on tasks. Uses `PUT /task/{id}` with `assignees.add` and `assignees.rem` arrays. TUI: assignee picker showing workspace members (similar to mention picker). CLI: `clickup task assign TASK_ID --add USER_ID --remove USER_ID`.

**Stories:** 3 (API model extension, CLI command, TUI picker)

---

<a id="f09"></a>
### F09 — Custom Field Editing

**Phase:** 3 | **Priority:** 🟠 High | **Complexity:** 8/12
**Scope:** API + CLI + TUI

Set custom field values on tasks. API: `POST /task/{id}/field/{field_id}`. Each field type requires different input handling (text input, number input, dropdown picker, date picker, checkbox toggle, etc.). Also needs `GET /list/{id}/field` to fetch available fields.

**Stories:** 5 (API endpoints, field type handlers, CLI command, TUI edit widget, tests)

---

<a id="f10"></a>
### F10 — Checklist Management

**Phase:** 3 | **Priority:** 🟠 High | **Complexity:** 6/12
**Scope:** API + CLI + TUI

Toggle checklist items, create/delete checklists and items. API: `POST /task/{id}/checklist`, `PUT /checklist/{id}`, `DELETE /checklist/{id}`, checklist item CRUD. TUI: press Space on checklist item to toggle, `a` to add item, `d` to delete. CLI: `clickup checklist` command group.

**Stories:** 4 (API endpoints + models, CLI commands, TUI interactions, tests)

---

<a id="f11"></a>
### F11 — Tag Management

**Phase:** 3 | **Priority:** 🟠 High | **Complexity:** 4/12
**Scope:** API + CLI + TUI

Add/remove tags on tasks. API: `GET /space/{id}/tag`, `POST /task/{id}/tag/{name}`, `DELETE /task/{id}/tag/{name}`. TUI: tag picker from space tags. CLI: `clickup task tag TASK_ID --add "bug" --remove "wontfix"`.

**Stories:** 3 (API endpoints, CLI command, TUI picker)

---

<a id="f12"></a>
### F12 — Time Tracking

**Phase:** 3 | **Priority:** 🟠 High | **Complexity:** 6/12
**Scope:** API + CLI + TUI

Log and view time entries. API: `GET /task/{id}/time`, `POST /task/{id}/time`. TUI: time tracking section on task detail with entry list + "Log Time" dialog. CLI: `clickup time log TASK_ID --duration "2h30m" [--description "..."]`, `clickup time list --task TASK_ID`.

**Stories:** 4 (API endpoints + models, CLI commands, TUI display + dialog, tests)

---

<a id="f13"></a>
### F13 — Dependency & Link Management

**Phase:** 3 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** API + CLI + TUI

Add/remove task dependencies and linked tasks. API: `POST/DELETE /task/{id}/dependency`, `POST/DELETE /task/{id}/link/{links_to}`. TUI: visual dependency indicators + management via keybinds. CLI: `clickup task depend TASK_ID --blocks OTHER_ID`.

**Stories:** 3 (API endpoints, CLI commands, TUI UI)

---

<a id="f14"></a>
### F14 — Deep Subtask Navigation

**Phase:** 4 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** TUI

Recursive subtask tree navigation. Build on F04 (basic subtask navigation) to support multi-level subtask hierarchies with indented tree view, breadcrumb tracking of parent chain, and efficient lazy loading.

**Stories:** 2 (tree rendering, recursive navigation)

---

<a id="f15"></a>
### F15 — Attachment Handling

**Phase:** 4 | **Priority:** 🟡 Medium | **Complexity:** 6/12
**Scope:** API + CLI + TUI

Upload attachments and open/download existing ones. API: `POST /task/{id}/attachment` (multipart). TUI: `o` on attachment to open with system default, `S` to save. CLI: `clickup task attach TASK_ID --file ./report.pdf`, `clickup task attachments TASK_ID`.

**Stories:** 4 (API upload endpoint, CLI commands, TUI open/save, tests)

---

<a id="f16"></a>
### F16 — Clipboard Support

**Phase:** 4 | **Priority:** 🟡 Medium | **Complexity:** 3/12
**Scope:** TUI

Copy task URL, task ID, or task name to system clipboard. TUI: `y` copies task URL, `Y` copies task ID. Uses `arboard` or `cli-clipboard` crate. Show brief "Copied!" flash message.

**Stories:** 2 (clipboard integration, UI feedback)

---

<a id="f17"></a>
### F17 — Cross-List Task Search

**Phase:** 4 | **Priority:** 🟡 Medium | **Complexity:** 6/12
**Scope:** API + CLI + TUI

Search tasks across all lists in a workspace. API: `GET /team/{id}/task` with `?include_closed=true&subtasks=true` and filter parameters. CLI: `clickup task search --query "bug fix" [--status "open"]`. TUI: global search overlay accessible from any screen.

**Stories:** 4 (API endpoint, CLI command, TUI search overlay, tests)

---

<a id="f18"></a>
### F18 — Quick-Jump Navigation

**Phase:** 4 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** TUI

Jump directly to a task by ID or quickly switch between recently visited lists/spaces. TUI: `g` → "Go to" dialog → type task ID → jump to TaskDetail. Also: recent items history (last 10 visited tasks/lists) with `Ctrl+R`.

**Stories:** 3 (go-to dialog, recent history, navigation logic)

---

<a id="f19"></a>
### F19 — Space CRUD

**Phase:** 5 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** API + CLI

Create, update, and delete spaces. API: `POST /team/{id}/space`, `PUT /space/{id}`, `DELETE /space/{id}`. CLI: `clickup space create`, `clickup space update`, `clickup space delete`. Dangerous operations require `--yes` flag.

**Stories:** 3 (API endpoints, CLI commands, tests)

---

<a id="f20"></a>
### F20 — Folder CRUD

**Phase:** 5 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** API + CLI + TUI

Create, update, and delete folders. API: `POST /space/{id}/folder`, `PUT /folder/{id}`, `DELETE /folder/{id}`. TUI: `N` on SpaceContent to create folder, `D` to delete with confirmation. CLI: `clickup folder create/update/delete`.

**Stories:** 3 (API endpoints, CLI commands, TUI actions)

---

<a id="f21"></a>
### F21 — List CRUD

**Phase:** 5 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** API + CLI + TUI

Create, update, and delete lists. API: `POST /folder/{id}/list`, `POST /space/{id}/list`, `PUT /list/{id}`, `DELETE /list/{id}`. TUI: `N` on SpaceContent to create list, `D` to delete. CLI: `clickup list create/update/delete`.

**Stories:** 3 (API endpoints, CLI commands, TUI actions)

---

<a id="f22"></a>
### F22 — Bulk Task Operations

**Phase:** 5 | **Priority:** 🟡 Medium | **Complexity:** 7/12
**Scope:** API + CLI + TUI

Multi-select tasks for bulk status change, assignment, or deletion. TUI: `Space` to toggle task selection, `B` for bulk actions menu (change status, assign, delete). CLI: `clickup task bulk-update --tasks "ID1,ID2,ID3" --status "done"`.

**Stories:** 4 (multi-select UI, bulk action menu, CLI command, tests)

---

<a id="f23"></a>
### F23 — OAuth2 Authentication

**Phase:** 6 | **Priority:** 🟡 Medium | **Complexity:** 7/12
**Scope:** API + CLI

Full OAuth2 authorization code flow. CLI: `clickup auth login --oauth` → opens browser → local callback server → exchange code → store token. Requires client_id/client_secret configuration. Support token refresh.

**Stories:** 4 (OAuth2 flow, local callback server, token refresh, config)

---

<a id="f24"></a>
### F24 — Pre-built Binaries & Release Pipeline

**Phase:** 6 | **Priority:** 🟡 Medium | **Complexity:** 5/12
**Scope:** CI/CD

GitHub Actions workflow for automated builds on tag push. Cross-compile for: macOS (x86_64 + aarch64), Linux (x86_64 + aarch64), Windows (x86_64). Create GitHub Releases with checksums.

**Stories:** 3 (CI workflow, cross-compilation, release automation)

---

<a id="f25"></a>
### F25 — Homebrew Formula

**Phase:** 6 | **Priority:** 🟡 Medium | **Complexity:** 3/12
**Scope:** Distribution

Create Homebrew tap with formula for macOS/Linux installation. `brew install rynaro/tap/termaup`. Auto-update formula on new releases via CI.

**Stories:** 2 (formula, CI integration)

---

<a id="f26"></a>
### F26 — Shell Completions

**Phase:** 6 | **Priority:** 🟡 Medium | **Complexity:** 3/12
**Scope:** CLI

Generate shell completions for bash, zsh, fish, PowerShell. Use clap's `clap_complete` crate. CLI: `clickup completions bash > ~/.bashrc.d/clickup.bash`. Include in distribution packages.

**Stories:** 2 (generation, packaging)

---

<a id="f27"></a>
### F27 — Offline Caching

**Phase:** 7 | **Priority:** 🔵 Low | **Complexity:** 9/12
**Scope:** API + TUI

Cache API responses locally for offline browsing and faster startup. Use SQLite or filesystem cache with TTL-based invalidation. Show stale data with "offline" indicator while fetching fresh data.

**Stories:** 5 (cache layer, storage backend, invalidation, UI indicators, tests)

---

<a id="f28"></a>
### F28 — Keyboard Shortcut Customization

**Phase:** 7 | **Priority:** 🔵 Low | **Complexity:** 6/12
**Scope:** TUI

User-configurable key bindings via `config.toml`. Default bindings remain vim-like. Support rebinding all actions. Config section: `[keybindings]`.

**Stories:** 3 (config schema, keymap loader, input dispatch refactor)

---

<a id="f29"></a>
### F29 — Goal Tracking

**Phase:** 7 | **Priority:** 🔵 Low | **Complexity:** 6/12
**Scope:** API + CLI + TUI

View and manage ClickUp Goals and Key Results. API: Full CRUD for `/goal/{id}` and key results. TUI: New Goals screen. CLI: `clickup goal list/get/create/update`.

**Stories:** 4 (API models + endpoints, CLI commands, TUI screen, tests)

---

<a id="f30"></a>
### F30 — View Management

**Phase:** 7 | **Priority:** 🔵 Low | **Complexity:** 5/12
**Scope:** API + CLI + TUI

Browse and apply ClickUp Views (saved filter/sort configurations). API: `GET /team/{id}/view`, `GET /space/{id}/view`. TUI: apply view filters to task list. CLI: `clickup view list`.

**Stories:** 3 (API endpoints, CLI commands, TUI integration)

---

<a id="f31"></a>
### F31 — ClickUp Docs Browsing

**Phase:** 7 | **Priority:** 🔵 Low | **Complexity:** 7/12
**Scope:** API + TUI

Browse and read ClickUp Docs in the terminal. ClickUp Docs API v2 is limited; v3 with full Docs support is expected. TUI: Docs screen with markdown rendering. CLI: `clickup doc list/view`.

**Stories:** 3 (API endpoints, TUI screen, CLI commands)

---

## Dependency Graph

```
Phase 1 (Core Lifecycle)
  F01 (Wire Comments) ──────────────────────────────────────→ standalone
  F02 (Task Creation) ──────────────────────────────────────→ standalone
  F03 (Quick Status) ───→ F06 (Field Editing) ──→ F07 (Description Edit)
  F04 (Subtask Nav) ────→ F14 (Deep Subtask Nav)  └──→ F08 (Assignees)
  F05 (Task Deletion) ──────────────────────────────────────→ F22 (Bulk Ops)

Phase 2 (Editing)
  F06 (Field Editing) ──→ F09 (Custom Fields)

Phase 3 (Enrichment)
  F09-F13 are all independent of each other

Phase 4 (Navigation)
  F14-F18 are mostly independent
  F17 (Cross-List Search) → F22 (Bulk Ops)

Phase 5 (Workspace Mgmt)
  F19-F22 are mostly independent

Phase 6 (Distribution)
  F24 (Binaries) → F25 (Homebrew)
  F23, F26 are independent

Phase 7 (Advanced)
  All independent
```

---

## Feature Plan Files Index

| Feature | Plan File |
|---------|-----------|
| F01 — Wire CLI Comment Commands | `features/F01-wire-cli-comment-commands.md` |
| F02 — Task Creation | `features/F02-task-creation.md` |
| F03 — Quick Status Changes | `features/F03-quick-status-changes.md` |
| F04 — Subtask Navigation | `features/F04-subtask-navigation.md` |
| F05 — Task Deletion | `features/F05-task-deletion.md` |
| F06 — Task Field Editing | `features/F06-task-field-editing.md` |
| F07 — Task Description Editing | `features/F07-task-description-editing.md` |
| F08 — Assignee Management | `features/F08-assignee-management.md` |
| F09 — Custom Field Editing | `features/F09-custom-field-editing.md` |
| F10 — Checklist Management | `features/F10-checklist-management.md` |
| F11 — Tag Management | `features/F11-tag-management.md` |
| F12 — Time Tracking | `features/F12-time-tracking.md` |
| F13 — Dependency & Link Management | `features/F13-dependency-link-management.md` |
| F14 — Deep Subtask Navigation | `features/F14-deep-subtask-navigation.md` |
| F15 — Attachment Handling | `features/F15-attachment-handling.md` |
| F16 — Clipboard Support | `features/F16-clipboard-support.md` |
| F17 — Cross-List Task Search | `features/F17-cross-list-task-search.md` |
| F18 — Quick-Jump Navigation | `features/F18-quick-jump-navigation.md` |
| F19 — Space CRUD | `features/F19-space-crud.md` |
| F20 — Folder CRUD | `features/F20-folder-crud.md` |
| F21 — List CRUD | `features/F21-list-crud.md` |
| F22 — Bulk Task Operations | `features/F22-bulk-task-operations.md` |
| F23 — OAuth2 Authentication | `features/F23-oauth2-authentication.md` |
| F24 — Pre-built Binaries & Release Pipeline | `features/F24-release-pipeline.md` |
| F25 — Homebrew Formula | `features/F25-homebrew-formula.md` |
| F26 — Shell Completions | `features/F26-shell-completions.md` |
| F27 — Offline Caching | `features/F27-offline-caching.md` |
| F28 — Keyboard Shortcut Customization | `features/F28-keyboard-shortcuts.md` |
| F29 — Goal Tracking | `features/F29-goal-tracking.md` |
| F30 — View Management | `features/F30-view-management.md` |
| F31 — ClickUp Docs Browsing | `features/F31-docs-browsing.md` |

---

*SPECTRA v4.2.0 — termaup roadmap*
