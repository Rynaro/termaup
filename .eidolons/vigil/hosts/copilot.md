# VIGIL on GitHub Copilot

Wiring notes for running VIGIL under GitHub Copilot (VS Code extension, Copilot CLI, Copilot Workspace).

## Installation

Installer default target: `./agents/vigil/` within the project repo.

```bash
bash /path/to/vigil/install.sh
```

## Copilot Instructions File

Create or extend `.github/copilot-instructions.md`:

```markdown
# Project Instructions for Copilot

## Debugger Agent: VIGIL

This project uses the VIGIL methodology for root-cause attribution of code
failures. VIGIL is a forensic specialist — load it when investigating:

- Failures that survived APIVR-Δ's Reflect loop (agents/apivr/)
- Heisenbugs, flaky tests, non-deterministic failures
- Regressions requiring bisect + attribution
- Post-hoc analysis of completed or abandoned sessions

**Do NOT load VIGIL for:** feature implementation (use APIVR-Δ), planning (use
SPECTRA), documentation (use IDG), or healthy-codebase exploration (use ATLAS).

**Load order:**
1. agents/vigil/agent.md — always first
2. agents/vigil/skills/<phase>/SKILL.md — on phase entry
3. agents/vigil/templates/<type>.md — on artifact composition

**Follow the V→I→G→I→L cycle strictly.** The invariants in agent.md are
non-negotiable. Every root-cause claim requires:
- Reproduction ≥2 deterministic runs (or statistical ≥85% CI)
- Information Dependency Graph construction (not temporal order)
- ≥3 competing hypotheses before any intervention
- Counterfactual intervention that FLIPS failure → success
- Bounded to 5 interventions max — escalate if exhausted

Full methodology: agents/vigil/VIGIL.md
```

## AGENTS.md (Open Standard)

Copilot honors the `agents.md` open standard when the file is present at
the project root. VIGIL's `AGENTS.md` is already compliant:

```bash
cp agents/vigil/AGENTS.md AGENTS.md           # if VIGIL is the primary agent
# OR
cp agents/vigil/AGENTS.md agents/vigil-AGENTS.md
# and reference from the root AGENTS.md
```

If you already have an `AGENTS.md` for another agent (e.g. APIVR-Δ or
SPECTRA), add a section:

```markdown
## Debugger Mode

For root-cause attribution on code failures, switch to VIGIL:

- Entry: agents/vigil/AGENTS.md
- Methodology: agents/vigil/VIGIL.md
- Cycle: V → I → G → I → L
- Trigger phrases: "debug this", "why did this fail", "investigate the
  regression", "this test is flaky", "APIVR-Δ escalated"
```

## Tool Permissions

Copilot has variable tool access depending on the surface:

| Surface | Tools Available | VIGIL Authority Limit |
|---------|-----------------|----------------------|
| VS Code chat (normal) | Read-only file access, search | `read-only` only |
| VS Code agent mode | Read/write files, terminal | Up to `sandbox` |
| Copilot CLI | Full shell, no sandbox by default | Up to `sandbox` if Docker is available |
| Copilot Workspace | Full codespace | Up to `write` |

Configure in `.vigil/config.yml`:

```yaml
authority:
  default_mode: read-only         # safest default for Copilot
  allow_write: false              # require explicit escalation

sandbox:
  adapter: docker | language_native
  # Workspace-specific settings...
```

## Copilot-Specific Behaviors

### Slash commands

Copilot chat supports slash commands. VIGIL can be invoked via:

```
/vigil investigate "test_ballot_token_generation failing intermittently"
```

This requires a custom slash command registration. See Copilot docs for
current setup. The command simply loads `agents/vigil/agent.md` into the
chat context.

### Repository-level rules

For teams with a shared Copilot config, the `.github/copilot-instructions.md`
approach propagates VIGIL's methodology to all developers automatically.
Individual developers can override locally via their VS Code settings.

### Copilot for Business / Enterprise

Enterprise administrators may restrict agent file access. Ensure
`agents/vigil/` is in the allowed paths list if your org uses this feature.

## Workspace Integration

In Copilot Workspace:

- VIGIL's sandbox adapter can use the workspace's built-in container
- Persistent memory (`memories/vigil-failures.yaml`) lives in the repo and
  survives workspace restarts
- Handoffs to APIVR-Δ happen naturally within the same workspace

## Example: APIVR-Δ → VIGIL handoff in Copilot

Given the shared `agents/` directory convention:

```
User: @workspace The APIVR-Δ agent just produced a repair-failed-report.md
      for the ballot token bug. Can VIGIL pick it up?

Copilot: I'll load the VIGIL debugger methodology.

[loads agents/vigil/agent.md]
[reads agents/apivr/sessions/20260416-ballot-token/repair-failed-report.md]

Mode: sandbox (per .vigil/config.yml)
Phase V — Verify: Attempting deterministic reproduction...
```

## Differences From Claude Code

- Copilot's context window is smaller; VIGIL's ≤3,500-token working set is
  essential for it to fit alongside code context
- Copilot does not natively support Anthropic's skill-loading pattern; skills
  are loaded by explicit instruction in the chat rather than by the model
  choosing autonomously
- Context compaction is less sophisticated; rely on structured artifacts
  (reproduction.md, idg.md, intervention-log.md) persisted to disk rather
  than in-context summary

## Handoff to IDG

If IDG is also installed in `agents/idg/`, VIGIL's root-cause report can be
chained:

```
User: VIGIL finished. Please have IDG write the incident chronicle from
      the root-cause-report.md.

Copilot: [unloads VIGIL] [loads agents/idg/SCRIBE.md]
         ...
```

---

*VIGIL v1.0.1 — GitHub Copilot host wiring*
