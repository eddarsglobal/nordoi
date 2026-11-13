# NORDOI G0.3 R6C — Two genuine same-admin signatures are NOT independent quorum

**RESEARCH ONLY / DRAFT / DENY NATIVE ADMISSION.** Direct additive child of owner-qualified R6B candidate `7a6c22bf6b092488bae23361497f8645141b70a9` (PR #119), which was replayed on actual owner macOS 14.5 Intel x86_64 with **967/967 regular tests PASS**, **173/173 exact source Git blob identities PASS**, real R4M native signed XPC/CDHash **6 PASS serial**, strict rustfmt and Clippy PASS, `SCRIPT_EXIT=0`. R6B GitHub CI **9/9 SUCCESS** after a targeted rerun of an Apple Silicon job originally cancelled before execution, without any source change or waiver. PR #119 remains DRAFT and unmerged.

## Scientific scope

R6A qualified a genuine OS-random 256-bit source-bound witness challenge. R6B qualified the exact response and a mathematically genuine, local OpenSSH Ed25519 SSHSIG signature. R6C asks whether an "owner" and a "guardian" can form an **independent two-party quorum** merely because two distinct keys, two operating-system process identities and two cryptographic signatures exist.

**They cannot.** R6C deliberately creates two distinct real local Ed25519 keypairs under the same real macOS/Linux account, with two actual subprocess PIDs observing identical UID/EUID. The owner and candidate each sign the exact same canonical R6C document with the independent OpenSSH SSHSIG signature namespaces `nordoi-r6c-owner` and `nordoi-r6c-guardian`. The original genuine R6B response signature is also cryptographically verified, yielding **three real valid signatures using two same-admin keys**. This is mathematically meaningful proof of key possession and source-message integrity, **not a quorum of independent operators**.

## Engineering controls

- Exact canonical protocol `NORDOI-G03-R6C-TWO-KEY-SAME-ADMIN-QUORUM-NO-AUTHORITY-V1` pinned to immutable R6B commit. It binds actual R6B signed response SHA-256, R6A OS-random challenge and request bytes, R5Z local signed witness SHA-256, historical height, ordered nonce-set SHA-256 and revocation floor, owner/candidate key pins, roles and a local timebox.
- Duplicate keys and noncanonical/oversized/invalid UTF-8 JSON, NaN, Boolean/negative/fractional counters, false branch/project/privileged-verb, same key reused for both purported signers, changed history or response, changed challenge, stale/future local clock and wrong public-key pin **fail closed**.
- Even genuinely signed claimed `owner_oob_root`, externally independent admin domain, protected external history root, remote nonce/revocation floors or `quorum_size=2` are **not accepted as independently authenticated evidence**. The fake claim is actually signed with a real local owner key as a native negative control.
- Real OS-level OpenSSH SSHSIG verification checks two separate domain-separated signatures and the original signed R6B response. Tampered content, incorrect allowlist, swapped signer signatures and wrong signature namespace are rejected; true valid signatures yield `TWO_REAL_SSHSIG_KEYS_SAME_ADMIN_NOT_INDEPENDENT_QUORUM` with **status BLOCK**, **admission NOT_AUTHORIZED** and all twelve native gates UNMET.
- Local spent nonce and local revocation sets are denial controls, not a globally monotonic external authority; local wall clock is not externally attested freshness.
- No network, no external key enrollment, no actual independent operator, no host privilege escalation, no protected external nonce or revocation history, no runtime/native/production modification.

## Portable and real tests

**24 R6C adversarial cases.** Sixteen portable schema/role/source/nonce/floor/timebox/false independence/revocation checks. Eight genuine POSIX OpenSSH/OS cases: verify three signatures under two distinct keypairs; altered signed envelope; wrong owner allowed signer; wrong guardian signature namespace; swapped signatures; locally signed false quorum and OOB claim; local key revocation and replay; full actual native `--exercise` with intentionally denied exit **2**. On Windows, eight POSIX-native cases are skipped without inventing native system testimony; 16 portable cases still execute.

Dedicated GitHub CI matrix: macOS Intel, macOS Apple Silicon, Ubuntu, Windows; all R6A and R6B and R5A–R5Z plus R4Y/R4Z prior negative-control regressions; unchanged original R4M **six genuine signed XPC/CDHash native tests serial once** on both macOS runners without retries/waivers. A separate general NORDOI Test Gate must pass. No existing Rust research files or previous PRs are changed.

## Owner Mac Intel acceptance — targets, not yet measured

**Five strictly additive R6C files** on exact source parent R6B. Fresh SHA-256 verified isolated owner kit; Git branch/HEAD pin; **178 exact Git source blob identities** (=173 existing through R6B +5 R6C); **991 regular tests =738 Python +253 Rust**, three Rust deliberately ignored; unchanged genuine original R4M native XPC/CDHash 6 PASS serial no retry or waiver; strict Rustfmt, Clippy and clean source checkout. `SCRIPT_EXIT=0` required. These are targets only until the actual owner submits the full Mac Intel Terminal replay.

## Scientific boundary

A valid 2/2 signature tally under one administrator is evidence of two keys, **NOT proof of separate administrative custody**. An actual independent quorum requires separately authenticated external operators, true owner out-of-band trust roots, independently controlled keys, externally retained monotonic history, cross-domain quorum, review of revocation/compromise/recovery and formal Bureau certification. All 12 exact native admission gate IDs retain `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`. Production NOT_AUTHORIZED, PR remains DRAFT and unmerged.

```ini
G03_R6C=RESEARCH_CANDIDATE
R6C_LOCAL_SIGNERS=2_DISTINCT_ED25519_KEYS_ONE_ADMIN
SIGNED_R6B_PARENT_RESPONSE=GENUINE_LOCAL_CRYPTO
INDEPENDENT_TWO_AUTHORITY_QUORUM=NOT_PROVEN
OWNER_OOB_ROOT=NOT_ENROLLED
INDEPENDENT_EXTERNAL_CUSTODIAN=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_OWNER_SOURCE_GIT_BLOBS=178/178_PASS
EXPECTED_OWNER_REGULAR_TESTS=991_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
BUREAU_FORMAL_CERTIFICATION=PENDING
PRODUCTION=NOT_AUTHORIZED
```
