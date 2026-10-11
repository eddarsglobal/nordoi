# NORDOI R0.22 — Witness Fork & Monotonic Replay Isolation (CANDIDATE)

**TEST-ONLY / RESEARCH_ONLY — no keys, no real signature verification, no production admission, no native authority.**

Certified baseline: `r0.21` / `a34bc552b31661570f270c6c75700849b261084c`. Eight additive files only; historical sources/tests/registries cannot be changed.

## Coverage

- 180 deterministic cases: 12 domains × 15 ordered witness/chain events.
- 12 INVALID, 84 BLOCKED, 84 RESEARCH_REVIEW_ONLY; **research-only is not an authorization**.
- 48 independent Rust witness tests + 2 inherited certified SHA-256 tests = **50 expected**.
- Two independent mock lineages A and B, synthetic `height`, `epoch`, `root`, parent-head, SHA-256 witness.
- Equal-height alternate forks, stale parents, cross-root claims, duplicate replay, resurrected revocation, forged verifier/signature, altered parent evidence and native escalation fail closed.
- Python and Rust reconstruct a canonical frozen **7654-byte** binary transcript.

Canonical registry SHA-256: `31a2f2ef5fbadf9d3a026e570db45e9da0c0d990ffafb52a3577bc267b14155f`
Golden binary SHA-256: `bfb3dd4b39d4bd6a0d756e8d0dff9b6f25bd94705e2e3883aa90f5863233d1da`
Certified R0.21 binary SHA-256: `cc08cac8ec0f196563ded1bebbd7c07633c47815a7a0eb3db49064079018c2ba`

All historical native gates: **12/12 UNMET**. All real external verifiers: **12/12 ABSENT**.

## Run on the certifying developer's Mac

```sh
python3 scripts/validate_r22_witness_forks.py
rustfmt --edition 2021 tests/resource_task_r22.rs
cargo fmt --all -- --check
cargo test --test resource_task_r22 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.21 -- .
git status --short
```

**Do not commit or tag** until Rust, Release Gate and later 5/5 GitHub CI succeed.

This does NOT implement a real trust store or external verifier. Native readiness and real external verification are NOT changed by R0.22. The research witness hashes have no cryptographic signing authority.
