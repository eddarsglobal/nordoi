# NORDOI R0.16 — Offline Evidence Byte Bundle Boundary

**CANDIDATE · TEST-ONLY · RESEARCH_ONLY · additive only**

Certified input baseline: `r0.15`, Git commit `3158ee20a894461df5402335b7db5509db9abd02`. This is a research fixture, **NOT** production evidence, identity authentication, or native execution authorization.

## Why this version is substantively different

R0.11–R0.15 use research-only registers and synthetic reviewer policies. R0.16 is the first bounded experiment here to check **actual binary byte slices** from a fixture checked into the research tree, including non-UTF-8, NUL and CRLF. A fixed catalog records offsets, lengths and SHA-256 digests. Tests intentionally corrupt real bytes and metadata and exercise a fail-closed decoder. No host path is followed, no external I/O happens during tests, and no production subsystem is changed.

- 12/12 native readiness requirements remain `UNMET`; R0.12–R0.15 external evidence and authentication remain `ABSENT`.
- 12 R0.16 mock binary records, every admitted status `ABSENT`, and no external trust root.
- Fixture SHA-256: `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`. This is a public test vector, **NOT** proof of who authored it.
- 44 new integration tests, plus two SHA-256 FIPS unit tests from the frozen implementation (46 expected); 44 governance witnesses.
- `ResearchReviewOnly` is the maximum success and requires local synthetic CI + synthetic review flags.
- No signature verification, no outside evaluator, no real provenance, no authorization to install or run native task code.

## Additive paths (8)

`README_R0_16.md`, `docs/NORDOI_R0_16_OFFLINE_BUNDLE_SPEC.md`, `governance/r16_offline_bundle_manifest_v1.tsv`, `governance/r16_bundle_witnesses_v1.tsv`, `research/R0_16_OFFLINE_BYTE_BOUNDARY_DECISION.md`, `research/fixtures/r16_synthetic_bundle_v1.bin`, `scripts/validate_r16_bundle.py`, `tests/resource_task_r16.rs`.

## Mac validation

```bash
python3 scripts/validate_r16_bundle.py
rustfmt --edition 2021 tests/resource_task_r16.rs
cargo fmt --all -- --check
cargo test --test resource_task_r16 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.15 -- .
git status --short
```

Expected: `R0.16 static contract: PASS`; `46 passed; 0 failed`; `NORDOI release gate: PASS`. These Rust results **have not been executed here**. Do NOT commit or tag the candidate before local checks and five green GitHub jobs on the exact R0.16 SHA.

Historical R0.10–R0.15 validators have tag-scoped Git clean-tree rules; if needed, run them in worktrees at their certified tags rather than amid R0.16 additions.
