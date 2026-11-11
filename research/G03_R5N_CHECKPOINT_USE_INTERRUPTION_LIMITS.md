# NORDOI G0.3 R5N — Interrupted checkpoint verification and deny-only use

**Research-only; no independent guardian, owner OOB key root, global protected witness or native admission.**

## Frozen parent

R5N stacks directly on R5M PR #104 exact owner-validated source SHA `e1073d35bb13a61ebe97d20968dbc2abbb28dba5`. Owner macOS Intel 14.5 R5M replay produced **607 regular tests PASS**, three intentionally ignored Rust, **98/98 SHA-pinned Git blobs**, Rustfmt/strict Clippy PASS and exit zero. R5M dedicated GitHub four-OS CI 4/4 SUCCESS, general NORDOI Test Gate 5/5 SUCCESS; PR #104 DRAFT, UNMERGED. R5M demonstrated real file damage detection with a preserved local witness, and the jointly restored same-admin local SQLite + witness counterexample.

## Actual R5N question

Can a *local* proof of R5L SQLite denial history verified against the R5M checkpoint remain unambiguous when another actual OS process replaces or truncates its files **between verification and a local denial recording operation**? Can interruption of an uncommitted local denial permit a claim to be treated as approved? What happens when all previously consistent files are restored?

R5N introduces a **canonical source-pinned deny-only evidence-use intent**: explicit `DENIAL_ONLY_EVIDENCE_USE` operation, nonce, nonzero R5L denial height and SHA-256 hash-chain head, actual SQLite file SHA-256, actual preserved R5M witness SHA-256, frozen exact R5M commit and repository/protocol identity. No alternative GRANT/APPROVE/UNREVOKE/RECOVER operation passes parsing. The receipt is not a cryptographic remote owner authorization: it is an opaque, local, non-secret research evidence snapshot.

The examiner uses unchanged real R5M `mode=ro` SQLite verification and the **actual preserved local witness**, validates before/after a laboratory-controlled second OS process file mutation and revalidates **once more** before attempting to persist a **deny-only** record in a separate actual R5L SQLite hash-chain journal with replay-nonce uniqueness. Only an exact, consistent local snapshot allows recording a local **BLOCK**; it **never allows native resource access or operational recovery**. Missing denial journal, primary DB, or witness causes quarantine **without new genesis creation**. An already consumed nonce returns quarantine, and persisted denial remains after process restart in the tested scope.

**Explicit safety limitation:** These three observations are *not* an atomic cross-process snapshot. A same-admin adversary can replace files after the final check, or restore all coherent old files and the denial log. No external protected monotonic counter, global spent nonce or independently enrolled custodian exists; no admissible authorization is derived. This is an experiment in refusal behavior and detectability, not native concurrency security proof.

## Genuine process and persistence experiments

The new R5N native `--exercise` only creates private disposable `nordoi-r5n-*` directories. It uses **real R5L SQLite deny-only journal** and **real R5M JSON file witness**:
1. Create a real persistent R5L denial and locally capture R5M witness at height 1; prepare a canonical intent bound to both actual files.
2. Verify valid intent and record **only a second local denial** in a third SQLite journal. Retry same nonce -> BLOCK, no replay admission.
3. Start a genuine R5L worker process that creates an **uncommitted** local denial, emits a flushed readiness marker, and is actually **SIGKILL**-terminated; reopening on a different OS process proves only the previous durable denial remains.
4. Start a real second OS process that **truncates** the SQLite file between R5N observations; evidence quarantined.
5. Advance actual R5L journal and R5M witness to height 2; start another real second OS process that replaces the witness with the old checkpoint **between the last two reads**; evidence quarantined **before local denial recording**.
6. Restore both earlier same-admin SQLite and JSON checkpoint files. The old local snapshot matches the old claim and cannot prove that a later height once existed; local decision remains **BLOCK**, not proof of global antirollback.

Every output remains `status=BLOCK`, `admission=NOT_AUTHORIZED` and CLI exit **2**. No real keys, credentials, owner production files or live external storage used.

## 24 new tests and CI requirements

**16 portable real-file controls:** actual R5L/R5M genesis and checkpoint, exact bytes/canonical packet, local denial recording but not authorization, duplicate and distinct nonce, missing journal/witness/primary refuse without silent re-init, missing/oversized/duplicate/noncanonical JSON, wrong source identity, GRANT/UNREVOKE/RECOVER refused, wrong nonce/digests, bool or genesis height refused, damaged actual SQLite, independent subprocess CLI exits 2.

**8 POSIX native tests:** height-2 real SQLite and checkpoint still denied, actual second process truncation between checks, deletion of local proof between checks, replacement of older proof before denial commit, replacement of older SQLite before denial commit, real SIGKILL of uncommitted local deny and subsequent process restart, simultaneous restoration of two old local artifacts and refusal despite inability to detect lost history globally, full native `--exercise` with expected exit 2.

Windows deliberately executes **16 portable controls and SKIPS 8 POSIX-only native controls**; skips never counted as actual runs. Dedicated CI must be 4/4 actual SUCCESS across macOS Intel, Apple Silicon, Ubuntu, Windows; NORDOI general Test Gate 5/5 actual SUCCESS. All R5M–R4Y regression suites unchanged and tested.

## Governance and acceptance

- Exactly **five additive research-only files** in a DRAFT/UNMERGED stacked PR: this dossier, actual R5N Python lab, its 24 tests, truthful no-external-custody JSON, dedicated multi-OS workflow. No change to earlier research, prior workflows, `.noi`, main, native compiler/kernel/runtime, original owner's source checkout, releases or production.
- Owner Mac Intel 14.5 x86_64 and Rust/Cargo 1.98.1: new isolated source checkout at pinned exact R5N head, R4U published observer pin preserved, **103 Git source blob pins** = 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + five each R5A–R5N. **631 expected regular PASS** = 24 new R5N + 607 prior R5M, three intentionally ignored Rust, Rustfmt/strict Clippy PASS. Real native R5N exercise, source and observer pinning, clean checkout, no original source writes, `SCRIPT_EXIT=0`.
- Owner ZIP **five files directly at archive root**. Verify outer SHA256 first, extract into a new unique `mktemp -d` folder and `cd "$WORK"`. No overwrite prompts and no guessed nested directory. Owner Mac replay must be provided; CI alone cannot certify owner hardware execution.
- No independent external guardian, owner-authenticated OOB trust root, external monotonic antirollback, protected global nonce register, recovery key custodian or safe authorization has been enrolled. All 12 native admission gates **UNMET**, **PRODUCTION NOT_AUTHORIZED**. R5N research success cannot promote a native permission.

```ini
G03_R5N=RESEARCH_CANDIDATE
CHECKPOINT_VERIFICATION=REAL_LOCAL_FILES_ONLY
LOCAL_EVIDENCE_USE=DENIAL_ONLY
RECOVERY_ADMISSION=BLOCK
OS_PROCESS_TRUNCATION_BETWEEN_CHECKS=QUARANTINED
OS_PROCESS_WITNESS_REPLACEMENT_BEFORE_DENIAL=QUARANTINED
SIGKILL_UNCOMMITTED_LOCAL_DENIAL=ROLLED_BACK
JOINT_OLD_LOCAL_EVIDENCE_RESTORE=NOT_GLOBALLY_DETECTABLE
INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
