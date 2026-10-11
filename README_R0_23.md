# NORDOI R0.23 — Evidence Chain Finality & Equivocation Detection

Status: **CANDIDATE**, not a production release. Baseline certified `r0.22` / `9d0bbdbb5153e5d38ff0f387ee381ad806c790d9`.

Strictly **TEST-ONLY / RESEARCH_ONLY**: 12/12 native readiness gates **UNMET**, 12/12 independent external verifiers **ABSENT**. No actual cryptographic signature, verifier, trust root, audit attestation, native admission, or production authority is exercised by this research fixture.

This additive campaign has 216 cases in 12 groups (18 per group): **12 INVALID, 96 BLOCKED, 108 RESEARCH_REVIEW_ONLY**. It tests monotonic finalized-height commitments, synthetic equivocation attempts, replay rejection, separated lineages, preserving a sealed A head when A is revoked, and legitimate continued advancement of B. A finality checkpoint is a *simulated irreversible historical commitment*, not a consensus finality guarantee. A descendant of a finalized checkpoint is allowed; replacement is not.

Canonical TSV SHA-256: `6da4eb212e7a8e8ed1c297808ad6c7beaae14e6f2cb665b12c0d5d9b6c56dba8`. Golden transcript SHA-256: `26fd97b314f458458b4f89cbdde5b55d41403c5dc5ea89c061a057b5ecf494b7` (9,598 bytes). Certified R0.22 transcript input SHA-256: `bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da`.

Run `python3 scripts/validate_r23_finality.py`, `cargo test --test resource_task_r23 -- --nocapture`, `./scripts/release_gate.sh`. Do not commit or tag until these pass locally and all five platform CI jobs pass on the exact commit.

See `docs/NORDOI_R0_23_FINALITY_SPEC.md` for the operational definition and **production exit criteria**. The production claim is explicitly prohibited by this phase.
