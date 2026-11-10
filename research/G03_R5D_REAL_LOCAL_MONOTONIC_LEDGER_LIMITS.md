# NORDOI G0.3 R5D — Real SQLite monotonic lab and independent antirollback gap

**RESEARCH ONLY. LOCAL durable storage is NOT an independently protected remote witness. NO production authority.**

## Immutable scientific baseline

Direct research successor to R5C PR #94 head `639272ce393f6005a93cf08da95e76859851e0f4`. Owner Intel macOS 14.5 R5C replay: **367/367 regular tests PASS** (R5C 24, R5B 24, R5A 24, R4Z 24, R4Y 18, 253 Rust), **3 intentionally ignored** Rust, **48/48 Git blob pins**, Rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`. R5C GitHub dedicated CI 4/4 SUCCESS and general Test Gate 5/5 SUCCESS. Neither source nor prior research tests may be rewritten by R5D.

## What this stage genuinely executes

R5D experiments with **actual local file-backed Python sqlite3/SQLite** using transactions and actual operating-system files, not in-memory lists or generated pseudo-evidence. An explicit caller-provided private temporary directory holds the ledger named `r5d_lab.sqlite3`. The module configures `PRAGMA synchronous=FULL`, rollback-journal mode DELETE, `BEGIN IMMEDIATE`, a single-row height/epoch/head metadata table, an append-only-by-application event table with **UNIQUE nonce**, and a SHA-256 predecessor-linked sequence across all accepted local events. It rereads and verifies the entire chain and metadata before writing each event and again before committing. Transactions commit to an actual local SQLite file; new processes reopen and inspect that file.

**Scope of durability:** the configuration uses SQLite's FULL synchronization setting. Tests verify reopening across processes and concurrent database transactions. They **do not prove resistance to loss of power, filesystem failure or malicious administrator control**. The directory and filename are caller-controlled and file permission checks are only local hygiene.

For one local valid event, the module reports:
- `status=LAB_LOCAL_COMMIT`,
- `admission=NOT_AUTHORIZED`,
- `reason=ACTUAL_SQLITE_FULL_SYNC_LOCAL_ONLY`,
- a local sequence/epoch/head for reproducibility,
- `global_antirollback=NOT_IMPLEMENTED`.

For an invalid event (duplicate locally consumed nonce, stale predecessor, epoch downgrade, broken chain or malformed data) the event is rejected **BLOCK / NOT_AUTHORIZED**. Both cases intentionally exit **2** at the CLI, never an operational success code.

## Negative scientific control: what local durability cannot guarantee

**The crucial R5D test actually restores an older copy of the local SQLite file and successfully opens that self-consistent older history.** Local metadata and hashes cannot distinguish an authenticated past snapshot from the most recent true state when the same publisher controls all copies. This is a **real demonstrated rollback vulnerability**, not an antirollback success. The overall authority remains `BLOCK` even when a local state is internally consistent.

A second test launches **two separate actual Python processes** that try to consume the identical fresh nonce against the same SQLite database. Exactly one local append commits and one fails, showing real SQLite local serialization. That does not establish durable global one-use consumption after filesystem restoration or across independent remote operators.

## Adversarial tests (24 new Python tests)

1. Initial real SQLite genesis has no external highwater.
2. Actual SQLite FULL-sync transaction adds one local event but grants nothing.
3. Reopening retains committed chain and nonce.
4. Duplicate nonce after reopening is refused.
5. Same nonce with different payload is refused.
6. Stale predecessor/fork rejected.
7. Contiguous new predecessor accepted locally with distinct head.
8. Epoch downgrade rejected.
9. Epoch advance survives reopening.
10. Invalid/boolean/negative epoch rejected.
11. Bad/noncanonical/zero nonce refused.
12. Invalid payload or predecessor digest refused.
13. Direct SQLite event modification detected on reopening.
14. Direct SQLite metadata modification detected.
15. Event deletion refused when metadata disagrees.
16. Injected unlinked event refused.
17. **Actual older-file snapshot restoration remains undetectable locally** and external authority stays BLOCK.
18. **Two real processes** competing for one nonce: exactly one local commit.
19. Symlink database path refused.
20. FIFO/nonregular database refused without blocking.
21. POSIX owner-only database/directory permissions checked (skip on Windows).
22. POSIX insecure directory refused (skip on Windows).
23. Real inspect CLI returns BLOCK with exit 2.
24. Real local append CLI still returns exit 2 and NOT_AUTHORIZED.

Windows may legitimately skip POSIX mode tests 21–22; Unix platforms must exercise them. No test should misreport intentionally skipped cases as executed.

## Requirements R5D does NOT satisfy

- **G1** Real independent operator, administration, signing and witness custody not enrolled.
- **G2** Owner-approved authentic OOB trust root still not enrolled.
- **G3** Independently protected remote monotonic highwater, non-equivocating append-only witness and externally durable consumed nonce ledger NOT IMPLEMENTED. The local SQLite lab is not its replacement.
- **G4** Deployed verifier and owner-approved trust pin not protected against unilateral primary-admin modification.
- **G5** Authenticated independent revocation, compromise, recovery, outage and key-rotation governance absent.
- **G6** True external signed provenance and independently reproducible third-party custody attestations absent.

No secrets, identities or private third-party records belong in this public repository. No externally protected infrastructure is instantiated, no guardian is contacted, and no production/admission gate is opened.

## Qualification and owner replay

The research PR must be a DRAFT child of R5C, with exactly five additional files: Python local SQLite lab, 24 tests, truthful no-external-witness JSON, this dossier and dedicated four-platform CI. Existing scientific sources, production code, `.noi`, `main`, old CI and original owner worktree unchanged.

- Dedicated real GitHub CI: 4/4 complete successful jobs (macOS Intel, Apple Silicon, Ubuntu and Windows) run R5D new tests and all R5C/R5B/R5A/R4Z/R4Y Python regressions; Windows platform exclusions reported accurately.
- General NORDOI Test Gate: all five real jobs PASS.
- Owner Mac Intel replay: exact branch SHA + R4U observer head pinned; **53 Git blobs** (24 Rust R4A–R4X + 5 R4Y + 4 R4Z + 5 R5A + 5 R5B + 5 R5C + 5 R5D), isolated clean git checkout, R5D 24 + R5C 24 + R5B 24 + R5A 24 + R4Z 24 + R4Y 18 + prior Rust 253 = **391 regular tests PASS**, 3 deliberately ignored Rust, strict Rustfmt/Clippy, `SCRIPT_EXIT=0`.
- A passing laboratory replay can only yield **PASS_WITH_CONTROLS_RESEARCH_ONLY**, never independent external custody admission. A current owner Mac replay must actually be observed before that scientific verdict.

## Authoritative current state

```ini
G03_R5D=RESEARCH_CANDIDATE
LAB_SQLITE_TRANSACTIONS=REAL_LOCAL_DISK
LAB_LOCAL_NONCE_CONSUMPTION=IMPLEMENTED
RESTORED_LOCAL_SNAPSHOT_ROLLBACK=UNDETECTABLE_WITHOUT_EXTERNAL_ANCHOR
REMOTE_INDEPENDENT_MONOTONIC_WITNESS=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
REAL_INDEPENDENT_CUSTODIAN=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
