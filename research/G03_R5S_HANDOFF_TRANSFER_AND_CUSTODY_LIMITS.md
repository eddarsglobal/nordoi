# NORDOI G0.3 R5S — Genuine offline challenge-bound handoff without independent custody

**DRAFT research only / NO NATIVE ADMISSION.** This is an executable protocol for **local transport integrity**, **not** a declaration that a real legal or administrative third-party guardian has received or authenticated anything.

## Frozen source lineage and purpose

R5S stacks directly on owner-validated R5R PR #109 source SHA `e9b10fbc0ec51f243399ea636c5d2b3fbcc3c6eb`. Owner macOS 14.5 Intel x86_64 replayed R5R with **727 regular tests PASS** (474 Python +253 Rust), **123/123 exact Git source blob pins PASS**, 3 deliberately ignored Rust, genuine signed XPC R4M 6/6 PASS serialized `--test-threads=1` with no retry/waiver, strict Rustfmt and Clippy PASS, `SCRIPT_EXIT=0`. Dedicated GitHub CI 4/4 and general Test Gate 5/5 PASS. PR #109 remains DRAFT/UNMERGED.

R5Q and R5R established that (a) three coherent same-admin local files can be restored after later durable history and (b) local Ed25519 signing, all twelve self-declared custodian conditions and a different local OS process **cannot** establish independent historical custody or global nonce protection.

R5S implements an **actual byte-level transfer** of three real files into a deterministic four-entry ZIP, verified by a different real OS process and a real local SQLite duplicate-nonce journal, while demonstrating the precise antirollback and independent-custody limitations.

## Wire and transport contract — V1, no online provider

The archive contains exactly these **four static paths** (no directory traversal and no extraction of untrusted paths):

```text
manifest.json
evidence/primary.sqlite3
evidence/local_witness.json
evidence/spent_nonces.sqlite3
```

The canonical strict UTF-8 manifest pins `NORDOI-G03-R5S-CHALLENGE-BOUND-HANDOFF-V1`, project, exact frozen R5R Git SHA, `OFFLINE_EVIDENCE_TRANSPORT_DENY_ONLY`, a 64-lowercase-hex challenge, untrusted local integer `created_at` and `expires_at` (1–300 seconds difference), SHA256 of the full R5Q **application-level historical projection**, and a SHA256 of each of three actual original evidence files. No secret, user credential, trusted signature or remote receipt is generated. Timestamps are **not a trusted independent time source**, and SHA256 proves integrity only against the manifest in the same untrusted archive. An actor who can rewrite both the archive and its manifest can create a new self-consistent archive. This mechanism **never authenticates a remote operator**.

The sender reads and verifies real R5Q primary SQLite (R5L hash chain) + separate R5M JSON checkpoint + real R5L SQLite spent nonce journal, compares real file bytes/FD identities and the R5Q historical projection before and after, enforces private disposable lab paths, writes a **new** ZIP with exclusive creation (no overwriting old source files, app checkout or real secrets). The research archive is never transmitted over an API or externally entrusted; `--inspect` is implemented as a local process protocol.

The receiver verifies exact four entries, no duplicates, no compression or encrypted entries, size limits and path allowlist, strict canonical manifest and challenge, untrusted timebox, SHA256 of all three included raw assets, then materializes **only its own private temporary directory** and checks the actual SQLite data and R5Q hash-chain/witness/nonce evidence by unchanged live research verifiers. It returns **`BLOCK/NOT_AUTHORIZED`**, never GRANT, even when all bytes are consistent.

With an explicitly existing private R5L SQLite receipt ledger, a successful observation writes **only a local denial record** for the 64-hex challenge. A second receipt with the same nonce returns a BLOCK with `ALREADY_RECORDED_DENIAL_NONCE`; missing receiver register is refused rather than silently recreated. A distinct real OS process can validate the transfer via `--inspect`; this is genuine transport/inter-process verification but **not a distinct security administration**. The resulting receipt is an **unsigned local JSON observation, not a guardian-signed receipt**.

## Real negative experiments and known limits

**16 portable actual-file cases** on Windows/macOS/Linux: real 3-file archive and proper SQLite header, canonical signed-source/challenge and all SHA256 fields, local receipt ledger first deny and same nonce replay refused, wrong challenge, expired/future local time, impossible >300s TTL or Boolean timestamp, duplicate/noncanonical manifest, damaged ZIP, mutated payload hash, forged R5Q historical projection digest, missing archive asset, malicious path-traversal ZIP member with no extraction, corrupt SQLite accompanied by a recomputed matching asset hash but inconsistent verified history, and real new Python receiver process exit **2 / NOT_AUTHORIZED**.

**Eight POSIX genuine OS experiments** on Mac Intel/Apple Silicon/Ubuntu: real second process receiving all actual files and writing a local SQLite denial, second process replay challenge refusal, same old A complete archive remains locally self-consistent after a real later world B is committed (proves **no external highwater**), distinct new world B bundle differs yet still only denied, actual different process restores a previous valid local receipt-ledger SQLite state and the same challenge can once again be recorded as local BLOCK, actual other-process `os.replace` of an archive with identical bytes while FD held is quarantined, separate process receipt states no independent custodian/monotonic root, and complete real `--exercise` exit **2** demonstrating those controls. Windows executes **16 portable tests and intentionally skips eight POSIX-specific tests**; skip is not counted as execution.

**Key limits:** contents are **not encrypted or signed by a real guardian**, challenge is **not generated by a real third party**, local clock is not authoritative, local SQLite spent-challenge register may be coherently rolled back under same admin, an old complete bundle is self-consistent and can appear valid without protected outside high-water, and this lab deliberately never sends data to a real external operator or uses a service/API/network. Neither the different process, the offline archive, the SHA256, nor the local denial receipt creates a legal and independently controlled custodian. No globally monotonic nonce guarantee, owner OOB root, signed independent receipt, quorum, protected revocation floor, custody audit or production admission is established.

The machine-readable `G03_R5S_NO_INDEPENDENT_RECEIPT_AUTHORITY.json` keeps all **12 native admission gates UNMET**, and names every absent external qualification, including genuine independent signed challenge receipt, owner OOB root, globally monotonic height/revocation/nonces, independent quorum and recovery. All verdicts `BLOCK / NOT_AUTHORIZED`; **`PRODUCTION=NOT_AUTHORIZED`**.

## CI and owner reproducibility

**Exactly five additive research files:** transfer protocol module, 24 negative controls, absent-real-authority JSON, this dossier and four-OS GitHub workflow. Earlier R4A–R5R science/workflows, `.noi`, compiler/kernel/runtime, main/release, original owner worktree and production unchanged. DRAFT stacked PR unmerged.

Dedicated CI Mac Intel, Apple Silicon, Ubuntu and Windows **actual 4/4 SUCCESS** plus general NORDOI Test Gate 5/5 at frozen exact R5S head. Preserve R5R–R4Y research regression chain. Both macOS jobs run genuine R4M native XPC/CDHash six-test suite with **`--test-threads=1`**, no retry, waived test, relaxed acceptance or original R4M source changes.

Owner Mac Intel macOS 14.5 x86_64, rustc/cargo 1.98.1, original isolated exact source pin + R4U observer SHA, **128 Git source blob pins** = 24 Rust R4A–R4X + 5 R4Y + 4 R4Z + five each R5A–R5S. Expected **751 regular tests** = 498 Python (24 new R5S + previous 474) + 253 Rust, three intentionally ignored Rust, Rustfmt/Clippy strict PASS, R4M 6 serial PASS, full actual R5S `--exercise` and earlier real negative controls, original owner worktree untouched and `SCRIPT_EXIT=0`. **Do not claim owner PASS before owner log.**

Five-file ZIP at archive root, verified outer SHA, unique `mktemp -d` fresh extraction, 4/4 inner SHA256SUMS, `bash -n`, then owner runner; no overwrite/no merge. Formal certification belongs to named RoadMap engineers, not a CI status. No native admission as long as 12 actual external gates remain unmet.

```ini
G03_R5S=RESEARCH_CANDIDATE
REAL_LOCAL_3_FILE_HANDOFF=ARCHIVE_AND_RECEIVER_SQLITE_VERIFIED
REAL_SECOND_OS_PROCESS_RECEIPT=BLOCK_ONLY
DUPLICATE_CHALLENGE_LOCAL_REGISTER=REJECTED
SAME_ADMIN_LOCAL_RECEIPT_REGISTER_RESTORE=REPLAY_LOCALLY_POSSIBLE
OLD_COHERENT_TRANSFER_AFTER_REAL_NEW_WORLD=NOT_GLOBALLY_DISQUALIFIED
REAL_INDEPENDENT_CUSTODIAN_RECEIPT=NONE
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_HEIGHT_OR_NONCE=NOT_PROVEN
REAL_EXTERNAL_OPERATOR=NOT_ENROLLED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
