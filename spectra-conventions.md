# SPECTRA Conventions — clickup-rs (termaup)

> Generated from spectra-adaptation-prompt.md for the clickup-rs project.
> SPECTRA v4.2.0 — https://github.com/Rynaro/SPECTRA

---

## Quick Reference

1. **Endpoint methods live on `ClickUpClient`** — business logic is `impl ClickUpClient` blocks in `crates/clickup-api/src/endpoints/*.rs`, not standalone service objects.
2. **Error handling is split by crate** — `thiserror` + `crate::Result<T>` in `clickup-api`; `anyhow::Result` + `.context()` in CLI/TUI binaries. Never `unwrap()` outside tests.
3. **All models derive `Debug, Clone, Serialize, Deserialize`** — use `#[serde(default)]` on Vec/Option fields; only `id` and `name` are required. See `docs/clickup-api-quirks.md`.
4. **Tests use `wiremock` for HTTP mocking** — never call the real ClickUp API. Inline `#[cfg(test)] mod tests` for unit tests; `tests/fixtures/` for integration tests.
5. **Quality gates are non-negotiable** — `cargo fmt --check` + `cargo clippy -- -D warnings` (zero warnings) + `cargo test --workspace` must all pass.

---

## 1. Convention Mapping

| SPECTRA Concept | Generic Example | clickup-rs Convention | Path Pattern |
|----------------|-----------------|----------------------|--------------|
| Service / Business Logic | `UserRegistrationService` | `impl ClickUpClient` endpoint method (e.g., `get_tasks()`, `create_comment()`) | `crates/clickup-api/src/endpoints/{resource}.rs` |
| Data Access / Repository | `UserRepository` | `ClickUpClient` HTTP methods (`get<T>()`, `post<T>()`, `get_all_pages()`) | `crates/clickup-api/src/client.rs` |
| Schema / Validation | `UserSchema` | Serde model struct with derives + custom deserializers | `crates/clickup-api/src/models/{entity}.rs` |
| UI Component | `UserProfile` | TUI screen render function `render(app: &App, frame: &mut Frame, area: Rect)` | `crates/clickup-tui/src/ui/{screen}.rs` |
| Background Job | `ImportJob` | `tokio::spawn` async task communicating via `tokio::sync::mpsc` | `crates/clickup-tui/src/data.rs` |
| API Endpoint | `POST /users` | CLI subcommand enum variant with async `run()` method | `crates/clickup-cli/src/commands/{resource}.rs` |
| Database Migration | `add_users_table` | N/A — no database; config changes go in `config.rs` | `crates/clickup-api/src/config.rs` |
| Test File | `user.test.ts` | Inline `#[cfg(test)] mod tests` block, or fixture-based integration test | `{module}.rs` (inline) or `crates/{crate}/tests/*.rs` |

---

## 2. Action Verb Mapping

| Verb | In clickup-rs | Example |
|------|--------------|---------|
| Create | Add a new `.rs` module with struct/impl, wire it into `mod.rs` | "Create `comment.rs` at `crates/clickup-api/src/models/`" |
| Extend | Add methods to an existing `impl` block or fields to a struct | "Extend `ClickUpClient` with `get_task_comments()`" |
| Modify | Edit existing function logic, struct fields, or serde attributes | "Modify `Task` model to add `time_spent` field" |
| Test | Write `#[tokio::test]` with `wiremock::MockServer` or fixture JSON | "Test with `wiremock` covering `get_spaces` pagination" |
| Configure | Update `Cargo.toml` workspace deps, `config.rs`, or CI workflow | "Configure `reqwest` timeout in `crates/clickup-api/src/client.rs`" |
| Migrate | N/A — no database migrations; closest is config format evolution | "Migrate config format using `config.rs` versioned loader" |

---

## 3. Validation Gates Template

```markdown
## Agent Hints:
- **Class:** [builder/reasoner/debugger]
- **Context:** [path to exemplar file in codebase]
- **Gates:**
  - [ ] P0: No `.unwrap()` outside `#[cfg(test)]`; no `panic!()` in `clickup-api`
  - [ ] P0: `clickup-api` has zero dependencies on CLI/TUI crates
  - [ ] P0: All API models derive `Debug, Clone, Serialize, Deserialize`
  - [ ] P1: `cargo test --workspace` passes with all new code covered
  - [ ] P1: New endpoint methods include `tracing::debug!` for the HTTP call
  - [ ] P2: `cargo fmt --all -- --check` passes clean
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes with zero warnings
```

---

## 4. Example Story

#### STORY: S-1 Add health check endpoint returning service status and version

**Description:** As a CLI/TUI user, I want a `status` command that calls `GET /user` and reports API connectivity, authenticated user, and client version so that I can quickly verify my setup is working.

**Action Plan:**

1. **Create** `crates/clickup-api/src/endpoints/status.rs` — add `pub async fn check_status(&self) -> Result<StatusInfo>` on `ClickUpClient` that calls `GET /user` and returns a `StatusInfo` struct with connectivity, user info, and latency.
2. **Create** `crates/clickup-api/src/models/status.rs` — define `StatusInfo` struct with fields: `connected: bool`, `user: Option<User>`, `latency_ms: u64`, `api_version: &'static str`.
3. **Extend** `crates/clickup-api/src/models/mod.rs` and `crates/clickup-api/src/endpoints/mod.rs` — wire in the new modules.
4. **Create** `crates/clickup-cli/src/commands/status.rs` — add `StatusCommand` with `pub async fn run()` that creates a client, calls `check_status()`, and prints a formatted result with ✓/✗ indicators.
5. **Extend** `crates/clickup-cli/src/main.rs` — register the `status` subcommand in the clap `Commands` enum.
6. **Test** inline in `status.rs` — mock `GET /user` with `wiremock`, verify `StatusInfo` fields. Test both success (200) and failure (401) scenarios.

**Acceptance Criteria:**

- **GIVEN** a valid API token is configured, **WHEN** I run `clickup status`, **THEN** I see a green `✓ Connected` message with my username and response latency.
- **GIVEN** an invalid or missing token, **WHEN** I run `clickup status`, **THEN** I see a red `✗ Authentication failed` message with guidance to run `clickup auth login`.
- **GIVEN** the API is unreachable, **WHEN** I run `clickup status`, **THEN** I see a red `✗ Connection failed` message with the network error.

**Technical Context:**

- Pattern: follow `crates/clickup-api/src/endpoints/users.rs` for endpoint structure
- Exemplar model: `crates/clickup-api/src/models/user.rs`
- CLI pattern: `crates/clickup-cli/src/commands/auth.rs` for simple command structure
- Deps: no new dependencies needed

**Agent Hints:**

```markdown
- **Class:** builder
- **Context:** crates/clickup-api/src/endpoints/users.rs
- **Gates:**
  - [ ] P0: No `.unwrap()` outside tests; `clickup-api` returns `Result<T>`
  - [ ] P0: CLI uses `anyhow::Result` with `.context()` for error propagation
  - [ ] P1: Tests cover success, auth failure, and network failure with `wiremock`
  - [ ] P1: `tracing::debug!` call in the endpoint method
  - [ ] P2: `cargo clippy --workspace -- -D warnings` clean
  - [ ] P2: `cargo fmt --all -- --check` clean
```

---

## 5. Project-Specific Rules

### Naming Conventions

- **Files**: `snake_case.rs` — one file per entity/resource (e.g., `task.rs`, `spaces.rs`)
- **Structs**: `CamelCase` — `Task`, `ClickUpClient`, `StatusInfo`
- **Functions**: `snake_case`, verb-first — `get_tasks()`, `create_comment()`, `check_status()`
- **Constants**: `SCREAMING_SNAKE_CASE` — `DEFAULT_TIMEOUT`, `BASE_URL`
- **Modules**: `snake_case` — match file name
- **Max line width**: 100 characters (enforced by `rustfmt.toml`)

### Architectural Boundaries

- **`clickup-api` is a pure library** — it must NEVER depend on `clap`, `ratatui`, `crossterm`, or any CLI/TUI crate
- **Dependency flow is one-way**: `clickup-cli` → `clickup-api` ← `clickup-tui`. The two binaries NEVER depend on each other.
- **No business logic in CLI commands** — commands call `ClickUpClient` methods and format the output. Logic belongs in `clickup-api`.
- **No blocking I/O in TUI** — all API calls go through `tokio::spawn`; the main thread only handles events and rendering.
- **Error handling split**: `thiserror` in library, `anyhow` in binaries. Never cross this boundary.

### Test Requirements

- All async tests use `#[tokio::test]`
- HTTP mocking uses `wiremock::MockServer` — never call the real ClickUp API
- Model tests must cover: full response, minimal (id+name only), null-heavy, and type-mixed payloads
- Test names are descriptive: `test_<what>_<condition>_<expected>`
- Fixture JSON files go in `tests/fixtures/` relative to each crate
- Snapshot tests (CLI output) use `insta`
- TUI widget tests use ratatui's `TestBackend`

### Deployment Constraints

- CI runs: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test --workspace` → `cargo build --release`
- Rust edition 2024 — use latest stable Rust features
- Workspace dependencies are defined in root `Cargo.toml` — member crates reference them with `dependency.workspace = true`
- Docker builds use multi-stage (`Dockerfile` at root)
- Secrets (API tokens) use OS keyring via `keyring` crate with file fallback — never hardcoded or committed

---

## 6. Artifact Storage

```
termaup/
├── docs/                      # Architecture docs, API quirks catalog
│   └── clickup-api-quirks.md  # Known ClickUp API v2 quirks
├── .github/
│   ├── skills/                # SPECTRA skill definitions (SKILL.md per skill)
│   └── workflows/             # CI/CD pipeline (ci.yml)
├── spectra-conventions.md     # This conventions file
├── spectra-project-profile.md # Auto-detected project profile
├── spectra-adaptation-prompt.md # Adaptation prompt (input)
├── SKILLS.md                  # Skill index and dependency graph
└── AGENTS.md                  # AI agent instructions and architecture
```

Planning artifacts (stories, specs, state files) should be placed in:

```
docs/spectra/                  # SPECTRA planning artifacts
├── stories/                   # .md story files (S-1.md, S-2.md, ...)
├── patterns/                  # Reusable story patterns
└── state/                     # .state.json tracking files
```

---

*Conventions mapped for clickup-rs by SPECTRA v4.2.0*
