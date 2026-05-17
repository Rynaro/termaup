# VIGIL — Design Rationale

Every non-obvious design decision in VIGIL traces to peer-reviewed research, documented production precedent, or explicit trade-off analysis with rejected alternatives named.

This document is the single authoritative map from *decision* → *evidence*. If a future iteration of VIGIL changes a design, the corresponding entry here must be updated.

---

## Core Design Thesis

> **The Debugger-class Eidolon is a forensic specialist operating under strict evidence discipline. Attribution requires reproduction; ranking requires dependency structure; blame requires counterfactual flip. Everything else is scaffolding.**

This thesis derives from four independent research threads converging on the same conclusion: modern agent debugging fails primarily because it relies on log inspection and temporal heuristics rather than reproduction and causal intervention. Each VIGIL invariant traces back to this thesis.

---

## Decision Map

### 1. Single-purpose Debugger, distinct from APIVR-Δ's Reflect phase

**Research input.** APIVR-Δ's Reflect phase [APIVR-Δ paper §4.7] handles ≤3 retry cycles on the 9-category failure taxonomy (TEST_ASSERTION, REGRESSION, BUILD_ERROR, TYPE_ERROR, LINT_VIOLATION, RUNTIME_ERROR, LOGIC_ERROR, INTEGRATION_ERROR, ENVIRONMENT_ERROR). It explicitly escalates when the cap is hit or confidence is low.

**Decision.** VIGIL is a distinct Eidolon, not an extended Reflect phase. Its scope extends APIVR-Δ's taxonomy with three new categories: `HEISENBUG`, `COMPOUND`, `SPEC_DEFECT`.

**Rationale.** APIVR-Δ's Reflect uses targeted-fix protocol (single hypothesis per failure, minimal change, retry). This is correct for straightforward failures where one focused attempt is likely to succeed. It is wrong for:

- Non-deterministic failures (Reflect assumes deterministic reproduction)
- Compound failures requiring multi-node analysis (Reflect handles one root per attempt)
- Spec defects (Reflect has no way to diagnose that the test is wrong)

Building this into APIVR-Δ would violate D2 (Single Responsibility) — the Coder class has no business doing forensic attribution; that's a different capability. The research on debugging-specific agents (AgenTracer, GraphTracer, CHIEF) consistently treats attribution as a distinct task with distinct cognitive demands.

**Precedent.** SPECTRA is a distinct Eidolon from APIVR-Δ for exactly parallel reasons — planning and building are different capabilities. VIGIL is the debugging analog.

---

### 2. V→I→G→I→L cycle with 5 phases, not 7 or 3

**Research input.**
- SPECTRA uses 7 phases (S→P→E→C→T→R→A) — appropriate for planning complexity
- IDG uses 3 phases (Intake→Draft→Gate) — appropriate for synthesis
- APIVR-Δ uses 5 phases (A→P→I→V→Δ/R) — appropriate for implementation
- Research pattern in AgenTracer, CHIEF, Lifecycle of Failures: reproduce → localize → hypothesize → intervene → verify — five steps

**Decision.** 5-phase cycle matching the research pattern: Verify (reproduce) → Isolate (localize) → Graph (analyze dependencies) → Intervene (hypothesize + counterfactual) → Learn (verify + emit).

**Rationale.** Three phases lose critical steps (no separate graph analysis → temporal attribution creeps back in; no separate learn → memory discipline breaks). Seven phases would either duplicate steps or invent ceremony. Five phases matches the empirical rhythm of debugging research.

**Rejected alternative.** Combining Isolate + Graph into one "Localize" phase. Rejected because the phase transition enforces the mental discipline of distinguishing **which nodes to consider** (Isolate — reduction) from **how those nodes relate** (Graph — structural analysis). Merging them has historically produced temporal attribution errors.

---

### 3. Reproduction gates attribution (Invariant I-1)

**Research input.**
- AgenTracer (arXiv:2509.03312) — counterfactual replay as minimum-cost causal proof; standalone LLM attribution accuracy is sub-10% without reproduction
- Lifecycle of Failures in Platform-Orchestrated Agents (arXiv:2509.23735) — empirically demonstrated attribution accuracy lift from 46.3% → 65.8% when replay is enforced vs. log-only analysis
- Classical Delta Debugging (Zeller) — reproduction is the foundational primitive; without it, there is no causality to investigate

**Decision.** Phase V mandates ≥2 consistent deterministic reproductions (or ≥3/5 consistent statistical reproductions) before any downstream phase begins. Schema validation rejects downstream artifacts emitted without valid reproduction artifact.

**Rationale.** The research convergence is unambiguous. Four independent threads reach the same conclusion: log-only attribution is ~50% accurate; reproduction-gated attribution is substantially higher. Making this mechanical (schema enforcement) rather than hortatory (prompt instruction) closes the "just one more prompt fix" drift that degrades agents under long-horizon pressure.

**Rejected alternative.** Soft recommendation ("try to reproduce"). Rejected because D3 (Mechanical Invariants Over Prompt Reminders) is a Prime Directive for the entire roster.

---

### 4. Information Dependency Graph over temporal attribution (Invariant I-2)

**Research input.**
- GraphTracer (arXiv:2510.10581) — IDGs achieve +18.18% attribution accuracy over temporal state-of-the-art in multi-agent failure scenarios
- CHIEF (NeurIPS 2024) — hierarchical causal graphs outperform flat log analysis by ranking root candidates via descendant reach
- Synthesis across all four research documents: the single most common attribution error is treating the earliest-observed symptom as the root cause

**Decision.** Phase G builds an IDG from trace evidence. Candidate ranking uses descendant count over the graph, not temporal order. The harness rejects attribution claims made without IDG construction.

**Rationale.** Failures propagate. The first node where failure is observed is usually downstream of where it originated. Temporal-ordering heuristics ("whatever error fires first is the root cause") are intuitive and wrong. The research quantifies the wrongness: systems using IDG attribution are measurably more accurate than systems using temporal attribution.

**Rejected alternative.** Using the test assertion location as root cause. Rejected for the same reason — the test observes failure; it rarely causes it.

**Related decision.** Cycles in the IDG halt the mission with `[DISPUTED]` rather than attempting auto-resolution. Cycles indicate either a false edge (fix the graph) or a genuine feedback loop (requires human reasoning). Neither is VIGIL's domain to resolve autonomously.

---

### 5. Hypothesis plurality — ≥3 competing hypotheses before intervention (Invariant I-3)

**Research input.**
- Consistent across AgenTracer, GraphTracer, CHIEF: single-hypothesis convergence produces confirmation-bias-driven misattribution
- APIVR-Δ's Reflect protocol specifies one hypothesis per failure — explicitly acknowledges this is insufficient for harder cases (escalation trigger)
- Classical Popperian falsification: the scientific protocol requires competing explanations before experiment, not before conclusion

**Decision.** Schema validation in `intervention-log.v1.json` requires ≥3 hypotheses before any intervention runs. The harness blocks intervention-type tool calls if fewer than 3 hypotheses are active.

**Rationale.** Forcing three hypotheses is a cognitive discipline, not a busywork requirement. It prevents the failure mode of "plausible first answer → design intervention to confirm it → intervention succeeds for the wrong reason." When three distinct mechanistic hypotheses compete, the intervention design is forced to be **falsification-optimal** — it must differentiate the hypotheses, not merely confirm one.

**Rejected alternative.** 2 hypotheses minimum. Rejected because binary choices invite the confirmation trap — a single intervention "confirming A over B" often just confirms A without discriminating A from the true cause C. Three hypotheses force true differential design.

---

### 6. Counterfactual replay as the causal-proof primitive (Invariant I-4)

**Research input.**
- AgenTracer — explicit demonstration that counterfactual replay (replace candidate's output with oracle value; re-run; observe flip) is the minimum-cost reliable causal proof
- Lifecycle of Failures — same finding, different methodology, same conclusion
- CHIEF — hierarchical causal graphs use oracle-guided backtracking, a variant of counterfactual replay
- Doc 4 source: "Logs are evidence; interventions are diagnosis" — this aphorism captures the entire research consensus

**Decision.** `[ROOT-CAUSE]` emission is gated on a `FLIPPED` counterfactual result. The schema rejects root-cause claims without a corresponding intervention log entry showing failure→success flip.

**Rationale.** This is the strongest causal proof available to VIGIL short of formal verification. A candidate whose correction eliminates the failure **is** the root cause by the standard scientific definition of "necessary and sufficient condition." No intervention flip → no claim of root cause. Period.

**Rejected alternative.** LLM-as-judge attribution (letting the model reason about likely causes from logs). Rejected because sub-10% baseline accuracy is documented across multiple papers. The counterfactual requirement is expensive but produces attribution quality that LLM reasoning alone cannot match.

---

### 7. Bounded intervention budget — 5 hard maximum (Invariant I-5)

**Research input.**
- CorrectBench (2025) — unbounded self-correction degrades output quality in open-ended tasks
- Reflexion (Shinn et al., NeurIPS 2023) — caps reflection memory at last 3 entries; explicit acknowledgment that unbounded reflection causes prose degradation
- APIVR-Δ's Reflect cap at 3 — same research basis, applied to a different capability class
- IDG's bounded revision (1 pass max) — same principle, synthesis version

**Decision.** Hard cap of 5 counterfactual interventions per mission. Schema enforces `max: 5`. Exhaustion triggers escalation brief emission, not continuation.

**Rationale.** Five interventions is enough to resolve most attributable failures — three hypotheses tested exhaustively leaves two budget slots for compound findings or second-pass hypotheses. Missions that exceed five interventions are genuinely ambiguous; continuing produces degraded output, not better attribution. The correct response to a 5-exhausted mission is to escalate with a structured evidence brief (FORGE for reasoning ambiguity, human for judgment).

**Rejected alternatives.**
- **Unbounded** — rejected per CorrectBench; unbounded self-correction degrades quality
- **3** (matching APIVR-Δ's Reflect) — rejected because VIGIL handles harder cases; 3 is too tight for compound failures
- **10** — rejected because double the cap doubles the risk of prose degradation without doubling attribution quality; diminishing returns curve is steep beyond 5

**Number chosen empirically.** Research on agent self-correction consistently shows steep diminishing returns after 3-5 attempts. Five is the empirical sweet spot — not too tight, not into the degradation zone.

---

### 8. Three entry modes with single methodology (escalation / consultant / post-hoc)

**Research input.**
- IDG's design (Scribe) demonstrates a single methodology working across invocation paths (user, agent, orchestrator) without compromise
- ATLAS's mission contract is identical regardless of who invokes (SPECTRA, human, parent agent)
- Research convergence: "methodology must be identical regardless of invocation path; only context arrives differently"

**Decision.** VIGIL runs the same V→I→G→I→L cycle in all three entry modes. Only the upstream artifact format and default authority differ.

**Rationale.** Different quality standards per invocation path = inconsistent outputs from the same agent = impossible to reason about. The upstream artifact structure (APIVR-Δ report vs trace file vs bug description) differs, but the core logic (reproduce, isolate, graph, intervene, learn) is identical.

**Rejected alternative.** Three distinct agent definitions with shared skills. Rejected because maintaining three parallel agents that are 90% identical is a maintenance disaster; the 10% variance (upstream artifact format) is better handled by polymorphic intake in Phase V.

---

### 9. Flag-gated authority — read-only / sandbox / write (Invariant I-6)

**Research input.**
- ATLAS's read-only guarantee is enforced mechanically, not by prompt instruction — shown in the ATLAS doctrine as the exemplar for D3
- Security and privacy research across all four source documents: authority must be explicit, not inferred
- Production agent observability research (dev.to 2025 Replit incident) demonstrates catastrophic failure modes when authority is implicit

**Decision.** Three explicit authority modes. Default for post-hoc: `read-only`. Default for escalation/consultant: `sandbox`. `write` requires explicit per-project configuration.

**Rationale.** Writing to the working tree is a qualitatively different action from simulating or sandboxing an intervention. Defaulting to the least-privileged option per mode matches the security principle of least authority. `write` as a flag-only option means a careless invocation cannot accidentally modify production code.

**Rejected alternative.** Single "safe" mode that does everything. Rejected because it either over-restricts (no interventions possible even in safe sandbox) or under-restricts (interventions can escape the sandbox).

**Related decision.** Even in `write` mode, VIGIL does NOT auto-apply patches to the working tree. It emits `verified-patch.diff` for downstream application. This preserves the handoff contract and gives the user (or APIVR-Δ) the final decision on whether to apply.

---

### 10. Pluggable sandbox adapter — harness chooses, VIGIL uses the interface (Invariant I-9)

**Research input.**
- OpenTelemetry's portable trace model — same pattern, different domain
- Docker/Podman for container isolation; language-native test harnesses; Firejail/bubblewrap for host-level isolation — all valid implementations of "isolated execution"
- Cross-host heterogeneity: Claude Code, Cursor, Copilot, OpenCode all have different native sandboxing capabilities

**Decision.** VIGIL specifies the sandbox adapter interface (apply patch, run command, revert, report result) but does not implement it. Host environments provide the adapter.

**Rationale.** Hard-coding Docker would break VIGIL on Copilot-in-VS-Code where Docker may not be available. Hard-coding language-native would break VIGIL on polyglot projects. The adapter interface is the migration-safe primitive; the implementation is host-specific.

**Rejected alternative.** Build-in Docker requirement. Rejected because D9 (Host-Agnosticism Is Explicit) forbids vendor lock-in at the core-methodology level.

---

### 11. Deterministic-first, statistical-second non-determinism protocol (Invariant I-8)

**Research input.**
- AgenTracer's counterfactual method explicitly assumes deterministic replay — an acknowledged limitation
- Doc 2 (ADA doctrine) flags non-determinism as a genuinely unsolved research question
- Wilson score interval for small-N confidence bands — classical statistics, well-established

**Decision.** Phase V attempts deterministic reproduction first. On two failed deterministic attempts, switches to statistical mode: 5 runs, Wilson 95% CI, requires ≥3/5 failures with matching signature. Subsequent phases inherit statistical mode; interventions require ≥4/5 flip to count as `FLIPPED`. The `[FLAKE]` marker is mandatory throughout.

**Rationale.** Many code failures are deterministic — deterministic-first avoids the cost of statistical mode when it's not needed. When determinism fails (heisenbugs, concurrency bugs, environment-sensitive failures), statistical attribution is the next best tool. Confidence bands are made explicit so downstream consumers can weight the finding appropriately.

**Rejected alternatives.**
- **Statistical only** — rejected because 5-run overhead per reproduction is unnecessary for the common deterministic case
- **User-flag choice** — rejected because the user often doesn't know if the failure is deterministic until they try
- **Quarantine non-deterministic failures** — rejected because heisenbugs are a major part of what escalates from APIVR-Δ; refusing them would leave a critical gap

**Honest limitation.** The 4/5 flip threshold in statistical mode is a heuristic. Research on statistical attribution under LLM non-determinism is not mature enough to provide a principled threshold. This is flagged in the research backlog.

---

### 12. Evidence-anchored findings with confidence tiers H/M/L (Invariant I-7)

**Research input.**
- ATLAS's `FINDING-NNN` schema with `path:line_start-line_end` + confidence tier — already established team-wide convention
- Research across all sources on agent hallucination: unanchored claims are systematically unreliable
- D6 (Evidence-Anchored Claims) is a Prime Directive for the entire roster

**Decision.** VIGIL inherits ATLAS's finding schema exactly — `[FINDING-NNN]` with `path:line_start-line_end` + confidence tier (`H|M|L`). Adds domain-specific fields: `counterfactual_result` (FLIPPED/NO_CHANGE/N/A) and `intervention_id`.

**Rationale.** Team-wide consistency matters more than local optimization. A finding from VIGIL should be indistinguishable in format from a finding from ATLAS; downstream consumers (APIVR-Δ, SPECTRA, IDG) work with a single schema.

**Extension.** VIGIL's confidence tiers have stricter conditions than ATLAS's because attribution is a stronger claim than exploration:

- `H` — deterministic reproduction + FLIPPED intervention + clean IDG (single root, no disputed edges)
- `M` — statistical reproduction with ≥0.85 CI + FLIPPED, OR deterministic + FLIPPED but one disputed edge
- `L` — **not admissible for `[ROOT-CAUSE]`**; only allowed in escalation briefs as `[HYPOTHESIS-N]`

The L-inadmissibility rule is enforced in the schema. VIGIL does not emit low-confidence root causes.

---

### 13. Failure taxonomy — 11 categories extending APIVR-Δ's 9

**Research input.**
- APIVR-Δ Reflect taxonomy: TEST_ASSERTION, REGRESSION, BUILD_ERROR, TYPE_ERROR, LINT_VIOLATION, RUNTIME_ERROR, LOGIC_ERROR, INTEGRATION_ERROR, ENVIRONMENT_ERROR
- Research-identified categories absent from APIVR-Δ's taxonomy: HEISENBUG (non-deterministic), COMPOUND (multi-root), SPEC_DEFECT (test is wrong)
- AgentDebug research (cited in APIVR-Δ) on fine-grained error typing producing 24% higher fix accuracy

**Decision.** VIGIL extends APIVR-Δ's taxonomy with three categories. Aligns the first 8 categories exactly (with `TEST_ASSERTION` → `LOGIC_ERROR` as the practical equivalent). The three new categories are precisely the failure classes APIVR-Δ escalates on.

**Rationale.** Taxonomy alignment enables clean handoffs. An APIVR-Δ `TEST_ASSERTION` escalation becomes a VIGIL `LOGIC_ERROR` mission with no translation loss. The three VIGIL-specific categories are where VIGIL adds value beyond what APIVR-Δ can do alone.

**Rejected alternative.** Inventing a new taxonomy. Rejected because it would force translation at the handoff boundary, creating error surface.

---

### 14. Walk-back to originating decision as mandatory Phase L output

**Research input.**
- Delta Debugging (Zeller) — finding the defect and finding when it was introduced are separate steps, both valuable
- Git bisect as standard industry practice for regression attribution
- Harness AI incident-scribe conventions (cited in IDG's DESIGN-RATIONALE) — "the commit that introduced the defect" is a mandatory field in incident reports

**Decision.** Phase L mandates a walk-back section: git blame the verified defect, identify the introducing commit, read the PR/issue context, classify the originating decision (implementation_bug / spec_defect / contract_drift / dep_change / missing_test / env_drift).

**Rationale.** Knowing **where** the bug lives is attribution. Knowing **why it got there** is prevention. The walk-back is how VIGIL's output becomes a learning artifact for the team — future missions can query the failure-signature ledger, see the originating decision pattern, and avoid repeating it. The classification also routes the fix correctly: `implementation_bug` goes to APIVR-Δ, `spec_defect` goes to SPECTRA, `env_drift` goes to human.

**Rejected alternative.** Skip walk-back when git history is unavailable. Rejected because the procedure gracefully degrades — if git is unavailable, emit `[GAP]` with `unknown_walkback_failed` classification. The discipline is mandatory; the result is optional.

---

### 15. Memory ledger with de-duplication and frequency counts

**Research input.**
- APIVR-Δ's Failure Catalog (cap 30, dedup by root cause) — directly precedent
- ATLAS's Memex (content-addressable excerpt store) — precedent for the storage pattern
- Research convergence on persistent semantic memory for agent systems: MemAct, three-tier memory architectures

**Decision.** VIGIL maintains a `failure-signature.yaml` ledger (default cap 50 entries), with de-duplication scoring (error_class + key_frames overlap + root_cause_path prefix) and frequency counts. Consolidation archives old single-occurrence entries; promotes high-frequency entries for review.

**Rationale.** Future missions should not re-investigate known failure patterns. De-duplication prevents ledger bloat; frequency counts surface recurring patterns worth structural attention (these often indicate a `COMPOUND` or `SPEC_DEFECT` at the architectural level, routable to SPECTRA).

**Rejected alternative.** Unlimited signature ledger. Rejected per MemAct research: unbounded semantic memory degrades retrieval quality. The 50-entry cap with consolidation is the APIVR-Δ-proven pattern.

---

### 16. Handoff contracts per downstream Eidolon

**Research input.**
- SPECTRA's three-artifact handoff (MD + YAML + state JSON) — precedent for structured handoff
- ATLAS's typed handoff recipients (SPECTRA / APIVR-Δ / human) — precedent for routing discipline
- Research on multi-agent handoff failures: bloated context transfer is the #1 cause of cross-agent misattribution

**Decision.** VIGIL emits one or more structured artifacts depending on handoff recipient:

- APIVR-Δ: `root-cause-report.md` + `verified-patch.diff`
- SPECTRA: `root-cause-report.md` (no patch — systemic)
- IDG: `root-cause-report.md` + session log (for chronicling)
- FORGE: `escalation-brief.md` + evidence bundle
- human: `escalation-brief.md`

The handoff recipient is determined by the `originating_decision.classification` + `confidence` combination. Rules are encoded in schema and routing tables.

**Rationale.** Structured handoffs avoid the free-text message problem. Each downstream Eidolon knows exactly what artifact shape to consume. The routing rules are explicit so the decision is auditable.

**Rejected alternative.** Free-form prose handoff "letter." Rejected per cross-agent handoff research — prose handoffs lose specificity and invite misinterpretation at the boundary.

---

## Rejected Architectural Alternatives (Summary)

| Rejected | Why |
|----------|-----|
| Extending APIVR-Δ's Reflect phase instead of separate Eidolon | Violates D2 (single responsibility); APIVR-Δ is Coder-class, not forensic |
| 7-phase cycle | Excess ceremony; research pattern is 5 steps |
| 3-phase cycle | Loses critical Graph/Learn separation |
| Soft reproduction recommendation | Violates D3 (mechanical invariants); drift under pressure |
| Temporal-order attribution | 18%+ accuracy loss per GraphTracer |
| 2 hypotheses minimum | Invites confirmation bias; 3 is empirical minimum for falsification |
| LLM-as-judge root-cause attribution | Sub-10% accuracy baseline |
| Unbounded intervention loop | CorrectBench: prose degradation |
| Hard-coded Docker sandbox | Violates D9 (host-agnosticism) |
| Statistical-only reproduction | Overhead without benefit for deterministic cases |
| Single unified "safe" authority mode | Either over- or under-restricts |
| Auto-applying verified patches in write mode | Violates handoff contract and least-surprise |
| New finding schema different from ATLAS | Violates team-wide consistency |
| Invented taxonomy not aligned with APIVR-Δ | Forces translation at handoff |
| Free-form handoff messages | Cross-agent handoff research: prose loses specificity |

---

## ECL v1.0 Adoption Rationale

> **Spec:** `/Users/henrique/workspace/oss/agents/eidolons/.spectra/vigil-ecl-adoption.md` (SPECTRA v4.2.11, generated 2026-05-08, status: READY FOR APIVR-Δ)

VIGIL v1.1.0 adopts ECL v1.0. Four decisions govern the adoption shape:

### D1 — Integrity method: sha256 (hmac-sha256 deferred)

ECL §6.3 RECOMMENDS `hmac-sha256` for `trust_level: high` edges (which both `vigil-to-apivr` and `vigil-to-spectra` carry). VIGIL v1.1.0 uses `sha256` only.

**Rationale.** No `ECL_HMAC_KEY` distribution mechanism exists in the nexus today. Introducing HMAC without a key-distribution contract would produce an unusable gate (no key → verify_fail on every receive). The sha256 choice is forwards-compatible: VIGIL v1.2 can promote to `hmac-sha256` without a SemVer break in either VIGIL or ECL.

**Precedent.** APIVR-Δ v3.1.0 ECL adoption made the same decision for the same reason.

**Future.** ECL v1.1 GA + nexus `ECL_HMAC_KEY` distribution unlock promotion. Tracked as F3 in the spec follow-ups.

### D3 — Schema vendoring: vendor-inlined-enum

ECL schemas are vendored at `schemas/ecl/` (envelope, base profile, root-cause-report profile). The `performative` enum from ECL §2.1 is **inlined** inside `schemas/ecl/envelope.v1.json` at both `properties.performative` and `properties.expected_response.properties.performative`.

**Rationale.** The gate command `jq empty schemas/ecl/*.json` (G0) must pass without a `$ref` resolver. Inlining makes the vendored set self-contained. The upstream source (`eidolons-ecl/schemas/envelope.v1.json`) uses a `$ref` to a sibling `performative.v1.json`; the vendored copy inlines that reference instead.

**Drift management.** `ECL_VERSION` declares the targeted spec version. On every ECL minor, diff `schemas/ecl/envelope.v1.json` against upstream (modulo the inlined enum) and bump VIGIL accordingly.

### D4 — Fan-out: one payload, N envelopes

When a finding routes to multiple recipients (e.g. SPEC_DEFECT → SPECTRA + IDG), VIGIL writes the payload `root-cause-report.md` **once** and writes **one envelope per recipient** with distinct `message_id` values but shared `thread_id` and `parent_id`.

**Rationale.** ECL §1.1.3 mandates that `to.eidolon` identifies a single recipient. There is no multicast performative in ECL v1.0. Fan-out via distinct envelopes satisfies the spec while allowing one-payload-many-recipients without duplicating payload bytes.

**Filename convention.** Fan-out envelopes use the suffix `<basename>.envelope.<recipient>.json` (e.g. `root-cause-report-20260510-001.envelope.spectra.json`) — chosen for human-readability since the recipient set is small (≤4). Documented in CHANGELOG.

### D6 — Receiver-side verification: verify-before-process

On escalation entry (APIVR-Δ → VIGIL), Phase V verifies the inbound `repair-failed-report.envelope.json` BEFORE processing the payload. A `verify_fail` halts the mission with a `[GAP]` finding referencing the ECL §5.3 failure code.

**Rationale.** ECL §6.2.2 says receivers SHALL recompute integrity before acting on the payload. VIGIL is the highest-trust artefact in the system — it would be contradictory for VIGIL to emit high-trust findings while itself accepting unverified payloads. The verify-before-process stance is the only consistent choice.

**Note on opt-in posture.** APIVR-Δ v3.1.0 adopted a warn-only (never-refuse) posture for its verify-incoming skill, citing ECL §0 (opt-in). VIGIL takes the stricter MUST-halt stance on verify_fail because: (a) VIGIL operates on forensic evidence where payload tampering is a meaningful threat; (b) the spec explicitly calls this out as the highest-trust edge in the system. This is a deliberate divergence from APIVR-Δ's pattern.

---

## Research Foundation (Citation Map)

| Decision | Primary research basis | Supporting evidence |
|----------|------------------------|---------------------|
| Reproduction-gates-attribution (I-1) | AgenTracer (arXiv:2509.03312), Lifecycle of Failures (arXiv:2509.23735) | Delta Debugging (Zeller); 46.3%→65.8% accuracy lift |
| IDG over temporal order (I-2) | GraphTracer (arXiv:2510.10581) | CHIEF (NeurIPS 2024); +18.18% accuracy |
| Hypothesis plurality ≥3 (I-3) | AgenTracer, GraphTracer confirmation-bias findings | Popperian falsification protocol |
| Counterfactual replay (I-4) | AgenTracer, CHIEF | Lifecycle of Failures; "logs vs interventions" aphorism |
| 5-intervention cap (I-5) | CorrectBench (2025) | Reflexion (NeurIPS 2023); APIVR-Δ's 3-cap precedent |
| Authority flag gating (I-6) | Production agent observability (Replit incident) | ATLAS read-only guarantee precedent |
| Evidence-anchored findings (I-7) | ATLAS schema (team precedent) | D6 (Prime Directive); agent hallucination research |
| Non-determinism protocol (I-8) | Wilson CI (classical statistics); AgenTracer acknowledged limitation | Open research question flagged in backlog |
| Pluggable sandbox (I-9) | OpenTelemetry pattern | D9 (host-agnosticism); cross-host heterogeneity |
| Layered loading architecture | APIVR-Δ precedent | JetBrains context-quality research; Anthropic context engineering |
| Failure taxonomy extension | APIVR-Δ Reflect (precedent) | AgentDebug fine-grained error-typing (+24% fix accuracy) |
| Walk-back procedure | Delta Debugging (Zeller) | Harness AI incident conventions |
| Memory ledger with de-dup | APIVR-Δ Failure Catalog | MemAct; three-tier memory research |
| Structured handoff contracts | SPECTRA/ATLAS precedent | Cross-agent handoff research |

---

## Research Backlog for Future VIGIL Iterations

Ranked by expected impact on design:

1. **Non-determinism attribution accuracy.** Current 4/5 flip threshold is heuristic. Empirical studies on statistical attribution under LLM non-determinism would let us set a principled threshold and a validated confidence decay function.

2. **IDG construction cost.** Building an IDG from trace evidence is not free — it costs tokens. An ablation study on the minimum trace granularity required for reliable IDG construction would optimize the cost/accuracy curve.

3. **Compound failure handling.** Current design handles compound failures by emitting multiple findings. Whether a more sophisticated "causal graph with multiple roots" approach (CHIEF-style) would produce higher attribution accuracy is unvalidated.

4. **Learned summarization for mission compaction.** SUPO-style learned summarization outperforms heuristic summarization in research settings but requires RL training. Inference-only approximations are an open question.

5. **Oracle validation for counterfactual replay.** Currently, the "correct" oracle value is assumed correct. When the oracle itself might be wrong (spec ambiguity, multiple valid interpretations), attribution becomes multi-step. Needs design thought.

6. **Meta-debugging.** When VIGIL's own run fails, the current design escalates to human. A formalized recursive trace protocol is an open research area.

---

*VIGIL v1.1.0 — Design Rationale*
