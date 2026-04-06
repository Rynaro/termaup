# Log Format Reference — clickup-rs

This document specifies the structured log format produced by clickup-rs binaries. Use this as a schema reference when parsing logs programmatically or building analysis tools.

## JSON Log Schema

Each line in a JSON-format log file is a self-contained JSON object with this structure:

```json
{
  "timestamp": "2026-03-25T22:38:47.976123Z",
  "level": "DEBUG",
  "target": "clickup_api::endpoints::tasks",
  "span": {
    "name": "<current_span_name>",
    "<field>": "<value>"
  },
  "spans": [
    {"name": "<outermost_span>", "<field>": "<value>"},
    {"name": "<inner_span>", "<field>": "<value>"}
  ],
  "fields": {
    "message": "<log message>",
    "<structured_field>": "<value>"
  }
}
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | string (ISO 8601) | UTC timestamp with microsecond precision |
| `level` | string | One of: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` |
| `target` | string | Rust module path (e.g., `clickup_api::client`) |
| `span` | object | Current tracing span with name and fields |
| `spans` | array | Full span stack from outermost to innermost |
| `fields.message` | string | Human-readable log message |
| `fields.*` | any | Structured data fields attached to the event |

### Common Structured Fields

| Field | Found In | Description |
|-------|----------|-------------|
| `request_id` | HTTP methods | 8-char hex UUID identifying one API request |
| `url` | HTTP methods | Full request URL |
| `params` | GET requests | Query parameters as debug-formatted array |
| `status` | Response handlers | HTTP status code |
| `body_len` | Response handlers (trace) | Response body length in bytes |
| `body_preview` | Deserialization errors | First 500 chars of response body |
| `wait_secs` | Rate limiter | Seconds to wait for rate limit reset |
| `remaining` | Rate limiter (trace) | Remaining API requests in window |
| `reset_in_secs` | Rate limiter (trace) | Seconds until rate limit resets |
| `team_id` | Workspace/space endpoints | Workspace (team) ID |
| `space_id` | Space/folder/list endpoints | Space ID |
| `folder_id` | Folder/list endpoints | Folder ID |
| `list_id` | List/task endpoints | List ID |
| `task_id` | Task endpoints | Task ID |
| `comment_id` | Comment endpoints | Comment ID |
| `page` | Paginated requests | Page number (0-indexed) |
| `original_type` | Serde helpers (trace) | Original JSON type before coercion |
| `kind` | Time value parsing (trace) | Type of time value encountered |
| `user_id` | Auth events | Authenticated user's numeric ID |
| `username` | Auth events | Authenticated user's display name |

## Target → Source File Mapping

| `target` value | Source file |
|----------------|------------|
| `clickup_api::client` | `crates/clickup-api/src/client.rs` |
| `clickup_api::rate_limiter` | `crates/clickup-api/src/rate_limiter.rs` |
| `clickup_api::endpoints::workspaces` | `crates/clickup-api/src/endpoints/workspaces.rs` |
| `clickup_api::endpoints::spaces` | `crates/clickup-api/src/endpoints/spaces.rs` |
| `clickup_api::endpoints::folders` | `crates/clickup-api/src/endpoints/folders.rs` |
| `clickup_api::endpoints::lists` | `crates/clickup-api/src/endpoints/lists.rs` |
| `clickup_api::endpoints::tasks` | `crates/clickup-api/src/endpoints/tasks.rs` |
| `clickup_api::endpoints::users` | `crates/clickup-api/src/endpoints/users.rs` |
| `clickup_api::endpoints::comments` | `crates/clickup-api/src/endpoints/comments.rs` |
| `clickup_api::serde_helpers` | `crates/clickup-api/src/serde_helpers.rs` |
| `clickup_tui::data` | `crates/clickup-tui/src/data.rs` |
| `clickup_tui::input` | `crates/clickup-tui/src/input.rs` |
| `clickup_tui` | `crates/clickup-tui/src/main.rs` |

## Example Log Entries by Category

### Successful GET Request (DEBUG)

```json
{"timestamp":"2026-03-25T22:38:47.100Z","level":"DEBUG","target":"clickup_api::client","fields":{"message":"GET request","url":"https://api.clickup.com/api/v2/task/abc123","params":[],"request_id":"a1b2c3d4"},"span":{"name":"get_task","task_id":"abc123"},"spans":[{"name":"get_task","task_id":"abc123"}]}
{"timestamp":"2026-03-25T22:38:47.350Z","level":"DEBUG","target":"clickup_api::client","fields":{"message":"response received","status":"200 OK"}}
```

### Rate Limit Warning (WARN)

```json
{"timestamp":"2026-03-25T22:38:47.500Z","level":"WARN","target":"clickup_api::rate_limiter","fields":{"message":"rate limit exhausted, waiting","wait_secs":42}}
```

### Rate Limit Header Update (TRACE)

```json
{"timestamp":"2026-03-25T22:38:47.351Z","level":"TRACE","target":"clickup_api::rate_limiter","fields":{"message":"rate limit updated","remaining":95,"reset_in_secs":58}}
```

### Deserialization Error (ERROR)

```json
{"timestamp":"2026-03-25T22:38:47.400Z","level":"ERROR","target":"clickup_api::client","fields":{"message":"deserialization failed","body_preview":"{\"id\":\"abc\",\"priority\":false,\"status\":{\"status\":\"open\",\"type\":\"open\"}}"}}
```

### Authentication Error

```json
{"timestamp":"2026-03-25T22:38:47.200Z","level":"DEBUG","target":"clickup_api::client","fields":{"message":"response received","status":"401 Unauthorized"}}
```

### Type Coercion — ID as Number (TRACE)

```json
{"timestamp":"2026-03-25T22:38:47.300Z","level":"TRACE","target":"clickup_api::serde_helpers","fields":{"message":"coerced number to string","original_type":"u64","value":12345}}
```

### Type Coercion — Boolean as Integer (TRACE)

```json
{"timestamp":"2026-03-25T22:38:47.301Z","level":"TRACE","target":"clickup_api::serde_helpers","fields":{"message":"coerced integer to bool","original_type":"i64"}}
```

### Priority as `false` (TRACE)

```json
{"timestamp":"2026-03-25T22:38:47.302Z","level":"TRACE","target":"clickup_api::serde_helpers","fields":{"message":"maybe_false: got `false` instead of null/object"}}
```

### Time Value Coercion (TRACE)

```json
{"timestamp":"2026-03-25T22:38:47.303Z","level":"TRACE","target":"clickup_api::serde_helpers","fields":{"message":"time_value type coercion","kind":"object"}}
```

### TUI User Authentication (INFO)

```json
{"timestamp":"2026-03-25T22:38:48.000Z","level":"INFO","target":"clickup_tui","fields":{"message":"authenticated user loaded","user_id":1001,"username":"alice"}}
```

### TUI Data Loading (DEBUG)

```json
{"timestamp":"2026-03-25T22:38:48.100Z","level":"DEBUG","target":"clickup_tui::data","fields":{"message":"loading workspaces"}}
{"timestamp":"2026-03-25T22:38:48.500Z","level":"DEBUG","target":"clickup_tui::data","fields":{"message":"loading spaces","team_id":"t1"}}
```

## Log Export Header Format

The `clickup logs export` command produces a file with this header:

```
=== clickup-rs log export ===
Generated: 2026-03-25T22:38:47Z
OS: macos arm64
Version: clickup-rs 0.1.0
Config: /Users/alice/.config/clickup-rs/config.toml
Sessions: 3 (last 24h)
===
```

Followed by session blocks:

```
[session: tui-2026-03-25T22-30-00.log]
{...json line 1...}
{...json line 2...}

[session: cli-2026-03-25T22-35-00.log]
{...json line 1...}
```

### Redaction Rules

| Data | Original | Redacted |
|------|----------|----------|
| API token | `pk_12345678_abcdefgh` | `pk_[REDACTED]` |
| Email | `alice@example.com` | `a***@example.com` |

## Useful `jq` Queries

For parsing JSON log files with `jq`:

```bash
# All errors
jq 'select(.level == "ERROR")' logfile.log

# Events for a specific request_id
jq 'select(.fields.request_id == "a1b2c3d4")' logfile.log

# All rate limit events
jq 'select(.fields.message | test("rate limit"))' logfile.log

# Deserialization failures with body preview
jq 'select(.fields.message == "deserialization failed") | .fields.body_preview' logfile.log

# All events from a specific endpoint module
jq 'select(.target | test("endpoints::tasks"))' logfile.log

# Timeline of events (timestamp + level + message)
jq '{t: .timestamp, l: .level, m: .fields.message}' logfile.log

# Count events by level
jq -s 'group_by(.level) | map({level: .[0].level, count: length})' logfile.log
```
