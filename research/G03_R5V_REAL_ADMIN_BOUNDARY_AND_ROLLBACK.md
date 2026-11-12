# NORDOI G0.3 R5V — Real OS administrative separation measurements, local signed custody and rollback

**RESEARCH ONLY / NO EXTERNAL AUTHORITY / NO NATIVE GRANTS OR PRODUCTION.**

## Source lineage

R5V is a direct candidate child of owner-qualified R5U FIX1 SHA `e2138af4c1a2dd3964d5ebdd9bf669129b63cfb1` (PR #112). Owner Mac Intel macOS 14.5 x86_64 Rust/Cargo 1.98.1 actually replayed **799 regular tests PASS** (546 Python +253 Rust), **138/138 frozen source Git blob IDs PASS**, three Rust deliberately ignored, genuine native R4M XPC/CDHash 6/6 serial PASS with no retry or waiver, Rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`. Exact GitHub same-head dedicated CI 4/4 and general Test Gate 5/5 SUCCESS. The first R5U Windows failed cleanup job (unclosed SQLite handle) remains documented; FIX1 corrected new test-only connection close without weakening rejection. Parent PR remains DRAFT/UNMERGED.

## Problem / measured evidence rather than claimed independence

A genuine second OS process, distinct local Ed25519 keys, different file paths, a folder with private permissions, a claimed domain label, or even a *reported* different UID are not automatically an externally authenticated independent administrative domain. R5V therefore records real **PID, PPID, UID/EUID and GID/EGID from actual OS syscalls** in two genuine separate process executions. On Windows where those POSIX identity observations are unavailable, fields are explicitly `null` and conclusion is `OS_USER_ID_SEPARATION_NOT_OBSERVABLE_ON_PLATFORM`, not a fictional credential.

The exact-source canonical `NORDOI-G03-R5V-ADMIN-SEPARATION-EVIDENCE-V1` report pins source SHA, role owner_proxy/custodian_proxy, claimed administrative label, observed process IDs and OS identities, actual signer public key SHA256, actual local R5U key epoch and ledger head, and exact 64-lowerhex local challenge. Parser strictly refuses duplicate/extra/missing/noncanonical JSON, invalid OS types, malformed PIDs, key pins and forged source. Comparing truly different PIDs with actual same UID/EUID returns `ACTUAL_SAME_OS_USER_ADMIN_DOMAIN_DISQUALIFIED / BLOCK / NOT_AUTHORIZED`. Different label and different real local key do **not** qualify independent custody. Even two genuinely different UIDs, **if ever observed**, would alone still fail to authenticate legal/administrative independence, trusted key roots and global monotonic history; this lab does not claim to perform privileged user switching.

Real `/usr/bin/ssh-keygen` creates two distinct Ed25519 keys under the SAME admin in a disposable 0700 research folder. Two reports originating in actual different child processes are signed by genuine `ssh-keygen -Y sign -n nordoi-r5v`, verified with `ssh-keygen -Y verify` using actual locally generated `allowed_signers`. The exact report key pin is checked against the real public key file. Changing canonical report bytes while preserving pin and schema produces genuine SSHSIG rejection. Wrong allowed signer or false pin is rejected. This is cryptographic key possession and real OS identity observation, **not** authenticated external authority.

R5V reuses unchanged R5U live SQLite signed key-epoch code: genuine dual-key signed rotation from epoch 0 to 1 and genuine successor signed revocation at epoch 2. The local SQL ledger identifies locally active and revoked keys while intact, but a **real second OS process atomically restores a prior on-disk SQLite snapshot**. The revoked successor may then appear unrevoked locally again, and a valid earlier signed OS report remains mathematically signed; neither establishes outside independent witness. A byte-identical atomic signature-file inode replacement is detected/quarantined through R5O held-FD checks. No production credential or host access is changed.

## Tests — exactly 24 new R5V cases

**16 portable tests**: actual OS PID/EUID observation, strict parent/source/protocol and no grant fields, same UID with different claimed domains still denied, different PIDs/public key fingerprints not independent, hypothetical altered UID claim remains **test input** and never becomes real independent proof, same PID refusal, identical key pin refusal, wrong challenge or history/epoch refusal, invalid role/domain, bogus UID/GID/eUID/eGID, bad PID/epoch Boolean, malformed 64hex signer/history/challenge, duplicate/noncanonical/extra/missing fields, huge/invalid UTF8/source spoofing, fresh actual subprocess with observed same UID (or unobservable on Windows) returning deny-only.

**8 native POSIX**: real two process PIDs and identical actual UID/eUID; genuine two separate local Ed25519 SSHSIG signatures with different keypins and real verification; **canonical modified** OS report rejected specifically at cryptographic verifier; wrong signer/allowlist/public pin refused; genuine dual-key R5U signed rotation plus unchanged disqualification; genuine successor signed revocation still no distinct admin control; actual second process restores prior SQLite key state and loses local revocation; complete actual native `--exercise` returns **exit 2** and exact controlled negative evidence. Windows deliberately **skips 8 POSIX/OpenSSH-only cases**, runs 16 portable ones; skips never count as native qualification.

Dedicated 4-OS CI executes new R5V 24 tests and all original R5U–R4Y negative regression suites, original real Mac Intel and Apple Silicon R4M signed XPC/CDHash 6 tests **serial `--test-threads=1`**, no retry, waiver or weakened CDHash admission. General Test Gate must independently run five jobs PASS. This is a five-file additive research-only candidate: module, tests, 12-gate JSON, dossier, workflow. Earlier R4A–R5U research, Rust/runtime/.noi/main, parent PR and owner checkout unchanged.

## Owner Mac Intel acceptance (targets, not current results)

macOS 14.5 Intel x86_64; rustc/cargo 1.98.1, exact HEAD+observer pin, isolated fresh clone; **143 pinned Git source blobs** (24 Rust R4A–R4X, five R4Y, four R4Z, five each R5A–R5V); **823 regular tests expected** = **570 Python** (new R5V 24 + earlier 546) + **253 Rust**, three deliberately ignored Rust, genuine original R4M six serial PASS no waiver/retry, rustfmt/strict Clippy PASS, real R5V `--exercise` as well as prior denial-only CLI controls, `SCRIPT_EXIT=0`, original owner worktree untouched. Preflight kit integrity/selftest is NOT equal to a full owner Mac run. Any failure blocks this qualification. RoadMap-appointed engineers retain formal independent certification authority.

A real independently enrolled external custodian, owner OOB-authenticated root, legally independent administrator, protected monotonic key epoch and signed revocation floor, globally durable nonce history and real independent quorum **are not present**; all twelve admission gates remain UNMET, production NOT_AUTHORIZED. R5V does not silently promote OS identity observation into third-party authority.

```ini
G03_R5V=RESEARCH_CANDIDATE
ACTUAL_TWO_OS_PROCESS_PIDS=DISTINCT
ACTUAL_UNIX_UID_EUID=IDENTICAL_SAME_ADMIN
TWO_REAL_ED25519_SIGNERS=KEY_POSSESSION_ONLY
ADMINISTRATIVE_INDEPENDENCE=NOT_PROVEN
TRUSTED_OWNER_OOB_ROOT=NOT_ENROLLED
REAL_R5U_SIGNED_ROTATION=BLOCK_ONLY
REAL_R5U_SIGNED_REVOCATION=BLOCK_ONLY
REVOCATION_AFTER_ACTUAL_SQLITE_RESTORE=LOST_LOCALLY
EXTERNAL_GLOBAL_MONOTONICITY=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
