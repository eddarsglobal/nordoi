# NORDOI G0.3 R4O — Pinned public trust root and dual-checkpoint anti-rollback limits

**LABORATORY ONLY · ADDITIVE RESEARCH PR · NO PRODUCTION · FAIL-CLOSED WHEN OBSERVABLE**

## Provenance and custody

The only R4O parent is owner-approved-with-controls R4N `5bcaae4eb5c2c4b00ec4b9a4baae78d935461e8d` (owner macOS 14.5 Intel: 126/126 regular native R4A–R4N tests PASS, 3 intentionally ignored; Rustfmt/Clippy PASS, 14/14 source blobs, local test scripts exit 0). R4A–R4N PRs #56, #58, #60, #62, #64, #66, #68, #70, #72, #74, #75, #76, #77 and #78 remain draft/unmerged. R4O must add only its research test, documentation and CI, without changing existing tests, original user Git checkout, public .noi syntax, compiler or runtime.

## Scope: two experimentally separate inputs

The laboratory creates an ephemeral OpenSSH Ed25519 **root signing key** and an **attacker key** in a temporary signing directory, then records a full SHA-256 digest of the canonical authorized root public key. This digest is explicitly passed into an isolated verifier API as the simulated **owner-approved pin**. A candidate root presented for enrollment is rejected unless it matches that already supplied digest. Enrollment requires a new `allowed_root` file; attempts to enroll twice fail. A post-enrollment root-public-key substitution is rejected by comparing to the original independent verifier argument *before* accepting any cryptographic signature, even when a rogue signature is otherwise valid under the replacement root.

`ssh-keygen -Y sign` and `ssh-keygen -Y verify` run as separate macOS processes using domain-scoped SSHSIG signatures (`nordoi-r4o-checkpoint@eddarsglobal`). Canonical version checkpoints include `epoch`, `seq` and a native Apple CodeDirectory `CDHash`. The verifier independently calls `codesign --verify --strict` and `codesign --display --verbose=4` for the **actual test executable**; it does not trust a caller-supplied CDHash. Wrong executable, namespace, root signature or altered checkpoint bytes must fail.

**Important distinction:** The test driver itself generates both the root key and the initial expected fingerprint, with directories and process calls separated for research. **This is NOT genuine owner out-of-band enrollment** and does not prove the owner has independently authorized a signing root. Authentic production enrollment will require a root public key fingerprint reviewed and retained outside the untrusted code and installer, with an actual independent custody channel.

## Two-witness rollback experiments

The verifier keeps one `cursor` and a second `checkpoint` in distinct temporary directories, stored with an exclusive temporary file, `sync_all` and rename. Acceptance requires both state files to be present, canonical, and **equal**, or neither to exist at initial epoch 1/sequence 1. An epoch/sequence rollback after epoch 2/sequence 2 is rejected, as are missing/corrupted witness files. If one file is restored from a saved old snapshot while the other remains newer, verification returns `WITNESSES_DIVERGED`, refusing both old and new releases until reconciliation.

**Adversarial counterexample (deliberately PASS as a limitation finding):** if the attacker restores **both writable copies** to an older checkpoint, the verifier cannot distinguish that restoration from a genuinely old local state and will admit the formerly valid signed old version. A test explicitly reproduces this vulnerability; it is NOT a security-success test. The two directories are owned by the same test user, not two independent access-control principals, and no OS-backed monotonic storage, append-only external audit, trusted time, remotely notarized head, secure enclave register, transactionally atomic dual-store update, or crash-safe witness protocol has been implemented.

## Native admission tests

1. Valid owner-pinned enrollment, root SSHSIG, first signed code version and exclusive second enrollment denial.
2. A substituted root key, paired with a different valid rogue SSHSIG, fails the original owner fingerprint.
3. Tampered checkpoint bytes and wrong SSHSIG namespace are rejected.
4. Native Mach-O changes retain valid ad-hoc signature but fail the signed code-version pin.
5. Local-only rollback is detected by the unmodified independent-directory witness.
6. Witness-only rollback is detected by the unmodified local cursor.
7. Missing and corrupted witness forbid silent reinitialization.
8. Replay of signed epoch 1/sequence 1 after advancing to epoch 2/sequence 2 is denied.
9. **Both-copy rollback attack is shown possible**, explicitly documenting the residual.

The 10th regular macOS test is the inherited NORDOI reference-model external-IO default-denial check. Linux and Windows never claim to run native macOS signature controls and only run portable default-denial checks.

## Security/non-authorizations

No production keys, certificate purchase, private key in repository, real VPS, root privilege, writes to system launchd settings or user original source, NORDOI native host-I/O entitlement, compiler syntax addition, merge or installation. **All 12 native admission gates remain UNMET.** `PRODUCTION=NOT_AUTHORIZED`.

For a future production-ready research step, require a truly independent owner public-key enrollment and a rollback-resistant external witness with explicit compromised-state failure tests. If both witness stores can be restored by the same adversary, R4O makes **no anti-rollback security guarantee**.

```ini
G03_R4O=RESEARCH_CANDIDATE
BASE_R4N=5bcaae4eb5c2c4b00ec4b9a4baae78d935461e8d
ROOT_OWNER_APPROVAL=SIMULATED_PIN_ONLY
ROOT_SUBSTITUTION_TEST=CI_PENDING
ONE_SIDED_ROLLBACK_DETECTION=CI_PENDING
BOTH_COPIES_RESTORED=KNOWN_UNDETECTED_ATTACK
HARDWARE_MONOTONIC_COUNTER=NOT_IMPLEMENTED
EXTERNAL_ROOT_CUSTODY=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

## Official reference

OpenSSH `ssh-keygen -Y sign/-Y verify` manual: https://man.openbsd.org/ssh-keygen.1.
