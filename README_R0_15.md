# NORDOI R0.15 — Independent Evidence Verification & Trust Policy

**CANDIDATE · TEST-ONLY · RESEARCH_ONLY · additive only**

Certified input baseline: `r0.14`, Git commit `86fc90a79f5d0cef83c730bcd7c02175307fffd3`.

## Scope

This is an additive, untrusted research policy exercise, **NOT** a production release, a cryptographic identity verification, or native execution authorization.

- 12/12 native readiness gates remain `UNMET` (R0.11).
- 12/12 R0.12 evidence entries remain `ABSENT`.
- 12/12 R0.13 integrity entries remain `ABSENT`.
- 12/12 R0.14 real authentication entries remain `ABSENT`.
- 12/12 R0.15 external verification entries are `ABSENT`.
- 39 new Rust policy tests and 2 inherited FIPS SHA-256 reference tests (41 expected).
- The 39 policy tests have 39 governance witnesses, in exactly the same order.
- No external verifier, signer, root of trust, or production evidence is admitted.

## Research question

Can NORDOI fail closed when the same data passes basic SHA-256 consistency checks but independent research-review POLICY requirements are not met? The tests exercise two **synthetic reviewer labels**, not authenticated people. Both must approve and their role identities must remain distinct; rejection, pending state, revocation, forgery, stale policy epoch, missing CI or missing approval prevent even research review.

A complete synthetic fixture yields `ResearchReviewOnly`, **NOT** production approval. Matching hashes and two untrusted textual reviewer labels do NOT prove artifact provenance, reviewer independence, factual validity, or security.

## Files (7 new paths)

- `README_R0_15.md`
- `docs/NORDOI_R0_15_INDEPENDENT_VERIFICATION_SPEC.md`
- `governance/r15_trust_policy_registry_v1.tsv`
- `governance/r15_verification_witnesses_v1.tsv`
- `research/R0_15_VERIFICATION_POLICY_DECISION.md`
- `scripts/validate_r15_verification.py`
- `tests/resource_task_r15.rs`

## Local check

```bash
python3 scripts/validate_r15_verification.py
rustfmt --edition 2021 tests/resource_task_r15.rs
cargo fmt --all -- --check
cargo test --test resource_task_r15 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.14 -- .
git status --short
```

Historical static validators r0.10–r0.14 enforce their own untracked-file boundaries and must be invoked only in isolated worktrees at their certified tags.

Do not commit or tag R0.15 until local 41/41 Rust tests, full release gate and five GitHub CI jobs on the exact commit SHA succeed.
