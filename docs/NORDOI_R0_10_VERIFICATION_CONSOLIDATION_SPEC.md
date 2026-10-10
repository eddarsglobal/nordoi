# NORDOI R0.10 — Verification Consolidation Contract

**Frozen certified baseline:** `r0.9` / `860d09774c881cf23f2a1233877109ed49c21a01`. **Research status:** TEST-ONLY, candidate (NOT certified).

## Purpose

R0.3–R0.9 produced independent, bounded resource/task verification evidence but their witness matrices are maintained in separate versions. R0.10 checks **their provenance and declared limitations**, without quietly promoting these experimental models to a NORDOI scheduler, compiler, NAIR or production host-authority API. We consolidate rather than widen the state model.

## Frozen lineage and evidence

| Release | Witnesses | Domain |
| --- | ---: | --- |
| R0.3 | 9 | Finite state-space exploration, selected profiles |
| R0.4 | 12 | Deterministic adversarial generation |
| R0.5 | 14 | Resource/task negative traces and interleavings |
| R0.6 | 15 | Single-root independent conformance |
| R0.7 | 18 | Two-scope independent conformance |
| R0.8 | 26 | Three-generation hierarchy |
| R0.9 | 23 | Sibling branch and cousin boundaries |
| **Total** | **117** | Finite, version-scoped witness declarations |

Exactly **18 R0.10 Rust tests** are mapped bijectively by `governance/r10_consolidation_witnesses_v1.tsv`; each row has a limitation. The Rust suite embeds seven historical witness matrices and test-source files at compile time and tests declared witnesses against actual `#[test]` functions. It independently counts the profile arithmetic (3,125 / 7,776 / 16,807 finite words, 90 / 2,520 pair-order counts), tests selected boundaries against the frozen R0.2 pure model and exercises synthetic mismatch detection. The deterministic FNV-1a fingerprint is a non-cryptographic consistency signal **not an attestation**.

The separate Python validator verifies the R0.10 governance contract and, when the full repository and `r0.9` tag exist, also enforces a Git tree boundary: *only six R0.10 paths may be added* and no path from the certified `r0.9` tree may change. A packaging-only static pass cannot substitute for a tagged Git-boundary check or a Rust Release Gate.

### Nonclaims and stop conditions

- Not a fresh 117-case behavioral re-execution beyond what `cargo test --all-targets` runs; it checks integrity of the **witness declarations**, not proof strength.
- Not an independent global oracle for all possible programs or OS threads.
- Not a security proof, formal model check, certification of real concurrency, host capability enforcement or a promotion to kernel runtime.
- Never edit frozen R0.2 source or historical R0.3–R0.9 files to make the consolidation suite pass. Investigate any discrepancy as potential evidence drift, a fixture defect, incomplete specification, or a genuine model problem.
- No `src/`, `Cargo.toml`, `Cargo.lock`, historical test, kernel, NAIR, compiler, scheduler or runtime modifications are part of R0.10.

## Release conditions

1. Validate the R0.10 static matrix, verify the Git boundary against `r0.9` and rerun R0.2–R0.9 historical validators.
2. `cargo fmt --all -- --check`, `cargo test --test resource_task_r10 -- --nocapture`, `./scripts/release_gate.sh` must PASS.
3. Exactly six additive paths; then commit/push and verify five GitHub CI jobs succeed on the exact commit SHA.
4. Only then publish annotated tag `r0.10`.
