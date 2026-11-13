# NORDOI G0.3 R6A — External witness challenge pre-enrollment, fail closed

**DRAFT RESEARCH CANDIDATE — NO EXTERNAL TRUST ENROLLED, ALL 12 NATIVE ADMISSION GATES UNMET.** This branch is an additive child of owner-qualified R5Z `381d0fd785ab7c572f133355cc482ca34e1a0a7f` (PR #117), which obtained **919 regular Mac Intel PASS**, **163/163 frozen Git blob identities PASS**, original R4M native signed XPC/CDHash **6/6 PASS serial**, Rustfmt and strict Clippy PASS, SCRIPT_EXIT=0, and GitHub CI 9/9 SUCCESS. Parent PR stays DRAFT, unmerged.

## Scientific transition from R5Z

R5Z established the limit of a second **same-admin** signed local witness: it detects partial SQLite rollback, but restoration of **both** local primary and witness state can return to an apparently coherent old history without invalidating the old cryptographic signature. R6A introduces the next missing artifact, an explicit, **fresh 256-bit OS-random witness challenge** that cryptographically binds an evidence request to the exact R5Z signed local witness bytes, height, ordered nonce-set SHA-256, local revocation floor, receipt digest and source commit.

This stage is a secure *pre-enrollment acceptance-format and denial protocol*. There is **no real independent operator available**, no out-of-band owner-authenticated trust root, no independently protected remote nonce/revocation high-water mark and no quorum. Therefore the mere receipt of a well-formed response or successful SSHSIG check **MUST NOT** grant authority. A genuine OS-random challenge only proves that the *local process generated a fresh random value*. It does not by itself prove an external custodian saw the challenge, independently retained the antirollback state or authenticated to the owner.

## Actual implemented controls

- Strict canonical UTF-8 JSON schema, source-protocol/project/operation pinned to R5Z candidate commit, no duplicate keys, NaN, unsigned grants, noncanonical serialization, invalid SHA-256 fields, oversized input, invalid counter/clock types or stale/future local TTL (bounded to 300 seconds). The local timebox is explicitly untrusted.
- Receipt binds byte-exact R5Z witness SHA-256, history height, nonce-set hash, source signed local receipt digest and revocation floor. Any altered source, nonce or epoch-like floor is refused. A real random 256-bit OS nonce from `secrets.token_hex(32)` is generated on real native executions.
- Optional claimed owner OOB root, guardian administrative domain, external history anchor, revocation floor or spent nonce floor are **untrusted user-supplied assertions**, quarantined even if syntactically valid. No input is automatically converted into an enrolled independent guardian.
- Real POSIX exercise: two distinct genuine local Ed25519 keypairs, different actual subprocess PIDs with **same UID/EUID**, true R5X signed receipt using `ssh-keygen -Y sign/verify`, true R5Y SQLite local spent-nonce event, true R5Z signed local witness and a genuine **R6A domain-separated OpenSSH SSHSIG** over the exact candidate challenge. Even a signature that verifies with a local allowed-signers file returns **BLOCK / LOCAL_KEY_POSSESSION_ONLY**.
- Wrong signer, wrong SSHSIG namespace, altered signed body, unsigned root assertions, locally spent challenge, stale or future local timebox, malformed file or symlink **never promote authority**. Native `--exercise` intentionally returns **exit 2** with status BLOCK on a qualified negative-control outcome.

## Acceptance evidence still missing (not simulated)

An actual independent human/organizational custodian must be identified and independently authenticated, their administrative domain proven separate from the owner, and their public trust root authenticated via a real owner out-of-band channel. The external custodian must prove that it independently retains unrollbackable signed key epochs, challenge history, revocation floors and spent nonce floors; its signer, time, host identity and quorum must be verified across distinct administrative domains. No local signed fixture, claimed external hash, second process, separate local user or remote hostname alone is sufficient. Independent authority requires separate authorization and formal Bureau certification.

All twelve exact R5Z externally independent admission IDs remain `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`. No signing-key export, actual external enrollment, runtime authority, main merge or production deployment is authorized. The SHA-bound R6A candidate is research-only and all paths return BLOCK.

## 24 adversarial tests / four operating systems

- **16 portable Python tests:** frozen source/protocol/operation, canonical duplicate/oversized/invalid UTF-8 controls, 256-bit random challenges, exact R5Z witness binding, invalid signer pins and root fingerprints, Boolean/negative counters, stale/future local timebox, spent local nonce, missing signature, false OOB claims, source/history tampering and primary-ahead local witness.
- **8 genuine native POSIX/OpenSSH tests:** real signed R5X/R5Z/R6A chain under same UID/EUID; modified signed bytes, wrong allowed signer, wrong namespace, locally signed fake OOB root, locally spent challenge, stale/future signed challenge, and standalone native exercise `exit 2`.
- Windows runs 16 portable cases and intentionally skips eight actual POSIX checks (no fabricated Windows user/OS facts). Dedicated CI matrix macOS Intel, Apple Silicon, Ubuntu, Windows runs all prior R5A–R5Z deny-only suites, R4Y and R4Z, and the **original R4M signed native XPC/CDHash six serial tests** on both macOS runners once without waivers/retries. General NORDOI Test Gate must complete independently.

## Owner Mac Intel acceptance targets — NOT measured until owner replay

Frozen source branch; fresh isolated clone with exact HEAD and five additive R6A paths; **168/168 Git blob pins** (24 R4A–R4X Rust, 5 R4Y, 4 R4Z, 5 per R5A–R5Z, 5 R6A); **943 regular tests** = **690 Python** (18 R4Y +24 R4Z +26×24 R5A–R5Z +24 R6A) plus **253 Rust**, **3 deliberately ignored Rust**; original R4M native signed XPC/CDHash **6 PASS serial** with no retry/waiver, strict Rustfmt/Clippy, exact HEAD before/after, clean source clone, `SCRIPT_EXIT=0`. None of those counts is claimed as an owner's measured result before receiving the Mac Intel output.

## Bureau decision boundary

R6A passes only when all negative controls and test/CI integrity pass. It cannot certify **real external independently protected history** without real external credentials and out-of-band evidence outside this same-admin laboratory. Formal Bureau certification, owner OOB identity and actual guardian enrollment remain separate and pending.

```ini
G03_R6A=RESEARCH_CANDIDATE
PARENT_R5Z=381d0fd785ab7c572f133355cc482ca34e1a0a7f
LOCAL_OS_RANDOM_CHALLENGE=256_BITS
SIGNED_SAME_ADMIN_SSHSIG=KEY_POSSESSION_ONLY
EXTERNAL_ADMINISTRATIVE_AUTHORITY=NOT_PROVEN
INDEPENDENT_EXTERNAL_WITNESS=NOT_ENROLLED
OWNER_OOB_TRUST_ROOT=NOT_ENROLLED
GLOBAL_MONOTONIC_HISTORY=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_OWNER_REGULAR_TESTS=943_PASS
EXPECTED_SOURCE_GIT_BLOBS=168/168_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```
