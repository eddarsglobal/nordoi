# NORDOI R0.20 — Trust Root Lifecycle & Revocation Boundary

**CANDIDATE — NOT CERTIFIED.** Base: annotated tag `r0.19`, `834c20d68e86f0ed6ab14141e0f9f9e1df241901`.

Eight additive, test/research/governance-only files. Read [spec](docs/NORDOI_R0_20_TRUST_ROOT_LIFECYCLE_SPEC.md). Deterministic, auditable simulation of a 12-gate × 11-scenario root register/rotate/revoke lifecycle; 132 synthetic cases. R0.19 evidence anchor `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`.

Registry root: `1a296c9adb4b828fd504169a3579b0c8addbdbc7add1c8b0fafeedd6352f5e2e`. Golden transcript root: `2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817` (5190 bytes). Outcomes 12 INVALID / 84 BLOCKED / 36 RESEARCH_REVIEW_ONLY. No real external identity signature, independent verifier, or native authority. `UNMET` 12/12, `ABSENT` 12/12. Strict `TEST-ONLY / RESEARCH_ONLY`.

## Gate commands

```bash
python3 scripts/validate_r20_trust_roots.py
rustfmt --edition 2021 tests/resource_task_r20.rs
cargo fmt --all -- --check
cargo test --test resource_task_r20 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.19 -- .
git status --short
```

Commit only after local PASS. Require exact-commit five-job GitHub CI PASS before annotated tag `r0.20`. No activation, production admission, or native readiness is implied.
