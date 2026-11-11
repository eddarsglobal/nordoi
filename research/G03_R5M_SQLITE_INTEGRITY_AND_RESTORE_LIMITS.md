# NORDOI G0.3 R5M — Real SQLite truncation and preserved local-file integrity quarantine

**Research-only, deny-only. No production. No independent guardian or independently protected witness.**

## Immutable parent and prior owner evidence

R5M directly stacks on R5L FIX1 PR #103 commit `db80d370bdd3ec6acebe01fb1f752b819412062b`. The owner executed R5L on **macOS 14.5 Intel x86_64**: 583/583 regular tests PASS, 3 deliberately ignored Rust, 93/93 SHA-pinned Git blobs PASS, rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`. R5L FIX1 dedicated GitHub CI 4/4 and NORDOI general Test Gate 5/5 SUCCESS. R5L proved actual SQLite uncommitted rollback on process SIGKILL, retention of previously committed local denial and same-nonce contention, but could not defeat same-admin snapshot rollback.

## Research question and real assets

Will a **preserved, locally separate, SHA256-pinned evidence file** allow a read-only examiner to refuse a corrupted, truncated, deleted or swapped real SQLite denial journal, including changes made by a second actual operating-system process? Does this prove independent antirollback if an administrator can restore BOTH the SQLite and witness files?

**Answer:** the separate retained local evidence detects many presented inconsistencies, but **cannot prove independent custody or global antirollback** if both artifacts remain under the same administrator. Every outcome `BLOCK / NOT_AUTHORIZED`.

R5M checks a real `r5l_local_denial_journal.sqlite3` created using the *unchanged* R5L implementation, with real committed local denial records. Its **distinct file** `r5m_preserved_local_checkpoint.json` is created only after R5L commits a denial at height >=1; it captures exact SHA-256 **bytes of the SQLite file**, last height and SHA256 chain head. The witness is a genuine file in a separate private sibling directory, created with exclusive create and file/directory fsync where POSIX provides it. An explicit **advance** operation atomically replaces that witness only if the R5L journal is at a strictly higher height and the earlier witness chain head is present at its former height. This is **application-level local-only provenance**, not an independent trust anchor, immutable backup or storage security boundary.

The examiner opens the *real primary* SQLite file with **read-only SQLite `mode=ro`, `PRAGMA query_only=ON`**, verifies `quick_check`, unique state, the complete actual R5L hash chain, and its file bytes before/after observation. It will **not** initialize a missing database or witness; it rejects symlinks, overlapping directory roots, badly formed or duplicate JSON keys and noncanonical witness bytes. It re-reads both the SQLite file and witness around an optional **lab-only interleaving hook**. A read or file integrity discrepancy quarantines the evidence, and even an exact witness match yields `BLOCK`.

## 24 actual tests

**16 portable tests (Windows/macOS/Linux):** missing witness refuses, genesis is not accepted as a revocation checkpoint, real matching file still refuses authority, existing witness cannot silently be recreated, canonical witness matches actual SHA256 file bytes, duplicate JSON field refused, whitespace/noncanonical JSON refused, extra key refused, forged digest refused, wrong pinned parent refused, actual SQLite denial-chain row tamper refused, actual SQLite header truncation refused, deleted primary does not silently reinitialize, witness symlink refused where supported, overlapping roots refused, and a **real subprocess CLI audit** returns exit 2 `NOT_AUTHORIZED`.

**Eight POSIX native tests** (macOS Intel, Apple Silicon, Ubuntu): verified actual height-2 R5L journal plus preserved witness; actual separate child process truncation between first/second observations; actual child truncates SQLite header; child unlinks SQLite, not recreated; child restores valid old height-1 SQLite with retained height-2 witness -> **BLOCK**; child restores an old witness between two observations -> **BLOCK**; child restores **both** older valid matching artifacts -> local height-1 match is **not distinguishable from legitimate history**; native `--exercise` runs the truncation/rollback research and returns exit **2**.

Windows deliberately skips the 8 POSIX/OpenSSH-specific tests, and runs 16 portable tests on actual Windows SQLite. They must not be misrepresented as native passes.

## Scientific limitations that cannot be promoted

- The witness file and SQLite database are **same-administrator local files**: no real independent external trustee, out-of-band owner root, independently held revocation key, externally protected high-water or global spent-nonce register is enrolled. The same operator can replace both with consistent old snapshots, which R5M explicitly demonstrates as **undetectable locally**.
- Detecting an interrupted file write with a second read is **not** an atomic filesystem snapshot, protected external observation or concurrency-safe authorization. Between read, verification and any later action, files may change. R5M never authorizes any later action.
- Crash tests are actual process actions on disposable files, **not physical power-loss or hardware storage durability certification**. They do not run on owner project sources, production files or external accounts.
- Witness creation/advance are explicit **laboratory evidence** operations and must not be used as a substitute for real owner-enrolled external custody.

All **12 native admission gates remain UNMET** and **PRODUCTION=NOT_AUTHORIZED**.

## Required acceptance gates

- Add **exactly five research-only files** in new DRAFT/UNMERGED PR directly on R5L FIX1: Python real SQLite/witness lab, 24 actual tests, no-external-authority JSON, this dossier, dedicated four-OS CI. Existing prior scientific sources/workflows, `.noi`, compiler/kernel/runtime, main and owner checkout unchanged.
- GitHub dedicated macOS Intel, macOS Apple Silicon, Ubuntu and Windows **4/4 actual SUCCESS**, NORDOI Test Gate **5/5 actual SUCCESS**. Preserve R5L–R4Y regression tests.
- Owner Mac Intel 14.5 x86_64 isolated exact-commit replay, **98 pinned source Git blobs** = 24 Rust R4A–R4X, five R4Y, four R4Z, five each R5A–R5M; **607 expected regular tests** = 24 new R5M + 583 prior R5L, 3 intentionally ignored Rust. Rustfmt/strict Clippy PASS, actual native file integrity `--exercise`, original owner worktree untouched, source exact HEAD unchanged, `SCRIPT_EXIT=0`.
- Owner ZIP five files directly at **archive root**. Verify its external SHA256, extract into a **fresh unique `mktemp -d` folder** and then `cd "$WORK"`. No interactive unzip overwrites. Owner full replay required for research-only acceptance; no invented Mac owner validation.
- No real outside operator or signatures required for **this bounded negative-control science**, but no native privilege may be admitted from it.

```ini
G03_R5M=RESEARCH_CANDIDATE
REAL_SQLITE_DAMAGE=FAIL_CLOSED_WHEN_PRESENTED
PRESERVED_LOCAL_FILE_WITNESS=LOCAL_SHA256_ONLY
MISSING_OR_TRUNCATED_PRIMARY=BLOCK_NO_REINITIALIZATION
PRIMARY_OLD_REPLACEMENT_WITH_RETAINED_NEW_WITNESS=BLOCK
SIMULTANEOUS_OLD_PRIMARY_AND_WITNESS_RESTORE=NOT_GLOBALLY_DETECTABLE
REAL_EXTERNAL_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
EXTERNAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
