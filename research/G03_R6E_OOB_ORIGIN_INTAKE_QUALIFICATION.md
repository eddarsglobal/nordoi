# NORDOI G0.3 R6E — Owner out-of-band origin and external custodian refusal

**DRAFT RESEARCH / DENY ONLY / NO NATIVE ADMISSION.** R6E extends the actual owner Mac Intel-validated R6D candidate `cc3816dac7bf677c3137df69adb34b3c44c23d19` (PR #121), with **1,015 real regular tests PASS (762 Python +253 Rust), 183/183 frozen Git source blobs PASS, original genuine native R4M XPC/CDHash six serial PASS**, `SCRIPT_EXIT=0`, and independently verified R6D GitHub CI **9/9 SUCCESS**. Earlier PRs remain DRAFT, unmerged.

## Research question and exact boundary

R6A established a real OS-random 256-bit challenge bound to a local signed witness; R6B demonstrated real signatures on its response, R6C examined same-admin local two-key quorum, and R6D examined five genuine signatures and two real different PIDs with one UID/EUID. R6E asks whether **a locally signed receipt that calls itself "out of band"** can establish the owner's real independent authentication of a separate guardian's public key or prove externally controlled, antirollbackable historical custody. **It cannot.**

The new `G03_R6E` envelope is a strictly canonical deny-only pre-enrollment record. It binds exact SHA-256 bytes of the R6D administrative provenance evidence, R6C same-admin quorum envelope, R6B signed response, R6A random challenge request, R5Z signed witness, challenge and two separately hashed local Ed25519 public keys. A disposable **same-admin local file** is expressly represented as such. A real OpenSSH SSHSIG under namespace `nordoi-r6e-claimed-oob` signs those bytes using the same locally controlled candidate key. The inherited five authentic OpenSSH signatures are independently rechecked; **six valid signatures are still NOT independent external custody**.

## Actual experiment and controls

- Strictly validated, canonical UTF-8 JSON source-pinned to R6D commit; reject duplicate fields, malformed/oversized/noncanonical input, NaN or wrong numeric types, false parent/operation, mismatched public keys, changed history, cross-request challenge replay, false local transport label and stale/future local wall clock. The clock is NOT an independent external time attestation.
- The R6E assessment verifies five real R6D and parent SSHSIG signatures plus the newly created genuine sixth SSHSIG on the exact candidate. It checks actual same-admin signatures, actual local OS subprocess observations and exact R6A–R6D source hashes. Genuine signature validity proves only **local cryptographic key possession and integrity**, not owner OOB enrollment.
- Claimed owner OOB root fingerprints, independently controlled guardian keys, external operator identity, external domain labels, remote history anchors, monotonic nonce/revocation floors or a 2/2 quorum are **untrusted self-assertions**, even if the same-admin key cryptographically signs them. The native test actually verifies a valid signature on a false root assertion and still blocks it.
- A locally spent nonce or revoked signer triggers fail-closed denial. It cannot prove global monotonicity or prevent coherent same-admin rollback. A different key, hostname, PID or transport *label* is not an independent custodian.
- The laboratory does **not** contact, create or enroll any real external operator, issue network requests, sign production authority, change native runtime, export secrets, waive tests or change preceding research. Its native `--exercise` intentionally returns **exit 2 and status BLOCK**.

## Tests, GitHub CI, owner Mac Intel acceptance

**24 new adversarial Python tests:** sixteen portable protocol/field/source/transport/clock/nonce/revocation/false-external-claim refusals, and eight genuine POSIX/OpenSSH native tests: six true signatures still BLOCK, signed body alteration, wrong allowlist, wrong SSHSIG namespace, genuinely signed false OOB root, cross-request replay, live local revoke/spent nonce, and native `--exercise` deliberately returning exit code 2. Windows runs the 16 portable cases and intentionally skips eight POSIX-native cases, never manufacturing OS or OOB evidence.

Dedicated CI matrix **macOS Intel, macOS Apple Silicon, Ubuntu and Windows** runs R6E and all original R6A–R6D, R5A–R5Z, R4Y and R4Z regressions and the **unchanged genuine signed native R4M XPC/CDHash six cases serial once** on both macOS runners (no waiver/retry). The general NORDOI Test Gate must complete separately. A PASS in the lab is acceptance of *refusal logic*, not external-custodian admission.

**Owner Mac Intel replay TARGETS, NOT owner-measured yet:** frozen R6E branch/head/parent, five strictly additive Git files, **188/188 exact Git source blob pins**, **1,039 regular tests =786 Python +253 Rust**, 3 intentionally ignored Rust, original R4M genuine signed native XPC/CDHash 6 PASS serial, Rustfmt PASS, strict Clippy PASS, clean isolated checkout, exact HEAD unchanged, `SCRIPT_EXIT=0`. All external admission gates must remain unmet. Owner source worktree and GitHub branches untouched.

## Prerequisite to actual external custody

A *real, separately authorized and authenticated operator* must supply independently verifiable ownership and control of a guardian key and a protected historical witness. The actual owner must separately establish the guardian fingerprint over a genuine independent channel, with recorded operator and out-of-band provenance and external high-water epoch, nonce, revocation and recovery history. The Bureau must certify the evidence and authorize any subsequent native admission. None of this is implemented, inferred, forged or simulated by R6E.

All 12 original R6D native admission gates retain `UNMET_INDEPENDENT_EXTERNAL_EVIDENCE`, owner OOB root remains NOT_AUTHENTICATED, guardian NOT_ENROLLED, independent custody NOT_PROVEN, protected global antirollback NOT_VERIFIED, formal Bureau certification PENDING and production NOT_AUTHORIZED.

```ini
G03_R6E=RESEARCH_CANDIDATE
SOURCE_PARENT_R6D=cc3816dac7bf677c3137df69adb34b3c44c23d19
SIX_LOCAL_REAL_SSHSIGS=SAME_ADMIN_KEY_POSSESSION_ONLY
INDEPENDENT_OWNER_OOB_CHANNEL=NOT_AUTHENTICATED
REAL_EXTERNAL_GUARDIAN=NOT_ENROLLED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_VERIFIED
NATIVE_ADMISSION_GATES=12_UNMET
EXPECTED_SOURCE_GIT_BLOBS=188/188_PASS
EXPECTED_REGULAR_TESTS=1039_PASS
OWNER_MAC_INTEL=PENDING
PR=DRAFT_UNMERGED
FORMAL_BUREAU_CERTIFICATION=PENDING
PRODUCTION=NOT_AUTHORIZED
```
