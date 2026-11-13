# NORDOI G0.3 R5Z — Preserved signed local witness and coordinated rollback counterexample

**RESEARCH CANDIDATE — ALL NATIVE ADMISSION DENIED.** Direct child of R5Y validated owner macOS Intel source `fb70bef860353895285e62d14e2f236402999cf7` (PR #116). The owner's submitted Mac Intel log confirmed **895 regular tests PASS**, **158/158 frozen Git blob pins PASS**, genuine R4M signed native XPC/CDHash **6/6 PASS serial**, Rustfmt/strict Clippy PASS, `SCRIPT_EXIT=0`. Dedicated R5Y GitHub CI 4/4 plus general Test Gate 5/5 SUCCESS. Parent PR remains DRAFT and unmerged.

## Why a second history is not enough

R5Y demonstrated a genuinely signed Ed25519 receipt, actual durable local SQLite spent-nonce tracking, and an actual second OS process restoring an old SQLite snapshot such that a formerly spent challenge was locally reusable. R5Z tests the next claim: **does a preserved second copy of the local history make the antirollback global?**

The experiment retains a local witness describing the primary ledger's exact height, ordered challenge-set SHA-256, locally recorded revocation floor, parent source and denied-only status. With the witness at height 2 and a restored primary at height 1, the comparison flags a rollback. However, the **same administrator can atomically restore both primary SQLite and signed witness+signature to the mutually coherent height-1 history**. An old genuine local Ed25519 signature is still cryptographically valid and the deleted second nonce can be committed again. Therefore this is a counterexample to treating a second **same-admin** file, even signed, as an independent monotonic authority.

## Genuine physical observations, not simulations

- Two genuinely separate OS child-process identity observations with the same real UID/EUID and two locally generated, distinct Ed25519 private/public keypairs. No privileged user switching, owner OOB root, external operator or actual external guardian.
- Actual `ssh-keygen -Y sign` and `ssh-keygen -Y verify` with a domain-separated `nordoi-r5z` namespace for both witness histories and genuine R5X signed receipt checks.
- Real SQLite commits using the R5Y durable denial-only spent-nonce register; two different challenges with heights 1 and 2; actual `synchronous=FULL` transactions. A genuine second OS process restores the old primary, then a genuine OS process restores the witness and signature. Local partial rollback is detectable with preserved witness; coherent double rollback is **not independently detectable**.
- The restoration helper is constrained to a disposable 0700 POSIX private laboratory directory with an exact prefix and **fixed filenames**, checks symlinks and 0600 permissions, and never operates on repository data, a production database, external keys, non-lab files, or another account.
- Genuine Ed25519 signatures prove **key possession only**; signer allowlists, local witness, path permissions, clocks, nonce counters and role labels cannot establish independent administrative authority.

## Schema and refusal rules

The R5Z canonical witness includes project, exact R5Y commit, local height, source signed receipt digest, SHA-256 over ordered spent challenges, local revocation floor, observed local clock and explicit **SAME_ADMIN_LOCAL_ONLY** custody. Owner OOB root and external guardian fields must be null; admission must remain **NOT_AUTHORIZED**. Duplicates, extra/missing fields, noncanonical JSON, invalid UTF-8, NaN, spoofed source, fake guardian identity, privileged operation, negative/Boolean counters and bogus hashes are refused. Comparisons and signature checks always return **BLOCK**.

The accepted negative categories are: `COHERENT_SAME_ADMIN_LOCAL_HISTORY_NOT_EXTERNAL`, `PRIMARY_AHEAD_OF_OLDER_LOCAL_WITNESS`, `PRESERVED_LOCAL_WITNESS_DETECTS_PRIMARY_ROLLBACK`, `SAME_HEIGHT_OR_FLOOR_HISTORY_MISMATCH`, and `REAL_LOCAL_WITNESS_SIGNATURE_VALID_NOT_EXTERNAL`. None is an admission or an independent trust-root attestation.

## Tests and formal gates

**24 R5Z adversarial cases:** 16 portable source/schema/private-lab/real SQLite and local witness tests, and 8 genuine POSIX native OpenSSH/OS cases. The native cases include real valid signed witness, tampered signed bytes, mismatched allowed signer, real second-process primary restoration, coordinated second-process signed witness restoration, fixed-path restrictions, false independent identity and full native CLI exit code **2**. Windows runs the 16 portable tests and intentionally skips the eight POSIX-only tests, with no fabricated native observation.

Dedicated CI covers macOS Intel, macOS Apple Silicon, Ubuntu and Windows, R5Z and prior R5A–R5Y Python negative tests, R4Y and R4Z; both macOS runners execute the unchanged genuine R4M signed native XPC/CDHash **six tests serial once without waiver/retry**. The general Test Gate runs separately. Each of the exact twelve externally independently authenticated custody gates from R5Y remains `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`.

## Owner macOS Intel acceptance targets — NOT owner results

Fresh integrity-checked isolated ZIP; frozen commit exact, **five additive sources**, **163/163 Git source blob pins** (24 R4A–R4X Rust, 5 R4Y, 4 R4Z, 5 each R5A–R5Z); **919 regular tests =666 Python +253 Rust**, three intentionally ignored Rust tests; native original R4M signed XPC/CDHash six serial PASS no retries/waivers; rustfmt/strict Clippy PASS; exact source HEAD before/after; clean isolated checkout; original Mac worktree untouched; `SCRIPT_EXIT=0`. These are *targets only* until the owner submits a complete Terminal log.

## Decision

A technical PASS in R5Z establishes that the locally preserved signed witness detects *some* partial rollbacks and fails to prove protection from coherent local rollback. It **does not enroll an independent guardian or owner OOB root**, verify real external administrative domain separation, produce a global monotonic history, establish an external quorum, open any of the 12 native gates, or authorize production. PR stays DRAFT/UNMERGED; formal RoadMap Bureau certification remains pending.

```ini
G03_R5Z=RESEARCH_CANDIDATE
LOCAL_SIGNED_WITNESS=PRESERVED_COPY_ONLY
PRIMARY_ONLY_ROLLBACK=DETECTABLE_WITH_PRESERVED_WITNESS
COHERENT_LOCAL_PRIMARY_AND_WITNESS_ROLLBACK=NOT_GLOBALLY_DETECTABLE
LOCAL_SSHSIG=KEY_POSSESSION_ONLY
OWNER_OOB_ROOT=NOT_ENROLLED
INDEPENDENT_CUSTODIAN=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
OWNER_MAC_INTEL=PENDING
EXPECTED_REGULAR_TESTS=919_PASS
EXPECTED_GIT_BLOBS=163/163_PASS
PRODUCTION=NOT_AUTHORIZED
```
