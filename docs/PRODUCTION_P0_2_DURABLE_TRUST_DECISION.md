# NORDOI Production Track P0.2 — Signed durable trust journal candidate

Baseline: `production/p0.1` @ `7f6855fa310cb936469c89e1141fa7204bec5f7a` (CI 38150634669; 5/5). This **production-track branch** is not a certified or deployed release. Historical `r0.23` and `main` are immutable reference points.

## What this implements

- Explicit journal initialization. A missing journal is **not** implicitly reinitialized on startup.
- An exclusive `create_new` lock file prevents two cooperative processes from writing the same journal. A lock remaining after an unclean shutdown must be investigated and cleared manually.
- Fixed-domain bootstrap fingerprint binds trusted initial Ed25519 root identities and public keys.
- Each event is a bounded canonical record of the **signed P0.1 envelope**, including the signed payload SHA-256 (never the raw execution payload).
- On recovery, all journal event hashes and **all historical Ed25519 signatures** are reverified from the original trusted public keys. Counters, key rotations, and revocations are reconstructed in canonical order.
- New entries are `write_all` + `File::sync_all` before replacing the in-memory candidate state or returning a receipt. I/O failure poisons the instance; partial and corrupted records block recovery.
- Log capped at 16 MiB for this prototype. On Unix, newly created log/lock files request mode `0600`; operators must protect their parent directory.
- Existing P0.1 tests remain required; 10 additional P0.2 integration tests target restart, tampering, malformed tail, bad signatures and lock contention.

## Security limitations — **production deployment is NOT authorized**

1. **Rollback of an entire valid journal is not detectable locally**. An attacker capable of replacing the entire journal with a valid older copy can bypass local sequence/revocation history. A hardware-bound monotonic counter, append-only external trusted witness or equivalent protected anchor is required before production.
2. The journal is neither externally authenticated nor automatically bound to a trusted system clock or independent verifier. The Ed25519 signatures authenticate events, but cannot prove no signed events have been removed by a filesystem attacker.
3. File-system privileges, durability semantics, power-loss behavior and symlink/race defenses require independent cross-platform review. A stale lock after a crash is deliberately fail-closed and requires manual operator recovery.
4. The public **P0.1 in-memory gate still exists**; callers can bypass the P0.2 wrapper. A future native runtime gate must make the durable authorization boundary mandatory.
5. P0.2 does not bridge to transactional effect dispatch, nor does it deliver exactly-once external effects; a verified receipt remains **non-authorizing** for external dispatch.
6. No external security audit, production key custody, backup/restore anti-rollback or hardware trusted root integration has been completed.
7. This candidate may require ordinary compiler fixes after the actual Mac Release Gate; it is NOT declared compiled until that evidence exists.

`PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED`

## Road to production

P0.2: persistent verified journal -> P0.3: crash-atomic native dispatch integration -> P0.4: independent rollback-resistant witness & key custody -> P0.5: security testing/audit, reproducible signed artifacts and controlled production pilot. These are engineering milestones, not release promises.
