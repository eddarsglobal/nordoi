# NORDOI R0.19 — Evidence Admission Gate Hardening

**CANDIDATE · TEST-ONLY · RESEARCH_ONLY · strictly additive**

Certified baseline: **r0.18 / `2839c377e78e3c0e58693bc5aa8e668d8e0e436f`**. This package changes none of the previous certified files. Native readiness **12/12 UNMET** and real external evidence **12/12 ABSENT** remain frozen.

This phase **separates local byte integrity from actual evidence admission**. Exactly **96 public, synthetic admission attempts** are mapped to 12 historical gates and 8 scenarios each. The bounded decision procedure can only output `INVALID`, `BLOCKED`, or `RESEARCH_REVIEW_ONLY`, never production/native admission.

- 12 valid-format mock review cases: `RESEARCH_REVIEW_ONLY`, not accepted evidence.
- 12 tampered artifact root cases: `INVALID`.
- 72 forged, missing, revoked or escalation cases: `BLOCKED`.
- The policy registry's SHA-256 is `7e7f3eccc8e06f6acc0bbb811ddd82b7813654b2b7a676764a5341a81643a3e2`.
- The byte-stable **3,498-byte** decision transcript SHA-256 is `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`.
- Frozen input SHA-256 roots: R0.16 pack `f3b3dcf7488ec5b4252b95550f9f8327acf87c5721104b2a75e30fc4a5ae1393`; R0.17 replay `1a8bb469182af60317634c0f1bf7bcee72b7d6ebcfb048ae73a8b5d720b09aab`; R0.18 campaign `9512d8f24a84bccdbd87ade72ea8c8ab18f9873864a859a11a3abd5612f86293`.
- 32 policy Rust tests/witnesses + 2 inherited SHA vector tests = **34 Rust tests expected**.

Exactly eight additive files, no native APIs, no real signature validation, no real independent reviewers, no legal admission decision. This gate does **NOT** certify external evidence or grant native authority.

## Local validation

```bash
python3 scripts/validate_r19_admission.py
rustfmt --edition 2021 tests/resource_task_r19.rs
cargo fmt --all -- --check
cargo test --test resource_task_r19 -- --nocapture
./scripts/release_gate.sh
git diff --exit-code r0.18 -- .
git status --short
```

Expected: Python static PASS, 34/34 tests PASS, Release Gate PASS. Rust compilation and CI are **NOT RUN by the Python validator**. Do not commit or tag before local checks and CI 5/5 on the exact SHA.
