# F23 — OAuth2 Authentication

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 6 — Distribution & Infrastructure
> **Complexity:** 7/12 | **Confidence:** 82%

---

## Problem Statement

termaup currently supports only personal API tokens (`pk_*`) for authentication. Many ClickUp users, especially those in organizations with SSO or security policies, cannot generate personal tokens and need OAuth2 authorization. Without OAuth2 support, termaup is inaccessible to a significant segment of ClickUp users, and third-party distribution (e.g., via Homebrew) cannot offer a seamless first-run experience.

## Approach

Implement the ClickUp OAuth2 Authorization Code flow with a local HTTP callback server. When a user runs `clickup auth login --oauth`, the CLI:

1. Reads `client_id` and `client_secret` from config or environment variables.
2. Starts a lightweight HTTP server on `127.0.0.1:9876` to listen for the OAuth callback.
3. Opens the user's browser to `https://app.clickup.com/api?client_id={ID}&redirect_uri=http://127.0.0.1:9876/callback`.
4. The user authorizes in the browser; ClickUp redirects to the callback URL with `?code={CODE}`.
5. The local server captures the code, shuts down, and exchanges it for an access token via `POST /api/v2/oauth/token`.
6. The access token is stored using the existing `TokenStorage` mechanism (keyring + file fallback).

The token is sent as `Authorization: Bearer {TOKEN}` instead of the bare `Authorization: {TOKEN}` used for personal tokens. `ClickUpClient` must detect the token type and set the header accordingly.

### Rejected Alternatives

1. **Device Authorization Grant (RFC 8628)** — ClickUp does not support this flow. Rejected because the API simply does not offer it.

2. **Manual code paste (no local server)** — User copies the authorization code from the browser URL and pastes it into the CLI. Simpler to implement but poor UX — the redirect URI would need to be a non-functional URL, and users would have to manually extract the `code` query parameter. Rejected for usability.

3. **Store client_id/secret in keyring** — These are not user secrets; they are application credentials typically shared across installations. Config file or environment variables are the standard approach. Rejected because keyring adds complexity for non-sensitive data.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| OAuth2 Authorization Code flow | OAuth2 PKCE extension (ClickUp doesn't support it) | TUI OAuth login flow |
| Local HTTP callback server | Token refresh (ClickUp tokens are long-lived, no refresh_token) | OAuth token expiry monitoring |
| Config fields for `client_id`/`client_secret` | Multi-tenant OAuth (multiple client_id per user) | OAuth scope selection |
| Env var support (`CLICKUP_CLIENT_ID`, `CLICKUP_CLIENT_SECRET`) | SSO/SAML integration | |
| Bearer token header detection in `ClickUpClient` | Revoking OAuth tokens via API | |
| CLI `clickup auth login --oauth` command | | |
| Browser auto-open via `open` / `xdg-open` | | |

## Stories

### S-1: OAuth Configuration Model

**As a** developer configuring OAuth2 credentials, **I want** `client_id` and `client_secret` available via config file or environment variables, **so that** the OAuth flow has the credentials it needs without hardcoding.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add OAuth fields to `Config` struct | `crates/clickup-api/src/config.rs` | Add `oauth_client_id: Option<String>` and `oauth_client_secret: Option<String>` with `#[serde(default)]` |
| 2 | Add env var fallback | `crates/clickup-api/src/config.rs` | Method `Config::oauth_client_id()` that checks field, then `CLICKUP_CLIENT_ID` env var |
| 3 | Add env var fallback for secret | `crates/clickup-api/src/config.rs` | Method `Config::oauth_client_secret()` that checks field, then `CLICKUP_CLIENT_SECRET` env var |
| 4 | Add `OAuthConfig` helper struct | `crates/clickup-api/src/auth.rs` | `OAuthConfig { client_id: String, client_secret: String, redirect_uri: String }` for passing around OAuth params |
| 5 | Add default redirect URI constant | `crates/clickup-api/src/auth.rs` | `const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:9876/callback"` |
| 6 | Unit tests | `crates/clickup-api/src/config.rs` | Test config round-trips with and without OAuth fields |

#### Acceptance Criteria

- **GIVEN** a `config.toml` with `oauth_client_id = "abc"`, **WHEN** `Config::load()` is called, **THEN** `config.oauth_client_id` is `Some("abc")`.
- **GIVEN** no `oauth_client_id` in config but `CLICKUP_CLIENT_ID=xyz` env var, **WHEN** `Config::oauth_client_id()` is called, **THEN** it returns `Some("xyz")`.
- **GIVEN** neither config nor env var set, **WHEN** `Config::oauth_client_id()` is called, **THEN** it returns `None`.
- **GIVEN** an existing config without OAuth fields, **WHEN** loaded, **THEN** deserialization succeeds with `None` for both OAuth fields.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/config.rs` (existing `Config` struct at ~line 40), `crates/clickup-api/src/auth.rs` (existing `TokenStorage`)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean

---

### S-2: Local OAuth Callback Server

**As a** CLI user initiating OAuth login, **I want** a temporary local HTTP server to capture the authorization code, **so that** I don't have to manually copy-paste codes from the browser.

**Timebox:** ≤2d | **Risk:** Medium (port conflicts, firewall) | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `tokio` + `hyper` or `tiny_http` dep | `crates/clickup-api/Cargo.toml` | Prefer lightweight HTTP: use `tokio::net::TcpListener` + manual HTTP parsing, or add `tiny_http` as optional dep behind `oauth` feature |
| 2 | Create OAuth module | `crates/clickup-api/src/oauth.rs` | New module for OAuth flow logic |
| 3 | Implement callback server | `crates/clickup-api/src/oauth.rs` | `async fn start_callback_server(port: u16) -> Result<String>` — binds TCP, waits for single GET request, extracts `code` param, returns HTML success page, shuts down |
| 4 | Handle error responses | `crates/clickup-api/src/oauth.rs` | If callback URL contains `error` param instead of `code`, return descriptive error |
| 5 | Add timeout | `crates/clickup-api/src/oauth.rs` | Server times out after 120 seconds if no callback received |
| 6 | Add `OAuthError` variants | `crates/clickup-api/src/error.rs` | `OAuthError(String)`, `OAuthTimeout`, `OAuthCallbackFailed(String)` |
| 7 | Unit test | `crates/clickup-api/src/oauth.rs` | Test with simulated HTTP request to callback endpoint |

#### Acceptance Criteria

- **GIVEN** the callback server is started on port 9876, **WHEN** a GET request arrives at `/callback?code=ABC123`, **THEN** the server extracts `"ABC123"` and returns it.
- **GIVEN** the callback server is started, **WHEN** the GET request contains `?error=access_denied`, **THEN** an `OAuthCallbackFailed` error is returned.
- **GIVEN** the callback server is started, **WHEN** no request arrives within 120 seconds, **THEN** an `OAuthTimeout` error is returned.
- **GIVEN** port 9876 is already in use, **WHEN** the server tries to bind, **THEN** a descriptive error is returned (not a panic).

#### Agent Hints

- **Class:** builder
- **Context:** Use `tokio::net::TcpListener` + `tokio::io` for minimal deps. Parse HTTP request manually — only need to extract query params from a single GET request. Return a simple HTML page: `<h1>Authorization successful!</h1><p>You can close this tab.</p>`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected (no CLI/TUI deps in clickup-api)
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P2: `cargo clippy` clean

---

### S-3: Browser Open + Token Exchange Flow

**As a** developer, **I want** a function that orchestrates the full OAuth flow (open browser → wait for code → exchange for token), **so that** the CLI can call a single function to complete OAuth login.

**Timebox:** ≤2d | **Risk:** Medium (browser opening varies by OS) | **Depends on:** S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add browser-open utility | `crates/clickup-api/src/oauth.rs` | `fn open_browser(url: &str) -> Result<()>` using `std::process::Command` with `open` (macOS), `xdg-open` (Linux), `start` (Windows) |
| 2 | Build authorization URL | `crates/clickup-api/src/oauth.rs` | `fn build_auth_url(config: &OAuthConfig) -> String` — constructs `https://app.clickup.com/api?client_id={}&redirect_uri={}` |
| 3 | Implement token exchange | `crates/clickup-api/src/oauth.rs` | `async fn exchange_code(client_id: &str, client_secret: &str, code: &str) -> Result<String>` — POST to `/api/v2/oauth/token` with JSON body `{ client_id, client_secret, code }`, parse `access_token` from response |
| 4 | Orchestrator function | `crates/clickup-api/src/oauth.rs` | `pub async fn perform_oauth_flow(config: &OAuthConfig) -> Result<String>` — starts callback server, opens browser, waits for code, exchanges, returns token |
| 5 | Update `ClickUpClient` for Bearer tokens | `crates/clickup-api/src/client.rs` | Detect if token starts with `pk_` → use `Authorization: {token}`, else use `Authorization: Bearer {token}` |
| 6 | Token exchange response model | `crates/clickup-api/src/oauth.rs` | `struct OAuthTokenResponse { access_token: String }` with serde derive |
| 7 | Integration test with wiremock | `crates/clickup-api/src/oauth.rs` | Mock the token exchange endpoint, verify correct request body and header parsing |

#### Acceptance Criteria

- **GIVEN** valid `client_id`, `client_secret`, and authorization `code`, **WHEN** `exchange_code()` is called, **THEN** it POSTs to `https://api.clickup.com/api/v2/oauth/token` and returns the `access_token`.
- **GIVEN** an invalid code, **WHEN** `exchange_code()` is called, **THEN** it returns an `OAuthError` with the API error message.
- **GIVEN** a token that does NOT start with `pk_`, **WHEN** `ClickUpClient` is created, **THEN** the `Authorization` header is set to `Bearer {token}`.
- **GIVEN** a token starting with `pk_`, **WHEN** `ClickUpClient` is created, **THEN** the `Authorization` header is set to `{token}` (no Bearer prefix).
- **GIVEN** the full OAuth flow, **WHEN** `perform_oauth_flow()` is called, **THEN** it opens the browser, waits for callback, exchanges code, and returns the access token.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/client.rs` (existing `ClickUpClient::new()` around line 30 sets headers). The token exchange endpoint is NOT under the base URL — it's at `https://api.clickup.com/api/v2/oauth/token` directly. Use `reqwest::Client` (not the `ClickUpClient` — it's not authenticated yet).
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Bearer vs personal token detection works
  - [ ] P2: `cargo clippy` clean

---

### S-4: CLI OAuth Login Integration

**As a** CLI user, **I want** `clickup auth login --oauth` to walk me through the OAuth flow, **so that** I can authenticate without a personal API token.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `--oauth` flag to login command | `crates/clickup-cli/src/commands/auth.rs` | Add `Login { #[arg(long)] oauth: bool }` variant or flag on existing login |
| 2 | Implement OAuth login handler | `crates/clickup-cli/src/commands/auth.rs` | If `--oauth`: load config, validate client_id/secret present, print instructions, call `perform_oauth_flow()`, store token via `TokenStorage::store_token()`, print success |
| 3 | Print pre-flight info | `crates/clickup-cli/src/commands/auth.rs` | Before opening browser, print: "Opening browser for ClickUp authorization...\nWaiting for callback on http://127.0.0.1:9876/callback..." |
| 4 | Error messaging | `crates/clickup-cli/src/commands/auth.rs` | If `client_id`/`client_secret` missing, print actionable error: "Set CLICKUP_CLIENT_ID and CLICKUP_CLIENT_SECRET environment variables, or add oauth_client_id and oauth_client_secret to ~/.config/clickup-rs/config.toml" |
| 5 | Update `auth status` | `crates/clickup-cli/src/commands/auth.rs` | Show token type (personal vs OAuth) in status output |

#### Acceptance Criteria

- **GIVEN** `CLICKUP_CLIENT_ID` and `CLICKUP_CLIENT_SECRET` are set, **WHEN** `clickup auth login --oauth` is run, **THEN** the browser opens, the callback server starts, and upon successful authorization the token is stored.
- **GIVEN** OAuth credentials are NOT configured, **WHEN** `clickup auth login --oauth` is run, **THEN** a clear error message explains how to configure them.
- **GIVEN** a successful OAuth login, **WHEN** `clickup auth status` is run, **THEN** it shows the user is authenticated (token type does not need to be distinguished in status).
- **GIVEN** a successful OAuth login, **WHEN** any API command is run, **THEN** the Bearer token is sent correctly and API calls succeed.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/auth.rs` (existing login flow uses `rpassword` for token input), `crates/clickup-cli/src/output.rs` (success/error prefix helpers: `"✓"`, `"✗"`, `"ℹ"`)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (config model) → S-2 (callback server) → S-3 (browser + exchange) → S-4 (CLI integration)
```

Linear dependency chain — each story builds on the previous.

## Assumptions

1. **ClickUp OAuth tokens are long-lived** and do not require a refresh flow. Risk if wrong: would need to add refresh token storage and auto-refresh logic in `ClickUpClient`. Mitigation: the current token storage can be extended with a refresh token field.
2. **Port 9876 is generally available** on developer machines. Risk if wrong: users behind corporate firewalls or with port conflicts may fail. Mitigation: S-2 returns a descriptive error; future work could make the port configurable.
3. **`open` / `xdg-open` / `start` commands are available** on the target OS. Risk if wrong: the browser won't open automatically. Mitigation: print the URL to the terminal so the user can open it manually.
4. **ClickUp's OAuth token response** returns `{ "access_token": "..." }`. Risk if wrong: adjust the response parsing model.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Touches config, auth, new OAuth module, client header logic, CLI command |
| Ambiguity | 2 | OAuth flow is standard but ClickUp-specific details (no PKCE, no refresh) need validation |
| Dependencies | 2 | May need new deps (tiny_http or manual TCP), touches client.rs header logic |
| Risk | 1 | Local server approach is proven; browser opening is best-effort with manual fallback |

**Total: 7/12** → Standard processing
