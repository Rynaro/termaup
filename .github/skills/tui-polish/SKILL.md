---
name: tui-polish
description: Add search, help overlay, themes, caching, and error handling to clickup-tui. Use this when polishing the TUI experience.
---

# TUI Polish

## Context

This skill adds the finishing touches to the TUI — search/filter functionality, a help overlay, theme support, data caching for back-navigation, error banners, and edge case handling.

## Crates

- `clickup-tui`

## Prerequisites

- Skill `tui-tasks` completed — all screens render with real data

## Deliverables

- `crates/clickup-tui/src/ui/search.rs` — Inline filter bar with fuzzy matching
- `crates/clickup-tui/src/ui/help.rs` — Modal overlay with keybinding reference
- `crates/clickup-tui/src/theme.rs` — Theme struct with ClickUp color conversion

## Implementation

### ui/search.rs

- Triggered by `/` key on any list screen
- Renders an input bar at the bottom of the main content area
- Filters the current list in real-time as the user types
- Uses case-insensitive substring matching (upgrade to fuzzy matching if desired)
- `Enter`: confirm filter and close search bar
- `Esc`: cancel filter and restore full list
- Filtered state persists until cleared

### ui/help.rs

- Triggered by `?` key from any screen
- Renders a centered modal overlay with semi-transparent background
- Shows keybinding table for the current screen context
- `?` or `Esc` closes the overlay
- Common keys shown on all screens: `q` quit, `?` help, `r` refresh, `/` search

### theme.rs

```rust
pub struct Theme {
    pub status_colors: HashMap<String, Color>,
    pub priority_colors: HashMap<String, Color>,
}
```

- Convert ClickUp hex color strings (e.g., `"#4194f6"`) to `ratatui::style::Color::Rgb`
- Provide fallback colors for unknown statuses
- Priority color mapping: Urgent=Red, High=Orange, Normal=Yellow, Low=Blue

### Data caching

- Already-loaded data stays in `App` struct fields
- Back-navigation (`Esc`) does NOT re-fetch — uses cached data
- `r` key explicitly refreshes (clears cache for current screen and reloads)
- Loading indicator shows only on initial load or explicit refresh

### Error handling

- API errors display as a dismissible banner at the top of the screen
- Network errors suggest checking connection
- Auth errors suggest re-running `clickup auth login`
- Banner auto-dismisses after 5 seconds or on any keypress

### Edge cases

- Terminal too small: show centered message "Terminal too small — resize to at least 80×24"
- Empty lists: show centered "No items found" message
- Mouse support: enable basic mouse scrolling for task detail view

## Acceptance

- `/` opens search bar, typing filters the list in real-time
- `?` toggles help overlay showing context-appropriate keybindings
- Status and priority colors match ClickUp's color scheme
- Back-navigation is instant (no loading delay)
- `r` refreshes data with a loading indicator
- API errors show as dismissible banners
- Small terminal shows a resize message instead of crashing
