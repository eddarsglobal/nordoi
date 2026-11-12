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

R5T **initially added five research-only files**; FIX1 also adjusts exactly one inherited R5S adversarial test assertion without changing its protocol or production source: Python receipt engine, 24 tests, authority-absence JSON, this dossier and 4-OS CI workflow; original R4A–R5S runtime/protocol source and workflows (one inherited R5S test assertion explicitly corrected by FIX1), `.noi`, kernel/compiler/runtime, main and owner worktree untouched. Stack R5T DRAFT PR directly on R5S owner validated source, do NOT merge.

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

## R5T FIX1 — Existing R5S Windows negative test assertion ordering (2026-11-12)

Initial exact R5T candidate `8533561a97bc8ab306cadf90794021b8f56f0af2` ran the new **16 portable R5T tests PASS** and Ubuntu native R5T **24/24 PASS**. The dedicated Windows run **FAILED on an inherited R5S test** `test_13_missing_an_asset_in_archive_is_quarantined`, *not* on a new R5T test: Windows returned `HANDOFF_TRANSFER_QUARANTINED` with `detail=TRANSFER_FILE_CHANGED_WHILE_READING` rather than the older test's single expected `ARCHIVE_MUST_CONTAIN_EXACT_FOUR_FIXED_NAMES` reason. The malformed archive was genuinely refused in both cases, without any admission; nevertheless **the original CI result is a real failure and cannot be called a pass**.

FIX1 performs one tightly bounded **test-only refinement** on the inherited `research/test_g03_r5s_challenge_bound_handoff.py` **in this R5T candidate branch only**. The frozen parent R5S PR #110 and its independently owner-tested commit `ebe695680e1992261f55a7d534fa9505c9133bc5` remain unchanged. The refined test explicitly invokes the **pure R5S decoder on the actual malformed ZIP bytes** and requires the exact `ARCHIVE_MUST_CONTAIN_EXACT_FOUR_FIXED_NAMES` rejection; it separately tests the opened-FD/file-identity path and requires `HANDOFF_TRANSFER_QUARANTINED` with *either* the exact malformed-shape refusal or the earlier fail-closed `TRANSFER_FILE_CHANGED_WHILE_READING` race. It additionally requires `status=BLOCK`, `admission=NOT_AUTHORIZED`, `native_authority_granted=false`. **No test skipped, expected-failure annotation, second-attempt/waiver, permission check modification, accepted malformed archive, or old R5S production code change.** The corrected test remains within the full R5S 24-test suite.

This changes a previously pinned **single inherited R5S test Git blob** in the R5T branch. The new R5T owner manifest must pin its exact new Git hash and **must NOT state all 128 parent blob hashes are unchanged**. There are still exactly **133 source blob paths**, but *one historical test pin differs from the frozen R5S owner-approved commit*; that change is disclosed in CI, the PR and replay dossier. Both initial CI failed case and a subsequent new exact-commit 4/4+5/5 verification are preserved separately. Owner replay remains PENDING until real hardware log. Final PR changes are **six files: five additive R5T research files plus one explicit R5S test-only FIX1**.

## R5T FIX2 — genuinely reach Ed25519 verification on canonical tampering

FIX1 dedicated CI at candidate SHA `91a1ddafd9bf9581e1fe4a3bec17d1423c1d275c` verified **four of four Mac Intel, Apple Silicon, Ubuntu and Windows jobs SUCCESS**, including actual Mac Intel R4M six signed XPC/CDHash tests PASS. But the original R5T test named "tampered genuine signed receipt" changed primary height; that mismatched the real R5S transfer and was rejected **before** the SSHSIG cryptographic verifier. An early conservative refusal is safe but is not evidence that the **actual signature verification** rejected a well-formed, context-consistent altered message.

FIX2 changes only the **new R5T module and its R5T test file**: alter a single signed `received_at` value from 1100 to 1101, keeping the message canonical, exact R5S ZIP/manifest, real SQLite height, SHA256 binding, challenge, receipt expiry and local timebox all valid. The original genuine `ssh-keygen -Y sign` signature bytes remain unchanged. Require the actual `ssh-keygen -Y verify` call to reject with `REAL_SSHSIG_SIGNATURE_INVALID_OR_UNTRUSTED_LOCAL_KEY`; require the final receipt to remain `SIGNED_RECEIPT_QUARANTINED / BLOCK / NOT_AUTHORIZED`. Apply the same check inside the full native `--exercise`, not just a unit test. **No retries, skipped native tests, new grants, weaker key/trust matching, changes to parent R5S production code, or XPC fixtures.** The original FIX1 branch history and initial Windows failure remain documented.

Owner kit and test evidence must now pin the **new exact FIX2 commit and new R5T Python source/test Git blobs**, preserving the explicitly disclosed one inherited R5S test-only correction (133 source file paths, still 6 PR changed files). Re-run dedicated four-OS and general gate on **exact FIX2 SHA** before declaring full CI completion. Owner Mac Intel full replay remains PENDING until the owner supplies its actual log.
