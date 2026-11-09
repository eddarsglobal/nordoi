# NORDOI G0.3 R4U — Remotely anchored two-signer checkpoint continuity and fork observation

**Research-only. Signed evidence is real; independent administration is NOT established. Production NOT_AUTHORIZED.**

## Parent and immutable limits

- Parent: owner Mac Intel macOS 14.5 validated G0.3 R4T PR #84, exact `5646986ebe6d63feab624785e7e3949ef8b0848f`. Owner log proved **199/199 regular tests PASS, zero failures, 3 intentionally ignored**, 20/20 source Git blobs, Rustfmt and strict Clippy PASS.
- Stacked R4U candidate PR is a **DRAFT** based only on R4T, with exactly three additive research files: this report, a single new native test and its GitHub workflow. Prior test suites, code public `.noi`, kernel, compiler, runtime, original owner checkout, `main` and releases stay unchanged.
- 0 USD; public GitHub evidence and open-source standard Ed25519 SSHSIG signatures, no commercial API.

## Actual GitHub-hosted signed evidence

A separate **public GitHub branch** `evidence/g03-r4u-signed-observer-ledger-20261109` is pinned in the verifier to its exact Git commit **`15adf075050a233705b3bad6eae62b6f9e6f946d`**. The single remote file `research/r4u_signed_observer_ledger/evidence_pack.txt` has SHA256 **`692809e55aed8ad188208e04653d3bf4fdb9945aa7fa661db6ee22fe8100400d`**.

The evidence contains actual, not invented, **RFC8709 Ed25519** detached SSHSIG signatures constructed using the documented OpenSSH SSHSIG format: an ephemeral guardian signs epoch 3, epoch 4 and an intentionally conflicting epoch-4 fork. A different ephemeral observer signs the accepted epoch 3 and epoch 4 only; the observer **does not approve the fork**. Both public signer keys are pinned independently as immutable constants in the candidate test. Ephemeral private keys were generated in memory for this fixture, never committed and never retained. Real macOS `ssh-keygen -Y verify` must accept the genuine signatures under the exact guardian/observer principals and **different signature namespaces**.

Canonical record fields:
```text
NORDOI-R4U-LOG-V1
project=eddarsglobal/nordoi
epoch=4
sequence=4
previous_sha256=11e000dc42cf66554c46d68a385959d0098bc516925b6bf33afa5bbb18f5b3c8
policy=accepted
```

Pin sequence and hash lineage:
- R4P authenticated V2 anchor bytes SHA256: `f69ab4ba094c555b0064e2128e1a90458abb82a2ea652e1a149fa217069e7aa8`.
- Guardian/observer-authenticated epoch3 record SHA256: `11e000dc42cf66554c46d68a385959d0098bc516925b6bf33afa5bbb18f5b3c8`.
- Both-authorized epoch4 record SHA256: `65b71123804659cea8b334c783f51d7e63ad3c8283057c22e432a8a54f6c9d1c`.
- Guardian-only conflicting epoch4 fork SHA256: `989377b80da386e7344aefa514614853da8e832b1bec826681dce96ffa218e0a`.

## Fresh read and rollback adversary

The Mac test must execute real `/usr/bin/git ls-remote` over GitHub HTTPS and verify the exact branch→commit binding; perform a **fresh detached temporary Git fetch** with global/system Git configuration and interactive credentials disabled; require `FETCH_HEAD` equals the expected hash; retrieve the exact evidence pack bytes from Git objects and check its SHA256. It then verifies public roots, canonical epoch/sequence records, SHA256 predecessor linkage and **both authentic SSHSIG endorsements**.

R4T proved an attacker may restore **the local verifier AND a previously authentic guardian-signed receipt** to an older epoch, permitting admission when the only claimed high-water is writable and restored. R4U's **central bounded trial** first admits the latest epoch4, then restores *both* local states plus both old signed epoch3 receipts and **recontacts GitHub after that restore**. The off-device, two-signature-backed current epoch4 ref is intact, so the old epoch3 request is rejected. Further trials:
- A guardian-signed *valid* same-epoch4 fork is refused because the independent observer's already signed accepted-epoch4 receipt does **not** sign that fork.
- Replayed old epoch3 observer SSHSIG remains authentic but obsolete.
- Mutation of the observer or guardian signature, message, signature namespace or public key is rejected by a real native SSHSIG verification.
- Duplicate/missing observer sections, manipulated predecessor digest, epoch gap, missing remote, diverged local copies and modified Git ref/pack hash are denied.

Only the tests explicitly named for live remote fetch guarantee an actual current network read. Other tests may reuse a previously authenticated local process snapshot to target specific signature/protocol negative controls; they do not demonstrate dynamic remote freshness.

## Critical non-claims

Both signatures are cryptographically different, but **both keypairs were created by ONE research process**; their private material is not custody-controlled by separate administrators. The witness pack and verifier live in GitHub resources governed by the **same `eddarsglobal` administrative account**. GitHub can rewrite branch refs; the verifier rejects a changed ref only while its own pinned source and code integrity remain trusted. An attacker able to replace the verifier/pins or control GitHub administration can bypass this guard. The evidence is a **static signed epoch4** only: no live epoch5 update protocol or independent append-only monotonic log exists, and the old signatures must never be treated as current on their own. In addition, the test has **no authenticated external service principal or owner-operated third-party transparency log**, and it does not solve global anti-equivocation if the observer is compromised or coordinated with the primary signer.

**Consequently**: `REAL_INDEPENDENT_ADMIN=NOT_IMPLEMENTED`, `GLOBAL_ANTIROLLBACK=NOT_IMPLEMENTED`, `PRODUCTION=NOT_AUTHORIZED`, `NATIVE_ADMISSION_GATES=12_UNMET`.

## CI and owner replay

- Native macOS 15 Intel and Apple Silicon jobs: real SSHSIG verification of published actual GitHub evidence, double-state rollback, fork denial, tampered signatures, namespace and missing-observer checks; previous R4T regression; Rustfmt and Clippy strict.
- Linux and Windows: reference-model host external-I/O default denial and explicit non-native platform claim only.
- Owner Mac Intel 14.5: fresh pinned research-only Git clone, exact **21 test source Git blobs**, expected cumulative **211 regular tests PASS, 0 failures, 3 intentionally ignored**, strict Rustfmt/Clippy, clean isolated checkout, source unchanged. Owner replay PENDING until actual user log.
- PR DRAFT_UNMERGED; prior PRs not merged; no production.

## Production-grade independent-custody prerequisites (not satisfied)

Before using this pattern in production, require a genuinely unrelated authorized custodian, independently controlled signing credentials and root fingerprint enrollment, protected persistent monotonic/append-only checkpoint, observer fork evidence exchange, verifier install protections, secure revocation and recovery, dynamic authenticated freshness, and measured cross-OS acceptance. This research cannot assert any of these.

Source specifications: <https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.sshsig>, <https://www.rfc-editor.org/rfc/rfc8709>.
