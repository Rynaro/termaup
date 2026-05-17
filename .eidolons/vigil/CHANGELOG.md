# Changelog

All notable changes to VIGIL are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version numbers follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html). For a methodology spec, "breaking" means any change to phase contracts or JSON schemas that requires existing implementations to be updated.

---

## [1.1.2] — 2026-05-13 — declare ECL v2.0 conformance

### Changed

- Declaration-only patch bump. No behaviour change, no schema change, no
  envelope-shape change.
- VIGIL emit envelopes are byte-compatible with ECL v2.0 (backward-compatible
  per ECL §7.3; compatibility window through 2027-05-13).

Files modified:
- `ECL_VERSION` — `1.2` → `2.0`
- `agent.md` frontmatter `comm.envelope_version` — `"1.2"` → `"2.0"`
- `install.sh` `EIDOLON_VERSION` — `1.1.1` → `1.1.2`

### Notes

- AGENTS.md has no YAML frontmatter — Markdown-only entry point; no comm-block
  edit there.
- Spec reference: `Rynaro/eidolons-ecl@v2.0.0` (`spec/ecl-2.0.md`, ISE trust
  hierarchy).
- Final patch of the Phase 2.D cycle: all six Eidolons now declare ECL v2.0
  conformance. ATLAS v1.5.2, SPECTRA v4.3.2, APIVR-Δ v3.1.2, IDG v1.2.2,
  FORGE v1.3.2 already released; VIGIL v1.1.2 closes the cycle.

---

## [1.1.1] — 2026-05-12 — Declare ECL v1.2 conformance

### Changed
- `ECL_VERSION` file: `1.0` → `1.2`. Targets the latest ECL spec
  (`Rynaro/eidolons-ecl@v1.2.0`); VIGIL's emit envelopes remain
  byte-compatible (v1.2 is backward-compatible with v1.0 per ECL §1.1.1).
- `agent.md` frontmatter: `comm.envelope_version` `"1.0"` → `"1.2"`.
- `install.sh`: `EIDOLON_VERSION` `1.1.0` → `1.1.1` (PATCH bump —
  declaration-only change; no behaviour change).

### Notes
- No envelope-format changes. v1.0 envelopes already emitted by older
  VIGIL releases are valid under v1.2 conformance.
- VIGIL's emit edges to APIVR-Δ and SPECTRA are `trust_level=high`
  per `vigil-to-apivr.yaml` and `vigil-to-spectra.yaml`. ECL v1.1 gate
  I-5 SHOULD-level warns when high-trust envelopes use `sha256`.
  Operators provisioning VIGIL at scale SHOULD pass
  `--integrity-method hmac-sha256` with an `ECL_HMAC_KEY` env per ECL
  v1.1 §6.4. The apivr-vigil escalation worked example in
  `Rynaro/eidolons-ecl@v1.1.0+` already demonstrates this pattern.
- Inbound verification (`repair-failed-report` from APIVR-Δ) continues
  to accept v1.0/v1.1/v1.2 envelopes; halt-on-fail semantics preserved.

## [1.1.0] — 2026-05-10 — ECL v1.0 adoption

### Added

- **`ECL_VERSION`** — root single-line file declaring `1.0`. Read by nexus during `eidolons sync` to warn on ECL spec mismatches (ECL §7.2).
- **`schemas/ecl/envelope.v1.json`** — vendored copy of `eidolons-ecl/schemas/envelope.v1.json` with the 10-value `performative` enum inlined (no `$ref` resolver required; `jq empty` validates standalone). Source: ECL v1.0 (2026-05-08).
- **`schemas/ecl/_base-profile.v1.json`** — vendored base profile for downstream compose checks.
- **`schemas/ecl/root-cause-report.v1.json`** — vendored copy of `eidolons-ecl/schemas/per-eidolon/root-cause-report.v1.json` with base-profile constraints inlined. Strict subset of local `schemas/root-cause-report.v1.json`, mirroring VIGIL P0 invariants (`reproduction_runs >= 2`, `hypotheses_count >= 3`, `interventions_count <= 5`, `blame_target` present).
- **`schemas/ecl/contracts/vigil-to-apivr.yaml`** — vendored outbound contract.
- **`schemas/ecl/contracts/vigil-to-spectra.yaml`** — vendored outbound contract.
- **`schemas/ecl/contracts/vigil-to-idg.yaml`** — vendored outbound contract.
- **`schemas/ecl/contracts/apivr-to-vigil.yaml`** — vendored inbound contract; consumed by Phase V verify gate.
- **`templates/root-cause-report.envelope.json`** — skeleton envelope sidecar for Phase L emission (root-cause-report to any recipient).
- **`templates/escalation-brief.envelope.json`** — skeleton envelope sidecar for FORGE escalation path (performative `ESCALATE`).

### Changed

- **`agent.md`** — version `1.0.1 → 1.1.0`; added `comm:` frontmatter block declaring `envelope_version: "1.0"`, `emits: [root-cause-report, escalation-brief]`, `consumes: [repair-failed-report]`.
- **`VIGIL.md`** — added I-11 invariant; extended §2.1 (Phase V) with inbound-verify step on escalation entry; extended §2.5 (Phase L) with envelope-emit step; added §10 ECL compatibility note.
- **`AGENTS.md`** — new section "## ECL Emission Contract" between "## Schema Validation" and "## Structural Markers"; five-line emit invariant + pointers to `schemas/ecl/` and `templates/*.envelope.json`.
- **`skills/learn/SKILL.md`** — new subsection "## Envelope Emission" after "Handoff Directive"; SHA-256 computation, UUIDv7 generation, fan-out procedure, `.envelope.<recipient>.json` suffix convention, trust-level mapping, trace-event append.
- **`skills/verify/SKILL.md`** — new subsection "## Inbound Envelope Verification (Escalation Entry)" at the top; verify-before-process: schema, contract match, integrity check, trace append, halt-on-fail.
- **`skills/intervene/SKILL.md`** — added reference at bottom for escalation-brief envelope construction.
- **`templates/root-cause-report.md`** — added top-of-file ECL v1.0 note.
- **`DESIGN-RATIONALE.md`** — new section "## ECL v1.0 Adoption Rationale" documenting D1, D3, D4, D6.
- **`install.sh`** — bumped `EIDOLON_VERSION` `1.0.3 → 1.1.0`; added `ECL_VERSION` to source sanity check; added `schemas/ecl/` mkdir; added `ECL_VERSION`, ECL JSON schemas, ECL contract YAML files, and two `.envelope.json` templates to copy loops; added `comm.envelope_version: "1.0"` to manifest (additive).
- **`CLAUDE.md`** — version footer `1.0.1 → 1.1.0`.

### Deferred

- **`hmac-sha256` integrity method** — `trust_level: high` edges use `sha256` in v1.1.0. Promotion deferred to ECL v1.1 GA + nexus `ECL_HMAC_KEY` distribution mechanism (D1).
- **`vigil-to-forge.yaml` upstream contract** — FORGE lateral edge uses `edge_origin: "roster"` with no vendored contract (follow-up PR to `eidolons-ecl` as F2).

### Fan-out filename convention

Fan-out envelopes use `<basename>.envelope.<recipient>.json` (e.g. `…envelope.spectra.json`, `…envelope.idg.json`). Chosen for human-readability per Q2 resolution.

## [1.0.3] - 2026-04-26 — Re-release v1.0.2 + manifest cleanup

### Fixed
- The previously-published `v1.0.2` tag pointed at the **pre-codex commit** by mistake. v1.0.3 is the same intended release re-cut at the correct `HEAD` (`bea134c`); v1.0.2 stays frozen but is effectively superseded.
- `install.sh` manifest emission no longer writes the non-EIIS `authority_mode` field (rejected by EIIS v1.1's `additionalProperties: false`); VIGIL's read-only stance is captured in `security.writes_repo` already.
- `install.sh` shared-dispatch helper now emits `mode: "overwritten"` instead of `mode: "rewritten"` to match the EIIS v1.1 enum (`created`, `appended`, `overwritten`).
- `examples/install.manifest.json` regenerated to reflect both manifest fixes; now passes `python -m jsonschema` against the vendored EIIS v1.1 schema.

## [Unreleased]

### Added

- **Release-integrity adoption.** `.github/workflows/release.yml` wraps
  the eidolons-nexus `eidolon-release-template.yml` (post-merge of
  `Rynaro/eidolons` PR #24) so a maintainer-triggered `workflow_dispatch`
  on this repo produces a tagged GitHub release with `release-manifest.json`
  (commit + tree + `archive_sha256`), `SHA256SUMS`, and a GitHub artifact
  attestation. Roster Intake on the nexus side consumes this metadata to
  populate `roster/index.yaml` `versions.releases.<v>`. Provenance is
  GitHub-attestation-based (sigstore-backed); no GPG keys, no per-Eidolon
  secrets. `manifest_sha256` is published as `null` for now because no
  `install.manifest.json` is committed at repo root; a follow-up may
  address `install.sh` determinism (target-path embedding in dispatch
  files) so a stable repo-root manifest can be committed.

## [1.0.2] — 2026-04-24 — EIIS-1.1 conformance + OpenAI Codex host

### Added

- **`EIIS_VERSION`** — root single-line file declaring `1.1`. Resolves drift
  D-6 (universal) and unlocks v1.1 features in the conformance checker.
- **`schemas/install.manifest.v1.json`** — verbatim vendored copy of the
  EIIS v1.1 manifest schema, kept in sync with future EIIS bumps so the
  installer-emitted manifest can be self-validated locally without
  network access (EIIS v1.1 §1.5).
- **OpenAI Codex host (`codex`)** — `install.sh` now accepts `--hosts codex`
  (and includes it in `--hosts all`), auto-detects `.codex/` or
  `AGENTS.md`-without-`.github/` consumer projects, writes
  `.codex/agents/vigil.md` with the EIIS v1.1 §4.5 frontmatter contract
  (`name: vigil`, `description:` describing VIGIL's role), and owns a
  marker-bounded block in root `AGENTS.md` whenever `codex` is wired
  (EIIS v1.1 §4.1.0 — co-owned by `copilot` and `codex`). The Codex body
  mirrors the existing Claude agent prompt and points at
  `${TARGET}/agent.md` as the canonical methodology entry.
- **`examples/install.manifest.json`** — sample manifest fixture so the
  EIIS conformance checker can structurally validate the emitted manifest
  shape (gates M1–M14).

### Changed

- **`install.sh`** — host list now validates entries up-front and rejects
  unknowns with exit 2; `--hosts none` is honoured explicitly. The
  generated `.vigil/config.yml` no longer embeds an installer timestamp,
  keeping `installed_at` as the only non-deterministic field
  (EIIS §3.5 / conformance gate I3).
- **Version footers** synchronised across the repo from `1.0.1` → `1.0.2`
  via the manifest emitter.

### Conformance

- `bash conformance/check.sh` (eidolons-eiis): `exit 3` (SHOULD-fail
  on the missing vendored schema) → `exit 0` after this release.

## [1.0.1] — 2026-04-23 — EIIS-1.0 conformance

### Changed

- **`install.sh`** — Full EIIS-1.0 §3 interface: `--target DIR`, `--hosts LIST`, `--force`, `--dry-run`, `--non-interactive`, `--manifest-only`, `--version`, `--shared-dispatch`/`--no-shared-dispatch`, `-h/--help`. Legacy positional target preserved with deprecation warning; existing `--mode=VALUE` and `--mode VALUE` both accepted. Emits `install.manifest.json` with token budget, hosts wired, handoffs declared, security posture, and authority mode. Writes per-host dispatch files for claude-code (`.claude/agents/vigil.md`), cursor (`.cursor/rules/vigil.mdc`), and opencode (`.opencode/agents/vigil.md`). Idempotency check: non-interactive mode exits 3 on existing manifest without `--force`.
- **Version footers** — synchronized across AGENTS.md, README.md, CLAUDE.md, VIGIL.md, DESIGN-RATIONALE.md, agent.md, all four host wirings, all four templates, and canary missions from 1.0.0 → 1.0.1.

### Unchanged

- FORGE cycle (Verify → Isolate → Graph → Intervene → Learn)
- Ten architectural invariants, eleven-category failure taxonomy
- Five phase skills, four decision templates, three JSON schemas
- Per-project `.vigil/config.yml` generation and memory ledger bootstrap

## [1.0.0] — 2026-04-16

Initial release of the VIGIL methodology — forensic debugger for code failures.

### Added

- **agent.md** — always-loaded entry point (~900 tokens) with identity, P0 invariants, skill-loading triggers, and team-wide structural markers.
- **VIGIL.md** — authoritative methodology specification covering all five phases (Verify, Isolate, Graph, Intervene, Learn), ten architectural invariants, eleven-category failure taxonomy, and six portable abstractions.
- **AGENTS.md** — agents.md open-standard compliant rules file for Copilot, Cursor, OpenCode hosts.
- **CLAUDE.md** — Claude Code entry pointer.
- **skills/** — five progressive-disclosure phase skills:
  - `verify/SKILL.md` — deterministic-first reproduction protocol, statistical switch on two-failure threshold, Wilson 95% CI for small-N flakes
  - `isolate/SKILL.md` — delta-debugging-style reduction with category-specific techniques (git bisect, dep walk, contract comparison, timing differential); ruled-out trail as first-class output
  - `graph/SKILL.md` — Information Dependency Graph construction, symptom/root discrimination, graph-shape handling (cyclic, disconnected, single-node)
  - `intervene/SKILL.md` — counterfactual replay protocol, ≥3-hypothesis plurality rule, 5-intervention hard cap, authority-gated execution, statistical-mode flip threshold (4/5)
  - `learn/SKILL.md` — finding emission with ATLAS-compatible schema, walk-back to originating decision, failure-signature memory with de-duplication, routing to downstream Eidolon
- **templates/** — four artifact skeletons:
  - `root-cause-report.md` — primary deliverable (~1,150 tokens)
  - `verified-patch.md` — conditional patch artifact when authority permits
  - `failure-signature.md` — memory ledger entry with de-duplication scoring
  - `escalation-brief.md` — budget-exhausted path with "what evidence would resolve this" section
- **schemas/** — three JSON Schema v2020-12 validators:
  - `reproduction.v1.json` — Phase V output; enforces ≥2 runs, statistical evidence on flaky verdict, GAP marker on intermittent
  - `intervention-log.v1.json` — Phase Intervene output; enforces ≥3 hypotheses, ≤5 interventions, null-survivor implies escalation
  - `root-cause-report.v1.json` — Phase L primary output; rejects L-confidence for root-cause emission, enforces SPEC_DEFECT→SPECTRA routing
- **hosts/** — per-host wiring documentation for Claude Code, Cursor, GitHub Copilot, OpenCode.
- **DESIGN-RATIONALE.md** — exhaustive mapping from every non-obvious design decision to its research basis (GraphTracer, AgenTracer, CHIEF, Lifecycle of Failures, Delta Debugging, TALE, CorrectBench, SWE-bench+, OpenTelemetry GenAI).
- **install.sh** — idempotent installer with `--mode` (read-only|sandbox|write), `--force`, per-project config generation, memory ledger initialization.
- **evals/canary/missions.md** — 23-mission evaluation dataset:
  - 15 deterministic missions (target ≥80% pass rate) covering all non-flake failure categories, three entry modes, confirmation-bias trap, plurality enforcement
  - 8 non-deterministic missions (target ≥65% pass rate) covering heisenbugs, CI-only flakes, external-dep flakes, concurrent writes, statistical edge cases
- **README.md** — architecture overview, quick start, team integration, research foundation.

### Architectural Invariants (v1.0)

1. Reproduction gates attribution
2. Dependency graph, not temporal sequence
3. Hypothesis plurality (≥3 before intervention)
4. Counterfactual gates blame
5. Bounded intervention budget (5 hard max)
6. Flag-gated authority
7. Evidence-anchored findings
8. Non-determinism declared, not masked
9. Sandbox adapter interface (pluggable)
10. Telemetry-driven compaction

### Handoff Contracts (v1.0)

- `APIVR-Δ → VIGIL` (escalation mode): consumes `repair-failed-report.md` + session log + delta history
- `VIGIL → APIVR-Δ`: emits `root-cause-report.md` + optional `verified-patch.diff` for surgical fixes
- `VIGIL → SPECTRA`: emits `root-cause-report.md` for systemic issues (SPEC_DEFECT, COMPOUND, structural drift)
- `VIGIL → IDG`: emits `root-cause-report.md` + session log for incident chronicling
- `VIGIL → FORGE`: emits `escalation-brief.md` when 5-intervention budget exhausts on ambiguity
- `VIGIL → human`: emits `escalation-brief.md` for environment errors, safety-critical judgments, or unreproducible failures

### Known Limitations

- **Non-determinism attribution is a genuinely open research area.** The 4/5 flip threshold in statistical mode is heuristic, not principled. Future iterations may refine this as research matures.
- **IDG construction cost is not yet benchmarked.** Building the dependency graph from trace evidence costs tokens. An ablation study on minimum trace granularity is in the research backlog.
- **Compound-failure handling emits multiple findings** but does not yet construct a unified causal-graph-with-multiple-roots (CHIEF-style). This is a candidate for v1.1 or v2.0 depending on evaluation results.
- **Oracle validation is single-source.** When the "correct" oracle value for counterfactual replay is itself uncertain (spec ambiguity), attribution currently requires `[DISPUTED]` marker and escalation. Multi-oracle validation is an open design question.
- **Meta-debugging is not supported.** If VIGIL's own mission fails, the current design escalates to human. Recursive self-diagnosis was explicitly rejected per D5 (Bounded Self-Correction).

### Token Budget

- Entry point: ~877 tokens (under 1,000-token cap)
- Individual skills: 1,056–1,760 tokens
- Typical working set (entry + skill + template): ~3,100 tokens (under 3,500 specialist cap)

---

## Versioning Policy

- **Major bump (v2.0.0)**: Changes to invariants (I-1 through I-10). Existing implementations must update fundamentally.
- **Minor bump (v1.1.0)**: Changes to phase contracts, JSON schemas, or failure taxonomy. Implementations must update artifacts but core logic unchanged.
- **Patch bump (v1.0.1)**: Clarifications, typos, additional canary missions, rationale expansions. No implementation changes required.

Implementations declare `methodology_version: "1.0"` in their `agent.md` frontmatter and MUST fail fast on a detected version mismatch with their loaded VIGIL spec.

---

*VIGIL v1.1.0 — Verify · Isolate · Graph · Intervene · Learn*
