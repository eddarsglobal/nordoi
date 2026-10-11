# NORDOI R0.21 — Trust Epoch Replay & Cross-Root Conflict Isolation

**CANDIDATE — NOT CERTIFIED. TEST-ONLY / RESEARCH_ONLY.** Exact frozen baseline: annotated tag `r0.20` / commit `bd54ef97523fc9f415ce0d6eaa41830b7105b7bf`.

Eight **additive** test/governance/research/documentation files, no frozen source modifications. Two **simulated** trust-root lanes A and B in each of the 12 historical gate domains, 13 ordered adversarial scenarios per domain = **156** cases. Results: **12 INVALID / 72 BLOCKED / 72 RESEARCH_REVIEW_ONLY**. The mock A lineage is irrevocably revoked; an independent B lineage is allowed to rotate afterward *only inside the research model*. Attempted cross-root writes, conflicting epochs, resurrection, stale replay, forged provenance, or native escalation cannot gain authority.

Pinned upstream R0.20 transcript root: `2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817` (5190 bytes).

Canonical R0.21 registry SHA-256: `179b9e332d9562d90df7bdaec8c46297b9a467cfd04b728c47750b0aec7a9a57`.

R0.21 binary golden transcript SHA-256: `cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba` (**6646 bytes**).

48 explicit Rust witnesses + 2 inherited SHA-256 vectors = **50 tests expected** (Rust and Clippy were **NOT run** in the packaging environment). Historical native gate statuses: **12/12 UNMET**. Real independent external verifiers: **12/12 ABSENT**. Zero real signatures, public-key trust, verified external identity, native execution permission, or production admission. `RESEARCH_REVIEW_ONLY` is NOT an authorization.

## macOS local validation

```bash
python3 scripts/validate_r21_epochs.py
rustfmt --edition 2021 tests/resource_task_r21.rs
cargo fmt --all -- --check
cargo test --test resource_task_r21 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.20 -- .
git status --short
```

Only after full local PASS, commit exactly the eight R0.21 additive paths and push `main`. Require five green jobs for the exact push commit (Check, Format & Clippy, Tests on ubuntu-latest, macos-latest and windows-latest). Publish annotated `r0.21` only then. Do NOT tag this candidate or confuse synthetic status with readiness.
