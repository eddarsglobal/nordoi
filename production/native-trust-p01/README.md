# Native Trust P0.1 — first cryptographic component

**CANDIDATE, NOT PRODUCTION-READY.** This is an independent Rust crate under the `production/p0.1` development branch. It does **not** change frozen `r0.23` files or integrate with the NORDOI runtime dispatch path.

Contains **actual strict Ed25519 verification** (ed25519-dalek), SHA-256 payload binding (sha2), a host-supplied multi-root allowlist, per-root epoch and in-memory sequence counters, cross-root signer isolation, and signed rotation/revocation with fail-closed errors. It tests authentic signatures, forgeries, replay, scope mismatch, payload tampering, rotation, permanent in-memory revocation, and unaffected neighboring roots.

**Critical gaps:** no independently authenticated/trusted bootstrap source, no atomically persisted root/sequence/revocation state, no kernel-native effect boundary integration, no crash-consistent replay protection, no external verifier, no independent crypto audit, no reproducible signed release artifacts. The `verify_execution` receipt is NOT authority to execute effects in production. Recreating this crate's `TrustGate` resets its counters and revocations, so using it as an effect authorization path across restarts would be unsafe.

Run from repo root:

```
cargo generate-lockfile --manifest-path production/native-trust-p01/Cargo.toml
cargo fmt --manifest-path production/native-trust-p01/Cargo.toml
./scripts/production_p01_gate.sh
./scripts/release_gate.sh
```

Commit the resulting `production/native-trust-p01/Cargo.lock` along with the eight included files after both gates pass. GitHub Actions workflow `production-p01.yml` supplies five specialized jobs, distinct from the existing main-branch CI gate.

The Ed25519 signable wire domain is `NORDOI:P0.1:ADMISSION:V1\0`, all integer fields use big endian, and execution requests commit to logical capability, logical resource, and SHA-256 of payload. NO secret keys are stored in the crate. Test secret keys are deterministic and never suitable for deployment.
