---
name: log-analysis
description: Diagnose issues from clickup-rs log files. Use this when analyzing logs, debugging errors, or helping users distill bug reports from log output.
---

# Log Analysis

## Context

This skill teaches you how to read, interpret, and diagnose issues from clickup-rs structured log output. Logs are produced by the `tracing` crate in both JSON and human-readable formats. The primary consumers are:

1. **LLM agents** debugging development issues during implementation
2. **Users** submitting bug reports with exported logs
3. **Maintainers** diagnosing reported issues from sanitized log bundles

## Crates

- `clickup-api` (produces all HTTP and API-level log events)
- `clickup-cli` (produces CLI-level lifecycle events, writes to stderr + file)
- `clickup-tui` (produces TUI-level lifecycle events, writes to per-session file)

## Log Locations

| Binary | Live Output | File Output |
|--------|-------------|-------------|
| CLI | stderr (pretty or JSON via `CLICKUP_LOG_FORMAT`) | `~/.config/clickup-rs/logs/cli-{timestamp}.log` |
| TUI | none (alternate screen) | `~/.config/clickup-rs/logs/tui-{timestamp}.log` |

## Log Format

### JSON (default for files)

Each line is a self-contained JSON object:

```json
{
  "timestamp": "2026-03-25T22:38:47.976123Z",
  "level": "DEBUG",
  "fields": {
    "message": "GET request",
    "url": "https://api.clickup.com/api/v2/task/abc123",
    "request_id": "a1b2c3d4"
  },
  "target": "clickup_api::client",
  "span": {
    "name": "get_task",
    "task_id": "abc123"
  },
  "spans": [
    {"name": "get_task", "task_id": "abc123"}
  ]
}
```

### Pretty (default for CLI stderr)

```
2026-03-25T22:38:47.976Z DEBUG clickup_api::client: GET request url=https://api.clickup.com/api/v2/task/abc123 request_id=a1b2c3d4
```

## Log Level Hierarchy

| Level | What It Captures | When To Use |
|-------|-----------------|-------------|
| `ERROR` | Deserialization failures, auth errors, unrecoverable errors | Always relevant — start here |
| `WARN` | Rate limiting waits, API quirk detection, degraded behavior | Important for API issues |
| `INFO` | Request/response summaries, state transitions, auth events | Good baseline for TUI |
| `DEBUG` | Full request URLs, query params, request IDs, config loading | Development debugging |
| `TRACE` | Response body content, serde type coercions, rate limit header updates | Deep API debugging |

## Diagnosis Workflow

When analyzing logs, follow this systematic approach:

### Step 1: Identify the Error Event

Search for `ERROR` level entries first. Common patterns:

```json
{"level":"ERROR","fields":{"message":"deserialization failed","body_preview":"..."}}
```

This indicates the ClickUp API returned a response body that couldn't be parsed into the expected Rust struct. The `body_preview` field contains the first 500 characters of the raw response.

### Step 2: Trace the Request Chain

Every HTTP request has a `request_id` field (8-character hex string). Use this to correlate:

1. The initial request log (`GET request`, `POST request`, etc.)
2. The response received log (`response received` with status code)
3. Any error that occurred during processing

Example correlation:
```
grep "a1b2c3d4" logfile.log
```

### Step 3: Check the Span Context

`#[instrument]` spans provide hierarchical context. The `spans` array shows the call stack:

```json
{
  "spans": [
    {"name": "get_task", "task_id": "abc123"},
    {"name": "get_with_params"}
  ]
}
```

This tells you: `get_task("abc123")` called `get_with_params()` internally.

### Step 4: Classify the Error

Map the error to one of these categories:

#### Authentication Errors
- **Pattern**: `"level":"ERROR"` + `"status":401` or `AuthError`
- **Cause**: Invalid/expired token, missing `Authorization` header
- **Fix**: Re-run `clickup auth login`

#### Rate Limiting
- **Pattern**: `"level":"WARN"` + `"rate limit exhausted, waiting"`
- **Fields**: `wait_secs` shows how long the client paused
- **Cause**: Too many API calls in the rate window
- **Note**: The client auto-waits; this is informational unless `wait_secs` is very high

#### Deserialization Errors
- **Pattern**: `"level":"ERROR"` + `"deserialization failed"` + `body_preview`
- **Fields**: `body_preview` contains raw API response, `endpoint` shows which URL
- **Cause**: ClickUp API returned unexpected field types or missing fields
- **Action**: Check `docs/clickup-api-quirks.md` for known quirks; the body_preview reveals what the API actually sent

#### API Quirk Detection (trace-level)
- **Pattern**: `"level":"TRACE"` + `"coerced number to string"` or `"coerced integer to bool"`
- **Cause**: API returned a number where a string was expected (or vice versa)
- **Note**: These are handled automatically by serde helpers; trace logging confirms the coercion

#### Network Errors
- **Pattern**: `NetworkError` in error fields
- **Cause**: DNS resolution failure, connection timeout, TLS errors
- **Action**: Check network connectivity, firewall rules, proxy settings

#### Not Found (404)
- **Pattern**: `"status":404` or `NotFound`
- **Cause**: The resource ID doesn't exist or the user lacks access
- **Action**: Verify the task/space/list ID is correct

### Step 5: Extract Minimal Reproduction Context

For a bug report, extract:
1. The first `ERROR` or `WARN` event
2. All events with the same `request_id`
3. The preceding 5-10 events (for context)
4. System info from the export header

## Common Error Scenarios

### Scenario: "deserialization failed" on task fetch

```json
{"level":"ERROR","target":"clickup_api::client","fields":{"message":"deserialization failed","body_preview":"{\"id\":\"abc\",\"priority\":false,...}"}}
```

**Diagnosis**: The `priority` field is `false` instead of `null` or an object. This is ClickUp API Quirk #4. The `deserialize_maybe_false` helper should handle it. If this error still occurs, the model struct may be missing the `#[serde(deserialize_with = "deserialize_maybe_false")]` attribute.

### Scenario: Rate limit wait during bulk operations

```json
{"level":"WARN","fields":{"message":"rate limit exhausted, waiting","wait_secs":45}}
```

**Diagnosis**: The client hit the 100 req/min limit. This is expected during bulk task loading. The client auto-waits. If `wait_secs` is consistently high, consider adding a delay between paginated requests.

### Scenario: 401 on comment edit

```json
{"level":"DEBUG","spans":[{"name":"update_comment","comment_id":"reply_123"}],"fields":{"message":"PUT request","url":".../comment/reply_123"}}
{"level":"ERROR","fields":{"message":"Authentication failed: Token invalid"}}
```

**Diagnosis**: This is ClickUp API Quirk #11. `PUT /comment/{id}` only works on top-level comments. Reply IDs return 401. The fix is to use delete-and-recreate for reply edits.

## Exported Log Format

The `clickup logs export` command produces a sanitized bundle:

```
=== clickup-rs log export ===
Generated: 2026-03-25T22:38:47Z
OS: macOS arm64
Version: clickup-rs 0.1.0
Config: ~/.config/clickup-rs/config.toml
Sessions: 3 (last 24h)
===

[session: tui-2026-03-25T22-30-00.log]
{"timestamp":"...","level":"INFO",...}
...
```

**Redaction**: Tokens (`pk_*`) are replaced with `pk_[REDACTED]`. Email addresses are masked (e.g., `a***@example.com`).

## Environment Variables

| Variable | Effect on Logging |
|----------|------------------|
| `CLICKUP_LOG` | Sets the log filter level (e.g., `debug`, `trace`, `clickup_api=trace`) |
| `CLICKUP_LOG_FORMAT` | Sets output format: `json` or `pretty` (default: `pretty` for stderr, `json` for files) |

## CLI Commands for Log Management

| Command | Purpose |
|---------|---------|
| `clickup logs list` | Show recent log session files |
| `clickup logs show [session]` | Display log file contents |
| `clickup logs show --tail 50` | Show last 50 lines |
| `clickup logs export --hours 24` | Export redacted logs for bug reports |
| `clickup logs export -o report.log` | Export to a file |
| `clickup logs clean --older-than 7` | Remove logs older than 7 days |
| `clickup logs clean --dry-run` | Preview what would be removed |

## Acceptance

- Can parse both JSON and pretty log formats from file or stdin
- Can correlate request chains using `request_id` fields
- Can identify the root cause error from a multi-line log sequence
- Can extract a minimal relevant log window for a bug report
- Can map `target` fields to source code locations (e.g., `clickup_api::endpoints::tasks` → `crates/clickup-api/src/endpoints/tasks.rs`)
- Can recognize all 11 ClickUp API quirks from their log signatures
- Produces structured diagnosis with: error category, root cause, affected endpoint, recommended fix
