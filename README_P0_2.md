# NORDOI Production P0.2 — Durable Trust Journal (CANDIDATE)

Base branch: `production/p0.1`
Base commit: `7f6855fa310cb936469c89e1141fa7204bec5f7a`
Verified upstream P0.1 CI: `38150634669` (5/5).

**This is a candidate patch, not a compiled or certified build.** Rust compiler, Clippy, Mac/Windows tests and GitHub CI have not been run by the package author. No changes to `main` or certified `r0.23` are intended.

## Patch contents

- New `production/native-trust-p01/src/durable.rs`: signed, bounded, hash-chained journal, explicit genesis, exclusive lock, fsync-before-memory-publication, full Ed25519 verification on reboot, denial on corruption.
- Updated `production/native-trust-p01/src/lib.rs`: one additional `pub mod durable;` export. Local `cargo fmt` will preserve the existing Rustfmt style.
- New `production/native-trust-p01/tests/durable.rs`: 10 tests for persistence, replay, rotation, revocation, tampering, bootstrap mismatch, concurrency and invalid signatures.
- New independent `.github/workflows/production-p02.yml` CI: 5 jobs. Existing P0.1 CI stays active.
- Dedicated test gate and security limitations in `docs/PRODUCTION_P0_2_DURABLE_TRUST_DECISION.md`.

## Important limitations

The journal is **not rollback-resistant** to replacement of the *entire* file with an earlier valid copy. No hardware monotonic counter / independent external anchor, no native runtime dispatch integration, no external audit. A stale lock after crash requires manual investigation. This is a protected-root local research candidate, not deployment authorization.

`PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED`

## Local validation

Run only after verifying `HEAD == 7f6855fa310cb936469c89e1141fa7204bec5f7a`, branch `production/p0.1` and clean Git tree. Copy package paths to repository root, then:

```bash
cargo fmt --manifest-path production/native-trust-p01/Cargo.toml
bash scripts/production_p02_gate.sh
bash scripts/production_p01_gate.sh
bash scripts/release_gate.sh
git diff --check
git status --short -uall
```

Do not commit/tag or claim P0.2 certification on test failure. Fix only production-track files. P0.2 CI runs along with P0.1 workflow on next push to `production/p0.1`; both must pass for the P0.2 milestone.
