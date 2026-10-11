# NORDOI R0.20 — Trust Root Lifecycle & Revocation Boundary

## Frozen baseline

`r0.19` commit `834c20d68e86f0ed6ab14141e0f9f9e1df241901` (certified GitHub CI 5/5) and R0.19 synthetic input `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`.

## Purpose and scope

`TEST-ONLY / RESEARCH_ONLY`. This work is a **deterministic offline mock state machine** for trust-root registration, rotation, revocation, stale-epoch rejection, scope isolation and anti-replay. **NOT** a real trust service, verifier, signature implementation, production admission or native authorization. Every R0.11 native gate remains `UNMET` and every real external verifier remains `ABSENT`.

## Canonical inputs

* Governance registry: `governance/r20_trust_root_lifecycle_v1.tsv`, SHA-256 `1a296c9adb4b828fd504169a3579b0c8addbdbc7add1c8b0fafeedd6352f5e2e`. LF canonical bytes; Windows CRLF checkout allowed, bare CR forbidden.
* Evidence anchor: the **unchanged** R0.19 public research transcript SHA-256 `ad3768da8d3c92e0989c723a9e3f9b148b444794a3f4f7b6696631c4cab39c33`; no external attestation is inferred.
* Mock transcript: `research/fixtures/r20_trust_root_transitions_v1.bin`, exactly 5190 bytes, SHA-256 `2209b2da72b393c7c9ed783ece2adb6b1ee1f82fe7cf150cdc6f7e45bbf04817`.
* 12 native gate areas × 11 fixed scenarios = 132 scenario rows, in canonical gate-major order.
* Outcomes: `INVALID` = 12, `BLOCKED` = 84, `RESEARCH_REVIEW_ONLY` = 36. `PRODUCTION_ADMITTED` and `NATIVE_AUTHORIZED` have **zero** reachable transitions.

## State machine (per gate)

Initial state: `(epoch=0, root=NONE, revoked=NO)`. First `REGISTER_MOCK` registers `MOCK-ROOT-XX-A` at epoch 1, then `ROTATE_MOCK` establishes `MOCK-ROOT-XX-B` at epoch 2; both are **research-only** transitions, never real key validation. Fault cases (stale epoch, foreign scope, fabricated signature, fabricated verifier, falsified evidence root) cannot advance state. `REVOKE_MOCK` marks epoch 2 as revoked in the **mock ledger**; `REPLAY_REVOKED`, `ROLLBACK_EPOCH`, and `NATIVE_ESCALATION` remain blocked and cannot resurrect it.

## Binary transcript v1

The transcript is `NDR20TR1` (8 ASCII bytes) + little-endian u16 row count (132) + 132 ordered records, each `(ordinal:u16, scenario:u8, decision:u8, before_epoch:u8, after_epoch:u8, revoked:u8, sha256(canonical TSV record + LF):[32]u8)` + sha256 of all previous transcript bytes (32 bytes). All independent digests are recalculated by Python `hashlib` and the frozen Rust SHA256 implementation.

## Certification policy

No new kernel source, no permissions, no external calls. Additive-only eight files. Run Python static boundary, rustfmt, Cargo tests, Clippy `-D warnings`, full Release Gate, GitHub five-job CI on exact SHA; tag only after full success. Real native authority is **NOT** conferred; statuses remain `UNMET`/`ABSENT`.
