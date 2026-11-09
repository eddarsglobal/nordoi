# NORDOI G0.3 R4M — Cryptographically pinned CodeDirectory identity (research)

**DRAFT · NO PRODUCTION IDENTITY · 0 USD · NO NEW PUBLIC .noi SYNTAX**

## Base and custody
Stacked on owner macOS 14.5 Intel **R4L FIX1**, commit `d004e33634036157d8efcdffc1c05204d449b5ff` (111/111 regular R4A–R4L tests PASS, 3 helper entries intentionally ignored, Rustfmt + Clippy strict PASS, owner source never opened, remote CI 4/4 and Test Gate 5/5 SUCCESS). R4A–R4L draft PRs remain unmerged, including #76; R4M uses a separate additive draft PR. No new NORDOI compiler, kernel, runtime, public grammar or host grants.

## Threat model and hypothesis
R4L enforced an XPC peer signing **identifier**. The identifier is signer-chosen and can be duplicated by a different independently signed executable. A **CodeDirectory cdhash** is a cryptographic hash of the signed code directory; a signing requirement `cdhash H"<40 hex digits>"` can require the exact version of a binary. Apple TN3126 notes a cdhash is architecture-specific and may represent a truncated hash; it is **not** a signature-authority certificate. Apple TN3127 documents both the `cdhash H` requirement and the need to combine `anchor apple generic` with appropriate signing Team ID and explicit identifier for trusted update-tolerant signing policy.

The R4M lab creates an app with a launchd-managed embedded **GUARDED** XPC service and an otherwise identically sandboxed **OPEN CONTROL** XPC service, both disposable. It builds two independently ad-hoc-signed clients with the **same exact identifier** `org.eddarsglobal.nordoi.r4m.client`. The second client has an intentionally changed but semantically inert compiled C branch, so the **real CodeDirectory CDHash differs**.

Before building the guarded service, the lab enrolls the exact `codesign --display --verbose=4` `CDHash=` of the approved client, validates 40 hexadecimal digits, and compiles a code-signing requirement:

```text
identifier "org.eddarsglobal.nordoi.r4m.client" and cdhash H"<ENROLLED_20_BYTE_CDHASH>"
```

`codesign --verify --strict -R` must accept the approved signed binary and reject the independently signed modified binary, **despite both having the same identifier**. The guarded native XPC service installs the exact requirement on every inbound connection before resuming it. This is a disposable same-session *lab* enrollment. Its approval source is the locally built test executable; it is not externally authenticated code distribution.

## Genuine native differential witnesses
Using the same R4L handler-owned, XPC-file-descriptor journal mechanism:
1. **Approved -> GUARDED**: genuine reply and exactly one service-written journal receipt.
2. **Same-ID modified -> OPEN CONTROL**: genuine reply and exactly two receipts; proves the adversary's binary can reach a sandbox-equivalent embedded XPC service.
3. **Same-ID modified -> GUARDED**: cannot produce a reply and must cause **no handler receipt**; native silent drop or a classified XPC rejection may be observed. A timeout, interruption, or generic error **alone** does not establish cryptographic rejection.
4. **Approved -> GUARDED again**: genuine reply, third handler receipt; eliminates the dead-service explanation.
5. Strict signed-code verification, independent CDHash comparison, test-source pin examination, guarded/open equal sandbox entitlements, all failure simulations fail-closed.

## Security limitations and future admission
**Exact-code integrity is not signer provenance.** SHA-256-based CDHash values usually are 20-byte truncated code directory fingerprints and differ by architecture; auto-pinning whatever executable an attacker presents is unsafe. The same-session enrolment used for this laboratory **does not create a trusted production authority**. R4M cannot claim a verified Developer ID, Team ID, independent trusted distribution, tamper-resistant policy storage, pinned code rotation/revocation, artifact transparency, third-party verification, or secure remote enrollment. An adversary who controls both the candidate binary and the enrolled policy could substitute a new valid signed code. The current test is also not a full confused-deputy authorization system; its journal FD is only a narrow, disposable laboratory measurement.

**Production gate** remains BLOCKED pending:
- Owner-managed trust root / verified Apple Developer ID and Team ID **or** independently authenticated, immutable version-pinned manifest distributed by a separate trusted channel.
- Actual production signer certificate and revocation verification, update policy, rollback prevention, multi-architecture pins, authorizer custody, hostile update tests, and real audit-token peer authorization.
- Explicit admission decision over 12 still-unmet native safety gates and no prior PR merge until separately authorized.

## CI acceptance (initially PENDING)
- macOS Intel 15 and Apple Silicon 15 each execute real guarded and open XPC services, same-ID code-hash positive and negative trials, strict Rustfmt and Clippy.
- Linux and Windows execute only pure-model external-IO default denial (no native macOS identity claim).
- Owner exact-head temporary-clone replay on macOS **14.5 x86_64** is mandatory before local R4M verdict. Cumulative target **117 regular tests** across R4A–R4M and 3 intentionally ignored, assuming the new R4M suite has 6 regular tests.
- PR remains draft/unmerged; `main` unchanged; production NOT_AUTHORIZED.

```ini
G03_R4M=RESEARCH_CANDIDATE
BASE_R4L=d004e33634036157d8efcdffc1c05204d449b5ff
CERTIFICATE_BASED_TRUST_ROOT=NOT_IMPLEMENTED
CDHASH_ENROLLMENT=CANDIDATE_NOT_EXTERNAL_AUTHORITY
FORGED_SAME_IDENTIFIER_DIFFERENT_CODE=CI_PENDING
ROTATION_AND_REVOCATION=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```

## Primary Apple references
- https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements
- https://developer.apple.com/documentation/technotes/tn3126-inside-code-signing-hashes
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement
