---
applyTo: "crates/clickup-cli/**/*.rs"
---

# clickup-cli — Coding Rules

## Context

The `clickup-cli` binary crate is the command-line interface for clickup-rs.

## Constraints

- Use `clap` with derive macros for all argument/command parsing
- Use `anyhow::Result` for the top-level return type and all command handlers
- Convert `ClickUpError` into anyhow errors using `?` (the `From` impl is automatic)
- User-facing output goes to stdout; errors and diagnostics go to stderr
- Use `owo-colors` for colored output — never raw ANSI escape codes
- Support three output formats via `--format` flag: table (default), json, markdown
- Use `comfy-table` for table output, `termimad` for markdown rendering
- Interactive prompts use `dialoguer` for selection, `rpassword` for hidden input
- Initialize tracing in `main()` with `CLICKUP_LOG` env var as the filter

## Patterns

Output prefix conventions:
- Success: green `"✓"` prefix
- Error: red `"✗"` prefix
- Info: blue `"ℹ"` prefix

Table output: bordered, with colored headers.
JSON output: pretty-printed with 2-space indent.
Dates: human-readable format (e.g., "Mar 16, 2026") — convert millisecond timestamps from ClickUp.

Command structure: `clickup <resource> <action> [options]`
Example: `clickup task get TASK_ID --format json`
