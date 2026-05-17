<!-- eidolon:atlas start -->
## ATLAS — Read-only codebase scout (v1.5.2)

Entry:     `./.eidolons/atlas/agent.md`
Full spec: `./.eidolons/atlas/ATLAS.md`
Cycle:     A (Assess) → T (Traverse) → L (Locate) → A (Abstract) → S (Synthesize)

**P0 (non-negotiable):** read-only (refuse edit/write/commit/deploy/migrate/refactor/fix); mission-first (requires `mission.md` + `DECISION_TARGET`); bounded ACI (`view_file` ≤100, `search_text` ≤50, `list_dir` ≤200); evidence-anchored claims (`path:line` + H|M|L); deterministic retrieval first, LLM search last.
<!-- eidolon:atlas end -->

<!-- eidolon:spectra start -->
## SPECTRA — Decision-ready specifications (v4.3.2)

Entry:     `.eidolons/spectra/agent.md`
Full spec: `.eidolons/spectra/SPECTRA.md`
Cycle:     CLARIFY → Scope → Pattern → Explore → Construct → Test → Refine → Assemble

**P0 (non-negotiable):** READ-ONLY during all planning phases (no code edits); dual-format output (Markdown + YAML/JSON); CLARIFY first (parse WHO/WHAT/WHY/CONSTRAINTS); confidence ≥85% at Assemble (else Refine, max 3 cycles); output is a specification, never an implementation.
<!-- eidolon:spectra end -->

<!-- eidolon:apivr start -->
## APIVR-Δ — Brownfield feature implementation (v3.1.2)

Entry:     `.eidolons/apivr/agent.md`
Full spec: `.eidolons/apivr/apivr.md`
Cycle:     A (Analyze) → P (Plan) → I (Implement) → V (Verify) → Δ (Delta) / R (Reflect)

**P0 (non-negotiable):** Internal First (USE → EXTEND → WRAP → CREATE); test-anchored (expected test cases before implementation); boundary-respect (no out-of-scope edits); evidence-based (no speculation); escalate early (3 failures at same category = STOP).
<!-- eidolon:apivr end -->

<!-- eidolon:idg start -->
## IDG — Documentation synthesis (v1.2.2)

Entry:     `.eidolons/idg/agent.md`
Full spec: `.eidolons/idg/IDG.md`
Cycle:     I (Intake) → D (Draft) → G (Gate)

**P0 (non-negotiable):** synthesis from provided context only (no retrieval or code analysis); structural markers ([DECISION], [ACTION], [DISPUTED], [GAP]) required; CHT verification gate (Completeness / Helpfulness / Truthfulness) with one revision max; provenance-first (every claim traces to source session).
<!-- eidolon:idg end -->

<!-- eidolon:forge start -->
## FORGE — Reasoner / structured deliberation (v1.3.2)

Entry: `./.eidolons/forge/agent.md`
Spec:  `./.eidolons/forge/REASONER.md`
Cycle: F (Frame) → O (Observe) → R (Reason) → G (Gate) → E (Emit)

**P0 (non-negotiable):** reasoning-only (no tools, no mutations); frame first
(refuse vague questions); ≥3 hypotheses with adversarial stress-tests;
evidence-anchored claims (H/M/L tiers); bounded deliberation (≤3 passes +
1 REFORGE); reversal conditions mandatory.

See `./.eidolons/forge/AGENTS.md` for full rules and the phase pipeline.
<!-- eidolon:forge end -->

<!-- eidolon:vigil start -->
## VIGIL — Forensic debugger (v1.1.2)

Entry:     `./.eidolons/vigil/agent.md`
Full spec: `./.eidolons/vigil/VIGIL.md`
Cycle:     V (Verify) → I (Isolate) → G (Graph) → I (Intervene) → L (Learn)
Authority: read-only

**P0 (non-negotiable):** reproduction gates attribution (≥2 deterministic runs or statistical CI ≥85%); dependency-graph ranking (never temporal order); ≥3 hypotheses before intervention; counterfactual-gated blame (minimal flip from fail→success); ≤5 intervention budget then escalate; flag-gated authority (read-only | sandbox | write — write never inferred); evidence-anchored findings with `path:line` + confidence tier; non-determinism declared, not masked.
<!-- eidolon:vigil end -->

<!-- eidolon:cortex start -->
## Eidolons Routing Cortex

When a free-form prompt arrives that doesn't already name an Eidolon, route it via the cortex.

**Read:** `.eidolons/cortex/EIDOLONS.md` — always-loaded descriptor table + dispatch protocol. It tells you which Eidolon (or chain) handles the prompt, at what tier (`standard` or `TRANCE`), and what hand-off contract to use.

**Deep tables** (load on demand): `.eidolons/cortex/trance-matrix.md`, `.eidolons/cortex/handoff-graph.md`, `.eidolons/cortex/validation-gates.md`.
<!-- eidolon:cortex end -->
