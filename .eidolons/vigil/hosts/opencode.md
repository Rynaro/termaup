# VIGIL on OpenCode

Wiring notes for running VIGIL under OpenCode (open-source agentic coding CLI).

## Installation

Installer default target: `./agents/vigil/` within the project repo.

```bash
bash /path/to/vigil/install.sh
```

## Agent Definition

Create `.opencode/agents/vigil.md`:

```markdown
---
name: vigil
description: Forensic debugger. Root-cause attribution for code failures via reproduction, IDG analysis, and counterfactual intervention. Specialist class — does not build, plan, or document.
methodology: VIGIL
methodology_version: "1.0"
entry_point: agents/vigil/agent.md
spec: agents/vigil/VIGIL.md
tools:
  - read_file
  - write_file
  - bash
  - search
triggers:
  - "debug this failure"
  - "why did this test fail"
  - "investigate the regression"
  - "this test is flaky"
  - "APIVR-Δ escalated"
  - "root cause analysis"
not_for:
  - feature implementation (route to APIVR-Δ)
  - planning (route to SPECTRA)
  - documentation (route to IDG)
  - healthy-codebase exploration (route to ATLAS)
---

Load VIGIL from agents/vigil/agent.md. Follow the V→I→G→I→L cycle strictly.
All invariants in agent.md are non-negotiable.
```

## OpenCode Configuration

In `.opencode/config.yaml`:

```yaml
agents:
  enabled:
    - vigil
    - apivr-delta       # if installed
    - spectra           # if installed
    - atlas             # if installed
    - idg               # if installed

vigil:
  authority:
    default_mode: sandbox
    allow_write: false

  sandbox:
    adapter: docker        # docker | podman | language_native | bubblewrap
    image: python:3.12-slim
    timeout_s: 300
    resource_limits:
      memory_mb: 2048
      cpus: 2

  memory:
    path: memories/vigil-failures.yaml
    signature_cap: 50
    consolidation_strategy: recency_weighted
```

## Tool Surface

OpenCode's open architecture makes it the most flexible host for VIGIL. Full
sandbox access is available out of the box:

| Tool | Purpose in VIGIL |
|------|------------------|
| `read_file` | All phases — trace inspection, source reading |
| `write_file` | Phase L — emit root-cause-report.md, verified-patch.diff |
| `bash` | Phase V (reproduction), Phase I-Intervene (counterfactual execution) |
| `search` | Phase I-Isolate — stack trace closure, churn filter |

For sandbox execution (Phase I-Intervene), OpenCode can use:

- **Docker/Podman** — via bash tool invoking `docker run --rm`
- **Language-native** — direct invocation of pytest, jest, rspec, etc.
- **Firejail/bubblewrap** — Linux host sandboxing via bash

## MCP Integration

OpenCode supports MCP servers natively. Recommended complementary servers:

- **atlas-aci** — shared code-search and graph capability; VIGIL reuses for Isolate and Graph
- **Docker MCP** — managed sandbox execution if preferred over direct shell
- **OTLP collector** — if the project emits OpenTelemetry GenAI traces, consume directly into Phase G

Configure in `.opencode/mcp.yaml`:

```yaml
servers:
  atlas-aci:
    command: atlas-aci-server
    args: ["--readonly"]

  docker-sandbox:
    command: docker-mcp-server
    args: ["--image", "python:3.12-slim"]

  otel-traces:
    command: otel-collector
    env:
      OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4318
```

## Multi-Eidolon Teams on OpenCode

OpenCode's agent routing makes Eidolon-to-Eidolon handoffs natural:

```yaml
# .opencode/routing.yaml
routes:
  - from: apivr-delta
    trigger: escalation
    to: vigil
    context_transfer:
      - path: agents/apivr/sessions/<session-id>/repair-failed-report.md
        as: upstream_artifact

  - from: vigil
    trigger: root_cause_found
    condition: classification in [LOGIC_ERROR, REGRESSION, BUILD_ERROR, TYPE_ERROR, LINT_VIOLATION, RUNTIME_ERROR, INTEGRATION_ERROR]
    to: apivr-delta
    context_transfer:
      - path: agents/vigil/sessions/<session-id>/root-cause-report.md
      - path: agents/vigil/sessions/<session-id>/verified-patch.diff

  - from: vigil
    trigger: root_cause_found
    condition: classification in [SPEC_DEFECT, COMPOUND]
    to: spectra

  - from: vigil
    trigger: escalation_brief
    to: human
    notify: slack
```

This produces a fully automated pipeline:

```
APIVR-Δ fails 3x → escalates → VIGIL investigates → verifies patch in sandbox
→ routes to APIVR-Δ for application (or SPECTRA for replan, or human if
budget exhausted)
```

## OpenCode-Specific Benefits

- **True open sandboxing** — no vendor restrictions on shell/container access
- **Native multi-agent routing** — explicit handoff contracts via YAML
- **Bring-your-own-model** — VIGIL's capability-class declaration (reasoner-class) lets you route the model choice per phase; e.g., Phase G (graph construction) can use a cheaper model than Phase I-Intervene (which needs strong causal reasoning)
- **Full observability** — OpenCode can emit OTEL spans for every phase transition, aligning with VIGIL's research foundation

## Example Invocation

```bash
opencode --agent=vigil --mission="Investigate ballot_token=nil failure; \
  upstream_artifact=agents/apivr/sessions/20260416/repair-failed-report.md; \
  authority=sandbox"
```

VIGIL runs autonomously:

1. Phase V — reproduces failure deterministically (2 consistent runs)
2. Phase I — narrows to 4 candidates via stack trace + git bisect + dep walk
3. Phase G — builds IDG; identifies 2 root candidates (`generate_uuid` and `Session#load`)
4. Phase I-Intervene — generates 3 hypotheses; intervention I-001 (oracle at `generate_uuid`) FLIPS on run 1
5. Phase L — emits `root-cause-report.md` + `verified-patch.diff`; writes VSIG to memory; routes to APIVR-Δ

Time: ~4 minutes wall clock, ~18k tokens total.

## Differences From Other Hosts

- **Most flexible authority:** OpenCode can grant `write` mode safely because sandbox execution is first-class
- **Best for production pipelines:** CI/CD integration is straightforward via `opencode` CLI invocation from build scripts
- **Best for team memory:** Memory persistence is repo-local and git-tracked by default; no vendor memory features to worry about

---

*VIGIL v1.0.1 — OpenCode host wiring*
