# VIGIL

> **Forensic debugger for code failures resistant to normal repair.** Part of the Eidolons team. Specialist class — does not build, plan, or document.

VIGIL is the team's **forensic specialist**. It is invoked when a failure resists normal repair — heisenbugs, regressions of unclear origin, compound failures spanning multiple modules, failures that survived APIVR-Δ's Reflect loop.

Where other Eidolons *act* on healthy code, VIGIL *investigates* sick runs. Its work is backward — from symptom to root cause, through reproduction, dependency analysis, and counterfactual intervention. Its output is evidence-anchored attribution that downstream members can act on.

## Characteristics

- **Patient.** Refuses premature convergence; keeps multiple hypotheses alive until falsification.
- **Methodical.** Reproduction before blame. Counterfactual before conclusion.
- **Evidence-bound.** No assertion without a trace span, a replay result, or an intervention outcome.
- **Final-authority on attribution.** When VIGIL says "this is the root cause," that claim survived falsification.
- **Vendor-agnostic.** Runs on Claude Code, Cursor, Copilot, OpenCode, or raw API. Uses a pluggable sandbox adapter — harness chooses the mechanism.
- **Token-disciplined.** ≤3,500-token working set via layered loading. Skills load per phase.

## VIGIL Cycle (v1.0)

```
V ──▶ I ──▶ G ──▶ I ──▶ L ──▶ EMIT
              ▲                │
              └── ≤5 counterfactuals ─┘
```

**V**erify · **I**solate · **G**raph · **I**ntervene · **L**earn

- **Verify** — Establish reproducible failure (deterministic-first; statistical on failure)
- **Isolate** — Narrow fault surface to ≤8 candidates via delta-debugging-style reduction
- **Graph** — Build Information Dependency Graph; distinguish symptoms from root candidates
- **Intervene** — Run counterfactual replays against ≥3 competing hypotheses (≤5 iterations)
- **Learn** — Emit verified finding + walk-back + memory entry + downstream handoff

## Quick Start

### Install

```bash
git clone https://github.com/Rynaro/vigil
bash vigil/install.sh [target-directory] [--mode=read-only|sandbox|write]
```

Default target: `./agents/vigil/`. Default mode: `read-only` (safest).

Per-project config is generated at `.vigil/config.yml`.

### Load into your AI host

| Host | Entry |
|------|-------|
| **Claude Code** | Add `@agents/vigil/agent.md` to `CLAUDE.md` or use as subagent |
| **Cursor** | Create `.cursor/rules/vigil.mdc` referencing `@agents/vigil/agent.md` |
| **GitHub Copilot** | Extend `.github/copilot-instructions.md` |
| **OpenCode** | Create `.opencode/agents/vigil.md` |
| **Raw API** | Load `agent.md` as system prompt |

See `hosts/` for detailed per-host wiring.

### Invoke

```
User: APIVR-Δ just escalated after 3 attempts at fixing the ballot token
      nil issue. Can VIGIL take over?

VIGIL: [loads agents/vigil/agent.md]
       Mode: sandbox (per .vigil/config.yml)
       Phase V — Verify: attempting deterministic reproduction from
       the repair-failed-report.md...
       ...
```

## Architecture

```
vigil/
├── agent.md                      # Always-loaded entry (~900 tokens)
├── VIGIL.md                      # Authoritative specification
├── AGENTS.md                     # Open-standard rule set
├── CLAUDE.md                     # Claude Code entry point
├── DESIGN-RATIONALE.md           # Research → design decision map
├── README.md                     # This file
├── CHANGELOG.md                  # Versioned evolution
├── install.sh                    # Idempotent installer
│
├── skills/                       # On-demand per phase
│   ├── verify/SKILL.md           # Reproduction protocol
│   ├── isolate/SKILL.md          # Fault surface reduction
│   ├── graph/SKILL.md            # IDG construction
│   ├── intervene/SKILL.md        # Counterfactual replay
│   └── learn/SKILL.md            # Finding emission + memory
│
├── templates/                    # Artifact skeletons
│   ├── root-cause-report.md      # Primary deliverable
│   ├── verified-patch.md         # Conditional patch artifact
│   ├── failure-signature.md      # Memory ledger entry
│   └── escalation-brief.md       # Budget-exhausted path
│
├── schemas/                      # JSON Schema v2020-12 validators
│   ├── reproduction.v1.json
│   ├── intervention-log.v1.json
│   └── root-cause-report.v1.json
│
├── hosts/                        # Per-host wiring
│   ├── claude-code.md
│   ├── cursor.md
│   ├── copilot.md
│   └── opencode.md
│
└── evals/canary/
    └── missions.md               # 23-mission evaluation set
```

**Typical working set:** entry point + current phase skill + current template = **~3,100 tokens**. Well under the 3,500-token specialist budget.

## Architectural Invariants

Ten invariants, mechanically enforced where possible:

| # | Invariant |
|---|-----------|
| I-1 | Reproduction gates attribution |
| I-2 | Dependency graph, not temporal sequence |
| I-3 | Hypothesis plurality (≥3 before intervention) |
| I-4 | Counterfactual gates blame |
| I-5 | Bounded intervention budget (5 hard max) |
| I-6 | Flag-gated authority (read-only / sandbox / write) |
| I-7 | Evidence-anchored findings |
| I-8 | Non-determinism declared, not masked |
| I-9 | Sandbox adapter interface |
| I-10 | Telemetry-driven compaction |

Full detail: `VIGIL.md`. Rationale: `DESIGN-RATIONALE.md`.

## Failure Taxonomy (11 categories)

`LOGIC_ERROR` · `REGRESSION` · `BUILD_ERROR` · `TYPE_ERROR` · `LINT_VIOLATION` · `RUNTIME_ERROR` · `INTEGRATION_ERROR` · `ENVIRONMENT_ERROR` · `HEISENBUG` · `COMPOUND` · `SPEC_DEFECT`

First 8 align with APIVR-Δ's Reflect taxonomy. Last 3 are the escalation classes — where VIGIL adds value beyond what APIVR-Δ can do alone.

## Structural Markers

- `[FINDING-NNN]` — evidence-anchored attribution (team-wide)
- `[HYPOTHESIS-N]` — candidate under falsification
- `[ROOT-CAUSE]` — counterfactual-verified
- `[SYMPTOM]` — propagated effect, NOT root cause
- `[INTERVENTION-N]` — minimal change applied in sandbox
- `[FLAKE]` — non-determinism; statistical attribution
- `[GAP]` — expected evidence missing
- `[DISPUTED]` — intervention evidence contradicts

## Team Integration

VIGIL fits into the Eidolons pipeline:

```
ATLAS ───▶ SPECTRA ───▶ APIVR-Δ ───▶ IDG
                          │
                          ▼ (3-attempt cap exhausted)
                       VIGIL ────┬──▶ APIVR-Δ (surgical fix)
                                 ├──▶ SPECTRA (replan)
                                 ├──▶ IDG (chronicle)
                                 ├──▶ FORGE (ambiguous)
                                 └──▶ human
```

VIGIL can also be invoked independently (consultant mode) or on completed sessions (post-hoc mode). Methodology is identical across entry modes.

## Design Principles

- **Single responsibility.** VIGIL attributes. It does not plan, implement, or chronicle.
- **Evidence over assertion.** Every claim anchored to a path:line + confidence + counterfactual result.
- **Mechanical over hortatory.** Invariants enforced by schema validation, not prompt instruction.
- **Portable over convenient.** Sandbox adapter is pluggable; no vendor lock-in.
- **Bounded over unbounded.** 5-intervention cap, not "keep trying."
- **Honest over confident.** Non-determinism declared; escalation is a success state.

## Research Foundation

VIGIL's design traces to four converging research threads:

- **GraphTracer** (arXiv:2510.10581) — Information Dependency Graphs achieve +18.18% attribution accuracy over temporal methods
- **AgenTracer** (arXiv:2509.03312) — Counterfactual replay as minimum-cost causal proof; standalone LLM attribution accuracy is sub-10% without it
- **CHIEF** (NeurIPS 2024) — Hierarchical causal graphs + oracle-guided backtracking
- **Lifecycle of Failures in Platform-Orchestrated Agents** (arXiv:2509.23735) — Attribution accuracy 46.3% → 65.8% via counterfactual replay
- **Delta Debugging** (Zeller, classical) — Minimal-intervention isolation as foundational technique
- **TALE** (ACL 2025 Findings) — Budget-aware reasoning reduces output token cost by 68.64%
- **SWE-bench+** (arXiv 2025) — Mutation-verified evaluation; raw SWE-bench overestimates by up to 54%
- **OpenTelemetry GenAI Semantic Conventions** — CNCF-standard portable trace interface
- **CorrectBench** (2025) — Unbounded self-correction degrades open-ended output; 5-iteration cap

Full mapping: `DESIGN-RATIONALE.md`.

## Evaluation

23-mission canary dataset in `evals/canary/missions.md`:

- **15 deterministic missions** — target ≥80% pass rate
- **8 non-deterministic missions** — target ≥65% pass rate (bound set by research state on statistical attribution)

Coverage: all 11 failure categories, three entry modes, escalation protocol, anti-patterns (confirmation bias, plurality enforcement), statistical edge cases.

Pass criteria per mission: root cause matches, classification matches, handoff recipient matches, intervention budget respected, escalation emitted when expected.

## Sibling Eidolons

VIGIL composes with the rest of the team:

- **APIVR-Δ** — Coder-class; builds features. VIGIL is its escalation target.
- **SPECTRA** — Planner-class; specifies work. VIGIL routes structural fixes here.
- **ATLAS** — Scout-class; maps healthy terrain. VIGIL's sibling — trained on sick runs.
- **IDG** — Scriber-class; chronicles what happened. Receives VIGIL's reports for incident docs.
- **FORGE** — Reasoner-class (in construction); handles VIGIL's escalations on ambiguity.

## Versioning

`VIGIL.md` is authoritative. Breaking changes to phase contracts or JSON schemas require minor-version bumps. See `CHANGELOG.md`.

## License

Apache 2.0. See [LICENSE](LICENSE).

---

*VIGIL v1.0.1 — Verify · Isolate · Graph · Intervene · Learn*
