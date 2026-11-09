# NORDOI G0.3 R4P — GitHub-hosted SSHSIG checkpoint independent of local file restoration

**Research candidate only · 0 USD · owner macOS 14.5 pending · PRODUCTION NOT_AUTHORIZED**

## Exact parent and untouched boundaries

R4O owner macOS 14.5 Intel accepted with controls on `e1831eef5267248ac31a5100914d718374be22e1` (136/136 cumulative regular R4A–R4O tests PASS, three intentionally ignored auxiliaries, Rustfmt and strict Clippy PASS). All 15 preceding R4A–R4O PRs remain draft/unmerged. This R4P branch stacks only on PR #79 and adds one new Rust native research test, this report, and a CI workflow. No code in the public `.noi` language, compiler, kernel or prior research suites changes.

## Distinct-domain signed checkpoint — ACTUAL read-only GitHub witness

A separate **evidence branch**, `evidence/g03-r4p-external-signed-checkpoint-20261109`, has immutable-*for-this-specific-test* expected head `ff8abe01cc1a527dd84b463dec1736d20a5391a4`. It contains canonically encoded **V1 epoch 1 / sequence 1** and **V2 epoch 2 / sequence 2** records, each with a real Ed25519 **SSHSIG** signature for namespace `nordoi-r4p-external-witness@eddarsglobal`. Signatures were generated offline using an ephemeral private Ed25519 key; **only the public verification key and signatures were uploaded to GitHub**. No private key was committed or retained in a file.

The Rust verifier embeds a fixed public verification key independent of the observed remote contents; never trusts a key supplied by a mutable GitHub file. It performs **live** `git ls-remote` against the official GitHub URL and exact evidence branch; the returned head SHA must equal the compiled-in expected commit. It fetches that remote branch into a new, temporary Git repository with global/system Git config disabled, checks `FETCH_HEAD` SHA again, then reads the two signed messages and public key from Git commit objects. No original owner NORDOI repository is opened, and no GitHub writes are required by the CI or local replay. Tests run `/usr/bin/ssh-keygen -Y verify` using the compiled-in SSH Ed25519 public key under an explicit allowed-signer principal and the correct namespace. A missing remote, altered remote head, root key substitution, malformed record, tampered signature/message, or cross-namespace replay causes rejection.

## Central negative experiment (R4O vulnerability closure *within the lab*)

R4O demonstrated an adversary can restore **BOTH local writable state copies**, so that epoch 1 / sequence 1 becomes locally admissible again. R4P first proves epoch 2 / sequence 2 matches the live signed GitHub checkpoint. Then the same test restores **BOTH** local copies to epoch 1 / sequence 1 and performs an **entirely new GitHub HTTPS ref lookup and detached temporary fetch after the restoration**, rather than relying on an in-process cached checkpoint. The GitHub-hosted witness still reports the separately signed epoch 2 / sequence 2. A version must match (a) both present and equal local states, (b) live authenticated remote state, and (c) the version requested. Both old and new requests are denied while restored local state diverges from the remotely authenticated state; re-enrollment/reconciliation is not silently performed. A genuinely signed V1 record is *intentionally not trusted* as the latest remote checkpoint.

Other negative controls: single local state rollback, local state ahead of the signed external witness, a simulated missing-external-witness admission path, missing local state, root substitution, tampering and wrong namespace.

## Precisely bounded independence

This is a **network- and device-separated, externally hosted witness** in a different Git branch, effective in the **narrow attacker model: a local attacker can restore the owner Mac's two writable cursor files, but cannot rewrite GitHub branch refs, replace the compiled verifier or falsify GitHub HTTPS responses**.

The witness branch and software repository are **both controlled by the same GitHub owner**, not independent administrative or multi-party trust domains. This is not a transparent public ledger, not append-only secure storage, and not hardware-protected. The test pins a particular remote commit SHA and a *static* signed epoch 2 checkpoint: it cannot admit new releases automatically. The signing private key is intentionally not retained, so advancing to epoch 3 requires a **new, separately governed witness** rather than a production rotation feature. GitHub admins could rewrite the branch; current pinned-head logic then fails closed only while a trusted verifier keeps the old expected SHA. An attacker who can change the verifier or its approved pins can bypass the witness. No owner out-of-band root key enrollment, cryptographically authenticated multi-party witness, certificate-based signer, production monotonic service, reliable distributed availability, revocation, or crash-safe runtime state update has been established.

This remote network read is confined to an isolated *research test*. It does not authorize NORDOI public `.noi` code or the production engine to perform any external host I/O. The reference model still denies `ResourceEffect::ExternalIo`. **All 12 native admission gates UNMET.**

## Admission gate and expected tests

- macOS 15 Intel + Apple Silicon: **R4P 10/10 regular native+reference tests**, real GitHub HTTPS `ls-remote` and Git fetch, SSHSIG signed V1 and V2 verification, dual-local-copy rollback denial, mismatch and outage tests, strict rustfmt and clippy. Expected R4O parent research suite 10/10 PASS.
- Windows/Linux: only pure model default-denial + platform non-claim; **no macOS-native or actual remote authority claim**.
- Owner macOS 14.5 Intel: independent, exact pinned temporary Git checkout and 16 Git test blobs; cumulative **146 regular tests**, three deliberately ignored auxiliaries, strict Rustfmt, offline Clippy, clean clone, no user repository mutations. **Pending actual replay.**
- No merging R4O/R4P or preceding PRs; no production or owner key activation.

```ini
G03_R4P=RESEARCH_CANDIDATE
SOURCE_PARENT_R4O=e1831eef5267248ac31a5100914d718374be22e1
REMOTE_WITNESS_BRANCH=evidence/g03-r4p-external-signed-checkpoint-20261109
REMOTE_WITNESS_HEAD=ff8abe01cc1a527dd84b463dec1736d20a5391a4
REMOTE_WITNESS_RECORD=EPOCH_2_SEQ_2
ED25519_PUBLIC_ROOT=PINNED_IN_RESEARCH_SOURCE
LOCAL_DOUBLE_RESTORE=CI_PENDING
EXTERNAL_NETWORK_WITNESS=CI_PENDING
ADMINISTRATIVELY_INDEPENDENT_WITNESS=NOT_IMPLEMENTED
TAMPER_PROOF_MONOTONIC_WITNESS=NOT_IMPLEMENTED
PRODUCTION_SIGNER_TRUST=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

## API sources

- OpenSSH SSHSIG format and API: https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig
- Git fetch and remote refs: https://git-scm.com/docs/git-fetch
