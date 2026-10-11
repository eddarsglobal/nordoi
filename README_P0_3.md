# NORDOI Production P0.3 — Guarded kernel EffectBackend adapter (CANDIDATE)

Base: production/p0.1 @ `e0068bc58442d99a7fed7f43cb00549e29530acf` (P0.2 FIXED1, both CI 5/5). Research `r0.23` unchanged.

Patch adds a standalone Rust crate depending on the NORDOI kernel and the P0.2 durable trust crate. It implements the **real** `nordoi_kernel::EffectBackend` trait and requires durable Ed25519 authorization with exact delivery key + fencing token before calling an owned underlying backend.

**Scope warning:** this is an opt-in adapter, not a mandatory kernel-wide security boundary; failure-atomic/destination-idempotent execution and external anti-rollback witness remain unsolved. Treat this code as a test candidate, not a deployable production enforcement system.

No existing files are changed; patch includes 8 new files. Generate the 9th file (`production/native-effect-p03/Cargo.lock`) with `cargo generate-lockfile --manifest-path production/native-effect-p03/Cargo.toml`. Execute `cargo fmt --manifest-path production/native-effect-p03/Cargo.toml`, then `bash scripts/production_p03_gate.sh`, `bash scripts/production_p02_gate.sh`, `bash scripts/production_p01_gate.sh`, `bash scripts/release_gate.sh`. Do not commit or tag until local validation passes. CI checks all three OSes and runs on production/p0.1.

**PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED**
