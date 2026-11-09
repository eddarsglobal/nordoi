# NORDOI G0.3 R4X — Cross-signed, time-limited guardian enrollment preflight

**Laboratory only — real signing, no independently administered custodian, no production activation.**

## Baseline and frozen research boundary

R4X directly follows R4W FIX2 **`d26ab27552b2d842ab3cf83cce36a907b24c44e6`** (draft PR #88). Original scientific R4W on the owner's Mac Intel macOS 14.5 **238/238 regular PASS**, 0 FAIL, 3 deliberately ignored, 23 exact Git source blob pins, Rustfmt and Clippy strict PASS, `SCRIPT_EXIT=0`. FIX2's targeted GitHub replay passed **2/2** (macOS Intel R4W 14/14 and R4V 13/13, plus Linux Check); new full general Test Gate **5/5 SUCCESS**. Historic original R4W workflow failures remain on record; their scheduler root cause was not determined.

R4X adds a new test-only Rust source, this research dossier and an exact-commit CI workflow. No changes to any previous R4A–R4W test, original R4W FIX2 workflow, existing `ci.yml`, public `.noi` grammar, compiler, kernel, runtime, original Mac owner worktree, `main`, permissions or production.

## What a future real external guardian would actually need

The canonical R4X preflight binds the following fields into ONE byte-for-byte signed document, under two **different OpenSSH SSHSIG namespaces**:

```text
NORDOI-R4X-DUAL-ENROLLMENT-V1
project=eddarsglobal/nordoi
owner_root_sha256=<SHA-256 of canonical public Ed25519 key>
candidate_guardian_sha256=<SHA-256 of canonical public Ed25519 key>
nonce=<new 256-bit lowercase hexadecimal challenge>
issued_unix=<issue epoch seconds>
expires_unix=<expiry epoch seconds, at most 300 seconds after issue>
floor_epoch=<at least epoch 4>
```

The Mac lab uses **actual** `/usr/bin/ssh-keygen -t ed25519`, `ssh-keygen -Y sign` and `ssh-keygen -Y verify` for independent *cryptographic signature roles*, the native `/dev/urandom` source for 256-bit challenges, and native `shasum -a 256` for public-key fingerprints. **Both ephemeral signing keys and both expected fingerprints are created by the same disposable lab process.** This establishes signature correctness, NOT owner out-of-band approval, separate operator custody, or actual external identity. No private keys are stored in GitHub or persisted outside temporary per-test directories.

The owner's signature is verified under `nordoi-r4x-owner-enrollment@eddarsglobal`; the guardian's under `nordoi-r4x-guardian-proof@eddarsglobal`. A valid dual-signature receipt can prove cryptographic possession and a consistent shared signed message, but **must not be accepted as independent administrative enrollment**. Therefore the only final lab admission decision after valid cryptography is `INDEPENDENT_ADMINISTRATIVE_PROVENANCE_NOT_ENROLLED` — even if both test signatures, local pins, clock and epoch are valid.

## Adversarial trials — macOS Intel / Apple Silicon

The 15 regular macOS tests consist of 14 native cases and one reference-model default-denial control:

- Two genuine Ed25519 signatures and distinct public keys verify, but independent-administrator admission remains denied.
- Reusing an old authentic pair against a fresh 256-bit challenge is rejected.
- Substituted owner public-root SHA256 and substituted guardian-root SHA256 fail.
- Owner and guardian SSHSIG namespace/domain replay both fail.
- Swapping a guardian signature from an otherwise valid older request fails.
- Mutation of the signed floor epoch fails canonical verification.
- Expired and not-yet-valid receipts fail; TTL over 300 seconds fails.
- Epoch floor downgrade is rejected, including correctly signed lower-epoch messages.
- Reusing the **same** private key for both 'owner' and 'guardian' roles is refused.
- Missing owner or guardian signature is refused even though the other is valid.
- One test checks an **actual live** `git ls-remote` for the R4U public same-owner signed observer branch, immutable expected commit `15adf075050a233705b3bad6eae62b6f9e6f946d`; its presence still cannot establish independent administration.
- Reference model `ResourceEffect::ExternalIo` remains default-denied.

Linux and Windows jobs run only the portable reference model, not an equivalent macOS native SSHSIG admission.

## Explicit limits on the trust boundary

The R4X test implementation **does not**:
- authenticate an unrelated external person or administrator,
- perform genuine separate-channel owner fingerprint enrollment,
- provide a production-safe monotonic witnessed head, transcript transparency or fork-detection service,
- protect the installed verifier binary or pinned public roots against privileged replacement,
- guarantee a single-use nonce across independent processes or device restarts (the nonce is fresh and compared per test session, NOT stored as an irreversible consumed-challenge ledger),
- authenticate a remote HTTP service, persist durable availability or implement revocation/rotation under real administrative custody,
- open any positive NORDOI execution or `ExternalIo` grant.

The new protocol thus models *precise signed-preflight syntax and denial*, not an actual enrollment or the resolution of R4V/R4W's external-custodian blocker. **12 native admission gates UNMET**; `PRODUCTION=NOT_AUTHORIZED`.

## Future real enrollment procedure (PENDING outside this lab)

1. Authorize an actual unrelated guardian operator under a separate administrative account and document verifiable, externally controlled key custody.
2. Obtain the real guardian root fingerprint through a separately authenticated owner-approved channel **before** accepting candidate-supplied signing material.
3. Exchange one new challenge with the independent guardian, collect signed proof under the protocol domain, verify expiration, full byte canonicality, current high-water and revocation.
4. Independently archive the root, operator authority and monotonic log receipts outside the repo account and Mac checkout, with protected verifier deployment and recovery review.
5. Only then design a separate governance decision with real reproducible evidence; R4X itself cannot grant or certify production.

## Research validation targets

- GitHub dedicated CI: macOS Intel / Apple Silicon native R4X tests and unchanged R4W regression; strict Rustfmt/Clippy, Linux/Windows portable deny-only.
- GitHub general Test Gate: all five expected jobs actually created and PASS; no confusion between absent jobs and failed tests.
- Owner Intel Mac 14.5: verify source pins for **24 distinct R4A–R4X test files**, expected cumulative **253 regular PASS, 0 FAIL, 3 intentionally ignored auxiliary child tests**, strict Rustfmt and Clippy, clean isolated HEAD; no changes to owner's original repo.
- PR remains draft/unmerged. Main unchanged. Production NOT_AUTHORIZED.

```ini
G03_R4W_FIX2=PASS_WITH_CONTROLS
G03_R4X=RESEARCH_CANDIDATE
DUAL_ED25519_SIGNED_PROTOCOL=MAC_LAB_ONLY
REAL_OWNER_OUT_OF_BAND_PIN=NOT_IMPLEMENTED
REAL_INDEPENDENT_CUSTODIAN=NOT_ENROLLED
PERSISTENT_NONCE_CONSUMPTION=NOT_IMPLEMENTED
GLOBAL_MONOTONIC_ANTIROLLBACK=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Reference: <https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig>.