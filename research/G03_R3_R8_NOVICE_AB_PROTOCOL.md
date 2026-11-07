# NORDOI G0.3 R3-R8 — Preregistered novice error-comprehension pilot

**RESEARCH-ONLY / NON-NORMATIVE / NO PRODUCTION / NO HUMAN EVIDENCE YET.**

Tracking: R3-R6 issue #25 and draft PR #26; R3-R7 issue #27 and draft PR #28. Source `main` pinned at `98da03b2ea1efcf665d205189284aa8b66fda5d3`, frozen production `e2898bf7c511e53636ff28c9fd3efc93165a0215`. Prior R2 independence unverified (A=1/B=0). The R3-R6 owner Mac technical log showed 10/10 case passes; the R3-R7 Mac verifier passed, but neither measured human comprehension.

## What is being compared?

**A / control:** exact stderr captured by the existing NORDOI binary in R3-R6 (including the full CLI help for invalid flags). **B / experimental:** the same exact stderr, plus R3-R7's *Problem / Why / Safe next action* **research hint**. This does not test a new compiler diagnostic, change the CLI, omit or rewrite existing error messages, grant any capability, or alter `.noi` grammar. The study measures *assisted interpretation while a hint is visible*, not durable learning.

Six pre-registered scenarios, in identical fixed order for both arms:

1. `missing_grant_console_run`: permission denial vs legacy plain run.
2. `undeclared_effect`: the CLI grant does not add the source `effect ConsoleWrite;`.
3. `misspelled_emits`: source syntax typo.
4. `invalid_option`: actual extensive usage help remains unmodified.
5. `missing_source_file`: path / working directory problem.
6. `legacy_plain_run`: old HALT-only route, not a `console-run` host denial.

All 10 R3-R6 cases are validated as the source inventory, but only these six are assessed in the pilot. Success, missing argument, unknown command, invalid UTF-8 are **not** scored and must not be included in any claim about those scenarios. No tasks execute code or request real permissions. Raw private R3-R6 evidence is never uploaded, committed, or included in CI artifacts.

## Pre-registration before seeing human outcomes

- **Pilot sample:** exactly 20 voluntarily consenting, previously unexposed NORDOI novices, 10 per arm. Recruitment should be independent of the software author; no names, emails, addresses, IPs or raw sources collected. This is a *feasibility pilot*, not a powered hypothesis test. If recruitment, consent or independence cannot be established by the researcher, report the gap and make no efficacy claim.
- **Assignment:** researcher runs `prepare` once with Python's `secrets.SystemRandom`, permuted blocks of size 4 (A,A,B,B), yielding 10 each, allocation stored only in a local private directory. Researcher should allocate consecutive slots by enrollment, not re-randomize to favor outcomes. Participants necessarily know when hints are shown; masking/blinding is impossible. The score function can only check self-reported fields, **not verify real human independence**.
- **Administration:** one person per assigned packet, same locale and instructions, six fixed scenario order, no code execution, no extra hints, no prompting corrections. Answers are closed choices for diagnosis and safe next action; keep the pre-existing stderr intact. Record duration in seconds for each scenario (external stopwatch) and do not collect identity data. Treat missing data as incomplete; do not fabricate.
- **Primary descriptive endpoint:** average per-participant proportion of six tasks where **both** the error class and the safe next action are correct. Compare means for A and B descriptively only; no p-values or “superior” verdict with this small sample.
- **Secondary endpoints:** median per-task time and count of dangerous “always grant” choices per arm. The research tool only reports aggregates; it cannot verify timers, consent or independence. Any increase in unsafe grant choices is a safety concern irrespective of apparent speed or comprehension gain.
- **Thresholds for later, separate studies:** prior to any prospective UX adoption, independently confirm an adequate sample, unchanged authority boundaries, no increase in unsafe permissions, and an appropriately powered transfer test with unseen fixtures and no help displayed. **R3-R8 cannot authorize adoption or production.**

## Safe operator workflow

1. Retrieve the exact pinned research script; verify its **Git blob hash** before use.
2. `python3 -B research/g03_r3_r8_novice_study.py selftest` — synthetic verification only, zero human subjects.
3. `python3 -B research/g03_r3_r8_novice_study.py prepare --report "$HOME/Downloads/NORDOI_G03_R3_R6_20261107T150311Z_90154.json" --out "$HOME/Downloads/NORDOI_G03_R3_R8_PRIVATE"` — creates a new private directory (must not already exist) with 20 preallocated Markdown packets, an allocation ledger, and a `responses_TEMPLATE.json`. No original repo modified.
4. Recruit genuine novices under the protocol; fill a **copy** of the local response template after actual sessions, one row per slot. The default template has null fields and **must not** score as human evidence.
5. `python3 -B research/g03_r3_r8_novice_study.py score --allocation "$HOME/Downloads/NORDOI_G03_R3_R8_PRIVATE/allocation.json" --responses "$HOME/Downloads/NORDOI_G03_R3_R8_PRIVATE/responses_ACTUAL.json"` — rejects missing and invalid data, prints **descriptive aggregates only**. Never upload response files.

The scoring tool is not a human-identity, participant-independence, or ethics verifier. Do not submit raw evidence, packets, response templates or private study reports to CI. Maintain separate written provenance of actual recruitment, consent and timing; do not claim independence without evidence.

## Verification and gating

- Python stdlib only; no process invocation of `nordoi`, no network I/O, no grant, no compiler modification.
- Synthetic `selftest` exercises data validation, safety gates and scoring; it asserts `HUMAN_TRIALS=0_IN_SELFTEST`.
- Actual owner Mac execution may prove that packet preparation works, **not** that human participants completed it.
- Study outcome state until independently completed: `HUMAN_COMPREHENSION=UNMEASURED`, `PEDAGOGICAL_SUPERIORITY=NOT_AUTHORIZED`, `PRODUCTION=NOT_AUTHORIZED`.
- NORDOI constitutional requirements are preserved: radically simple public language surface and extreme execution performance; R3-R8 adds no runtime overhead.
