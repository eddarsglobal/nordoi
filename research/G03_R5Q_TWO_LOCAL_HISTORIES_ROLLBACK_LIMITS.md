# NORDOI G0.3 R5Q — Two real local histories become indistinguishable after coherent same-admin rollback

**Research only. All access decisions BLOCK. No independent guardian, OOB root or production authority.**

## Exact frozen parent and prior owner Mac evidence

R5Q is stacked directly on [G0.3 R5P FIX1 PR #107](https://github.com/eddarsglobal/nordoi/pull/107), exact owner-validated SHA `a9968141e628ec4ccfae560830414c9c8cb5dd92`. The owner actually executed R5P FIX1 on **macOS 14.5 Intel x86_64** with Rust/Cargo 1.98.1: **679 regular tests PASS (426 Python +253 Rust)**, 3 intentionally ignored Rust, 113/113 source Git blob pins PASS, genuine native signed XPC R4M **6/6 PASS with `--test-threads=1` and no retry**, Rustfmt/strict Clippy PASS, `LAST_STAGE=COMPLETE`, `SCRIPT_EXIT=0`. GitHub dedicated four OS jobs 4/4 SUCCESS and general Test Gate 5/5 SUCCESS. The earlier owner R5P nonserial R4M failure remains recorded, not erased. Neither PR #107 nor R5Q may be merged into main.

## Question and scientific scope

R5M–R5P established actual SQLite and local-witness corruption, open descriptor versus pathname replacement, and closed/reopened pathname identity checks. But these check only *local current* evidence. Can a new real operating-system process distinguish a historic **height-2 committed world B** from **height-1 world A**, after a single same-administrator actor restores *all three* evidence files to valid old copies, including the spent nonce register?

**No independently protected history exists to answer this.** R5Q creates the following actual research sequence on disposable private directories under `nordoi-r5l-r5q-*`:

1. Create the actual R5L SQLite primary denial journal (height 1), actual R5M separate JSON checkpoint at height 1, and an actual distinct R5L SQLite **local spent-nonce/denial register**. Commit a real R5P deny-only use, consuming nonce C locally at height 1. This is historic world A; store byte-for-byte copies of the **three files** in a same-admin disposable snapshot folder.
2. Commit a second R5L primary denial at height 2; advance the R5M checkpoint to height 2 and record a second R5P local denial, now consuming nonce D at height 2. This is historic world B. Verify the true SQL chain and nonce records and that B's hash projection differs from A.
3. In a **distinct actual operating-system subprocess**, copy only the old primary from A while the newer R5M proof remains at height 2. The examiner **quarantines** this mismatch. A separate test also restores only the old JSON witness against the newer primary and detects the mismatch.
4. In another **real separate OS process**, restore **all three old coherent files** (primary SQLite, local JSON proof and SQLite spent-nonce register). The original process's verified **application-level evidence projection**—actual SHA-256 primary/witness bytes, R5L primary height/head, local spent-nonce height/head and full denied nonce list—now exactly matches A. A new real OS process invoked via `subprocess` also reports this former projection. No protected outside high-water exists that can say B ever happened.
5. An **R5P ticket from world B** must still fail against the restored old world. This negative control distinguishes stale-ticket detection from true historical antirollback. A **fresh** R5P ticket obtained after the old coherent restore is consistent locally and yields **only BLOCK**, never admission. Nonce D was genuinely consumed in B; after the same-admin spent nonce ledger is reverted to A, D can be **recorded locally again**. This exposes missing *globally durable* replay protection without ever authorizing a native operation.

The proposition is **application-layer historical observational equivalence**, *not* identical filesystem metadata across worlds: inode, ctime, file-descriptor state or process IDs may and often do differ. R5P uses current device/inode/mtime/ctime in a fresh ticket, but none of those is an independent protected historical root. No simulated power loss, fake external signatures, false independent custodian or native privilege is used.

## Implementation and tests

Exactly five additive research files:
- `research/g03_r5q_coherent_local_rollback.py`: real three-file read-only evidence projection; strict denial on corrupted/missing source files; disposable lab-only copy/restore allowlist; actual second OS process file restore; fresh real OS process examiner; nonce D historical replay counterexample; CLI audit and full `--exercise` exit **2** always.
- `research/test_g03_r5q_coherent_local_rollback.py`: **24 actual adversarial unit tests**, no relaxed negative receipts.
- `research/G03_R5Q_NO_EXTERNAL_COHERENT_ROLLBACK_AUTHORITY.json`: explicit no external/global authority contract.
- This scientific dossier.
- `.github/workflows/g03_r5q_coherent_rollback.yml`: real four-OS research CI and R5P–R4Y regression chain, plus serialized genuine native R4M XPC adversarial tests on both macOS flavors.

**16 portable tests** (Windows, macOS and Linux) verify real files and JSON+SQLite formats, real content SHA256 projection and actual local nonce list, deletion/corruption of each of three stores refuses without silent new genesis, actual second consistent local history is distinct, duplicate local nonce refuses, local valid ticket still only records BLOCK, overlapping roots refused, copy snapshots contain real equal bytes but are same-admin, and an actual separate OS process CLI `--audit` reports identical local-only projection and exit 2.

**Eight genuine native POSIX tests** on macOS Intel, Apple Silicon and Linux: verify two committed distinct real histories; restore old primary only => mismatch with retained newer witness quarantined; restore old witness only => mismatch quarantined; restore **only** spent-nonce SQLite ledger => primary/witness still locally coherent and no globally protected shared high-water exists; restore all three via real subprocess => original old application-level evidence projection; stale R5P B ticket denied with fresh R5P A ticket still only local denial; nonce consumed at height2 B is no longer globally known after coordinated all-file restore and can be refused-and-recorded locally again, not an admission; full native `--exercise` proves these results via live processes and exit 2. Windows **16 portable PASS with eight intentionally skipped POSIX-only controls**; do not falsely count skipped native tests as PASS.

## Rigorous limitations / research admission boundary

- A preserved local file, its SHA256 or a source-pinned Git commit **cannot** provide a globally monotonic history if the same admin can roll all evidence back.
- Inodes, open descriptor state, mtime, ctime and file SHA can reveal certain atomic swaps or single-file corruption but are not independent immutable credentials. **This R5Q claim compares security-relevant application projections, not every possible OS observation.**
- No real independent third-party guardian, owner authenticated out-of-band enrollment, protected monotonic highwater, independent cross-signing custody, durable global nonce uniqueness, or independently authorized recovery/rotation exists. The experiment cannot establish one or bypass its absence.
- A local nonce D recorded again after rollback is *another local BLOCK record*, not unauthorized native access. All operations remain deny-only in the research harness.
- Physical power-loss durability, tamper-proof logging and cross-machine historical consensus are not certified. Laboratory snapshots are same-administrator disposable files and must never be represented as trusted.
- **All 12 native admission gates remain UNMET. PRODUCTION NOT_AUTHORIZED. No merges.**

## Acceptance and owner Intel replay contract

Exact parent R5P FIX1 SHA pinned. Five additive files only. Dedicated CI macOS Intel, Apple Silicon, Ubuntu and Windows **4/4 actual SUCCESS**; NORDOI general Test Gate Check, Format+Clippy, macOS, Ubuntu, Windows **5/5 actual SUCCESS** at exact R5Q head. Preserve genuine R4M native test **all six cases, `--test-threads=1`, no retries and no waivers** on native macOS CI and owner Mac.

Owner Mac **14.5 Intel x86_64 / Rust + Cargo 1.98.1**, isolated fresh exact-source clone plus old pinned observer branch. **118 pinned Git source blobs** = 24 Rust R4A–R4X, 5 R4Y, 4 R4Z, five each R5A–R5Q. Expected **703 regular tests** = 24 new R5Q + 679 prior R5P (450 Python + 253 Rust), 3 intentionally ignored Rust; Rustfmt / Clippy strict PASS, all genuine negative-control live exercises, no source/worktree changes, `SCRIPT_EXIT=0`. Owner ZIP exactly five files at **archive root**, verify exterior SHA256, extract only into fresh unique `mktemp -d`, verify 4/4 internal SHA256SUMS, Bash -n, then run. Owner actual replay PENDING until supplied log.

```ini
G03_R5Q=RESEARCH_CANDIDATE
REAL_HISTORY_A=LOCAL_SQLITE_AND_JSON_HEIGHT_1
REAL_HISTORY_B=LOCAL_SQLITE_AND_JSON_HEIGHT_2_WITH_SPENT_NONCE
PRIMARY_ONLY_RESTORE_WITH_NEW_WITNESS=QUARANTINE
ALL_THREE_OLD_SAME_ADMIN_FILES_RESTORED=OLD_APPLICATION_PROJECTION_MATCHES
NEW_OS_PROCESS_AFTER_RESTORE=SEES_ONLY_OLD_LOCAL_HISTORY
WORLD_B_SPENT_NONCE_AFTER_GLOBAL_ROLLBACK=FORGOTTEN_LOCALLY
REISSUED_LOCAL_NONCE_AFTER_ROLLBACK=BLOCK_ONLY
EXTERNALLY_PROTECTED_GLOBAL_HIGH_WATER=NOT_IMPLEMENTED
INDEPENDENT_EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
