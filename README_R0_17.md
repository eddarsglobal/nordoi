# NORDOI R0.17 — Reproducible Evidence Verification & Independent Replay

**CANDIDATE · TEST-ONLY · RESEARCH_ONLY · strictly additive**

Certified input baseline: `r0.16` / `a5d7e9669a9b6d4be6a47bed17756946df24c9bb`. R0.11–R0.16 remain byte-for-byte frozen.

This bounded experiment replays the **559-byte public R0.16 fixture** through two deliberately different local readers: (A) cursor-based binary traversal, and (B) manifest-offset-based direct indexing. Python `hashlib` generates and validates a **595-byte canonical replay transcript**; Rust recomputes the same transcript with the existing frozen SHA-256 implementation and compares exact bytes and anchored SHA-256. Thus an independent implementation language checks reproducibility, **NOT independent external authorship or trust**.

- Frozen R0.16 root SHA-256: `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`.
- Canonical R0.17 transcript SHA-256: `1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab`.
- 12/12 bounded records, 12/12 `ABSENT` external verification statuses, 12/12 native gates `UNMET`.
- Python static checks + 44 new Rust integration tests / 44 corresponding governance witnesses, with two inherited FIPS SHA-256 tests (46 expected).
- The only favorable assessment is `ResearchReviewOnly` with synthetic local CI and review flags; never approval for production/native installation/execution.
- No signature verification, external reviewers, identity authentication, network access, scheduler, host path interpretation, or trust-root import.

## Exactly eight new files

`README_R0_17.md`, `docs/NORDOI_R0_17_INDEPENDENT_REPLAY_SPEC.md`, `governance/r17_replay_registry_v1.tsv`, `governance/r17_replay_witnesses_v1.tsv`, `research/R0_17_REPLAY_BOUNDARY_DECISION.md`, `research/fixtures/r17_independent_replay_transcript_v1.bin`, `scripts/validate_r17_replay.py`, `tests/resource_task_r17.rs`.

## Mac validation

```bash
python3 scripts/validate_r17_replay.py
rustfmt --edition 2021 tests/resource_task_r17.rs
cargo fmt --all -- --check
cargo test --test resource_task_r17 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.16 -- .
git status --short
```

Expected: static contract `PASS`, Rust `46 passed`, `NORDOI release gate: PASS`. Rust and CI are **NOT RUN here**, so do not commit/tag before obtaining them locally and on five GitHub jobs for the exact commit SHA.
