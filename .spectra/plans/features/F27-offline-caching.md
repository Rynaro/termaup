# F27 — Offline Caching

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 7 — Advanced Features
> **Complexity:** 9/12 | **Confidence:** 72%

---

## Problem Statement

Every TUI screen transition requires a full API round-trip, resulting in 1–3 second loading delays even on fast connections. Users navigating back to a previously viewed screen wait for data they already fetched seconds ago. On slow or unreliable networks, termaup becomes unusable. An intelligent caching layer with TTL-based invalidation and stale-while-revalidate semantics would eliminate redundant API calls, provide instant navigation for recently viewed data, and enable graceful degradation on poor connections.

## Approach

Implement a filesystem-based JSON cache at `~/.cache/clickup-rs/` (respecting `XDG_CACHE_HOME`) as a transparent layer between the `ClickUpClient` and its consumers. The cache stores serialized API responses keyed by endpoint path + parameter hash, with configurable TTL per resource type:

| Resource | TTL |
|----------|-----|
| Workspaces | 1 hour |
| Spaces | 30 minutes |
| Folders/Lists | 15 minutes |
| Tasks (list) | 5 minutes |
| Task detail | 2 minutes |

The TUI uses a **stale-while-revalidate** pattern: if cached data exists (even expired), show it immediately and spawn a background revalidation fetch. When fresh data arrives, update the UI seamlessly. The CLI always fetches fresh data (no caching) to maintain predictable scripting behavior.

Write operations (create, update, delete) invalidate related cache entries to prevent stale data display.

### Rejected Alternatives

1. **SQLite cache** — More robust for complex queries and atomic writes, but adds a heavy dependency (`rusqlite`) and is overkill for simple key-value response caching. JSON files are human-inspectable and trivially debuggable. Rejected for simplicity.

2. **In-memory-only cache** — Loses all data when TUI exits, providing no benefit for cold starts. Rejected because the primary value of caching is across sessions.

3. **Cache in `ClickUpClient` (transparent)** — Making caching invisible inside the client violates the principle that `clickup-api` is a pure API client. The CLI should not cache, and the TUI needs cache metadata (freshness, source). Rejected to maintain crate responsibilities.

4. **HTTP-level cache (ETag/Last-Modified)** — ClickUp API v2 does not consistently support conditional requests (`If-None-Match`, `If-Modified-Since`). Rejected due to API limitation.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Filesystem-based JSON cache at `~/.cache/clickup-rs/` | CLI caching | Cross-device cache sync |
| TTL configuration per resource type | Cache encryption | Cache size limits / LRU eviction |
| Stale-while-revalidate for TUI | SQLite backend | Cache compression |
| Cache invalidation on write operations | Cache warming/prefetch | Configurable TTL via config.toml |
| TUI cache indicator (⚡) | Offline-first mode (queue writes) | |
| Cache clearing command (`clickup cache clear`) | | |

## Stories

### S-1: Cache Storage Layer

**As a** developer, **I want** a cache module that stores and retrieves serialized API responses by key with TTL metadata, **so that** both TUI data loading and future consumers have a reusable caching foundation.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create cache module | `crates/clickup-api/src/cache.rs` | New module for cache types and storage logic |
| 2 | Define `CacheEntry<T>` struct | `crates/clickup-api/src/cache.rs` | `struct CacheEntry { data: serde_json::Value, cached_at: i64 (unix millis), ttl_secs: u64 }` |
| 3 | Define `CacheKey` | `crates/clickup-api/src/cache.rs` | Struct with `endpoint: String, params_hash: String`; `fn to_path(&self) -> PathBuf` converts to filesystem path |
| 4 | Implement `FileCache` struct | `crates/clickup-api/src/cache.rs` | `pub struct FileCache { base_dir: PathBuf }` with methods `get<T>()`, `set<T>()`, `invalidate()`, `clear()` |
| 5 | Implement `get<T>()` | `crates/clickup-api/src/cache.rs` | Read JSON file, deserialize `CacheEntry`, check if expired, return `CacheResult::Hit(T)`, `CacheResult::Stale(T)`, or `CacheResult::Miss` |
| 6 | Implement `set<T>()` | `crates/clickup-api/src/cache.rs` | Serialize data + metadata to JSON, write to file atomically (write to `.tmp` then rename) |
| 7 | Implement `invalidate()` | `crates/clickup-api/src/cache.rs` | Delete cache file for a given key, or all files matching a prefix pattern |
| 8 | Implement `clear()` | `crates/clickup-api/src/cache.rs` | Delete all files in `base_dir` |
| 9 | Hash params | `crates/clickup-api/src/cache.rs` | Use `std::hash::DefaultHasher` on serialized params to generate `params_hash` |
| 10 | Register module | `crates/clickup-api/src/lib.rs` | `pub mod cache;` |
| 11 | Unit tests | `crates/clickup-api/src/cache.rs` | Test set/get round-trip, TTL expiry, stale detection, invalidation, clear, concurrent access |

#### Acceptance Criteria

- **GIVEN** a cache entry is stored with TTL of 300s, **WHEN** retrieved at 200s, **THEN** `CacheResult::Hit(data)` is returned.
- **GIVEN** a cache entry is stored with TTL of 300s, **WHEN** retrieved at 400s, **THEN** `CacheResult::Stale(data)` is returned (data still accessible).
- **GIVEN** no cache entry exists for a key, **WHEN** `get()` is called, **THEN** `CacheResult::Miss` is returned.
- **GIVEN** a cache entry exists, **WHEN** `invalidate()` is called for its key, **THEN** subsequent `get()` returns `CacheResult::Miss`.
- **GIVEN** cache files exist, **WHEN** `clear()` is called, **THEN** all cache files are deleted.
- **GIVEN** a cache key with path `/team/123/space` and params `{archived: false}`, **WHEN** serialized to a file path, **THEN** the path is deterministic and filesystem-safe.

#### Agent Hints

- **Class:** builder
- **Context:** Place in `clickup-api` so the TUI crate can depend on it. Use `dirs::cache_dir()` (already a dependency via `dirs` crate) for `XDG_CACHE_HOME` compliance. The cache stores `serde_json::Value` not typed data — deserialization happens at the consumer level to keep the cache generic.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected (no TUI deps)
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P2: `cargo clippy` clean

---

### S-2: TTL Configuration

**As a** developer, **I want** per-resource TTL values defined centrally, **so that** different resource types are cached for appropriate durations.

**Timebox:** ≤0.5d | **Risk:** Low | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Define `CacheTtl` constants | `crates/clickup-api/src/cache.rs` | `pub mod ttl { pub const WORKSPACES: u64 = 3600; pub const SPACES: u64 = 1800; pub const FOLDERS: u64 = 900; pub const LISTS: u64 = 900; pub const TASKS: u64 = 300; pub const TASK_DETAIL: u64 = 120; }` |
| 2 | Helper method | `crates/clickup-api/src/cache.rs` | `fn ttl_for_endpoint(path: &str) -> u64` — pattern-match on path segments to determine TTL |
| 3 | Unit tests | `crates/clickup-api/src/cache.rs` | Verify TTL mapping for each endpoint pattern |

#### Acceptance Criteria

- **GIVEN** endpoint path `/team/123/space`, **WHEN** `ttl_for_endpoint()` is called, **THEN** it returns 1800 (30 minutes).
- **GIVEN** endpoint path `/task/abc123`, **WHEN** `ttl_for_endpoint()` is called, **THEN** it returns 120 (2 minutes).
- **GIVEN** an unknown endpoint path, **WHEN** `ttl_for_endpoint()` is called, **THEN** it returns a sensible default (e.g., 300s).

#### Agent Hints

- **Class:** builder
- **Context:** Keep TTLs as constants, not config values — configurable TTLs are deferred scope. Pattern matching can use simple string `contains` or prefix matching.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test -p clickup-api` passes

---

### S-3: Stale-While-Revalidate Integration

**As a** TUI user, **I want** screens to load instantly from cache while fresh data is fetched in the background, **so that** navigation feels responsive even when the API is slow.

**Timebox:** ≤3d | **Risk:** Medium (state management complexity) | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `CachedClient` wrapper | `crates/clickup-tui/src/cached_client.rs` | Wraps `ClickUpClient` + `FileCache`; provides same endpoint methods but with cache logic |
| 2 | Implement cache-first fetch | `crates/clickup-tui/src/cached_client.rs` | On data request: check cache → if Hit, return immediately; if Stale, return stale + spawn revalidation; if Miss, fetch from API + cache result |
| 3 | Add `DataPayload` variants for cache | `crates/clickup-tui/src/event.rs` | Add `is_cached: bool` flag to `DataPayload` variants, or new variants like `WorkspacesRevalidated(Vec<Workspace>)` |
| 4 | Update `spawn_load_*` functions | `crates/clickup-tui/src/data.rs` | Each `spawn_load_*` function uses `CachedClient` instead of raw `ClickUpClient`. First emit cached data, then revalidation data. |
| 5 | Handle revalidation events | `crates/clickup-tui/src/app.rs` | When revalidation data arrives, update `App` state and trigger re-render only if data changed |
| 6 | Register module | `crates/clickup-tui/src/main.rs` | Initialize `CachedClient` at startup, pass to data functions |
| 7 | Integration test | `crates/clickup-tui/src/cached_client.rs` | Test cache-first behavior with wiremock: first call caches, second call returns from cache |

#### Acceptance Criteria

- **GIVEN** workspaces were loaded 10 minutes ago (within 1h TTL), **WHEN** the user navigates to WorkspaceSelect, **THEN** cached data is shown immediately (no loading spinner).
- **GIVEN** workspaces were loaded 2 hours ago (past 1h TTL), **WHEN** the user navigates to WorkspaceSelect, **THEN** stale data is shown immediately AND a background fetch is triggered.
- **GIVEN** a background revalidation completes, **WHEN** new data differs from stale data, **THEN** the screen updates seamlessly without user interaction.
- **GIVEN** no cache exists (first launch), **WHEN** the user navigates to any screen, **THEN** behavior is identical to current (loading spinner → data).
- **GIVEN** the network is unavailable, **WHEN** stale cached data exists, **THEN** it is displayed (no error screen) with a cache indicator.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/data.rs` (existing `spawn_load_*` functions pattern). The `CachedClient` wraps but does NOT replace `ClickUpClient` — it composes. For the "data changed" check, compare serialized JSON to avoid unnecessary re-renders. Place `CachedClient` in the TUI crate, not in `clickup-api` — caching policy is a consumer concern.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy` clean

---

### S-4: Cache Invalidation on Writes

**As a** TUI user, **I want** cache entries to be invalidated when I modify data, **so that** I see fresh data after creating, updating, or deleting tasks.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Define invalidation rules | `crates/clickup-tui/src/cached_client.rs` | Map: task create → invalidate task list for that list; task update → invalidate task detail + task list; task delete → same; comment CRUD → invalidate task detail |
| 2 | Invalidate after writes | `crates/clickup-tui/src/data.rs` | After each `spawn_*` write operation completes successfully, call `cache.invalidate()` for affected keys |
| 3 | Invalidate by prefix | `crates/clickup-api/src/cache.rs` | Add `invalidate_prefix(prefix: &str)` — delete all cache files whose key starts with prefix (e.g., `/list/123/task` invalidates all pages) |
| 4 | Unit tests | `crates/clickup-tui/src/cached_client.rs` | Test that write operations clear related cache entries |

#### Acceptance Criteria

- **GIVEN** a cached task list for list `L1`, **WHEN** a task is created in list `L1`, **THEN** the task list cache for `L1` is invalidated.
- **GIVEN** a cached task detail for task `T1`, **WHEN** `T1` is updated (status change, field edit), **THEN** the task detail cache for `T1` is invalidated.
- **GIVEN** a cached task detail for task `T1`, **WHEN** `T1` is deleted, **THEN** both the task detail and parent task list caches are invalidated.
- **GIVEN** a cached task detail, **WHEN** a comment is added, **THEN** the task detail cache is invalidated (comments are embedded in task detail).

#### Agent Hints

- **Class:** builder
- **Context:** Write operations don't exist yet for most resources (F02–F06 are future features), but the invalidation hooks should be designed now so that future write operations can call `cache.invalidate()` trivially. For now, implement invalidation for comment CRUD (which already exists).
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test -p clickup-tui` passes

---

### S-5: TUI Cache Indicators

**As a** TUI user, **I want** a visual indicator showing when displayed data is from cache, **so that** I understand the data's freshness and know when a background refresh is in progress.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add cache metadata to `App` | `crates/clickup-tui/src/app.rs` | `pub data_source: DataSource` enum (`Fresh`, `Cached { age_secs: u64 }`, `Revalidating`) per screen |
| 2 | Update top bar rendering | `crates/clickup-tui/src/ui/layout.rs` | Show `⚡ cached 2m ago` or `↻ refreshing...` in the top bar next to the breadcrumb |
| 3 | Update data event handlers | `crates/clickup-tui/src/app.rs` | Set `data_source` when data events arrive: `Cached` when from cache, `Revalidating` when stale + fetching, `Fresh` when live data arrives |
| 4 | Style the indicator | `crates/clickup-tui/src/theme.rs` | Use dimmed/yellow for cached indicator, green for fresh |

#### Acceptance Criteria

- **GIVEN** data is served from cache, **WHEN** the screen is rendered, **THEN** a `⚡ cached Xm ago` indicator appears in the top bar.
- **GIVEN** stale data is served and revalidation is in progress, **WHEN** the screen is rendered, **THEN** a `↻ refreshing...` indicator appears.
- **GIVEN** fresh data arrives (from network), **WHEN** the screen is rendered, **THEN** no cache indicator is shown (or a brief green `✓ fresh` fades out).

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/layout.rs` (existing top bar rendering). The indicator should be right-aligned in the top bar to not interfere with the breadcrumb. Use `chrono::Utc::now().timestamp()` to calculate age from `cached_at`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: Indicator renders correctly
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (storage layer) → S-2 (TTL config) → S-3 (stale-while-revalidate) → S-4 (write invalidation)
                                                                       → S-5 (TUI indicators)
```

S-4 and S-5 can execute in parallel after S-3.

## Assumptions

1. **Filesystem-based JSON cache is fast enough** for TUI responsiveness (<10ms reads). Risk if wrong: migrate to SQLite or in-memory cache. Mitigation: cache files are small (1–50KB each) and SSD I/O is fast.
2. **Cache directory `~/.cache/clickup-rs/` is writable**. Risk if wrong: fall back to no caching with a warning. Mitigation: `FileCache::new()` validates directory access at startup.
3. **Concurrent TUI access** from multiple terminals won't cause cache corruption. Risk if wrong: low — atomic writes (tmp + rename) prevent partial reads, and stale data is acceptable.
4. **JSON serialization overhead** for caching is negligible. Risk if wrong: serde_json serialization of pre-parsed models adds ~1ms — acceptable.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 3 | New module in API crate, wrapper in TUI crate, UI changes, data flow changes |
| Ambiguity | 2 | Stale-while-revalidate flow has subtle state management edge cases |
| Dependencies | 2 | Touches data loading, event system, app state, and UI rendering in TUI |
| Risk | 2 | State management complexity; potential for stale data bugs; filesystem edge cases |

**Total: 9/12** → Complex processing — consider splitting S-3 into sub-tasks during implementation.
