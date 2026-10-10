# R0.15 — Independent Evidence Verification & Trust Policy Specification

**Status:** TEST-ONLY / RESEARCH_ONLY. **Baseline:** `r0.14` `86fc90a79f5d0cef83c730bcd7c02175307fffd3`.

## Non-negotiable boundaries

1. Preserve the certified Git tree at r0.14 (no changes to source, Cargo manifests, CI, historical tests, existing governance, or docs).
2. Preserve all twelve R0.11 `UNMET` production-readiness gates and all R0.12, R0.13 and R0.14 `ABSENT` records.
3. Register all twelve R0.15 external verification policies as `ABSENT` with `verifier_ref=NONE` and `trust_root=NONE`.
4. No native runtime/NAIR/compiler authority may be exported by R0.15.
5. A digest is **not** a signature, an identity, a scientific result, or a permission.
6. Two synthetic reviewer labels are **not** two authenticated independent people.

## Synthetic trust-policy decision semantics

- `Invalid`: missing/malformed/unexpected fields, hash mismatch, duplicate artifacts, forged baseline, policy drift, reviewer role collision, claimed authenticating signature or trust root, stale epoch.
- `Blocked`: no fixture, unanswered/negative review, revoked record, missing synthetic CI or missing explicit synthetic approval.
- `ResearchReviewOnly`: twelve well-formed non-revoked synthetic proposals, both distinct synthetic reviewers marked approved, mock CI=true and mock approval=true.
- **No `ProductionAuthorized` state exists.** Even `ResearchReviewOnly` is not a claim of independent verification.

## Genuine independent verification gaps

A future *separate* authorization would require verified producer credentials and trust roots; cryptographically authenticated signatures over bound artifacts; independent retrieval from appropriate sources; semantic and adversarial validation by real authorized reviewers; evidenced conflict handling and revocation; and explicit human/governance authorization. This R0.15 release delivers none of those: `ABSENT`, `UNMET`, `RESEARCH_ONLY`.

## Verification and bounded scope

39 defined synthetic research tests plus two frozen SHA-256 FIPS reference tests = 41 Rust test functions expected in `tests/resource_task_r15.rs`. Work is bounded to exactly 12 gates, with finite in-memory fixtures, no network, no asynchronous activity, no unsafe code or host authority.
