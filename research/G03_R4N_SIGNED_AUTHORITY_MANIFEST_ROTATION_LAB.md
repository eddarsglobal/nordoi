# NORDOI G0.3 R4N — Independent research-root Ed25519 authority and signed release manifest

**Research only · separate root and delegated release signing keys · fail-closed · no product signing or distribution authority**

## Exact provenance
Stacked directly on owner-validated G0.3 R4M `fa68904b7e5702adbb20bde15b2b7d1a29dd4540` (owner macOS 14.5 Intel: 117/117 regular research tests PASS; 3 intentionally ignored, Rustfmt strict PASS, Clippy strict PASS, original worktree untouched). All previous 13 R4A–R4M research PRs are still DRAFT/UNMERGED; no changes to `main`, public `.noi` language or production kernel.

## Scope and explicitly limited root independence
R4M used an exact Apple CodeDirectory `CDHash` pin **enrolled from the same local build**. It proved exact code/version discrimination, not *who* authorized that code. R4N establishes a separate **public verification trust-anchor file** in a distinct test verifier directory. Test-only **Ed25519** private root and release signing keys live in a separate signer directory, not in the verifier directory; the verifier receives only the root public key, signed authority policy, signed release manifest, last accepted custody cursor and observed native `CDHash`.

Both keys are generated **locally in the disposable test fixture at zero cost** using macOS `/usr/bin/ssh-keygen`; these are not externally trusted vendor keys. A production verifier MUST use an independently distributed, immutable/owner-controlled root public key, NOT a freshly generated root or an untrusted key bundled alongside an executable.

## Concrete cryptographic wire contracts

All source bytes are restricted to exact canonical ASCII, unique field order and final newline:

```text
NORDOI-R4N-POLICY-V1
epoch=1
delegate=ssh-ed25519 <BASE64_KEY>
revoked=none
```

A policy carries an **OpenSSH SSHSIG** detached Ed25519 signature from the root under exclusive namespace `nordoi-r4n-policy@eddarsglobal`, checked with `ssh-keygen -Y verify -f <INDEPENDENT_ROOT_ALLOWED_SIGNERS> -I nordoi-r4n-root -n ... -s ...`.

The delegated signer publishes an independent signed release manifest:

```text
NORDOI-R4N-RELEASE-V1
epoch=1
seq=1
cdhash=<40 lowercase hexadecimal characters>
```

The verifier derives a temporary allowed-signers file **only** from the already-authenticated policy's delegated public key, using principal `nordoi-r4n-release`. It verifies the release signature in the **different** namespace `nordoi-r4n-release@eddarsglobal`, compares release `cdhash` with the actual `codesign --display --verbose=4` hash of a separately ad-hoc-signed macOS Mach-O binary, and checks a strictly increasing release sequence. No signature check may be replaced by a hash comparison or "signed" status alone.

## Root-controlled rotation / explicit revocation experiment

- First epoch must be 1 with `revoked=none`; first signed release sequence must be positive.
- Same-epoch publication cannot change the delegate or revocation list, and a duplicate or lower release sequence is refused.
- To increment the policy epoch by exactly one, the root must sign the new delegate AND explicitly name the previously active delegate key as revoked. Old delegate release signatures no longer verify under the new active key. Reusing the old epoch and releases is denied.
- Unknown future epoch gaps, invalid root signer, unauthorized delegated signer, cross-namespace signature, tampered manifest, wrong actual CodeDirectory hash and malformed/noncanonical policy/cursor are denied.
- A trial cursor is atomically replaced in a *disposable file* to demonstrate ordinary session-state recovery, **not** anti-rollback if an attacker can tamper with or restore the state file.

## Threats and unimplemented production mechanisms

**The generated test-only root does not establish independent external trust custody.** The laboratory's Rust test process generates all keys before the verifier is invoked, so process isolation of signer from verifier is modeled by distinct fixture directories and API boundaries; it is not a remote HSM or hardware-separated private key. An attacker with write access to the installed root public key, private root key, signing directory or monotonic state could still replace or roll back the authorization record. Ed25519 proves possession of a particular signing key, not the real-world identity of a developer or code-review authority.

**Known missing controls:** owner out-of-band root public key enrollment; trusted signer/team identity when required; production key custody; independent signature/certificate trust and revocation checks; monotonic hardware-backed anti-rollback; durable atomic process crash recovery; multi-arch trusted release hash manifests; secure update/rotation validation; offline root key management; global XPC production authorization; anti-TOCTOU file descriptor mediation; public language exposure. No external I/O is granted in NORDOI reference model, and no production deployment or PR merge is authorized. All **12 native admission gates still UNMET**.

## Acceptance plan
1. macOS 15 Intel and Apple Silicon: native `/usr/bin/ssh-keygen` Ed25519 SSHSIG signing and verification, two distinctly signed delegate keys, native Mach-O `codesign CDHash`, wrong signer/namespace/hash, valid rotation/revocation, replay denial, strict Rustfmt and Clippy.
2. Linux and Windows: only portable NORDOI external-host-I/O default-denial tests; no cryptographic native Mac assertions.
3. Owner macOS 14.5 Intel: exact commit, 14 test Git blobs and cumulative **126 regular R4A–R4N test PASS**, 3 deliberately ignored, native cross-domain source isolation, formatting and Clippy strict. CI does not substitute for owner replay.
4. No merge, release, install, new `.noi` syntax or enabling production after any test result.

```ini
G03_R4N=RESEARCH_CANDIDATE
BASE_R4M=fa68904b7e5702adbb20bde15b2b7d1a29dd4540
SSHSIG_ROOT_VERIFIER=CI_PENDING
ROOT_PUBLIC_KEY_CUSTODY=DISPOSABLE_FIXTURE_ONLY
DELEGATED_SIGNER_ROTATION=CI_PENDING
REVOCATION_AND_ROLLBACK_TESTS=CI_PENDING
HARDWARE_BACKED_ROLLBACK=NOT_IMPLEMENTED
EXTERNAL_TRUST_ENROLLMENT=NOT_IMPLEMENTED
PRODUCTION_SIGNER=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

### References
- https://man.openbsd.org/ssh-keygen.1
- https://developer.apple.com/documentation/technotes/tn3126-inside-code-signing-hashes
- https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements
