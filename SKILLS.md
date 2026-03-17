# SKILLS.md — clickup-rs Development Skills

This document defines the development skills (workstreams) for the clickup-rs project.
Each skill has clear boundaries, dependencies, and acceptance criteria.
Reference this document when executing prompts against the Copilot Coding Agent.

---

## Skill 1: Foundation

**Scope:** Project scaffolding, workspace setup, Docker, CI/CD
**Crates:** All (workspace root)
**Dependencies:** None

**Deliverables:**
- Cargo workspace with 3 member crates compiling
- Dockerfile (multi-stage) + docker-compose.yml
- GitHub Actions CI pipeline (fmt, clippy, test, build)
- rustfmt.toml, clippy.toml, .gitignore, LICENSE, README stub

**Acceptance:** `cargo build --workspace` succeeds, `docker build .` succeeds, CI passes

---

## Skill 2: Error Handling & Configuration

**Scope:** Shared error types, config system, token storage
**Crates:** `clickup-api`
**Dependencies:** Skill 1

**Deliverables:**
- `error.rs` — `ClickUpError` enum with thiserror, `Result<T>` alias
- `config.rs` — `Config` struct with TOML load/save at `~/.config/clickup-rs/config.toml`
- `auth.rs` — `TokenStorage` with OS keyring + encrypted file fallback

**Acceptance:** Config round-trips through TOML, tokens persist in keyring, error Display is human-readable

---

## Skill 3: HTTP Client

**Scope:** Core HTTP client with rate limiting, retry, pagination
**Crates:** `clickup-api`
**Dependencies:** Skill 2

**Deliverables:**
- `client.rs` — `ClickUpClient` with `get()`, `get_with_params()` methods
- `rate_limiter.rs` — Tracks `X-RateLimit-*` headers, auto-waits on exhaustion
- `pagination.rs` — Generic page-crawling helper

**Acceptance:** Client handles 429/401/404 correctly, rate limiter sleeps transparently, pagination collects all pages

---

## Skill 4: Domain Models

**Scope:** Serde structs for all ClickUp API entities
**Crates:** `clickup-api`
**Dependencies:** Skill 1

**Deliverables:**
- `models/user.rs` — User, AuthenticatedUser
- `models/workspace.rs` — Workspace, WorkspaceMember, WorkspacesResponse
- `models/space.rs` — Space, SpacesResponse
- `models/status.rs` — Status
- `models/folder.rs` — Folder, FoldersResponse
- `models/list.rs` — List, ListsResponse
- `models/task.rs` — Task, TasksResponse, TaskStatus, TaskPriority, Tag, CustomField
- `models/comment.rs` — Comment, CommentsResponse
- `models/mod.rs` — Re-exports all

**Acceptance:** All models deserialize from ClickUp API JSON fixtures without error

---

## Skill 5: API Endpoints

**Scope:** Typed async endpoint methods on ClickUpClient
**Crates:** `clickup-api`
**Dependencies:** Skill 3 + Skill 4

**Deliverables:**
- `endpoints/users.rs` — `get_authenticated_user()`
- `endpoints/workspaces.rs` — `get_workspaces()`
- `endpoints/spaces.rs` — `get_spaces()`, `get_space()`
- `endpoints/folders.rs` — `get_folders()`
- `endpoints/lists.rs` — `get_lists_in_folder()`, `get_folderless_lists()`, `get_list()`
- `endpoints/tasks.rs` — `get_tasks()`, `get_tasks_with_filters()`, `get_task()`
- `endpoints/comments.rs` — `get_task_comments()`

**Acceptance:** Each endpoint tested with wiremock mock server and realistic JSON fixtures

---

## Skill 6: CLI Auth

**Scope:** Authentication commands for the CLI
**Crates:** `clickup-cli`
**Dependencies:** Skill 5

**Deliverables:**
- `commands/auth.rs` — login (token + interactive), logout, status, switch workspace
- `output.rs` — success/error/info helpers with colored prefixes
- `client_factory.rs` — create authenticated client from stored token

**Acceptance:** Full auth lifecycle works: login → status → switch → logout

---

## Skill 7: CLI Data Commands

**Scope:** Space, List, Task browsing commands with rich output
**Crates:** `clickup-cli`
**Dependencies:** Skill 6

**Deliverables:**
- `commands/workspaces.rs` — `workspace list`
- `commands/spaces.rs` — `space list`, `space get`
- `commands/lists.rs` — `list list` (by space or folder), `list get`
- `commands/tasks.rs` — `task list`, `task get`, `task view` (rich markdown)
- Enhanced `output.rs` — table formatter, JSON printer, markdown renderer, status/priority/date formatters

**Acceptance:** All commands produce correct, colored, well-formatted output in all three formats

---

## Skill 8: TUI Scaffold

**Scope:** App skeleton, event loop, state machine, layout framework
**Crates:** `clickup-tui`
**Dependencies:** Skill 5

**Deliverables:**
- `app.rs` — App struct with Screen enum, selection state, breadcrumbs
- `event.rs` — EventHandler with crossterm polling + tokio channels
- `main.rs` — Terminal setup/teardown, main loop, panic hook
- `ui/mod.rs` — Main render dispatcher
- `ui/layout.rs` — Standard 3-row layout (breadcrumb, content, keys)
- Placeholder screens for all views

**Acceptance:** TUI launches, shows placeholder content, responds to q/Ctrl+C to quit, terminal restores cleanly

---

## Skill 9: TUI Workspace & Space Screens

**Scope:** First real TUI screens with data loading
**Crates:** `clickup-tui`
**Dependencies:** Skill 8

**Deliverables:**
- `data.rs` — Async loader functions that send DataPayload events
- `ui/workspace_select.rs` — Selectable workspace list
- `ui/space_list.rs` — Selectable space list with private indicators
- `input.rs` — Key routing per screen
- Loading spinner in status bar

**Acceptance:** Can launch TUI → see workspaces → select one → see spaces → navigate with j/k/Enter/Esc

---

## Skill 10: TUI List & Task Screens

**Scope:** List view (task table) and task detail with markdown
**Crates:** `clickup-tui`
**Dependencies:** Skill 9

**Deliverables:**
- `ui/list_view.rs` — Folder/list hierarchy + task table with colored status/priority
- `ui/task_detail.rs` — Full task view with metadata header + scrollable markdown body + comments
- `widgets/markdown.rs` — pulldown-cmark → ratatui Line/Span converter

**Acceptance:** Can drill from space → list → task → see rendered markdown description, scroll through it, go back

---

## Skill 11: TUI Polish

**Scope:** Search, help, themes, error handling, caching
**Crates:** `clickup-tui`
**Dependencies:** Skill 10

**Deliverables:**
- `ui/search.rs` — Inline filter bar with fuzzy matching
- `ui/help.rs` — Modal overlay with keybinding reference
- `theme.rs` — Theme struct with ClickUp color conversion
- Data caching (back-navigation doesn't re-fetch)
- Refresh (`r` key), error banners, terminal-too-small guard, mouse support

**Acceptance:** Search filters lists in real-time, help overlay shows/hides, colors match ClickUp statuses

---

## Skill 12: Documentation & Release

**Scope:** Docs, testing infrastructure, release automation
**Crates:** All
**Dependencies:** Skill 11

**Deliverables:**
- Comprehensive README.md with install/usage/architecture/keybindings
- CONTRIBUTING.md
- `.github/ISSUE_TEMPLATE/` (bug report + feature request)
- `.github/workflows/release.yml` (cross-compile + GitHub Release)
- Integration test suite with JSON fixtures
- Snapshot tests for CLI output
- TUI render tests with TestBackend
- `justfile` with common recipes

**Acceptance:** README is complete, release workflow produces binaries for all platforms, all tests pass

---

## Dependency Graph

```
Skill 1 (Foundation)
  ├── Skill 2 (Errors/Config) → Skill 3 (HTTP Client) ─┐
  └── Skill 4 (Models) ────────────────────────────────┘
                                                         ↓
                                                    Skill 5 (Endpoints)
                                                    ↙           ↘
                                          Skill 6 (CLI Auth)   Skill 8 (TUI Scaffold)
                                              ↓                    ↓
                                          Skill 7 (CLI Data)  Skill 9 (TUI Workspaces)
                                              ↓                    ↓
                                              ↓               Skill 10 (TUI Tasks)
                                              ↓                    ↓
                                              ↓               Skill 11 (TUI Polish)
                                              ↓                    ↓
                                              └──── Skill 12 (Docs & Release) ────┘
```

Skill 6→7 (CLI track) and Skill 8→9→10→11 (TUI track) are parallelizable after Skill 5.
