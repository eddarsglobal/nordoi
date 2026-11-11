# NORDOI G0.3 R5H — Actual SSHSIG revocation and the rollbackable revocation-log gap

**Scope: research-only negative-control lab. Not independent authority, not production.**

## Frozen research baseline

R5H stacks directly on R5G PR #98 commit `c4356e24aeb1638dcd110f10d3439f1d471b3667`. The owner completed an actual **macOS 14.5 Intel** replay for R5G: **463/463 research regular tests PASS**, **3 deliberately ignored Rust cases**, **68/68 pinned Git blobs**, rustfmt and strict clippy PASS, `SCRIPT_EXIT=0`; R5G dedicated CI **4/4 SUCCESS**, NORDOI Test Gate **5/5 SUCCESS**. R5G proved that a separate process using the same Unix owner can chmod the supposed read-only witness (0400 to 0600) and restore both SQLite databases to an older mutually consistent state, without detecting rollback.

## Scientific question

Does a **genuine Ed25519 SSHSIG signature** on a revocation, combined with an actual durable local SQLite revocation log and anti-replay controls, prove that a separately governed external guardian can revoke or refuse NORDOI powers, even after the publisher restores an old database snapshot?

**Answer tested: no.** Signatures demonstrate possession of the corresponding *laboratory-generated* private key and bind the signed message. They do not prove real guardian identity, independent custody of that private key, an owner-approved out-of-band public-key root, independent administrator control of the database, durable global revocation state, or compromise/recovery governance.

## Actual test method

The new research module uses the native `/usr/bin/ssh-keygen` OpenSSH binary where available to generate an **ephemeral Ed25519 fixture key** inside a fresh private temporary directory, sign a canonical JSON `REVOKE` record with a **new R5H-specific SSHSIG namespace**, and verify the genuine signature with an exact fixture public key. No private key leaves the ephemeral directory. The fixture signer is controlled by the test process, *not an outside guardian*.

The signed revocation record binds:
- R5H protocol and original project and exact R5G parent commit;
- `REVOKE` only, sequence, epoch and predecessor hash;
- canonical nonce and resource SHA-256 digest;
- fixture Ed25519 key fingerprint;
- issuance and bounded expiry (up to 300 seconds).

The new real file-backed SQLite `r5h_local_revocations.sqlite3` applies `PRAGMA synchronous=FULL`, rollback journal and `BEGIN IMMEDIATE` local transactions. It verifies an append-only-by-application SHA-256 chain of local revocations. A correctly signed `REVOKE` commits locally, sets the revocation latch and rejects a duplicate. That local commit **always returns `admission=NOT_AUTHORIZED`**, never grants a native production capability; the CLI is always exit code **2**.

**Critical real negative experiment:** after opening an empty local SQLite ledger, the lab copies its actual file. It then commits a cryptographically valid signed `REVOKE`, confirms that the state is revoked and that duplicate submission is refused, then restores the previous empty SQLite file from the same-administrator copy. The old database is internally consistent, and the revocation latch now reads **false**. Neither signature verification nor the local hash chain can recover the missing signed revocation without an independently held high-water or append-only revocation history. This finding is precisely `SIGNED_LOCAL_REVOCATION_LOST_AFTER_SAME_ADMIN_SNAPSHOT_RESTORE`, and authority **remains BLOCK** even in that case.

## 24 new adversarial tests

16 portable negative controls: missing/invalid JSON, malformed UTF-8, duplicate keys, noncanonical and oversized records, incorrect protocol/project/parent, unsupported `GRANT`/`UNREVOKE`, invalid sequence/epoch, invalid digests/nonces, expiry and TTL failures, invalid Ed25519 wire key/pin, fraudulent signature, and no automatic trust enrollment.

8 **genuine native OpenSSH** controls where `/usr/bin/ssh-keygen` is installed: real Ed25519 SSHSIG verifies only fixture key possession, signed local `REVOKE` commits but denies admission, local nonce replay refused across reopen, real second key substitution refused, signed message tamper detected, signed receipt after expiry refused, older SQLite snapshot actually erases the signed revocation, and real CLI demonstrates this rollback yet exits 2.

Windows is **not** claimed to execute native Unix `/usr/bin/ssh-keygen`; its eight native controls must be reported as skipped, while its 16 portable tests must succeed. macOS Intel, Apple Silicon and Ubuntu must execute all 24 with no skips. All R5G/R5F/R5E/R5D/R5C/R5B/R5A/R4Z/R4Y regression tests remain untouched and checked on GitHub.

## Governing limits

- **No real guardian:** external independent operator, independent custody of signing keys, distinct root/backup/CI and revocation powers not enrolled.
- **No owner OOB root:** a fixture public-key SHA-256 pin is not independently authenticated OOB trust.
- **No externally durable revoke:** the same local database admin can restore a past copy and erase a signed revocation event. The experiment proves lack of globally monotonic revocation.
- **No production verifier:** main, native compiler/kernel/runtime and `.noi` remain unchanged, so nothing in this lab is a deployed protected admission or revocation control.
- **No independent recovery:** real third-party revocation, key rotation, compromise isolation, fallback, outage and recovery custody are not enrolled.
- **No fabricated evidence:** owner terminal replay and same-owner CI logs do not count as independently controlled third-party signed evidence.

All **12 native admission gates UNMET**, `PRODUCTION=NOT_AUTHORIZED`.

## Acceptance checklist

Exactly **five additive research-only files**: this dossier, actual SSHSIG plus SQLite revocation module, 24 adversarial tests, honest no-external-guardian JSON, and a dedicated four-platform GitHub Actions workflow. DRAFT child PR of R5G, no merge. Existing science files and CI, `.noi`, compiler/kernel/runtime, main, owner original worktree and production stay untouched.

- Dedicated CI: actual macOS Intel, Apple Silicon, Ubuntu, Windows jobs **4/4 SUCCESS**. Mac/Linux native 24/24 PASS; Windows 16 PASS and 8 native skips. Regression suites through R4Y PASS, with native skips exactly reported.
- General NORDOI Test Gate: **5/5 actual SUCCESS**.
- Owner Mac Intel 14.5, Rust/Cargo 1.98.1, isolated exact-source replay: **73 pinned Git blobs** = 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + 5 each R5A–R5H; **487 regular PASS** = R5H 24 plus all 463 R5G prior regular tests, **3 deliberately ignored Rust**. Bash syntax, selftest, Git pins, exact head, rustfmt/strict clippy, real signed revocation rollback counterexample, original checkout clean and `SCRIPT_EXIT=0`.
- Research verdict can be `PASS_WITH_CONTROLS_RESEARCH_ONLY` only after actual owner replay, and does not promote admission; no authentic outside guardian can be inferred from test cryptography.

```ini
G03_R5H=RESEARCH_CANDIDATE
REAL_ED25519_OPENSSH_SSHSIG=FIXTURE_SIGNING_ONLY
REAL_LOCAL_SQLITE_REVOCATION=LAB_ONLY
SIGNED_REVOCATION_ANTI_REPLAY=LOCAL_UNRESTORED_HISTORY_ONLY
SIGNED_REVOCATION_SNAPSHOT_RESTORE=ERASES_LOCAL_REVOKED_LATCH
REAL_INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_AUTHENTICATED_OOB_ROOT=NOT_ENROLLED
PROTECTED_GLOBAL_REVOCATION=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
