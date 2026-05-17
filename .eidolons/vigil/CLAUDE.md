# VIGIL — Claude Code Entry Point

VIGIL is the team's forensic debugger. This file points Claude Code at the authoritative methodology.

## Quick Reference

- **Authoritative spec:** `VIGIL.md`
- **Always-loaded entry:** `agent.md`
- **Methodology:** `VIGIL` v1.0
- **Phase cycle:** V → I → G → I → L (Verify · Isolate · Graph · Intervene · Learn)

## Load Order

1. `agent.md` — always loaded, ~900 tokens, identity + invariants + skill triggers
2. `skills/<phase>/SKILL.md` — loaded on phase entry
3. `templates/<type>.md` — loaded on artifact composition
4. `VIGIL.md` — consulted only for spec clarification; not part of the default working set

## Invocation Triggers

Load VIGIL when the user:

- Hands off from APIVR-Δ with a `repair-failed-report.md`
- Reports a heisenbug, flaky test, or non-deterministic failure
- Asks to diagnose a regression after APIVR-Δ's Reflect loop was exhausted
- Requests forensic post-mortem on a completed or abandoned session
- Uses phrases like: "why did this fail", "debug this heisenbug", "what's the root cause", "this test fails sometimes", "bisect this regression"

Do NOT load VIGIL for:

- Feature implementation (that's APIVR-Δ)
- Planning or specification work (that's SPECTRA)
- Documentation or chronicling (that's IDG)
- Healthy-codebase exploration (that's ATLAS)

## Default Working Set

Entry point + current phase skill + current template = **~3,100 tokens**, well under the 3,500-token specialist budget.

## Key Design Patterns

- **Reproduction gates attribution** — I-1 invariant enforced mechanically via Phase V schema validation
- **IDG over temporal order** — Phase G rejects time-ordered attribution; mandates dependency analysis
- **Hypothesis plurality** — harness refuses intervention with <3 active hypotheses
- **Counterfactual gating** — `[ROOT-CAUSE]` emission blocked without `FLIPPED` intervention result
- **5-intervention cap** — hard-enforced in schema; exhaustion triggers escalation brief, not continuation
- **Authority flags** — read-only/sandbox/write set at mission start; write never inferred

## Schema Validation

All structured artifacts validate against JSON schemas:

- `schemas/reproduction.v1.json` — Phase V
- `schemas/intervention-log.v1.json` — Phase Intervene
- `schemas/root-cause-report.v1.json` — Phase L

Artifacts that fail schema validation are rejected; VIGIL does not emit invalid findings.

## Canary Evaluation

The `evals/canary/` directory contains a curated mission dataset spanning deterministic regressions, heisenbugs, compound failures, SPEC_DEFECTs, and APIVR-Δ escalation fixtures. Pass targets:

- **Deterministic cases:** ≥80%
- **Non-deterministic cases:** ≥65% (bound set by current research on statistical attribution)

## Versioning

`VIGIL.md` is authoritative. Breaking changes to phase contracts or JSON schemas require a minor-version bump. Implementations declare `methodology: VIGIL` and `methodology_version: 1.0` in `agent.md` frontmatter.

## Reference Implementations

VIGIL is host-agnostic. See `hosts/` for wiring notes per host:

- `hosts/claude-code.md` — this file's detailed companion
- `hosts/cursor.md` — Cursor via `.cursor/rules/`
- `hosts/copilot.md` — GitHub Copilot via `.github/copilot-instructions.md`
- `hosts/opencode.md` — OpenCode via `.opencode/agents/`

---

*VIGIL v1.1.0 — see `VIGIL.md` for the full specification*
