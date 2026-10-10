# NORDOI R0.12 — Evidence Admission & Provenance Boundary (TEST-ONLY)

**Certified prerequisite:** `r0.11` / `d61487bd2392890b3aaaa6ef33fce4371cfb6850`. This specification is additive research governance, NOT a change in scheduler, NAIR, compiler, kernel APIs, capabilities, or production authorization.

## Research question
Can the laboratory define an unambiguous, fail-closed intake schema for *proposed research evidence* without treating proposals, metadata locators, a green CI or mock reviewer approvals as native readiness proofs?

## Frozen invariants
1. Read-only R0.11 registry: 12 native requirements, 12 `UNMET`, all `RESEARCH_ONLY`.
2. `governance/r12_evidence_ledger_v1.tsv` maps exactly one `R12-E01..E12` to `R11-G01..G12` in canonical order and exact area, pinned to the full R0.11 commit SHA.
3. Every canonical ledger row is `ABSENT`, has `artifact_ref=NONE`, `reviewer_ref=NONE`, and explicitly documents what is missing; there is no native proof, authenticated reviewer or artifact digest in this milestone.
4. In isolated tests, `PROPOSED` rows are accepted only with `research://synthetic/` locators, unique per gate, and synthetic reviewer references. This is a **parsing and admission policy simulation**, not the verification of an artifact, identity, signature or digest.
5. Every malformed, duplicate, reordered, cross-gate, stale-SHA, unsupported-status, or foreign-scope record fails closed. Missing proposal data blocks review.
6. The strongest synthetic advisory is `ResearchReviewOnly` and it requires all proposals plus mock CI/mock approval flags; this never means `NATIVE_VERIFIED`, readiness or permission to deploy.
7. The frozen R0.2 reference model is used only inside isolated tests. The production source and toolchain are unchanged.

## Evidence limitations
The fixture is bounded to twelve declared areas and 29 Rust tests; it does not implement certificate chains, signatures, trusted timestamps, artifact retrieval, content hashing, native concurrency, executable host-boundary controls, independent security review, or production approval. Test-only status is mandatory. A GitHub CI success validates only the existing test suite, not the readiness requirements.

## Certification discipline
A R0.12 package must only contain seven additive paths. Run the R0.12 static validator, rustfmt, focused Rust tests, and complete release gate on macOS. Since the historical R0.11 validator legitimately rejects future untracked files, run it separately on an isolated `r0.11` worktree, never modify it. Require exact-SHA five-job GitHub CI before making annotated tag `r0.12`.
