# VIGIL on Claude Code

Wiring notes for running VIGIL under Anthropic's Claude Code CLI.

## Installation

Installer default target: `./agents/vigil/` within the project repo.

```bash
# From repo root
curl -fsSL https://raw.githubusercontent.com/Rynaro/vigil/main/install.sh | bash
# or, if cloned locally:
bash /path/to/vigil/install.sh
```

Then add to your project's `CLAUDE.md`:

```markdown
## Debugger Agent

For code failures that resist normal repair (heisenbugs, regressions, compound
failures, post-APIVR-Δ escalations), consult VIGIL:

- Entry: `@agents/vigil/agent.md`
- Spec: `@agents/vigil/VIGIL.md`
- Invocation: "Load VIGIL and investigate [failure description]"
```

## Subagent Configuration (Claude Code 0.14+)

Define VIGIL as a subagent in `.claude/agents/vigil.md`:

```markdown
---
name: vigil
description: Forensic debugger. Root-cause attribution for code failures via reproduction, IDG analysis, and counterfactual intervention. Use when APIVR-Δ's Reflect loop exhausts, when facing heisenbugs, or for post-hoc failure analysis.
tools: Read, Bash, Grep, Glob, Edit
---

Load the full VIGIL methodology from @agents/vigil/agent.md and follow the V→I→G→I→L cycle strictly. All invariants in the entry point are non-negotiable.
```

### When Claude Code Should Invoke VIGIL

- After APIVR-Δ's Reflect phase escalates (3 failed attempts)
- When the user describes a heisenbug or flaky test
- When the user asks for root-cause attribution on a completed/abandoned session
- When a regression requires bisect + attribution

### When Claude Code Should NOT Invoke VIGIL

- During initial feature implementation — that's APIVR-Δ
- For specification/planning questions — that's SPECTRA
- For writing incident documentation — that's IDG (VIGIL's output feeds IDG)
- For exploring an unfamiliar healthy codebase — that's ATLAS

## MCP Server Considerations

VIGIL uses a **pluggable sandbox adapter** (per invariant I-9). Recommended MCP servers to connect:

### For code search and graph (shared with ATLAS)

- `atlas-aci` — if installed, provides `view_file`, `search_symbol`, `graph_query`, `search_text` with bounded ACI. VIGIL reuses these for Isolate and Graph phases.

### For sandbox execution

Choose one based on project stack:

- **Docker/Podman MCP** — most portable, highest isolation; recommended for containerized stacks
- **Language-native test harness** — pytest, jest, rspec, cargo test — lowest overhead, highest fidelity to production behavior
- **Firejail/bubblewrap** — Linux host-level sandboxing for CLI tool failures

Configure the sandbox adapter in `.vigil/config.yml`:

```yaml
sandbox:
  adapter: docker | language_native | firejail | bubblewrap
  image: <if docker>
  working_dir: /tmp/vigil-sandbox
  timeout_s: 300
  resource_limits:
    memory_mb: 2048
    cpus: 2
```

### For OpenTelemetry trace ingestion (optional but recommended)

If the project emits OTEL GenAI-compatible traces, VIGIL's Graph phase can consume them directly as input to IDG construction:

```yaml
trace_source:
  type: otel | otlp_file | structured_log
  endpoint: http://localhost:4318   # for live OTLP
  retention_days: 7
```

## Authority Flag

Set in `.vigil/config.yml` per project:

```yaml
authority:
  default_mode: read-only | sandbox | write
  allow_write: false              # write mode requires explicit true
  sandbox_required_for: [sandbox, write]
```

Per-invocation override:

```
vigil --mode=sandbox investigate [failure]
```

Claude Code will respect the config unless explicitly overridden by the user in the chat.

## Memory Persistence

VIGIL writes failure signatures to `memories/vigil-failures.yaml` (or wherever the project's shared agent memory lives). On mission start, VIGIL queries this ledger for signature matches to avoid re-investigating known patterns.

For multi-agent teams, consider a shared memory MCP server (e.g., based on `atlas-aci`'s Memex pattern) so VIGIL's findings become queryable by other Eidolons (APIVR-Δ especially benefits from prior failure signatures during its Analyze phase).

## Claude Code-Specific Behavior

### TodoWrite integration

VIGIL phases map naturally to Claude Code's TodoWrite pattern. Recommended task template:

```
1. [VIGIL Phase V] Reproduce failure
2. [VIGIL Phase I] Isolate fault surface
3. [VIGIL Phase G] Build IDG
4. [VIGIL Phase I] Run counterfactual interventions (up to 5)
5. [VIGIL Phase L] Emit root-cause report + handoff
```

### Context compaction

VIGIL's long-running mission is a natural fit for Claude Code's checkpoint/compact behavior. At phase boundaries, compact the context to: mission brief + current phase artifact + active findings list. Raw traces and tool outputs go to content-addressable storage.

### Handoff to APIVR-Δ

When VIGIL emits a verified patch, the default Claude Code flow is:

1. VIGIL produces `root-cause-report.md` + `verified-patch.diff`
2. User reviews (or auto-approves if `authority: write`)
3. APIVR-Δ receives both artifacts, runs Analyze → Plan → Implement → Verify on the patch
4. APIVR-Δ's Verify confirms the fix or re-escalates to VIGIL with new evidence

## Example Invocation

```
User: APIVR-Δ just escalated after 3 attempts at fixing the ballot token
      nil issue. The repair-failed-report.md is at agents/apivr/sessions/
      20260416-ballot-token/repair-failed-report.md. Can VIGIL take over?

Claude Code: [loads agents/vigil/agent.md]
             [sets mode=sandbox based on .vigil/config.yml]
             [begins Phase V: Verify reproduction from APIVR-Δ's artifact]
             ...
```

---

*VIGIL v1.0.1 — Claude Code host wiring*
