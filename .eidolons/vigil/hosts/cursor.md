# VIGIL on Cursor

Wiring notes for running VIGIL under Cursor.

## Installation

Installer default target: `./agents/vigil/` within the project repo.

```bash
bash /path/to/vigil/install.sh
```

## Rule File Setup

Create `.cursor/rules/vigil.mdc`:

```markdown
---
description: Forensic debugger for code failures resistant to normal repair. Load when investigating heisenbugs, regressions, compound failures, or escalations from APIVR-Δ.
globs: ["**/*"]
alwaysApply: false
---

# VIGIL — Debugger Agent

Load VIGIL when asked to:
- Investigate a failure that survived APIVR-Δ's Reflect loop
- Debug a heisenbug or flaky test
- Perform post-hoc root-cause analysis
- Diagnose a regression requiring bisect

Do NOT load VIGIL for feature implementation (APIVR-Δ), planning (SPECTRA),
documentation (IDG), or healthy-codebase exploration (ATLAS).

Full methodology: @agents/vigil/agent.md
Specification: @agents/vigil/VIGIL.md

Follow the V→I→G→I→L cycle strictly. Invariants in agent.md are non-negotiable.
```

## Cursor-Specific Considerations

### Agent mode vs chat mode

VIGIL works best in Cursor's **Agent mode** because:

- Multi-step reproduction requires sustained context
- Counterfactual interventions need tool-execution capability
- Phase-based skill loading requires agent autonomy

Chat mode can be used for **post-hoc analysis** where the user has already gathered evidence and wants VIGIL's attribution reasoning — but without sandbox access, authority is limited to `read-only` and only `[HYPOTHESIS-N]` (not `[ROOT-CAUSE]`) can be emitted.

### Tool access

Cursor's Agent mode provides file read/write and terminal access. For VIGIL's sandbox adapter requirement (invariant I-9):

- **Docker adapter:** Ensure Docker daemon is accessible from the Cursor shell environment
- **Language-native adapter:** Test commands must run from project root; use project's existing test runner (pytest, jest, rspec, etc.)
- **No system-level sandbox:** Cursor does not natively integrate with Firejail/bubblewrap; prefer Docker or language-native on Cursor

### `.cursorrules` (legacy)

If using legacy `.cursorrules` instead of `.cursor/rules/`, add:

```
## Debugger Agent

For failures requiring root-cause attribution, load the VIGIL methodology
from agents/vigil/agent.md. Follow V→I→G→I→L cycle strictly.
```

## Authority Configuration

Place `.vigil/config.yml` at project root:

```yaml
authority:
  default_mode: sandbox
  allow_write: false

sandbox:
  adapter: docker
  image: python:3.12-slim       # adapt to your stack
  working_dir: /tmp/vigil-sandbox
  timeout_s: 300
```

Cursor respects per-project config; users can override per-invocation via chat
("run VIGIL in read-only mode on this failure").

## Context Management

Cursor's composer has a smaller effective context than Claude Code's extended
modes. VIGIL's ≤3,500-token working set fits comfortably. If the reproduction
trace is very large:

- Use Cursor's `@file` references to point VIGIL at trace files rather than
  pasting full content
- Rely on VIGIL's Phase G skill for IDG construction directly from trace files
  on disk, not from in-context content

## Example Invocation

```
User: This test is flaky — passes locally, fails in CI ~40% of the time.
      Can you investigate? Trace logs are in logs/ci-failures/run-42.log.

Cursor [Agent mode]:
  Loading agents/vigil/agent.md
  Mode: sandbox (per .vigil/config.yml)
  Phase V: attempting deterministic reproduction locally...
  ...
```

## Handoff to APIVR-Δ

If APIVR-Δ is also installed as a Cursor rule, VIGIL's `verified-patch.diff`
can be handed off explicitly:

```
User: Apply the VIGIL verified patch via APIVR-Δ.
Cursor: Loading agents/apivr/agent.md
        Reading verified-patch.diff from sessions/20260416-ballot-token/
        Starting APIVR-Δ cycle: A→P→I→V→Δ ...
```

---

*VIGIL v1.0.1 — Cursor host wiring*
