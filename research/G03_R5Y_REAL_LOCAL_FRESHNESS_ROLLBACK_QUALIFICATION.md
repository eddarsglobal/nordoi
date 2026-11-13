# NORDOI G0.3 R5Y — Signed receipt freshness, real local nonce journal and coherent rollback

**DRAFT RESEARCH CANDIDATE / NO EXTERNAL TRUST OR NATIVE ADMISSION.** R5Y is an exact additive child of owner Mac Intel validated R5X SHA `fbef91f4dccc22d3a73a8cb4f466e738d3bedbdf` (PR #115), which reported 871 regular PASS, 153/153 Git source pins PASS, original true signed native XPC/CDHash R4M 6/6 PASS serial, rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0` and GitHub CI 9/9 SUCCESS. R5X PR stays DRAFT/UNMERGED.

## Research problem

A candidate receipt can have a **genuine OpenSSH Ed25519 signature** and a fresh, source-bound challenge; its key may still be controlled by the same local administrator. R5X rightly denies external trust. R5Y measures the **persistence limits of a real SQLite local spent-nonce and revocation-floor register**. In the intact local state, each signed receipt can be recorded as a denial-only event once; duplicate challenges and lowered local revocation floors are refused. After a **real second OS process atomically restores an earlier coherent SQLite file**, a previously consumed signed challenge may become locally unused again and can be re-recorded. Cryptographic signatures do not become invalid as a result of the restored database.

This is actual SQLite file I/O and real operating system process isolation, **not a simulation of an external guardian**. The experiment preserves durable local commits, a 0700 disposable research root, 0600 SQLite/private test files on POSIX, a canonical source/nonce/history/key-epoch-bound signed receipt, OpenSSH `ssh-keygen -Y sign/verify`, a deliberately controlled restoration and a newly started OS process. No actual third-party operator, production credentials, user-switching, privileged helper or remote trust root is involved.

## Fail-closed implementation

- Exact new protocol `NORDOI-G03-R5Y-LOCAL-FRESHNESS-ROLLBACK-V1`, pinned to R5X parent SHA, with strictly canonical local evidence fields. Duplicate, missing/extra, noncanonical, invalid digest, negative/Boolean counter, privileged status and forged project/source are refused.
- Real SQLite `PRAGMA synchronous=FULL`, transactional one-use challenge register and local revocation-floor high-water mark. Local replay returns `LOCAL_SIGNED_NONCE_ALREADY_SPENT / BLOCK`; fresh local denial append returns `LOCAL_ONLY_SPENT_NONCE_COMMITTED / BLOCK`. This is **not an admission or an externally protected antirollback floor**.
- Portable tests may use structurally canonical unsigned candidate receipts **only to test local database mechanics**. They do not claim cryptographic authenticity or external provenance; native POSIX tests add actual OpenSSH signature validation before controlled journal experiments.
- Real second-process restoration in a disposable lab restores only `before_second.sqlite3` to `r5y_local_nonce_history.sqlite3`. The helper is confined to an absolute private directory with an exact research prefix and fixed source/target filenames; symlink, outside-lab and non-private directories fail closed.
- The true native `--exercise` uses two independently spawned OS observation processes with identical UID/EUID, separate actual local Ed25519 keys and two different challenge-bound receipts. It confirms denial-only first and second durable commits, local replay refusal, a real separate process coherently restoring the old database, and **reissued local nonce** after this restore. It exits intentionally **2** for the qualified negative result.

## Exact security limits

Two OS processes, two locally controlled keys, a valid SSHSIG and two different challenge values do not authenticate a real independent external administrative custodian. A successful local nonce check and a local revocation floor can both be rolled back by an administrator controlling all files. There is **no authenticated OOB owner root**, **no independent external guardian**, **no independently protected global nonce/revocation history**, and **no independent quorum**. Every native admission gate remains **12 UNMET** and `PRODUCTION=NOT_AUTHORIZED`.

## Tests and compatibility

**24 new R5Y tests:** 16 portable canonical SQLite and source-check cases (actual local write, one-use challenge, local timebox, revoked floor, real file snapshot restore, schema failures, permissions and no grant), plus 8 POSIX genuine OpenSSH/OS negative controls (real locally signed candidate, valid signed nonce one-use, modified signed body, wrong real allowlist signer, expired genuine receipt, local signed revocation floor, real second-process coherent rollback with signature still valid, full native CLI exit 2). Windows intentionally skips eight POSIX-only cases and validates sixteen portable ones; no Windows process ID is fictionalized as an independent custodian.

The dedicated four-OS CI runs R5Y and all R5X–R5A negative regressions, plus R4Z/R4Y. Both macOS runners run original R4M **six genuine signed XPC/CDHash native tests serial once** without retry or waiver. The independent general NORDOI Test Gate remains separate.

## Owner Mac Intel technical acceptance — target, not yet observed

Fresh verified ZIP and internal SHA-256 manifest; isolated clone pinned to exact R5Y commit and unmodified R5X base; **exactly five additive R5Y files, 158/158 pinned Git source blobs** (24 Rust R4A–R4X, 5 R4Y, 4 R4Z, 5 each R5A–R5Y); **895 regular tests** = **642 Python** (18 R4Y +24 R4Z +25×24 R5A–R5Y) and **253 Rust**, three deliberately ignored Rust; true R4M native XPC/CDHash 6 serial, rustfmt/strict Clippy PASS, HEAD before/after exact, clean clone, `SCRIPT_EXIT=0`, original owner worktree unchanged. These figures are **targets** until owner Terminal evidence is received.

## Bureau disposition

A R5Y technical PASS can certify only correct negative controls and their **demonstration of local antirollback limitations**; it cannot certify a globally protected external nonce floor. Independent external custody and formal Bureau certification require separately authenticated trust roots and external operators. PR remains DRAFT and unmerged, no main, runtime, .noi, release or native admission changes.

```ini
G03_R5Y=RESEARCH_CANDIDATE
SIGNED_RECEIPT_VALIDITY=LOCAL_KEY_POSSESSION_ONLY
REAL_SQLITE_SPENT_NONCE=LOCAL_DURABLE_ONLY
REAL_COHERENT_ROLLBACK=OLD_NONCE_REISSUABLE_LOCAL_ONLY
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
INDEPENDENT_EXTERNAL_CUSTODIAN=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_OWNER_SOURCE_GIT_BLOBS=158/158_PASS
EXPECTED_OWNER_REGULAR_TESTS=895_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```
