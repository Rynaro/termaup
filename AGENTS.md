# AGENTS.md — clickup-rs

You are building **clickup-rs**, a fast CLI and TUI client for the ClickUp project management platform, written entirely in Rust.

## Identity & Mission

- This is a greenfield Rust project — there is NO existing ClickUp CLI/TUI in the Rust ecosystem.
- The goal is to replace the bloated ClickUp web app for power users who prefer the terminal.
- Quality, performance, and developer experience are paramount.

## Architecture

The project is a **Cargo workspace** with three member crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `crates/clickup-api` | Library | Pure async API client for ClickUp API v2 |
| `crates/clickup-cli` | Binary | clap-based CLI application |
| `crates/clickup-tui` | Binary | ratatui-based TUI application |

Dependency flow: `clickup-cli` and `clickup-tui` both depend on `clickup-api`. The two binaries NEVER depend on each other.

## ClickUp API v2 Reference

Base URL: `https://api.clickup.com/api/v2`

### Authentication
- **Personal Token**: Sent as `Authorization: {token}` header (token starts with `pk_`)
- **OAuth2**: Authorization code flow → `POST /oauth/token` to exchange code for access token, then `Authorization: Bearer {token}`
- Token validation: `GET /user` returns the authenticated user

### Resource Hierarchy
```
Workspace (Team) → Space → Folder → List → Task
                          └→ List (folderless) → Task
```

### Key Endpoints (MVP scope)
| Method | Endpoint | Returns |
|--------|----------|---------|
| GET | `/user` | Authenticated user info |
| GET | `/team` | All workspaces (teams) |
| GET | `/team/{team_id}/space` | Spaces in a workspace |
| GET | `/space/{space_id}` | Space details |
| GET | `/space/{space_id}/folder` | Folders in a space |
| GET | `/folder/{folder_id}/list` | Lists in a folder |
| GET | `/space/{space_id}/list` | Folderless lists in a space |
| GET | `/list/{list_id}` | List details |
| GET | `/list/{list_id}/task` | Tasks in a list (paginated, `page` param) |
| GET | `/task/{task_id}` | Task details (supports `?include_subtasks=true&include_markdown_description=true`) |
| GET | `/task/{task_id}/comment` | Task comments |

### Rate Limits
- 100 req/min (Free/Unlimited/Business), 1000 (Business+), 10000 (Enterprise)
- Response headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset` (Unix timestamp)
- HTTP 429 on rate limit exceeded — must implement backoff

### Pagination
- Task lists use `page` parameter (0-indexed)
- Response includes a `last_page` boolean field in `TasksResponse`
- Always paginate until `last_page == true`

### Error Responses
```json
{ "err": "error message", "ECODE": "ERROR_CODE" }
```

## Skills

Skill definitions for each development workstream are in `.github/skills/`. Copilot loads them automatically when relevant to the current task. See `SKILLS.md` for the dependency graph and index.

## Dependency Stack

| Purpose | Crate | Notes |
|---------|-------|-------|
| HTTP | `reqwest` 0.12 | features: json, rustls-tls |
| Async | `tokio` 1.x | features: full |
| Serialization | `serde` 1 + `serde_json` 1 | features: derive |
| CLI | `clap` 4 | features: derive, env |
| TUI | `ratatui` 0.29 | — |
| Terminal | `crossterm` 0.28 | — |
| Errors (lib) | `thiserror` 2 | — |
| Errors (bin) | `anyhow` 1 | — |
| Logging | `tracing` 0.1 + `tracing-subscriber` 0.3 | features: env-filter |
| Secrets | `keyring` 3 | OS-native credential storage |
| Config | `toml` 0.8 + `dirs` 6 | Config at ~/.config/clickup-rs/ |
| Markdown (CLI) | `termimad` 0.30 | Terminal markdown rendering |
| Markdown (TUI) | `pulldown-cmark` 0.12 | Parse MD → ratatui spans |
| Tables (CLI) | `comfy-table` 7 | — |
| Colors | `owo-colors` 4 | — |
| Dates | `chrono` 0.4 | — |
| Prompts | `dialoguer` 0.11 | Interactive selection |
| Password | `rpassword` 7 | Hidden token input |
| URLs | `url` 2 | — |
| Secrets wrapping | `secrecy` 0.10 | — |
| Testing HTTP | `wiremock` 0.6 | dev-dependency |
| Snapshot testing | `insta` | dev-dependency |

## File Organization Patterns

### API Client Crate (`crates/clickup-api/src/`)
```
lib.rs          → pub mod declarations
error.rs        → ClickUpError enum + Result type alias
config.rs       → Config struct, load/save, paths
auth.rs         → TokenStorage (keyring + file fallback)
client.rs       → ClickUpClient struct, HTTP methods
rate_limiter.rs → Rate limit tracking and auto-wait
pagination.rs   → Generic pagination helpers
models/         → One file per entity (user.rs, workspace.rs, space.rs, etc.)
endpoints/      → One file per resource (users.rs, workspaces.rs, spaces.rs, etc.)
```

### CLI Crate (`crates/clickup-cli/src/`)
```
main.rs          → Clap app setup, tracing init, command routing
commands/        → One file per command group (auth.rs, spaces.rs, lists.rs, tasks.rs)
output.rs        → Formatters (table, json, markdown, colors)
client_factory.rs → Helper to create authenticated ClickUpClient
```

### TUI Crate (`crates/clickup-tui/src/`)
```
main.rs     → Terminal setup/teardown, main loop
app.rs      → App state machine, Screen enum
event.rs    → Event system (crossterm events + async data events)
data.rs     → Async data loading functions
input.rs    → Key event routing per screen
theme.rs    → Color theme definitions
ui/         → One file per screen (workspace_select.rs, space_list.rs, list_view.rs, task_detail.rs)
  layout.rs → Standard layout (top bar, main, bottom bar)
  search.rs → Search/filter overlay
  help.rs   → Help modal overlay
  markdown.rs → MD→ratatui spans converter
widgets/    → Custom ratatui widgets
```

## Configuration & Storage

- Config file: `~/.config/clickup-rs/config.toml`
- Token: OS keyring (service: "clickup-rs", username: "api_token"), fallback to `~/.config/clickup-rs/.token`
- TUI log: `~/.config/clickup-rs/tui.log`
- Environment variables: `CLICKUP_LOG` (log level), `CLICKUP_TOKEN` (token override)

## Defensive Deserialization

The ClickUp API v2 is type-unstable and endpoint-inconsistent. See [`docs/clickup-api-quirks.md`](docs/clickup-api-quirks.md) for the full catalog of known quirks.

**Golden rule**: Only `id` and `name` fields should be required on model structs. Everything else must tolerate absence, `null`, or unexpected types.

Key patterns:
- All `Vec` fields: `#[serde(default, deserialize_with = "deserialize_null_as_default")]`
- All ID fields: `deserialize_string_or_number` or `deserialize_default_string_or_number`
- Sub-objects (`list`, `folder`, `space`, `creator`): `Option<T>` + `#[serde(default)]`
- Boolean fields: consider `deserialize_bool_or_int` (API sends `0`/`1`)
- Priority: `deserialize_maybe_false` (API sends `false` not `null`)
- Time values: `deserialize_time_value` (number, `{"time": ms}`, or string)
- Every model must have tests for: full, minimal (id+name only), null-heavy, and type-mixed responses

## Code Quality Gates

Every PR must pass:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
4. `cargo build --workspace --release`

<!-- eidolon:atlas start -->
## ATLAS — Read-only codebase scout (v1.5.2)

Entry:     `./.eidolons/atlas/agent.md`
Full spec: `./.eidolons/atlas/ATLAS.md`
Cycle:     A (Assess) → T (Traverse) → L (Locate) → A (Abstract) → S (Synthesize)

**P0 (non-negotiable):** read-only (refuse edit/write/commit/deploy/migrate/refactor/fix); mission-first (requires `mission.md` + `DECISION_TARGET`); bounded ACI (`view_file` ≤100, `search_text` ≤50, `list_dir` ≤200); evidence-anchored claims (`path:line` + H|M|L); deterministic retrieval first, LLM search last.
<!-- eidolon:atlas end -->

<!-- eidolon:spectra start -->
## SPECTRA — Decision-ready specifications (v4.3.2)

Entry:     `.eidolons/spectra/agent.md`
Full spec: `.eidolons/spectra/SPECTRA.md`
Cycle:     CLARIFY → Scope → Pattern → Explore → Construct → Test → Refine → Assemble

**P0 (non-negotiable):** READ-ONLY during all planning phases (no code edits); dual-format output (Markdown + YAML/JSON); CLARIFY first (parse WHO/WHAT/WHY/CONSTRAINTS); confidence ≥85% at Assemble (else Refine, max 3 cycles); output is a specification, never an implementation.
<!-- eidolon:spectra end -->

<!-- eidolon:apivr start -->
## APIVR-Δ — Brownfield feature implementation (v3.1.2)

Entry:     `.eidolons/apivr/agent.md`
Full spec: `.eidolons/apivr/apivr.md`
Cycle:     A (Analyze) → P (Plan) → I (Implement) → V (Verify) → Δ (Delta) / R (Reflect)

**P0 (non-negotiable):** Internal First (USE → EXTEND → WRAP → CREATE); test-anchored (expected test cases before implementation); boundary-respect (no out-of-scope edits); evidence-based (no speculation); escalate early (3 failures at same category = STOP).
<!-- eidolon:apivr end -->

<!-- eidolon:idg start -->
## IDG — Documentation synthesis (v1.2.2)

Entry:     `.eidolons/idg/agent.md`
Full spec: `.eidolons/idg/IDG.md`
Cycle:     I (Intake) → D (Draft) → G (Gate)

**P0 (non-negotiable):** synthesis from provided context only (no retrieval or code analysis); structural markers ([DECISION], [ACTION], [DISPUTED], [GAP]) required; CHT verification gate (Completeness / Helpfulness / Truthfulness) with one revision max; provenance-first (every claim traces to source session).
<!-- eidolon:idg end -->

<!-- eidolon:forge start -->
## FORGE — Reasoner / structured deliberation (v1.3.2)

Entry: `./.eidolons/forge/agent.md`
Spec:  `./.eidolons/forge/REASONER.md`
Cycle: F (Frame) → O (Observe) → R (Reason) → G (Gate) → E (Emit)

**P0 (non-negotiable):** reasoning-only (no tools, no mutations); frame first
(refuse vague questions); ≥3 hypotheses with adversarial stress-tests;
evidence-anchored claims (H/M/L tiers); bounded deliberation (≤3 passes +
1 REFORGE); reversal conditions mandatory.

See `./.eidolons/forge/AGENTS.md` for full rules and the phase pipeline.
<!-- eidolon:forge end -->

<!-- eidolon:vigil start -->
## VIGIL — Forensic debugger (v1.1.2)

Entry:     `./.eidolons/vigil/agent.md`
Full spec: `./.eidolons/vigil/VIGIL.md`
Cycle:     V (Verify) → I (Isolate) → G (Graph) → I (Intervene) → L (Learn)
Authority: read-only

**P0 (non-negotiable):** reproduction gates attribution (≥2 deterministic runs or statistical CI ≥85%); dependency-graph ranking (never temporal order); ≥3 hypotheses before intervention; counterfactual-gated blame (minimal flip from fail→success); ≤5 intervention budget then escalate; flag-gated authority (read-only | sandbox | write — write never inferred); evidence-anchored findings with `path:line` + confidence tier; non-determinism declared, not masked.
<!-- eidolon:vigil end -->

<!-- eidolon:cortex start -->
## Eidolons Routing Cortex

When a free-form prompt arrives that doesn't already name an Eidolon, route it via the cortex.

**Read:** `.eidolons/cortex/EIDOLONS.md` — always-loaded descriptor table + dispatch protocol. It tells you which Eidolon (or chain) handles the prompt, at what tier (`standard` or `TRANCE`), and what hand-off contract to use.

**Deep tables** (load on demand): `.eidolons/cortex/trance-matrix.md`, `.eidolons/cortex/handoff-graph.md`, `.eidolons/cortex/validation-gates.md`.
<!-- eidolon:cortex end -->
