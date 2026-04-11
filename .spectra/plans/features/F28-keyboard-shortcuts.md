# F28 — Keyboard Shortcut Customization

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 7 — Advanced Features
> **Complexity:** 6/12 | **Confidence:** 85%

---

## Problem Statement

All keyboard shortcuts in the TUI are hardcoded in `crates/clickup-tui/src/input.rs`. Users who prefer non-vim keybindings (e.g., arrow-key-only users, Emacs users) or who have conflicting terminal keybindings cannot remap controls. The help overlay (`?`) displays static key hints that don't reflect any customization. This rigidity limits adoption among users with accessibility needs or strong keybinding preferences.

## Approach

Introduce a `[keybindings]` section in `~/.config/clickup-rs/config.toml` where users map semantic actions (e.g., `move_up`, `select`, `quit`) to key combinations. At startup, the TUI loads this keymap into a `HashMap<KeyBinding, Action>` and the input dispatch in `input.rs` resolves key events through this map instead of hardcoded `match` arms.

Default bindings remain the current vim-like keys. The keymap abstraction layer intercepts key events, resolves them to actions, and the existing handler logic switches on actions rather than raw key events.

### Rejected Alternatives

1. **JSON keybinding file** — Separate file (e.g., `keybindings.json`) for keybinding config. Adds file management complexity when config.toml already exists and supports nested tables. Rejected for simplicity.

2. **Modal keybinding modes (vim-like normal/insert/visual)** — Full modal editing with distinct keymaps per mode. Extreme complexity for a browsing-focused TUI. Rejected as overengineering.

3. **Lua scripting for keybindings** — Runtime-configurable keybindings via embedded Lua. Massive dependency and attack surface for a simple key remapping feature. Rejected.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `[keybindings]` config section in `config.toml` | Per-screen keybinding overrides | `clickup config keybindings reset` command |
| `Action` enum for all bindable actions | Keybinding recording/wizard | Multi-key sequences (chords beyond Ctrl/Alt/Shift) |
| `Keymap` struct with `resolve(KeyEvent) -> Option<Action>` | Mouse button remapping | Keybinding conflict detection warnings |
| Default vim-like bindings as fallback | | Keybinding presets (vim, emacs, arrows) |
| Refactored `input.rs` dispatch using actions | | |
| Dynamic help overlay showing configured bindings | | |

## Stories

### S-1: Config Schema + Keymap Parser

**As a** user, **I want** to define custom key bindings in `config.toml`, **so that** I can remap TUI controls to keys I prefer.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Define `Action` enum | `crates/clickup-tui/src/keymap.rs` | New module. `pub enum Action { MoveUp, MoveDown, Select, Back, Quit, Search, ToggleHelp, Refresh, PageUp, PageDown, ToggleFilter, ToggleComments, CycleView, ... }` — one variant per bindable action |
| 2 | Define `KeyBinding` struct | `crates/clickup-tui/src/keymap.rs` | `pub struct KeyBinding { pub code: KeyCode, pub modifiers: KeyModifiers }` with `From<KeyEvent>`, `Hash`, `Eq` |
| 3 | Define config format | `crates/clickup-tui/src/keymap.rs` | `[keybindings]` section: `move_up = "k"`, `move_down = "j"`, `select = "Enter"`, `quit = "q"`, `search = "/"`, `help = "?"`, `refresh = "r"`. Modifier syntax: `"Ctrl+c"`, `"Alt+j"` |
| 4 | Implement parser | `crates/clickup-tui/src/keymap.rs` | `fn parse_key_binding(s: &str) -> Result<KeyBinding>` — parses strings like `"k"`, `"Enter"`, `"Ctrl+c"`, `"Alt+j"`, `"Esc"`, `"Tab"`, `"F1"` |
| 5 | Define `Keymap` struct | `crates/clickup-tui/src/keymap.rs` | `pub struct Keymap { bindings: HashMap<KeyBinding, Action> }` with `fn resolve(&self, event: &KeyEvent) -> Option<Action>` |
| 6 | Default keymap | `crates/clickup-tui/src/keymap.rs` | `impl Default for Keymap` populates all current hardcoded bindings |
| 7 | Config integration | `crates/clickup-api/src/config.rs` | Add `keybindings: Option<HashMap<String, String>>` to `Config` with `#[serde(default)]` |
| 8 | Merge logic | `crates/clickup-tui/src/keymap.rs` | `Keymap::from_config(config_bindings: &HashMap<String, String>) -> Keymap` — starts with defaults, overrides with config entries |
| 9 | Unit tests | `crates/clickup-tui/src/keymap.rs` | Test parsing: `"k"` → `KeyCode::Char('k')`, `"Ctrl+c"` → `Char('c') + CONTROL`, `"Enter"` → `KeyCode::Enter`, `"F5"` → `KeyCode::F(5)`. Test merge: config overrides default, unset keys retain defaults |

#### Acceptance Criteria

- **GIVEN** config contains `[keybindings]\nmove_up = "i"`, **WHEN** the keymap is loaded, **THEN** pressing `i` triggers `Action::MoveUp`, and `k` no longer triggers `MoveUp`.
- **GIVEN** config contains no `[keybindings]` section, **WHEN** the keymap is loaded, **THEN** all default vim-like bindings are active.
- **GIVEN** config contains `move_up = "Ctrl+p"`, **WHEN** parsed, **THEN** `KeyBinding { code: Char('p'), modifiers: CONTROL }` is mapped to `MoveUp`.
- **GIVEN** config contains an unknown action name `fly = "f"`, **WHEN** parsed, **THEN** it is silently ignored (no panic, log a warning).
- **GIVEN** config contains an unparseable key string `move_up = "???"`, **WHEN** parsed, **THEN** a warning is logged and the default for `move_up` is retained.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (existing key dispatch — this story does NOT modify it yet, just builds the abstraction). `crates/clickup-api/src/config.rs` (existing `Config` struct). The `keybindings` field in `Config` should be `HashMap<String, String>` (action name → key string) at the config level; the TUI crate converts to `Keymap`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P1: `cargo test` passes (both clickup-api for config, clickup-tui for keymap)
  - [ ] P2: `cargo clippy` clean

---

### S-2: Keymap Abstraction Layer

**As a** TUI developer, **I want** a `Keymap` that resolves `KeyEvent` to `Action`, **so that** input dispatch is decoupled from hardcoded key codes and all keybinding logic is centralized.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Keymap` to `App` | `crates/clickup-tui/src/app.rs` | Add `pub keymap: Keymap` field to `App` struct |
| 2 | Initialize keymap at startup | `crates/clickup-tui/src/main.rs` | Load config, build `Keymap::from_config()`, pass to `App::new()` |
| 3 | Add `resolve()` method validation | `crates/clickup-tui/src/keymap.rs` | Ensure `resolve()` handles modifier masking correctly (ignore `SHIFT` for letters since crossterm includes it) |
| 4 | Add context-aware resolution | `crates/clickup-tui/src/keymap.rs` | `fn resolve_for_screen(&self, event: &KeyEvent, screen: &Screen) -> Option<Action>` — future-proofing for per-screen overrides |
| 5 | Integration test | `crates/clickup-tui/src/keymap.rs` | Test full flow: config TOML → parse → keymap → resolve KeyEvent → Action |

#### Acceptance Criteria

- **GIVEN** a `Keymap` with `MoveUp` bound to `k`, **WHEN** `resolve(KeyEvent { code: Char('k'), modifiers: NONE })` is called, **THEN** `Some(Action::MoveUp)` is returned.
- **GIVEN** a `Keymap` with no binding for `x`, **WHEN** `resolve(KeyEvent { code: Char('x'), modifiers: NONE })` is called, **THEN** `None` is returned.
- **GIVEN** `Quit` bound to `Ctrl+c`, **WHEN** `resolve(KeyEvent { code: Char('c'), modifiers: CONTROL })` is called, **THEN** `Some(Action::Quit)` is returned.
- **GIVEN** the `App` struct, **WHEN** inspected, **THEN** it contains a `keymap` field of type `Keymap`.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/app.rs` (existing `App::new()` constructor). The `Keymap` should be immutable after construction — no runtime modification.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy` clean

---

### S-3: Refactor Input Dispatch

**As a** TUI user with custom keybindings, **I want** all key handling to go through the keymap abstraction, **so that** my configured keybindings actually take effect.

**Timebox:** ≤2d | **Risk:** Medium (large refactor, regression risk) | **Depends on:** S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Refactor global key handling | `crates/clickup-tui/src/input.rs` | Replace `match key.code { KeyCode::Char('q') => ... }` with `match app.keymap.resolve(&key) { Some(Action::Quit) => ... }` |
| 2 | Refactor screen-specific handlers | `crates/clickup-tui/src/input.rs` | Update `handle_workspace_select()`, `handle_space_list()`, `handle_task_list()`, `handle_task_detail()` — replace hardcoded keys with action matches |
| 3 | Preserve non-bindable keys | `crates/clickup-tui/src/input.rs` | Text input (comment compose, search) should NOT go through keymap — raw key events are needed for typing. Add guard: if in text input mode, skip keymap resolution |
| 4 | Update help overlay | `crates/clickup-tui/src/ui/help.rs` | Render key hints from `Keymap` instead of hardcoded strings. `keymap.binding_for(Action::Quit)` → display the configured key |
| 5 | Update bottom bar hints | `crates/clickup-tui/src/ui/layout.rs` | Bottom bar key hints should reflect configured bindings |
| 6 | Regression tests | `crates/clickup-tui/src/input.rs` | Test that all default keybindings still work after refactor (same behavior, different dispatch path) |

#### Acceptance Criteria

- **GIVEN** default keybindings, **WHEN** `q` is pressed on WorkspaceSelect, **THEN** the app quits (existing behavior preserved).
- **GIVEN** default keybindings, **WHEN** `j`/`k` are pressed, **THEN** selection moves down/up (existing behavior preserved).
- **GIVEN** custom binding `quit = "x"`, **WHEN** `x` is pressed, **THEN** the app quits. **WHEN** `q` is pressed, **THEN** nothing happens.
- **GIVEN** comment compose mode is active, **WHEN** any key is pressed, **THEN** it is treated as text input (keymap is bypassed).
- **GIVEN** search mode is active, **WHEN** any key is pressed, **THEN** it is treated as search input (keymap is bypassed).
- **GIVEN** the help overlay is shown, **WHEN** inspected, **THEN** it displays the currently configured key for each action, not hardcoded defaults.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` is the largest file to modify. The function `handle_key()` has global dispatch first, then screen-specific dispatch. Preserve the structure: global actions first (quit, help, search), then delegate to screen handlers. The comment compose and search input guards already exist — just ensure they remain ABOVE the keymap resolution. `crates/clickup-tui/src/ui/help.rs` renders the help overlay — update it to read from the keymap. `crates/clickup-tui/src/ui/layout.rs` renders the bottom bar.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: All existing keybindings still work with default keymap
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P1: Custom keybinding override works end-to-end
  - [ ] P2: `cargo clippy` clean
  - [ ] P2: Help overlay and bottom bar show configured keys

---

## Execution Sequence

```
S-1 (config schema + parser) → S-2 (keymap abstraction) → S-3 (refactor input dispatch)
```

Linear chain — each story builds on the abstraction from the previous.

## Assumptions

1. **crossterm `KeyEvent` provides consistent modifier flags** across platforms. Risk if wrong: some terminal emulators may not report modifiers correctly. Mitigation: document known terminal compatibility issues; support simple keys as primary bindings.
2. **Users will not need per-screen keybinding overrides** in v1. Risk if wrong: the `Action` enum can be extended with screen-scoped variants later.
3. **The `SHIFT` modifier is implicit** for uppercase letters in crossterm. Risk if wrong: the keymap parser needs to handle `Shift+k` → `K` correctly. Mitigation: strip SHIFT from letter key events before lookup.
4. **Single-key bindings are sufficient** — no multi-key sequences (e.g., `gg` for "go to top"). Risk if wrong: would need a key sequence buffer. Deferred.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | New module (keymap), config extension, large refactor of input.rs, help/layout updates |
| Ambiguity | 2 | Key parsing edge cases (modifiers, special keys, platform differences) |
| Dependencies | 1 | Touches config (API crate) and input + UI (TUI crate); well-defined boundaries |
| Risk | 1 | Regression risk in input dispatch, but default keymap ensures backward compatibility |

**Total: 6/12** → Standard processing
