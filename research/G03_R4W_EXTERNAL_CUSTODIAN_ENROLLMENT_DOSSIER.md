# NORDOI G0.3 R4W — External guardian enrollment dossier and custody qualification

**Research only. No externally administered guardian has been enrolled. All production/native authority remains BLOCKED.**

## Provenance and boundary

R4W is stacked directly on owner-validated G0.3 R4V commit `4b22847375459d839c30a76ee2297a86651a5e6c` (PR #86). Owner Mac Intel 14.5: 224/224 regular R4A–R4V tests PASS, 3 deliberately ignored, 22/22 pinned source blobs, Rustfmt/strict Clippy PASS, script exit zero; R4V dedicated CI 4/4 and general Test Gate 5/5. These results validate research counterexamples, not actual independently governed custody.

**Exact R4W additions:** one new macOS native Rust test file, this enrollment dossier and one dedicated CI workflow. No other code changes. PR draft/unmerged, no changes to public `.noi`, original test files, compiler/kernel/runtime, original owner worktree, `main`, signed install, host authorization or production. The 12 native admission gates remain **UNMET**.

## Candidate onboarding dossier — required real-world evidence

A prospective third-party guardian must independently supply the following *outside* the NORDOI repository and with explicit owner authorization. A string, check box, repository branch or synthetic private key is **not** a substitute.

| Gate | Proof required from a real independent operator | Failure result |
|---|---|---|
| G1 — Control separation | Verified accountable operator identity and a documented separate administrative controller, with NORDOI release administrator lacking signing/witness-head mutation privileges | SAME_ADMINISTRATOR or EXTERNAL_OPERATOR_IDENTITY_NOT_VERIFIED |
| G2 — Owner trust enrollment | Independent owner-reviewed Ed25519 fingerprint received and approved through an authenticated out-of-band channel, before accepting any untrusted network key | OWNER_APPROVED_ROOT_NOT_OBTAINED_OUT_OF_BAND |
| G3 — Witness high-water | Protected monotonic/append-only current head that can detect joint rollback of app state, receipts and locally writable mirrors; independently anchored durable history | IMMUTABLE_EXTERNAL_HIGHWATER_NOT_ATTESTED |
| G4 — Trusted verifier | Protected installed verifier and protected root pin, not replaceable by compromised primary source/release administrator alone | VERIFIER_PIN_REPLACEABLE_BY_PRIMARY_ADMIN |
| G5 — Recovery | Independently reviewed policy for revocation, planned guardian rotation, compromise, loss of signing key, crash, witness outage, split brain and emergency owner recovery | RECOVERY_NOT_AUTHORIZED |
| G6 — Verifiable provenance | An actual external trust chain/custody audit, not booleans or labels produced by the same signer/process or GitHub owner | EXTERNAL_PROVENANCE_NOT_PRESENT / RESEARCH_CANNOT_CERTIFY_REAL_OPERATOR_CUSTODY |

**There is no R4W positive native admission path.** Even if all names/flags are asserted to be true, the research gate returns `RESEARCH_CANNOT_CERTIFY_REAL_OPERATOR_CUSTODY` because none of those assertions constitutes externally authenticated provenance.

## Canonical proof-of-possession request (lab-only)

The macOS test generates an ephemeral Ed25519 key using real `/usr/bin/ssh-keygen` and signs the following exact canonical bytes under the dedicated `nordoi-r4w-custodian-request@eddarsglobal` SSHSIG namespace:

```text
NORDOI-R4W-CUSTODIAN-ENROLLMENT-V1
project=eddarsglobal/nordoi
guardian_key=<canonical ssh-ed25519 public key>
nonce=<fresh random 256-bit lowercase hexadecimal>
requested_epoch=4
```

Native `ssh-keygen -Y verify` authenticates possession of the candidate private key and its message bytes, not the candidate's operator identity, regulatory permissions, separate administrative control, or monotonic service state. Replayed nonce, substituted public key, altered epoch, wrong SSHSIG namespace or missing signed request must fail closed.

The native test reads the actual public GitHub R4U evidence reference `refs/heads/evidence/g03-r4u-signed-observer-ledger-20261109` at pinned commit `15adf075050a233705b3bad6eae62b6f9e6f946d` via `/usr/bin/git ls-remote`. It demonstrates that public off-device existence of two real signed research roles under the same `eddarsglobal` administrator **cannot** open G1. That reference is not proof of independent external control.

## Adversarial verification checklist

The R4W laboratory executes real native macOS key-generation/signature/verification and the following fail-closed checks:

1. Genuine Ed25519 SSHSIG proves private-key possession only; same-admin enrollment denied.
2. New 256-bit nonce invalidates an older but correctly signed request.
3. A substituted public key cannot reuse another candidate's valid signature.
4. Cross-namespace SSHSIG fails.
5. Signed message mutation and different requested epoch fail.
6. Missing enrollment signature fails without a fallback.
7. Same repository owner denies despite valid signature.
8. Arbitrarily renamed operator lacking independent authorization denies.
9. Self-asserted operator authorization without out-of-band owner fingerprint denies.
10. No protected monotonic remote log denies.
11. Replaceable verifier pin or ungoverned recovery denies.
12. All-green self declarations are insufficient.
13. Real GitHub R4U ref is checked but same-admin control still denies.
14. Reference model `ResourceEffect::ExternalIo` remains default denied.

Portable Linux/Windows jobs check only the reference model and explicitly do not claim native Mac signature enforcement or independent third-party identity.

## Unmet requirements for actual future external enrollment

- No third-party identity was contacted, authenticated or approved. No real external custodian keys were enrolled; only disposable test keypairs were created locally.
- There is no independent HSM/secure monotonic store, independently governed signed current head, live custodian challenge endpoint, owner-verified out-of-band key fingerprint, externally reviewed real recovery policy, tamper-resistant verifier update path, or independent admin separation.
- A GitHub CI success, successful local key PoP, static signed evidence, or self-reported attestations cannot close those gaps. If the third-party operator is unavailable, network evidence fails closed and NORDOI cannot silently reset the trusted epoch.
- NORDOI production execution and all 12 native admission gates remain locked.

## Acceptance evidence to collect

- Dedicated CI: macOS Intel + Apple Silicon native Ed25519 SSHSIG tests and R4V predecessor regression, strict Rustfmt and Clippy; Linux and Windows portable deny-only.
- General NORDOI Test Gate: all checks PASS.
- Owner macOS 14.5 Intel: pinned isolated Git clone, exact **23 source Git blobs**, expected **238 regular R4A–R4W tests PASS**, **0 failed, 3 deliberately ignored**, Rustfmt and strict Clippy, script exit 0 and original worktree untouched.
- Production and independent-admin gates **must remain BLOCKED** after any research-only PASS.

```ini
G03_R4W=RESEARCH_CANDIDATE
OWNER_MAC_INTEL_R4W=PENDING
CRYPTOGRAPHIC_KEY_POSSESSION=RESEARCH_ONLY
REAL_UNRELATED_OPERATOR_IDENTITY=NOT_VERIFIED
REAL_INDEPENDENT_CUSTODIAN=NOT_ENROLLED
OUT_OF_BAND_OWNER_TRUST_ROOT=NOT_ENROLLED
EXTERNALLY_PROTECTED_MONOTONIC_LOG=NOT_IMPLEMENTED
PRODUCTION_ADMISSION=ALWAYS_BLOCKED_IN_R4W
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

Reference: <https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig>.