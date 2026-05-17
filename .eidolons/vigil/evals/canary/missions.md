# VIGIL Canary Missions — Evaluation Dataset v1.0

Hand-curated missions covering the VIGIL capability surface. Used to regression-test implementations after changes to methodology, skills, or schemas.

## Pass Criteria (global)

Per mission:
1. `[ROOT-CAUSE]` (or `[HYPOTHESIS-N]` in escalation) matches expected
2. Classification matches expected failure taxonomy category
3. Handoff recipient matches expected downstream Eidolon
4. Interventions used ≤ expected budget (for successful missions)
5. Escalation emitted when expected (for hard missions)

**Overall pass targets:**
- Deterministic missions: **≥80%** (12 of 15)
- Non-deterministic missions: **≥65%** (5 of 8)

---

## Deterministic Tier (15 missions)

### VIGIL-CANARY-D-001 — Classic null-guard regression

**Upstream:** APIVR-Δ `repair-failed-report.md` after 3 attempts.

**Setup:** Failing test: `test_record_vote_with_valid_session`. Error: `NoMethodError: undefined method 'bytes' for nil:NilClass`. Stack top: `RecordVote#call:56` → `TokenGenerator#generate_uuid:14`. Git log shows commit `abc123` from 3 days ago changed `generate_uuid` to accept a session argument.

**Expected:**
- Classification: `LOGIC_ERROR`
- Root cause: `generate_uuid` returns `nil` when `session.secret_key` is absent
- Survivor intervention type: `oracle_injection` OR `code_change`
- Interventions used: ≤2
- Handoff: APIVR-Δ
- Walk-back: commit `abc123`, classification `implementation_bug`

**Pass:** `[ROOT-CAUSE]` cites `generate_uuid`; classification LOGIC_ERROR; handoff APIVR-Δ.

---

### VIGIL-CANARY-D-002 — Git-bisect regression

**Upstream:** CI failure report. Test `test_calculate_tax` was passing 2 weeks ago.

**Setup:** Test fails because `calculate_tax(100, "CA")` returns `7.25` instead of `8.25`. Git log has 47 commits in the last 2 weeks across 12 files. No explicit stack trace link to the failure.

**Expected:**
- Classification: `REGRESSION`
- Phase I uses `git bisect` to find introducing commit
- Root cause: specific commit that changed tax rate table
- Interventions used: ≤2
- Handoff: APIVR-Δ

**Pass:** Bisect identifies correct commit; verified patch reverts the tax-rate change.

---

### VIGIL-CANARY-D-003 — Missing nil-guard in helper, multi-call-site

**Setup:** `UserProfile#display_name` occasionally returns `nil`. Helper `truncate(name, 20)` does not handle nil. 8 call sites. Failing test calls helper directly. Reproduction is deterministic (specific fixture).

**Expected:**
- Classification: `RUNTIME_ERROR`
- IDG shows `UserProfile#display_name` as root candidate (not `truncate`)
- Interventions distinguish between "fix helper" (symptom) and "fix display_name" (root)
- Handoff: APIVR-Δ with recommended fix at root, not symptom

**Pass:** `[ROOT-CAUSE]` at display_name; `truncate` marked `[SYMPTOM]`.

---

### VIGIL-CANARY-D-004 — Integration error at component boundary

**Setup:** API endpoint returns 500. Service A sends `user_id: "42"` (string); Service B expects `user_id: 42` (integer). Recent schema change in Service B's contract but caller not updated.

**Expected:**
- Classification: `INTEGRATION_ERROR`
- Phase G identifies contract boundary as fault surface
- Walk-back finds schema change commit
- Handoff: APIVR-Δ (caller update) OR SPECTRA (if contract change was unplanned — needs replan)

**Pass:** Integration error correctly attributed to type mismatch at boundary.

---

### VIGIL-CANARY-D-005 — Type error after library upgrade

**Setup:** TypeScript project, `strictNullChecks` was turned on. Build fails in 14 files. Error: `Type 'string | undefined' is not assignable to type 'string'`.

**Expected:**
- Classification: `TYPE_ERROR`
- Root cause: compiler flag change or library upgrade (not individual files)
- Interventions: 1 (reverting the flag shows it's the single change; then the real question is how to migrate)
- Handoff: SPECTRA (migration plan) — this is a structural change, not surgical fix

**Pass:** Classified as TYPE_ERROR; routed to SPECTRA for migration planning.

---

### VIGIL-CANARY-D-006 — Build error from dep version conflict

**Setup:** `npm install` produces conflicting peer dependency errors; build fails. Package A requires `react@17`, Package B requires `react@18`.

**Expected:**
- Classification: `BUILD_ERROR`
- Root cause: conflicting peer dep constraints
- Phase V may not even require full reproduction — the error message is deterministic
- Handoff: APIVR-Δ (dep resolution) or human (if no resolution exists without removing one package)

**Pass:** Root cause cites both packages and their conflicting constraints.

---

### VIGIL-CANARY-D-007 — Lint violation that masks a real bug

**Setup:** ESLint fires on `no-unused-expressions`. Line: `user.save;` (missing parentheses — ruby-ism in JS). Test passes the lint rule as flagged, but also the test expecting save to persist — fails.

**Expected:**
- Classification: Could be `LINT_VIOLATION` OR `LOGIC_ERROR` — VIGIL should investigate both
- Root cause: the real bug is missing parens; the lint rule just surfaced it
- Handoff: APIVR-Δ

**Pass:** Attribution identifies the missing-parens bug, not just the lint rule.

---

### VIGIL-CANARY-D-008 — Spec defect (test is wrong)

**Setup:** Test asserts `format_date("2026-04-16")` returns `"16/04/2026"`. Implementation returns `"04/16/2026"`. Product spec says US format is expected. Team is US-based.

**Expected:**
- Classification: `SPEC_DEFECT`
- Phase L walk-back finds the test was authored by a non-US contributor; no prior spec review
- Handoff: **SPECTRA** (not APIVR-Δ) — spec needs clarification, not code change
- Verified patch: **not emitted** — no code fix exists

**Pass:** Classified as SPEC_DEFECT; routed to SPECTRA; no patch emitted.

---

### VIGIL-CANARY-D-009 — Environment error (works locally, fails in CI)

**Setup:** Test passes on developer machine. Fails in CI with `File not found: /etc/app/config.yml`. CI container doesn't have the file.

**Expected:**
- Classification: `ENVIRONMENT_ERROR`
- Root cause: env parity gap between dev and CI
- Walk-back: config.yml generation step missing from CI pipeline
- Handoff: **human** (or APIVR-Δ if env-as-code)

**Pass:** Classified correctly; handoff human or APIVR-Δ for CI fix.

---

### VIGIL-CANARY-D-010 — Compound failure (two independent roots)

**Setup:** Failing end-to-end test. Database query returns incorrect results (off-by-one in pagination) AND the display layer also has a bug (rendering the wrong field). Fixing either alone still fails the assertion.

**Expected:**
- Classification: `COMPOUND`
- Phase G identifies 2 disconnected components in IDG
- Interventions: 2+ runs (one per component), potentially chained
- Handoff: **SPECTRA** (replan needed — compound fix)

**Pass:** Both roots identified; routed to SPECTRA.

---

### VIGIL-CANARY-D-011 — Read-only mode (post-hoc analysis)

**Setup:** User supplies a 3-week-old bug report + CI log + failing test. Says "I just want to know what caused this, I'll fix it manually."

**Expected:**
- Authority: `read-only`
- Interventions are **simulated**, not executed
- Max confidence: `[HYPOTHESIS-N]` with HIGH flag; no `[ROOT-CAUSE]`
- Verified patch: not emitted (can't verify without sandbox)
- Handoff: human with explicit note that attribution is unconfirmed

**Pass:** Read-only discipline held; no `[ROOT-CAUSE]` emitted; hypothesis described textually.

---

### VIGIL-CANARY-D-012 — Correct escalation on exhausted budget

**Setup:** Failing test with 8 plausible candidates across 3 modules. All 5 interventions run; none flip (or multiple flip partially, none cleanly).

**Expected:**
- Phase I-Intervene exhausts budget
- **Escalation brief emitted** — not another intervention
- All 5 interventions documented with NO_CHANGE or partial outcomes
- Handoff: FORGE (reasoning ambiguity) or human

**Pass:** Mission escalates cleanly; no 6th intervention attempted; brief has ≥3 remaining hypotheses.

---

### VIGIL-CANARY-D-013 — Confirmation-bias trap

**Setup:** Failing test. Obvious-looking cause: a recent commit modified the exact function in the stack trace. Careful analysis: the commit changed formatting only; real cause is upstream in a different module.

**Expected:**
- Phase I-Isolate flags the recent commit as top candidate (correct)
- Phase I-Intervene: intervention at the obvious location produces NO_CHANGE
- Iteration shifts to upstream hypothesis → FLIPPED
- Interventions used: 2-3

**Pass:** Does not commit to the first plausible hypothesis; survives to correct attribution.

---

### VIGIL-CANARY-D-014 — Invariant I-3 hypothesis plurality enforcement

**Setup:** Artificial test — upstream provides just one obvious-looking candidate, no ambiguity.

**Expected:**
- Even with one clear candidate, VIGIL generates ≥3 hypotheses (different mechanisms)
- At least one hypothesis is "the other candidates in the stack trace are actually the cause" — forces genuine differential analysis
- Interventions used: 1 (the obvious one flips)

**Pass:** Schema validation confirms ≥3 hypotheses before intervention; no shortcut.

---

### VIGIL-CANARY-D-015 — Walk-back with deleted code

**Setup:** Failing test caused by a function that was deleted in a refactor 6 months ago. The caller was not updated. CI didn't catch it then because the test was skipped (recently un-skipped).

**Expected:**
- Classification: `LOGIC_ERROR` (but complex)
- Walk-back: identifies the refactor commit; classification `missing_test` because the un-skipping is what revealed the old bug
- Handoff: APIVR-Δ for surgical fix + test un-skip

**Pass:** Walk-back correctly identifies the 6-month-old commit; classification accurate.

---

## Non-Deterministic Tier (8 missions)

### VIGIL-CANARY-N-001 — Classic timing heisenbug

**Setup:** Test passes 7/10 locally, 3/10 in CI. Race condition between cache warm-up and first query.

**Expected:**
- Phase V: deterministic attempts inconsistent → switches to statistical mode
- `[FLAKE]` marker set
- Classification: `HEISENBUG`
- Root cause: missing `await` or sync primitive
- Statistical evaluation of intervention: ≥4/5 required for FLIPPED

**Pass:** Flake handled gracefully; attribution at `H` confidence if statistical CI sufficient, else `M`.

---

### VIGIL-CANARY-N-002 — Flaky only in CI, not locally

**Setup:** Test passes locally 10/10. Fails in CI 40% of the time. Environment difference (concurrent CI jobs competing for resources).

**Expected:**
- Phase V: local reproduction fails (PASS×5 → not_reproduced)
- Escalation with `[GAP]`: requires CI environment to reproduce
- Recommended next action: get CI-env access or logs
- Handoff: human with specific evidence request

**Pass:** Clean escalation; does not fabricate attribution without reproduction.

---

### VIGIL-CANARY-N-003 — Test depends on unstable external service

**Setup:** Test hits a public API. Fails 20% of the time due to rate limits or transient 500s. The test itself has no retry logic.

**Expected:**
- Classification: `HEISENBUG` with subcategory `external_dependency_flake`
- Root cause: test design depends on external service availability
- Handoff: **SPECTRA** (test-design issue) — not APIVR-Δ
- Recommended fix: mock the external service or add retry with backoff

**Pass:** Correctly routed to SPECTRA; doesn't try to "fix" the external service.

---

### VIGIL-CANARY-N-004 — Memory-layout-dependent failure

**Setup:** Test fails intermittently due to hash-order dependence (older Python, before dict order guarantee). Stack traces vary.

**Expected:**
- Phase V: statistical mode
- Phase G: signature consistency low — multiple failure modes observed
- Challenging for VIGIL — may escalate

**Pass:** Either: (a) identifies hash-order dependence with confidence M+, OR (b) escalates cleanly with observation about signature drift.

---

### VIGIL-CANARY-N-005 — Concurrent database writes

**Setup:** Two services writing to the same record. Test simulates load. Occasionally the record state is inconsistent. No locks.

**Expected:**
- Classification: `HEISENBUG` subcategory `race_condition`
- Statistical reproduction required
- Root cause: missing transactional boundary or optimistic locking
- Handoff: **SPECTRA** (structural fix — needs concurrency strategy)

**Pass:** Correctly identifies race; routes to SPECTRA.

---

### VIGIL-CANARY-N-006 — Intermittent crash with no clear signature

**Setup:** Production logs show crashes with varying stack traces. No reliable local reproduction. Happens ~0.1% of requests.

**Expected:**
- Phase V: not_reproduced locally
- Escalation with specific evidence request: production telemetry, memory dumps
- `[GAP]` marker
- Handoff: human with actionable next steps

**Pass:** Doesn't fabricate attribution; escalates with specific evidence needs.

---

### VIGIL-CANARY-N-007 — Flake fixed by intervention but statistics confounded

**Setup:** Baseline 4/5 failures. Intervention produces 3/5 failures. Better, but not 4/5 pass (the FLIPPED threshold). Genuine improvement but below the signal bar.

**Expected:**
- Intervention marked `NO_CHANGE` (3/5 is below the 4/5 flip threshold)
- Harness suggests narrower intervention or broader hypothesis refinement
- May lead to iteration or escalation

**Pass:** Does not claim FLIPPED on below-threshold statistical evidence. May escalate or iterate.

---

### VIGIL-CANARY-N-008 — Determinism verdict changes across phases

**Setup:** Phase V concludes `flaky` with 4/5. During Intervene, baseline unexpectedly stabilizes (environment becomes quieter). Some runs produce 5/5 failures (pure flake) while others show 0/5 (fully deterministic fix).

**Expected:**
- `[DISPUTED]` marker raised mid-mission
- Phase V re-entered OR mission escalated
- Robustness test — does VIGIL notice the regime shift?

**Pass:** Detects the change; handles gracefully rather than drawing false conclusions from a moving baseline.

---

## Summary

**23 total canary missions:**
- 15 deterministic (targeting ≥80% pass rate — allow ≤3 failures)
- 8 non-deterministic (targeting ≥65% pass rate — allow ≤3 failures; acknowledged harder domain)

**Coverage by failure category:**
- LOGIC_ERROR: 3 missions (D-001, D-003, D-015)
- REGRESSION: 1 mission (D-002)
- RUNTIME_ERROR: 1 mission (D-003)
- INTEGRATION_ERROR: 1 mission (D-004)
- TYPE_ERROR: 1 mission (D-005)
- BUILD_ERROR: 1 mission (D-006)
- LINT_VIOLATION: 1 mission (D-007 — masking case)
- SPEC_DEFECT: 1 mission (D-008)
- ENVIRONMENT_ERROR: 1 mission (D-009)
- COMPOUND: 1 mission (D-010)
- HEISENBUG: 4 missions (N-001, N-003, N-004, N-005)
- Escalation protocol: 3 missions (D-012, N-002, N-006)
- Read-only mode: 1 mission (D-011)
- Anti-patterns (confirmation bias, plurality): 2 missions (D-013, D-014)
- Statistical edge cases: 2 missions (N-007, N-008)

**Mission attributes recorded per run:**
- Interventions used
- Phases entered
- Markers raised
- Handoff recipient
- Confidence tier
- Wall clock
- Tokens in/out

**Regression triggers:** Any pass-rate drop below targets after a methodology or schema change indicates regression. Canary missions must be re-run after any breaking change per CHANGELOG policy.

---

*VIGIL v1.0.1 — Canary Missions*
