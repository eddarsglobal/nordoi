# NORDOI G0.3 R5P — Real file close/reopen continuity and inode reuse limits

**DRAFT research-only. No independent guardian or production admission.**

## Source lineage, owner evidence and disposition

R5P directly stacks on frozen R5O FIX2 [PR #106](https://github.com/eddarsglobal/nordoi/pull/106), SHA `d97886abdf2b57324d7e05cd73182837dd112917`, whose owner actually replayed on macOS 14.5 Intel x86_64 with **655 regular PASS, 108/108 SHA-pinned source Git blobs, 3 intentionally ignored Rust, rustfmt and strict Clippy PASS, `SCRIPT_EXIT=0`**. R5O dedicated 4/4 and general 5/5 GitHub CI SUCCESS; R5O PR remains DRAFT/unmerged. The new R5P PR must remain a direct child and add exactly five research-only files. No earlier R4A–R5O source/workflow rewrites, no `.noi`, native kernel/compiler/runtime, main, release or production modifications.

## Research problem

R5O established that a held open descriptor can refer to an older inode after a distinct actual OS process atomically replaces its pathname with a **byte-identical** file. R5P investigates the different interval after the descriptor has been **closed**, and a new descriptor has been opened later; repeated replacements and possible file-system **inode number reuse** are assessed in real disposable files.

A file's **device/inode tuple is NOT a permanent globally unique identity**: the filesystem may reuse inode numbers. Even inode+mode+size+mtime+ctime+content SHA256 are locally observed, mutable under the same administrator, and cannot be considered a protected monotonic generation. Closing and reopening is not an atomic authorization or a lock.

## Real R5P technique

- Use unchanged actual R5L SQLite denial journal, unchanged R5M canonical local witness and R5O held-descriptor verifier. Capture an `epoch()` from two genuine opened FDs: `fstat` device/inode/mode/size/mtime-ns/ctime-ns, SHA256 bytes read via FD, R5M read-only SQL verification, real local witness, current R5L positive denial height/hash-chain head. Then **close both FDs**.
- R5P canonical `LOCAL_DENIAL_AFTER_REOPEN_ONLY` ticket binds exact R5O frozen parent, project/protocol, disposable nonce, actual journal height/head, exact content hashes and full six-element metadata for each file. Tickets are untrusted local source-pinned observations, **not remote signatures or authorizations**.
- Actual second OS process can run genuine `os.replace` against SQLite primary and JSON witness after the earlier descriptors close, or just before a third verification. R5P freshly reopens both files, reruns the same actual read-only SQLite/witness verification, compares all metadata and content, and **quarantines** changed or missing files. Even if the source remains identical, a new inode/ctime can invalidate an earlier ticket. A reissued ticket for newly matched files is *not an authorization*: use unchanged R5O/R5N to store **only a local SQLite refusal**, and replayed local nonces are refused.
- Separate native test calls `actual_reuse_probe`: a **bounded number of genuine separate-process atomic replacements**, observing inode/device tuples. It reports `OBSERVED_NO_AUTHORITY` if reuse happens, or `NOT_OBSERVED_BOUNDED_NO_AUTHORITY` otherwise. The experiment never invents inode reuse or relies on observing it. Reuse is possible on some filesystems, not guaranteed, and can make dev/inode matches misleading. It does not weaken denial-only behavior in either outcome.
- Successful R5P checks cannot create an atomic cross-file snapshot: a same-admin actor can replace/restore all older coherent SQLite, witness, ticket and denial journals; changes after the last check remain outside the observation window. Root filesystem privileges could forge metadata or affect file handling. No real independent owner OOB, hardware protected generation, third-party monotonic anchor or custodian enrolled.

## 24 new adversarial controls

**16 portable**: actual file close/reopen metadata consistency; exact canonical source/height/digest ticket; valid matching identity yields only a local R5N/R5O denial; same nonce replay blocked; distinct nonce creates only another denial; missing primary/witness/local journal never initialized automatically; invalid recovery/GRANT/UNREVOKE/APPROVE operations rejected; wrong frozen project, parent or protocol; duplicate JSON key and noncanonical whitespace rejected; malformed, oversized, zero or bool height, invalid SHA/nonce and noninteger/negative inode and device fields rejected; separate real Python process CLI always exit code **2 / NOT_AUTHORIZED**.

**8 POSIX native experiments**: real close/reopen with no swap permits only local refusal; distinct OS process swaps the primary *after both descriptors closed*, witness likewise; swaps primary just before local deny; two consecutive actual byte-identical `os.replace` calls; stale ticket after close/reopen rejected but refreshed ticket remains denial-only; **bounded real inode reuse observation, with both outcomes accepted and neither yielding authority**; genuine native `--exercise` reproduces complete chain including repeated nonce and only local refusal. Windows runs the 16 portable tests and deliberately **SKIPS** eight POSIX/Unix-native controls; do not count skips as actual executions.

## Tests, approval and research boundary

- Dedicated GitHub 4-OS CI actual **4/4 SUCCESS**, general NORDOI Test Gate actual **5/5 SUCCESS**. Preserve all R5O–R4Y Python research regression suites.
- Owner macOS 14.5 Intel x86_64, Rust/Cargo 1.98.1, isolated freshly cloned pinned candidate head and observer ref. **113 exact Git blob pins** = 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + 5 each R5A–R5P. Expected **679 regular tests** = 24 new R5P + prior R5O 655; three Rust deliberately ignored; Rustfmt and strict Clippy PASS, real native `--exercise`, intact owner original worktree and clean isolated checkout, expected `SCRIPT_EXIT=0`. Do not claim owner Mac PASS before owner log.
- ZIP exactly five files at archive root; verify external SHA256 first, extract into a unique `mktemp -d` folder without interactive overwrites, `cd "$WORK"`, verify `SHA256SUMS.txt`, `bash -n`, run.
- No real independent external guardian, owner authenticated OOB root, protected global generation/revocation floor, global spent nonce ledger or independent recovery. **All 12 native admission gates UNMET; `PRODUCTION=NOT_AUTHORIZED`.** PR stays DRAFT/UNMERGED, main unchanged, no native privilege grant.

```ini
G03_R5P=RESEARCH_CANDIDATE
ACTUAL_CLOSE_REOPEN=REAL_FILES_AND_NEW_DESCRIPTORS
TWO_CONSECUTIVE_ATOMIC_REPLACEMENTS=REAL_SEPARATE_OS_PROCESSES
INODE_REUSE=OBSERVATIONAL_NOT_DETERMINISTIC
STALE_EPOCH=QUARANTINE_WHEN_DETECTED
FRESH_MATCHING_EPOCH=LOCAL_DENIAL_ONLY
SAME_ADMIN_RESTORES_ALL_OLD_HISTORY=NOT_GLOBALLY_DETECTABLE
EXTERNAL_INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
