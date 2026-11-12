# NORDOI G0.3 R5U — Genuine dual-signature key rotation, local revocation and custody limits

**RESEARCH ONLY — no native admissions, no independent external guardian enrollment, no production.**

## Frozen source and parent evidence

R5U stacks strictly on R5T FIX2 PR #111, **owner-validated commit `3372ddbd691d5c6102bd395b4589c7a27ec030b3`**. Owner macOS 14.5 Intel x86_64 / Rust-Cargo 1.98.1 genuinely replayed R5T FIX2: **775 regular PASS** (522 Python +253 Rust); **133/133 exact source Git blob pins PASS**; real native R4M signed XPC/CDHash six test controls PASS sequentially with no waiver or retry; three intentionally ignored Rust; Rustfmt and strict Clippy PASS, `SCRIPT_EXIT=0`. Its dedicated R5T 4/4, inherited R5S 4/4 and general Test Gate 5/5 GitHub jobs all SUCCESS, **13/13 actual jobs**. Original R5T initial Windows negative assertion failure, FIX1 correction in inherited R5S test, and cryptographic tamper FIX2 remain documented and unchanged. R5T PR remains DRAFT/UNMERGED.

## R5U research problem

R5T proved that a *genuine* signed receipt with a valid Ed25519 signature under a local same-admin allowlist binds its bytes and transfer context but **does not establish a cryptographically authenticated independent custodian**. To qualify a next step without inventing an operator, R5U constructs an *actual two-key local change of custody* and demonstrates precisely what local rotation/revocation can and cannot attest.

The laboratory uses two **distinct real OpenSSH Ed25519 keypairs**, generated in a fresh 0700 disposable directory as local/owner controlled fixtures. A strict canonical `NORDOI-G03-R5U-LOCAL-KEY-CUSTODY-CONTINUITY-V1` intent binds exact parent R5T SHA, operation `ROTATE_LOCAL_RESEARCH_KEY_BLOCK_ONLY` or `REVOKE_LOCAL_RESEARCH_KEY_BLOCK_ONLY`, historical R5T receipt SHA, strictly monotonically incremented **local** epoch, prior+next public-key SHA256 pins and an exact 64-character lowercase-hex local challenge nonce.

**Actual rotation requires BOTH** `ssh-keygen -Y sign -n nordoi-r5u` signatures by the distinct prior and new private keys, and separately verifies both through real `ssh-keygen -Y verify` using their corresponding same-admin `allowed_signers`. The verifier checks that the authentic public keys inside those allowlists match the canonical signed intent's SHA256 pins. One signature alone, a wrong signer, a mismatched key pin, identical old/new pins or invalid epoch is quarantined. No absence of an external operator is treated as an implicit grant.

The real `r5u_local_key_lineage.sqlite3` stores a local genesis key pin and an append-only *logical* chained table of actual key transitions, key revocations and consumed local transition nonces. SQLite integrity, hash chain, sequence, previous head, active pin, single-use nonce and revocation terminal state are audited in read-only mode; missing/corrupted registry is rejected, never silently regenerated. A genuine key rotation commits height 1 and the old signer is **locally quarantined**, while the newly active signed R5T receipt still produces **only a local BLOCK**. A genuine successor-key-signed revocation commits height 2 and the successor is also **locally quarantined**. These are research outcomes and not a trusted custodian key lifecycle.

Then **real second OS processes** atomically restore prior SQL ledger snapshots. After restoring the pre-revocation height-1 copy, the formerly revoked successor can again verify as a *local* active key, but results stay DENY-only. Restoring height-0 genesis permits the previously rotated-out older signer again, and the **same already used signed rotation nonce** can be committed a second time locally, still with `BLOCK/NOT_AUTHORIZED`. This demonstrates that a same-admin SQLite hash-chain and signed local rotation/revocation **cannot establish a globally protected key epoch, revocation floor or spent-nonce register**. Distinct actual Ed25519 keypairs, two signatures and separate OS processes do **not** imply administrative independence or independent recovery authority.

## Program of negative tests

R5U adds **exactly five additive research-only files**: actual local-key research Python module, 24 tests, machine-readable external-authority absence JSON containing 12 unmet native gates, scientific dossier (this file), dedicated 4-OS CI workflow. All prior R4A–R5T source, the explicitly corrected R5S FIX1 test, `.noi`, native runtime/kernel/compiler, main, production and owner original worktree unchanged.

**16 portable** tests check actual SQLite file initialization and read-only durable local genesis audit (not synthetic report); exact protocol/source and canonical signed-intent schema; revocation cannot change the identity pin; reject GRANT/ENROLL or unknown opcode, identical-key rotation, malformed key/receipt/nonce digests, invalid Boolean/negative/noninteger epoch, forged project/source, duplicate/unknown/missing fields and bad UTF8; absent/corrupt SQLite ledger never autoinitializes; real SQL genesis corruption detected; no forged absent-signature rotation can be committed; and a new actual Python process audits only the local file and returns `BLOCK` and exit 2.

**Eight genuine POSIX native** controls on Mac Intel/Apple Silicon/Ubuntu require two different actual SSHSIG Ed25519 keys signing the *same exact* canonical context, reject a one-signature rotation, reject wrong signer/allowed key, rotate old out/new into local denial state, sign and commit actual successor-key revocation, restore the previous SQLite snapshot via an actual other OS process and observe revocation lost locally, restore genesis and observe old key return and signed old nonce replay as another local refusal, and run the complete actual CLI `--exercise` with exit **2**, verifying a truthful denial-only summary. Windows runs **16 genuine portable tests**, eight POSIX-only native signature tests deliberately SKIPPED, never represented as executed.

Dedicated CI should run all 24 R5U tests plus unchanged R5T–R4Y regressions across **macos-15-intel, macos-15, ubuntu-latest, windows-latest**; both macOS jobs must run genuine native original R4M XPC/CDHash six test controls with `--test-threads=1`, no retries/waivers. General Test Gate must check original Rust source, Clippy, formatting and platform tests. No false independent guardian assertions.

## Acceptance / current owner target

Owner macOS 14.5 Intel x86_64 and Rust/Cargo 1.98.1, exact pinned remote candidate SHA + same-owner observer `15adf075050a233705b3bad6eae62b6f9e6f946d`, fresh isolated clone, **138 expected exact source Git pins** = 24 Rust R4A–R4X, five R4Y, four R4Z, five each R5A–R5U (including the disclosed R5S inherited test FIX1 pin). **799 expected regular tests** = 546 Python (24 new R5U + previous 522 R5T–R4Y) +253 Rust; three Rust intentionally ignored; original real R4M XPC signed-peer tests six serial once, no retries, Rustfmt/strict Clippy PASS, live R5U `--exercise` and all older live negative controls, exact clone unchanged, original worktree untouched and `SCRIPT_EXIT=0`. These are acceptance **targets until owner execution is actually evidenced**.

Owner package must contain five files **at ZIP root**, verify outer SHA256, unique `mktemp -d` extraction, internal 4/4 SHA256SUMS and Bash syntax before actually running owner real Mac tests. PR DRAFT and UNMERGED, no production access, no automatic admission. Formal stage certification remains with RoadMap-designated engineering Bureau.

## Security and scientific limitations

The lab does **not** authenticate independent control of private keys, out-of-band owner pinning, remote legal custodial status, independent quorum, a trusted external timestamp, cross-domain signed key event feed, externally durable monotonic key epoch/revocation floor or globally durable consumed-nonce register. Local signatures and the local SQLite history share one administrative control domain and are rollbackable. Every successfully verified lab key still yields a BLOCK result. No key files are distributed outside ephemeral private research fixtures. **All 12 actual native admission gates UNMET, PRODUCTION NOT_AUTHORIZED.**

```ini
G03_R5U=RESEARCH_CANDIDATE
REAL_TWO_DISTINCT_LOCAL_ED25519_SIGNATURES=VERIFIED
ONE_SIGNER_ROTATION=QUARANTINED
OLD_LOCAL_KEY_AFTER_ROTATION=QUARANTINED
CURRENT_LOCAL_KEY=DENIAL_ONLY
SUCCESSOR_SIGNED_REVOCATION=QUARANTINED_LOCAL_USE
REAL_SECOND_PROCESS_REVOCATION_LEDGER_ROLLBACK=REVOCATION_LOST_LOCALLY
REAL_SECOND_PROCESS_FULL_KEY_HISTORY_ROLLBACK=OLD_KEY_LOCALLY_VALID_AGAIN
REPLAYED_OLD_SIGNED_ROTATION_AFTER_ROLLBACK=SECOND_LOCAL_DENIAL
INDEPENDENT_EXTERNAL_KEY_PROVENANCE=NOT_PROVEN
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
PROTECTED_GLOBAL_KEY_EPOCH_AND_REVOCATION_FLOOR=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
