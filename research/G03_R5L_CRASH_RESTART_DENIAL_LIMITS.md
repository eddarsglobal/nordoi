# NORDOI G0.3 R5L — Actual SIGKILL, restart, durability and competing deny-only decisions

**Research-only, deny-only. Independent guardian and globally protected revocation are NOT ENROLLED.**

## Parent and actual previously verified evidence

R5L is stacked directly on immutable R5K PR #102 head `7a22152f5bc4c74f014601bfd56338195d78a647`. The owner performed a complete **macOS 14.5 Intel** R5K replay: 559/559 regular tests PASS, three Rust intentionally ignored, **88/88 Git SHA-pinned source blobs**, Rustfmt and Clippy strict PASS, `SCRIPT_EXIT=0`. Dedicated R5K CI 4/4 SUCCESS and general NORDOI Test Gate 5/5 SUCCESS. The new PR is research-only and must remain DRAFT/UNMERGED, with original sources, `.noi`, native compiler/kernel/runtime, main, release and production untouched.

## Real R5L scientific problem

After genuine R5H temporary Ed25519 revocation, the real R5I SQLite second checkpoint, R5J signed recovery-intent quarantine and R5K concurrent signed intent refusals, what can a local *deny-only* journal guarantee across:
- an actual operating-system process being **forcibly terminated while holding an uncommitted SQLite transaction**;
- a new real process starting afterward and reopening persistent files;
- two separately launched actual OS processes racing to record the **same nonce** or separate denials;
- a same-administrator **physical old SQLite snapshot restore** despite valid earlier commits?

R5L creates a third real *local*, file-backed SQLite `r5l_local_denial_journal.sqlite3` with `synchronous=FULL`, `journal_mode=DELETE`, `BEGIN IMMEDIATE`, strict `quick_check`, continuous SHA-256 predecessor hash chain, exact nonce uniqueness, immutable decision literals `BLOCK / NOT_AUTHORIZED`, and an explicit fail-closed re-open if a previously existing state row has been removed. Paths must be absolute private non-symlink directories/files. There is no permission grant or runtime recovery code.

R5L does not claim that a SQLite durable commit defeats OS/filesystem root, snapshot restore, disk loss or an external threat. The same owner controls every local file; permanent protection against restore requires a genuinely independent trusted monotonic anchor.

## Real process and storage tests

The standalone `--exercise` makes fresh private temporary directories and actual R5H/R5I revocation files; two actual **distinct disposable Ed25519 key pairs** sign R5J recovery intents. The existing R5K checker verifies their genuine cryptographic signatures and still **BLOCKS** recovery. A SHA256 hash of the batch is bound into only a *local denial log*, not granted any authority.

Then actual subprocesses use the R5L script to:
1. Commit a **real SQLite denial** at height 1. A separate Python process reopens the file and verifies that it persisted.
2. Start another **real OS process** that begins an immediate write transaction, writes an updated row and emits a flushed `R5L_CHILD_TXN_UNCOMMITTED_READY` marker **before COMMIT**, and remains running. The parent sends **actual SIGKILL / `Popen.kill()`** and waits for termination. SQLite recovery after a fresh process reopen must show the **previous committed height 1**, without the uncommitted row.
3. Start another real process and record the interrupted nonce as a new **locally committed denial at height 2**. Attempting to replay that now-spent nonce is blocked. Do **not** interpret this as performing or approving recovery.
4. Start two **concurrent child processes** trying to insert the *same nonce*, one per local candidate role. Exactly one denial event may be committed; the other must be refused as a replay. Both processes must exit code 2, and the final local denied-event height must be 3.
5. A separate adversarial test snapshots the entire local journal at height 0, commits a real denial, restores the old file and shows a now internally valid height-0 history. **The lost denial cannot be globally detected if the same administrator controls both current and previous snapshots**. Even the restored, apparently clean state still yields **BLOCK**.

This is actual local storage and process termination, **not a claim of real third-party external custody, an operational failover or an HSM**. Tests do not run against owner or production paths: all native demonstrations stay inside newly created disposable `nordoi-r5l-*` directories.

## New 24 case controls

**16 cross-platform**: actual genesis SQLite and file header, persisted denial and replay refusal, reopen durability, invalid/bool nonce and claim hash, unexpected role, deleted existing state cannot self-heal, modified state or event chain refuses, broken SQLite file refuses, symlink refuses (when supported), nonprivate directory refusal on Unix, audit checks non-mutation, and a real child audit that exits 2 without admitting anything.

**8 actual POSIX/native tests**: true R5K dual-Ed25519 signing on genuine R5H/R5I SQLite evidence with R5L kill/restart exercise; committed record visible across separate OS processes; actual SIGKILL of uncommitted transaction and SQLite rollback; pre-crash denial preserved across kill and subsequent real new commit; concurrent same-nonce subprocesses serialize to one local commit and one refusal; two distinct parallel nonces both record denials with no permission grant; local same-admin snapshot restoration hides previously committed denial; full real native `--exercise` demonstrates every guard and CLI **exit code 2**.

On **Windows**, the 8 POSIX/OpenSSH/SIGKILL controls are correctly **SKIPPED**; 16 portable tests run on actual SQLite. Do not count skips as passes.

## Scientific interpretation / security limitations

No independent third-party custodian, owner-OOB trust root, protected global monotonic revocation floor, independently held recovery signing keys or global durable spent-nonce ledger exists. Local SQLite crash recovery establishes limited durability and transaction atomicity for the tested termination scenario; it does not solve concurrency at a native admission boundary, protected antirollback, file replacement between two checks, root-level compromise, physical power loss, or cross-host recovery. All decisions and CLI JSON results are **status=BLOCK, admission=NOT_AUTHORIZED**; CLI always exits **2**, even on a successful research negative-control exercise. No `ALLOW`, `APPROVE` or `UNREVOKE` behavior is implemented.

All **12 native admission gates UNMET**; **PRODUCTION NOT_AUTHORIZED**. This milestone cannot be promoted based solely on test fixture cryptographic signatures.

## Expected research acceptance

- Exactly five additive files in a stacked **DRAFT, UNMERGED** PR: this dossier, real SQLite/process lab, 24 actual tests, explicit missing-real-custodian JSON and dedicated 4-OS workflow. No edits to earlier science/workflows, `.noi`, main, native kernel/compiler/runtime or owner original worktree.
- CI: dedicated macOS Intel, Apple Silicon, Ubuntu, Windows **4/4 actual SUCCESS** plus general NORDOI Test Gate **5/5 actual SUCCESS**. Check all R5K–R4Y research Python suites unchanged; Windows skips must be explicitly reported.
- Owner Mac Intel 14.5 exact-head disposable clone and SHA256 manifest **93 pinned Git blob identities** (24 Rust R4A–R4X, 5 R4Y, 4 R4Z, and five per R5A–R5L). **583 regular tests expected PASS** = 24 new R5L + prior R5K 559; 3 intentionally ignored Rust, Rustfmt/Clippy strict PASS; real R5L process SIGKILL and restart tested; exact HEAD unchanged and original checkout untouched, `SCRIPT_EXIT=0`.
- The owner ZIP is **flat** with its five files directly at archive root; external outer SHA256 verified. Extract only into a new unique `mktemp -d` folder, `cd "$WORK"` directly, without replacing anything or invoking unzip replacement prompts.
- Owner R5L acceptance **PENDING** until full Mac Intel run log. GitHub CI cannot replace owner hardware execution; neither constitutes an outside independent guardian.

```ini
G03_R5L=RESEARCH_CANDIDATE
REAL_SIGKILL_UNCOMMITTED_SQLITE_TRANSACTION=ROLLED_BACK
PRECRASH_COMMITTED_DENIAL=LOCAL_DURABLE
RESTART_NEW_PROCESS=DENY_ONLY
COMPETING_SAME_NONCE_PROCESSES=ONE_LOCAL_DENY_ONE_REPLAY_BLOCK
SAME_ADMIN_OLD_SQLITE_RESTORE=NOT_GLOBALLY_DETECTABLE
INDEPENDENT_GUARDIAN=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
