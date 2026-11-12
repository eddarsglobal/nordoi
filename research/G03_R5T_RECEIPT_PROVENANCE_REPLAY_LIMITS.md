# NORDOI G0.3 R5T — Genuine signed *local* transfer receipts: provenance and replay boundaries

**DRAFT / RESEARCH ONLY / ALL LOCAL DECISIONS BLOCK. An Ed25519 signature by an owner-controlled test key is NOT an independently authenticated guardian receipt.**

## Source and conservative continuity

R5T stacks directly on owner-tested R5S PR #110, frozen source SHA `ebe695680e1992261f55a7d534fa9505c9133bc5`, with **751 regular tests PASS** on owner macOS 14.5 Intel x86_64 (498 Python + 253 Rust), 128/128 source Git blob hashes PASS, three Rust intentionally ignored, real R4M signed macOS XPC/CDHash six PASS serialized with no retry/waiver, Rustfmt and strict Clippy PASS, owner `SCRIPT_EXIT=0`, and dedicated four-OS CI 4/4 plus Test Gate 5/5 SUCCESS. R5S PR stays DRAFT/UNMERGED.

R5S already established a real four-entry archive containing a manifest plus three actual SQLite/JSON evidence files. It bound a fresh local challenge, exact source lineage, SHA256 assets, application-level history and local claimed timebox, and used a separate OS process and a local durable SQLite spent-challenge denial journal. The technical limitation remained: **neither local transport integrity, different processes, nor a local spent-nonce register proves an external independently governed historical witness**. It remains possible to verify an older coherent bundle after a later world and to restore an older same-admin nonce journal.

## What R5T actually implements and what it does not

R5T generates a **real OpenSSH Ed25519 keypair** inside one disposable 0700 research directory using `/usr/bin/ssh-keygen -t ed25519`. It signs actual canonical receipt bytes with real `ssh-keygen -Y sign -n nordoi-r5t`; the verifier invokes genuine `ssh-keygen -Y verify` against the corresponding **same-admin-created `allowed_signers` file**. A valid signature is cryptographic evidence of **possession of that local private key only**. It is not external custodian enrollment, a genuine owner-OOB authenticated pin, external time attestation, quorum, trusted identity, or a protected monotonic root.

The canonical `NORDOI-G03-R5T-SIGNED-LOCAL-RECEIPT-V1` message pins: the exact parent R5S Git SHA, research-only `SIGNED_LOCAL_RECEIPT_BLOCK_ONLY` operation, explicit same-admin fixture signer and signature namespace, the **actual R5S archive SHA256**, SHA256 of its exact canonical manifest, the actual **R5Q application projection SHA256**, exact 64-lowercase-hex challenge, primary R5L committed height and local (non-authoritative) signed-receipt timebox. It does not carry an authorization opcode, production trust-store key or remotely enrolled guardian ID.

The read-only verifier rejects duplicate/noncanonical JSON, unknown fields, wrong project/parent/namespace/signer/opcode, non-SHA values, Boolean/negative counters, backwards untrusted time, missing or swapped source file identity, altered receipt/signature, wrong untrusted allowed-signer file, bad cryptographic signature, mismatch to the actual R5S archive, incorrect challenge and real SQLite/JOSN history, and expiring/future local time. Valid signature + all hashes + native OpenSSH returns:

```ini
reason=REAL_LOCAL_ED25519_SIGNATURE_VALID_NO_EXTERNAL_AUTHORITY
cryptographic_signature_verified=true
independently_authenticated_signer=false
independent_custodian=NOT_ENROLLED
external_signed_guardian_receipt=NONE
native_authority_granted=false
admission=NOT_AUTHORIZED
status=BLOCK
```

With an existing R5L SQLite local denial register, an actual receipt hash plus nonce creates **only a local BLOCK record**; replaying the same signed challenge is refused. **An actual second OS process may restore the register's older SQLite file**, after which the exact valid signature/nonce is accepted again **only for another local denial**. This is a scientifically honest demonstration that a local register cannot establish globally durable spent-receipt nonces.

R5T also checks that an actual later world B does not inherently invalidate an old cryptographically valid receipt over a self-consistent R5S world-A bundle. A **second real local Ed25519 keypair** can sign the same local receipt and be verified with an *equally same-admin authored allowed-signers file*: true local key rotation, but **no independent provenance**. A second actual OS process atomically swaps a byte-identical receipt path while an opened FD is held: R5O pathname/FD inode mismatch is quarantined.

This is **NOT a trusted external receipt protocol**. The project has not installed, contacted or enrolled any external provider, custodian, guardian service, legal operator, owner authenticated OOB root, externally signed challenge, globally protected high-water, globally durable spent nonce service or real independent quorum. R5T has no network, no external provider/API, no native GRANT branch and makes no claim of hardware power-cut tests. The locally provided timestamp is NOT an authenticated clock; the self-authored `allowed_signers` is explicitly NOT a trust root. All twelve actual native admission gates remain **UNMET**, production **NOT_AUTHORIZED**.

## Exactly 24 new research tests

**16 portable:** actual R5S SQLite/JSON ZIP and canonical receipt SHA/lineage/challenge/height/time binding; exact deny-only opcode and parent; wrong challenge/expired time; duplicate fields, noncanonical JSON, extra fields, grant/enroll or wrong source pin, wrong signer/SSH namespace, bad SHA-256 fields, Boolean/noninteger/negative timestamps and heights, backwards timebox, truncated/oversized payload, an **old fully coherent ZIP after actual newer history B** still yielding the old legitimate message, corrupted actual ZIP cannot form a valid receipt, missing signature/allowed-signers always refused, separate real Python CLI never fabricates a valid signature and returns exit 2.

**Eight native POSIX experiments** on macOS Intel, Apple Silicon and Linux (Windows intentionally SKIPS eight): real local OpenSSH Ed25519 signing and verification yet no independent guardian; authentic signed receipt context altered and refused; wrong real local public key refused; genuine SQLite first nonce refusal and second signed challenge replay blocked; real second OS process restores the earlier local spent-nonce SQLite and repeat challenge yields **another BLOCK only**; old A receipt verifies after newer world B is genuinely committed (no global external highwater); a different genuine locally generated signing key can verify another receipt but is still not independent; real second OS process performs byte-identical atomic receipt swap during held FD check followed by full `--exercise` (exit **2**). Original R4M true XPC/CDHash signed peer suite all six with `--test-threads=1`, no waiver or test retry. All R5S–R4Y regression experiments remain unchanged.

## CI, Mac Intel and governance acceptance

R5T adds **exactly five research-only files**: Python receipt engine, 24 tests, authority-absence JSON, this dossier and 4-OS CI workflow; original R4A–R5S source/workflow, `.noi`, kernel/compiler/runtime, main and owner worktree untouched. Stack R5T DRAFT PR directly on R5S owner validated source, do NOT merge.

Dedicated real GitHub CI Mac Intel, Apple Silicon, Ubuntu, Windows **4/4 SUCCESS required**, including all new R5T native suites on Unix, and Test Gate **5/5 SUCCESS required** at exact frozen R5T Git SHA. Owner replay baseline macOS 14.5 Intel x86_64, rustc/cargo 1.98.1. Target **133 exact source Git blob pins** = 24 Rust R4A–R4X, five R4Y, four R4Z, five each R5A–R5T; expected **775 regular tests PASS** = **522 Python** (R5T 24 + previous R5S 498) and **253 Rust**, 3 ignored Rust, genuine R4M XPC 6 serial with no retry/waiver; strict Rustfmt and Clippy PASS, live R5T `--exercise`, previous real negative controls, owner source checkout unmodified and expected `SCRIPT_EXIT=0`. Do NOT report owner run completed until owner supplies actual Mac evidence.

Five owner ZIP archive-root files, outer SHA256 check, fresh unique `mktemp -d` extraction, internal 4/4 SHA256SUMS, `bash -n`, then runner. No overwrite or production permission. Certification is held by the RoadMap-named engineers. The exact limitations are machine-readable in `G03_R5T_SIGNED_RECEIPT_NO_EXTERNAL_AUTHORITY.json`.

```ini
G03_R5T=RESEARCH_CANDIDATE
REAL_ED25519_SSHSIG_LOCAL_RECEIPT=VALID_KEY_POSSESSION_ONLY
ALTERED_RECEIPT_SIGNATURE_OR_WRONG_SIGNER=QUARANTINE
SIGNED_RECEIPT_WRONG_ZIP_CHALLENGE_OR_HISTORY=QUARANTINE
SAME_ADMIN_LOCAL_SPENT_RECEIPT_NONCE=REPLAY_BLOCKED_WHILE_INTACT
REAL_SAME_ADMIN_SPENT_NONCE_ROLLBACK=REPLAY_POSSIBLE_LOCAL_DENIAL_ONLY
OLD_AUTHENTIC_SIGNED_BUNDLE_AFTER_REAL_ADVANCED_HISTORY=VERIFIABLE_LOCALLY
NEW_ACTUAL_SIGNING_KEY_UNDER_SAME_ADMIN=NOT_INDEPENDENT
REAL_SECOND_PROCESS_RECEIPT_INODE_SWAP=QUARANTINE
AUTHENTICATED_EXTERNAL_CUSTODIAN=NOT_ENROLLED
EXTERNAL_SIGNED_GUARDIAN_RECEIPT=NONE
OWNER_OOB_ROOT=NOT_ENROLLED
GLOBAL_PROTECTED_NONCE_AND_HEIGHT=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
