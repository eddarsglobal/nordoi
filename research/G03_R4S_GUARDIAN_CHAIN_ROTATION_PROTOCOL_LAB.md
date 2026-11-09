# NORDOI G0.3 R4S — Guardian custody protocol, signed checkpoint lineage and dual-key rotation

**Research protocol and Mac native laboratory only. Real administrative independence NOT IMPLEMENTED. No production authority.**

## 1. Provenance, scope and governance

Direct, draft-only successor of owner-replayed G0.3 R4R PR #82 on commit \`cb1033784154bbea87545381d77f90b971dfbe10\`. R4R was validated on owner's macOS **14.5 Intel**: **171/171** regular R4A–R4R tests PASS, **3 intentionally ignored** subprocess entrypoints, 18 Git blob pins PASS, Rustfmt and Clippy strict PASS. Its exact-head dedicated CI was 4/4 SUCCESS and general NORDOI Test Gate 5/5 SUCCESS. The original NORDOI owner worktree was never opened or modified.

R4S adds **only** its native Rust research test, this report and one GitHub CI workflow. Prior PRs remain draft/unmerged. \`main\`, the public \`.noi\` language, compiler/kernel, native admission semantics and production host I/O authority remain untouched. All **12 native admission gates UNMET**, \`PRODUCTION=NOT_AUTHORIZED\`.

## 2. Threat-model motivation

R4R proved a freshly nonce-signed obsolete B checkpoint can still be denied if the independent *off-device but same-GitHub-owner* static signed V2 floor remains intact. R4S addresses a different requirement: **how a future externally governed guardian might publish successive checkpoints and rotate signing authority without accepting gaps, forks, forged lineage or silently authorized new keys**.

This experiment does **not** create an unrelated administrator. The test driver creates and owns all five temporary Ed25519 identities in its own OS account, including first and successor guardian, rogue signer, first and successor publisher. The signer and verifier merely reside in different *test directories*. An actual external witness, append-only log, secure third-party service, separately governed signing key, out-of-band owner enrollment or protected verifier does **not** exist in R4S. Tests exercise *a protocol candidate*, not deployed independent custody.

## 3. Proposed signed checkpoint contract

An exact canonical ASCII record:

\`\`\`text
NORDOI-R4S-RECORD-V1
epoch=3
sequence=3
previous=<64 lowercase hex SHA256 of previously trusted checkpoint>
publisher=<authorized OpenSSH ssh-ed25519 public key>
next_guardian=none
\`\`\`

- The first research trust state is **epoch=2 / sequence=2** with \`previous\` anchor derived from the exact R4P public signed V2 checkpoint. A dedicated native test does **live** GitHub \`ls-remote\`, fetches pinned evidence commit \`ff8abe01cc1a527dd84b463dec1736d20a5391a4\` into a fresh temporary Git directory and verifies the **real R4P Ed25519 SSHSIG** against a compiled-in public root.
- A new R4S checkpoint must have exactly \`epoch=prior.epoch+1\` and \`sequence=prior.sequence+1\`, and \`previous\` must match the full SHA256 of the verifier's last-trusted canonical record. No silent gap or skip is accepted.
- The current guardian signs the exact checkpoint bytes using native OpenSSH SSHSIG namespace \`nordoi-r4s-checkpoint@eddarsglobal\`; verifier checks the **previously pinned** current guardian public key, not a key supplied by the next candidate.
- An intentional guardian rotation specifies \`next_guardian=<new Ed25519 public key>\`. It must be co-signed by the **new key** in a *different* SSHSIG namespace \`nordoi-r4s-guardian-rotation@eddarsglobal\` on identical canonical message bytes. After successful admission, the new key becomes the current trusted guardian and the former signer is rejected for the next checkpoint. A same-key rotation is denied.
- The record includes an explicitly guardian-endorsed \`publisher\` key for the next epoch. No public NORDOI runtime authority or compiler syntax is created by these record fields.

This provides *lineage consistency relative to an intact last-trusted verifier head*. It does **not** guarantee global non-equivocation, persistent monotonicity or trustworthy deployment.

## 4. Required adversarial trials

1. Actually contact pinned public GitHub R4P evidence and verify live signed V2 anchor.
2. Generate real Ed25519 checkpoints for epochs 3, 4 and 5, verifying SHA256 continuity and a two-signature guardian rotation.
3. Reject an authentically signed older checkpoint after advancing to a newer trusted head.
4. Reject a second different checkpoint for an epoch already accepted (fork/equivocation relative to the local trusted head).
5. Reject a wrongly chained predecessor digest and an epoch/sequence gap.
6. Reject a rogue signer and a cross-namespace signature.
7. Reject guardian rotation absent new-key consent and reject signatures from old guardian after rotation.
8. Reject tampered signed records and corrupted successor-key co-signatures.
9. Deny missing guardian signatures (fail closed, no fallback to local-only assertions).
10. **Mandatory counterexample:** restoring BOTH the last-trusted verifier head and old guardian-signed history allows reacceptance of an older checkpoint. This test passes as a demonstration of the *remaining vulnerability*, not as anti-rollback protection.

The portable Windows/Linux test only checks reference-model default host-I/O denial and declines to claim native Mac signer or independent guardian authority.

## 5. Deployment design requiring external authorization (NOT EXECUTED)

Before any real independent-guardian pilot, a distinct authorized custodian must own the signing credentials and external append-only state. A future protocol shall require:

- Explicit enrollment of an independently reviewed guardian public-key fingerprint by the owner **outside** the untrusted app/installer/repository.
- Distinct administrative credentials and operational control of signer and immutable checkpoint history. A second Git branch under the same GitHub owner is insufficient.
- Authenticated freshness (nonce with strictly bounded validity) and a persistent externally controlled monotonic checkpoint or transparency log resistant to **joint restoration** of the local verifier and witness.
- A verifiable, explicitly authorized recovery protocol after outage/partial commit; a missing guardian must fail closed without unsafe automatic reset.
- Signed key-rotation and revocation with a separately protected owner recovery key, bounded key lifecycle, no circular self-authorization, proof that a compromised original GitHub administrator cannot change installed verifier pins.
- Real multi-OS coverage and controlled Mac/Windows/Linux native trust before claiming platform-wide availability.
- Security response to chain fork, global equivocation, replay, key theft, signer compromise, timeout, DoS, GitHub ref rewrite, credential loss, unrecoverable witness and rolling upgrades.

Until an unrelated administrator actually operates and attests to such an external service, **R4S cannot close the administrative-independence gate**. In particular, a live GitHub check in the *first* test does not authenticate the later mock guardian's distinct identity; that guardian remains a local fixture.

## 6. Acceptance, tests and release prohibitions

- CI: macOS Intel and Apple Silicon real Ed25519 keygen/sign/verify, GitHub live pinned V2 anchor, chain, key rotation, replay and whole-history rollback counterexample, strict Rustfmt and Clippy; Windows/Linux portable default-deny only.
- Owner Mac **Intel 14.5**, if the candidate CI passes: pinned isolated owner kit, exact research source Git HEAD and **19/19** source test blobs, cumulative **182 regular tests PASS, 0 failed, 3 intentionally ignored**, Rustfmt + Clippy \`-D warnings\` PASS. Owner replay remains distinct from GitHub CI.
- No model training, no executable installation, no new production signing or root privileges, no change to public \`.noi\`, no PR merge, no production enablement.
- Future R4T should target a **separately operated authenticated append-only guardian** only with explicit owner-selected external administrator; otherwise continue strict protocol-level counterexample research.

\`\`\`ini
G03_R4S=RESEARCH_CANDIDATE
R4R_OWNER_MAC_INTEL=PASS_WITH_CONTROLS
CHAIN_SIGNATURES=CI_PENDING
GUARDIAN_ROTATION_DUAL_SIGN=CI_PENDING
GLOBAL_EQUIVOCATION=NOT_PREVENTED
JOINT_HISTORY_RESTORE=KNOWN_VULNERABILITY
REAL_SEPARATE_ADMIN=NOT_IMPLEMENTED
EXTERNAL_MONOTONIC_STATE=NOT_IMPLEMENTED
OWNER_MAC_INTEL_R4S=PENDING
NATIVE_ADMISSION_GATES=12_UNMET
PR=RESEARCH_DRAFT_ONLY
PRODUCTION=NOT_AUTHORIZED
\`\`\`

## References

- OpenSSH SSHSIG verification: https://man.openbsd.org/ssh-keygen.1
- Git ref and fetch: https://git-scm.com/docs/git-ls-remote
- Apple code signing identity limitations: https://developer.apple.com/documentation/technotes/tn3126-inside-code-signing-hashes
