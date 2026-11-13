# NORDOI G0.3 R5X — Genuine signed provenance without independent external authority

**DRAFT RESEARCH CANDIDATE.** R5X is an exact additive child of owner-validated R5W `38fcadb6b659e9774c8677a14a7fd27df328f4b3` (PR #114). Owner macOS Intel 14.5 x86_64 R5W replay: **847 regular PASS**, **148/148 exact-source Git blobs PASS**, R4M genuine signed native XPC/CDHash **6 PASS serial**, 3 deliberately ignored Rust, strict rustfmt/Clippy and `SCRIPT_EXIT=0`. Dedicated R5W CI 4/4 and general Test Gate 5/5 SUCCESS. The R5W PR remains DRAFT/UNMERGED.

## Scientific question

**What can a real cryptographic signature over a candidate guardian's receipt establish without an owner-authenticated external root?** Only key possession and signed message integrity. It does not establish an independent administrative custodian, provenance of the key, host/domain identity, legal independence, reliable time, globally persistent nonce/revocation floors, or quorum.

R5X builds a canonical deny-only signed candidate receipt linked to exact R5W dossier SHA-256, frozen source, R5V challenge, history head, key epoch, and guardian public-key pin. It rejects duplicate, oversized, invalid UTF-8, noncanonical, unpinned, malformed and grant-verb payloads. In the POSIX lab, genuine OS child-process identities and real OpenSSH SSHSIG Ed25519 signatures are exercised with the `nordoi-r5x` signature namespace, actual `ssh-keygen -Y sign` and `-Y verify`. Signature modification, wrong locally configured signer, wrong namespace, stale/future untrusted timebox, nonce replay and local revocation-floor regressions are rejected.

The lab keys and `allowed_signers` file are generated under the **same real OS administrator** in a temporary private research directory. A valid signed receipt therefore yields **REAL_SSHSIG_VALID_BUT_EXTERNAL_AUTHORITY_UNPROVEN / BLOCK**, not evidence of an enrolled independent guardian. A claimed OOB root hash or claimed OS/host attestation hash remains an unauthenticated string even when covered by a genuine local signature. The `now` input remains a caller-controlled local clock; `local_spent_challenges` and `local_revocation_floor` are local negative checks, NOT globally protected state. Coordinated local rollback remains outside reliable detection.

## Test plan and governance

- **16 portable tests:** exact source/operation, canonical parser and duplicate-key refusal; hash/challenge/history/key-epoch binding; invalid sources/grant verbs, bogus key IDs, malformed times, NaN/Boolean counters, claimed external hashes, missing signature, stale/future timebox, local nonce/floor rollback refusal, changed owner/candidate report.
- **8 genuine POSIX/OpenSSH tests:** two child-process identities and two true local Ed25519 keys, valid signature still BLOCK, tampered signed canonical body, wrong signer, wrong namespace, authentic local signature over claimed OOB root/host evidence, signed nonce replay, signed local floor regression and full `--exercise` returning intentional exit **2**.
- Windows runs only the 16 portable cases, with 8 genuine POSIX checks intentionally skipped; no simulated OS identity or claimed Windows native signing authority.
- Four-runner dedicated CI: macOS Intel, macOS Apple Silicon, Ubuntu, Windows. It executes R5X–R5A previous negative-control suites plus R4Z and R4Y, and the unchanged genuine R4M signed XPC/CDHash native six on both macOS runners in series without retry/waiver. A separate NORDOI general Test Gate must also succeed.
- The machine-readable R5X JSON copies the **12 exact R5W gate IDs**, keeps every status `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`, and declares owner OOB root, external guardian, protected nonce/revocation histories and independent quorum as NOT ENROLLED/NOT VERIFIED. PR remains **DRAFT/UNMERGED**, no main/runtime/.noi/production modification.

## Owner Mac Intel acceptance — TARGETS only until observed

Fresh SHA-256 verified ZIP extraction; branch and exact HEAD pinned; independent isolated Git clone; exactly **5 additive R5X files**; **153/153 source Git blobs** (=24 Rust R4A–R4X, 5 R4Y, 4 R4Z, 5 each R5A–R5X), original owner worktree unchanged. Full **871 regular tests** = **618 Python** (R4Y 18 + R4Z 24 + 24 stages R5A–R5X ×24) + **253 Rust**, plus **3 intentionally ignored Rust tests**. Native R4M 6/6 serial no waiver/retry, strict rustfmt and Clippy, `SCRIPT_EXIT=0`, source HEAD exact and clean after. **These are targets, not a claimed owner result.**

## Absolute decision boundary

The R5X scientific PASS, if obtained, would only certify **correct refusal of insufficient signed provenance evidence**. It cannot certify real independent administrative authority or open any of the twelve admission gates. No external operator is enrolled; owner OOB root is absent; protected global antirollback is not proven; a valid same-admin SSHSIG is not an external guardian receipt. No native admission, feature unlock, PR merge, or production deployment is authorized without real independently established trust and formal Bureau review.

```ini
G03_R5X=RESEARCH_CANDIDATE
SIGNED_RECEIPT=LOCAL_KEY_POSSESSION_ONLY
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
INDEPENDENT_EXTERNAL_CUSTODIAN=NOT_ENROLLED
PROTECTED_GLOBAL_REVOCATION_HISTORY=NOT_VERIFIED
EXTERNAL_NONCE_MONOTONICITY=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_OWNER_SOURCE_GIT_BLOBS=153/153_PASS
EXPECTED_OWNER_REGULAR_TESTS=871_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```
