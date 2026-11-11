# NORDOI G0.3 R5I — Actual signed-revocation checkpoint and joint rollback boundary

**RESEARCH ONLY. No independent guardian, owner OOB root, external protected monotonic floor, or native admission.**

## Frozen baseline

R5I is an unmerged DRAFT child of R5H PR #99 exact commit `43192354c023cf7f952f0d09a6f7c68e97a7f03f`. The owner completed R5H Mac Intel macOS 14.5 replay: **487 regular tests PASS**, 3 deliberately ignored Rust, 73/73 exact Git blob SHA pins, Rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`, and validated real temporary Ed25519 OpenSSH SSHSIG signing and local SQLite rollback of the signed revocation. Dedicated R5H four-OS GitHub CI 4/4 SUCCESS and general NORDOI Test Gate 5/5 SUCCESS.

R5H established that valid signing-key possession and locally durable `REVOKE` are **not** a permanent global revocation guarantee: the same administrator can restore a pre-revocation SQLite snapshot and erase the revocation latch.

## R5I: new actual revocation floor

R5I introduces a **second, separately persisted real SQLite database** `r5i_local_revocation_floor.sqlite3` in a **different private sibling directory**. It has its own transactional `BEGIN IMMEDIATE` commit path, `synchronous=FULL`, `journal_mode=DELETE`, sequential height/epoch, unique nonce, previous primary head, primary R5H signed-revocation event digest, and a distinct SHA-256-linked checkpoint chain.

It reads and verifies the **actual existing R5H signed-revocation SQLite database** `r5h_local_revocations.sqlite3` (not an invented or synthetic approval record) through the unmodified R5H implementation. Real R5H `REVOKE` records must first be signed with a **genuine temporary OpenSSH Ed25519 test key**, validated by R5H, and committed to the primary ledger. Only then may the R5I laboratory record the next checkpoint. Every operation returns `admission=NOT_AUTHORIZED`; the CLI always exits **2**, even for `LAB_LOCAL_CHECKPOINT_COMMIT`.

The local floor can demonstrate a useful narrower property **only while the floor snapshot is preserved**:

1. **Signed REVOKE committed at height 1**, then checkpointed to a real separate SQLite floor at height 1; both ledgers match.
2. **Primary-only rollback:** restore a pre-revocation R5H database copy while preserving the floor at height 1. The R5I audit explicitly returns `SIGNED_REVOCATION_LOSS_DETECTED_WITH_PRESERVED_LOCAL_FLOOR`, **BLOCK**.
3. **Same-height signed-revocation fork:** restore genesis primary then sign a different `REVOKE` with a different nonce under the same temporary key; floor remains pinned to the original height-1 event hash. R5I reports `REVOCATION_FLOOR_PRIMARY_FORK_OR_EQUIVOCATION`, **BLOCK**.
4. **Both snapshots restored:** restore **both** pre-revocation primary and pre-checkpoint floor files. Two internally consistent height-0 databases result; the audit returns `LOCAL_REVOCATION_FLOOR_MATCH_NOT_INDEPENDENT` and **cannot detect the lost height-1 history**. **This is the decisive real counterexample** to claiming global antirollback from separately named databases under shared administration.
5. **Floor-only rollback:** preserve the signed height-1 primary, but restore a height-0 floor. The discrepancy returns `PRIMARY_REVOCATION_AHEAD_OF_UNCHECKPOINTED_LOCAL_FLOOR`, **BLOCK**. It is not external custody.
6. **Separate real process:** the CLI reads the actual two-file checkpoint across a Python subprocess, and a checkpoint commit still exits **2** without any authority grant.

This is a real on-disk *same-admin* two-store experiment. It is **not** an operational remote observer, an HSM, an independently held revocation journal, proof against host root, power-loss evaluation, or an atomic multi-database transaction. Time-of-check-to-time-of-use conditions and concurrent administrator-controlled filesystem replacement are not solved by this lab.

## 24 tests, no invented guardian

**16 portable real-SQLite tests:** actual genesis, physically distinct files, refused pre-revocation checkpoint, real CLI exit 2, identical and nested paths, absent paths, absent primary database, corrupted floor, mutated/absent state metadata, forged floor checkpoint chain, non-mutating audit, invalid operation and relative paths.

**8 native Unix/OpenSSH tests** (actual ephemeral Ed25519 SSHSIG key possession in a private temporary laboratory directory): real signed revocation checkpoint, duplicate local checkpoint refusal, reopen across real process boundary, primary-only rollback detection with preserved floor, signed same-height fork detection, **joint old snapshot restoration invisibility**, floor-only rollback detection, and actual native checkpoint CLI exit 2. Windows intentionally skips these eight platform-specific controls; its portable 16 run on actual SQLite.

No source in R4A–R5H is modified. The entire R5I source is additive, remains in `research/`, and does not touch native NORDOI compiler/kernel/runtime, public `.noi`, main, old tests, existing workflows, releases or production.

## Real independent-custody admission requirements still absent

- G1: genuinely separate guardian operator with independently evidenced administrator/root, backup, CI, recovery and access control distinct from publisher.
- G2: owner-authenticated OOB fingerprint/pin for actual externally held guardian signing root, not a fixture digest.
- G3: external protected monotonic revocation floor, append-only audit and durable global consumed-nonce history outside publisher's unilateral rewrite/restore powers.
- G4: deployed verifier with authenticated, protected trust roots and independent fork/equivocation refusal.
- G5: independently exercised revocation, outages, compromise, revocation of the guardian itself, key rotation and incident recovery.
- G6: genuinely independently attributable signed evidence and custody chain, independently reproducible without granting publisher control.

Do not upload external private keys, identifying documents, secret recovery materials or OOB communications to a public repository. **All 12 native admission gates remain UNMET; PRODUCTION NOT_AUTHORIZED.**

## Acceptance gates

- One new **DRAFT/UNMERGED** stacked PR exactly on R5H head; **exactly five additive files**: Python two-store actual checkpoint module, 24 actual tests, truthful missing external floor JSON, this dossier, dedicated four-OS CI workflow.
- Dedicated GitHub CI 4/4 actual jobs SUCCESS (macOS Intel, Apple Silicon, Ubuntu, Windows). General Test Gate 5/5 actual jobs SUCCESS. R5H/R5G/R5F/R5E/R5D/R5C/R5B/R5A/R4Z/R4Y regression suites unchanged and checked. Report skipped native tests honestly.
- Owner Mac Intel actual isolated replay against pinned exact new R5I SHA and same-owner R4U observer SHA, **78 exact Git blob pins** (24 Rust R4A–R4X; 5 R4Y; 4 R4Z; 5 each R5A–R5I). Expected **511 regular PASS** = R5I 24 + prior R5H 487; 3 deliberately ignored Rust. Rustfmt/Clippy strict PASS; actual signed-revocation local preserved-floor and both-stores rollback controls executed; original source worktree and GitHub unchanged; `SCRIPT_EXIT=0`.
- The Mac ZIP must contain its four signed-hash-manifest-checked files **at archive root**, verified against outer SHA-256, and be extracted into a *new unique `mktemp -d` folder* with `cd "$WORK"`, not into an existing directory or a guessed nested folder. Never prompt for file replacement.
- Research-only PASS_WITH_CONTROLS requires the owner Mac transcript, not just GitHub CI.

```ini
G03_R5I=RESEARCH_CANDIDATE
SIGNATURE=REAL_EPHEMERAL_ED25519_FIXTURE_ONLY
SECOND_REVOCATION_FLOOR=REAL_LOCAL_SQLITE_ONLY
PRESERVED_FLOOR_PRIMARY_ROLLBACK=DETECTABLE
BOTH_LOCAL_STORES_RESTORED=UNDETECTABLE
REAL_INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_AUTHENTICATED_OOB_ROOT=NOT_ENROLLED
EXTERNAL_PROTECTED_REVOCATION_FLOOR=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
