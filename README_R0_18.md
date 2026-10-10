# NORDOI R0.18 — Adversarial Evidence Replay & Differential Fault Injection

**CANDIDATE · TEST-ONLY · RESEARCH_ONLY · strictly additive**

Certified baseline: `r0.17` / `c5f76bc2e393f95f38552778f870fac4be8206c3` (includes FIXED2 Windows LF/CRLF compatibility). **No certified files are modified**. Native readiness: **12/12 UNMET**, real external verification **12/12 ABSENT**.

This experiment supplies **60 fully deterministic, bounded synthetic fault-injection cases** against the existing public R0.16 binary fixture, R0.17 transcript and the canonical R0.16/R0.17 TSV records. Two independently implemented local decoders classify whether the mutated packet remains structurally parseable; a pinned SHA-256 trust boundary rejects **every** mutation even if both decoders can still parse it.

- Immutable input SHA-256 (pack): `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`.
- Immutable input SHA-256 (transcript): `1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab`.
- Normalized LF manifest SHA-256: `f4dfbb95a61e1a9184ca459f7279a0c686166de10411554c8364445cb75dcb24`.
- Normalized LF registry SHA-256: `c0d649990dbeb318408a8e82af34478fc7cf1f50834e8f69a05d0464d1e2e242`.
- Campaign SHA-256: `281ea03f893b54e2c03d12e7df1236da249ff1d8a61cbe0a8f5b4fb18d26b783`.
- Canonical **2,307-byte** result transcript SHA-256: `9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293`.
- 60 fault cases + 38 Rust governance tests / witnesses + 2 inherited FIPS SHA-256 tests = **40 expected tests**.

**Exactly eight additive paths**:
`README_R0_18.md`, `docs/NORDOI_R0_18_DIFFERENTIAL_FAULT_SPEC.md`, `governance/r18_fault_campaign_v1.tsv`, `governance/r18_fault_witnesses_v1.tsv`, `research/R0_18_ADVERSARIAL_BOUNDARY_DECISION.md`, `research/fixtures/r18_differential_fault_results_v1.bin`, `scripts/validate_r18_faults.py`, `tests/resource_task_r18.rs`.

## Local macOS validation

```bash
python3 scripts/validate_r18_faults.py
rustfmt --edition 2021 tests/resource_task_r18.rs
cargo fmt --all -- --check
cargo test --test resource_task_r18 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.17 -- .
git status --short
```

Expected: static contract `PASS`, Rust **40 passed**, Release Gate `PASS`. Rust execution/CI are **NOT RUN by the static validator**. No commit or tag until local success and five GitHub jobs on the exact SHA.
