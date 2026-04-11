# F01 — Wire CLI Comment Commands

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 1 — Critical MVP Gaps
> **Complexity:** 2/12 | **Confidence:** 98%

---

## Problem Statement

The CLI comment commands (`list`, `create`, `reply`, `edit`, `delete`) are fully implemented in `crates/clickup-cli/src/commands/comments.rs` with a complete `CommentCommands` enum and `run()` method, but users cannot access them because the module is neither exported from `commands/mod.rs` nor registered as a `Commands` variant in `main.rs`. This is a dead-code wiring gap — the feature is 100% built but 0% reachable.

## Approach

Add three lines of code: a `pub mod comments;` export, a `Comment` variant in the `Commands` enum, and a match arm in the dispatch block. This follows the exact pattern of all five existing command groups (`Auth`, `Workspace`, `Space`, `List`, `Task`).

### Rejected Alternatives

1. **Nest comments under the `Task` subcommand** (`clickup task comment list`) — While hierarchically logical (comments belong to tasks), this conflicts with the existing `CommentCommands` implementation which has its own `--task` argument and operates at the top level. Would require rewriting the entire `comments.rs` module for no functional gain. Rejected for unnecessary churn.

2. **Feature-flag the comment commands** (`#[cfg(feature = "comments")]`) — Over-engineering for code that is already written, reviewed, and passing compilation when exported. Feature flags add CI matrix complexity. Rejected for adding process overhead to a trivial wiring fix.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Export `comments` module from `commands/mod.rs` | Changing CommentCommands API or behavior | Adding `--format` flag to comment output |
| Add `Comment` variant to `Commands` enum in `main.rs` | Adding new comment subcommands | Comment search/filter commands |
| Add dispatch match arm for `Comment` | Modifying comment output formatting | Batch comment operations |
| Verify existing comment tests still pass | | |

## Stories

### S-1: Wire comment commands into CLI dispatch

**As a** CLI user, **I want** to run `clickup comment list --task TASK_ID` and other comment subcommands, **so that** I can manage ClickUp task comments from the terminal.

**Timebox:** ≤1d | **Risk:** P2 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod comments;` after line 1 (alongside existing module exports: `auth`, `lists`, `spaces`, `tasks`, `workspaces`) |
| 2 | Modify | `crates/clickup-cli/src/main.rs` | Add `use commands::comments::CommentCommands;` to the imports block (lines 3-8) |
| 3 | Modify | `crates/clickup-cli/src/main.rs` | Add `/// Manage task comments.` + `Comment { #[command(subcommand)] command: CommentCommands }` variant to `Commands` enum (after line 56, following the `Task` variant pattern) |
| 4 | Modify | `crates/clickup-cli/src/main.rs` | Add `Commands::Comment { command } => command.run(fmt, ws).await,` to the match dispatch block (after line 79, following existing dispatch pattern) |
| 5 | Test | `crates/clickup-cli/` | Run `cargo build -p clickup-cli` to verify compilation, `cargo test -p clickup-cli` to verify no regressions |

#### Acceptance Criteria

- [ ] GIVEN the CLI binary is built WHEN the user runs `clickup comment --help` THEN a help message listing `list`, `create`, `reply`, `edit`, `delete` subcommands is displayed
- [ ] GIVEN a valid token is configured WHEN the user runs `clickup comment list --task TASK_ID` THEN comments for the specified task are fetched and displayed (or a connection error is shown if no network)
- [ ] GIVEN the CLI binary is built WHEN the user runs `clickup --help` THEN `comment` appears in the list of available commands alongside `auth`, `workspace`, `space`, `list`, `task`
- [ ] GIVEN the existing test suite WHEN `cargo test --workspace` is run THEN all tests pass with no regressions

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/main.rs` (lines 31-58 Commands enum, lines 73-79 dispatch match), `crates/clickup-cli/src/commands/mod.rs` (lines 1-5), `crates/clickup-cli/src/commands/comments.rs` (lines 14-88 CommentCommands enum + run method)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test --workspace` — all tests pass (existing + new)
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes
  - [ ] P2: All public items have `///` doc comments

## Execution Sequence

```
S-1 (wire comment commands)
```

Single story, no dependencies. Entire feature is a 3-line wiring change.

## Assumptions

1. `CommentCommands::run()` signature `(self, format: &str, workspace_override: Option<&str>) -> Result<()>` matches the pattern used by all other command `run()` methods — Risk if wrong: compilation error, trivially fixable by adjusting the dispatch call (LOW)
2. The `comments.rs` file compiles cleanly when exported — Risk if wrong: compilation errors from unused imports or dead code warnings, but since the code was written to be exported, this is extremely unlikely (NEGLIGIBLE)

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | 3 lines of code across 2 files; no logic changes |
| Ambiguity | 1 | Exact pattern exists 5 times already in the same files |
| Dependencies | 1 | No external dependencies; comments.rs is self-contained |
| Risk | 1 | Zero behavioral change; purely additive wiring |
