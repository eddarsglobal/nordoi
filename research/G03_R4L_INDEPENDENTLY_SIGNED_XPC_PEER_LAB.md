# NORDOI G0.3 R4L FIX1 — Differential signed XPC admission with handler-origin receipts

**Research draft · test-only code · strict denial · no product authorization**

## Provenance

Exact accepted-with-controls parent **R4K FIX3**: `5c2fa93cd3dbc729e0d09195ef1a0084a18f92f3`. Owner's macOS 14.5 Intel verification: **106/106 regular R4A–R4K native tests PASS**, 3 intentionally ignored, Rustfmt PASS, Clippy strict PASS, worktree untouched. All prior 11 draft R4A–R4K PRs (#56, #58, #60, #62, #64, #66, #68, #70, #72, #74, #75) remain unmerged. R4L PR #76 stacks on #75 and changes **only three R4L research files**.

## Original R4L evidence — REMEDIATE

Owner macOS 14.5 x86_64 R4L diagnostic against head `73af758df41344e50ef2dfb1ea37e1f84916a73d`: exact SHA verified, 12/12 test blobs; **109 PASS, 2 FAIL, 3 intentionally ignored**, Rustfmt and Clippy PASS, SCRIPT_EXIT=101. The two negative tests failed because an independently ad-hoc-signed client with the wrong identifier was silently dropped until **SIGALRM=14**; it did **not** receive `XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT`. Native CI macOS 15 Intel and Apple Silicon reproduced the same outcome. The previous test's coordinator counter was invalid as a cross-client witness: embedded XPC services can have separate processes per client. Thus original R4L is **not certified**.

Apple's peer requirements allow a nonconforming initiating client to be silently prevented from having its message delivered: an explicit signing-error reply is not guaranteed to the initiator. A `SIGALRM` in isolation is NOT proof of signature enforcement.

## FIX1: controlled differential experiment

A new test creates two real, launchd-managed XPC services embedded in one disposable app, each separately signed with the **same strict App Sandbox entitlement** and the same service C source:

- **GUARDED** service: installs `xpc_connection_set_peer_code_signing_requirement` for signing identifier `org.eddarsglobal.nordoi.r4l.client` before resuming the peer. No authorization bypass.
- **OPEN CONTROL** service: test-only negative control; does **not** install the peer-signing requirement. This temporary service has *only* the same sandbox as the guarded service, and no ambient rights to read/write arbitrary files.

The app coordinator launches two separately ad-hoc-signed binaries built from **identical C source** in the same `Contents/MacOS` directory, under the same user:

- **Good** sibling: signing identifier `org.eddarsglobal.nordoi.r4l.client`.
- **Bad** sibling: signing identifier `org.eddarsglobal.nordoi.r4l.impostor`.

The driver owns a **new, disposable, empty journal file** inside its private test directory. Each sender opens that file solely for this experiment and passes an exact open descriptor as the XPC `receipt_fd` value. Only when the XPC service's event handler actually receives an authenticated-format `ping` does it write and `fsync` one `R4L|HANDLER|PID=<native PID>` entry through the specifically passed descriptor **before replying**.

Exactly these conditions are required in sequence:

1. `good -> guarded`: genuine `pong`, journal has exactly **one** service-produced handler marker.
2. `bad -> open control`: genuine `pong`, journal increases to exactly **two** handler markers, proving this same bad executable can reach a service with equivalent bundling/sandbox conditions.
3. `bad -> guarded`: **no** `pong`, only the explicit native code-signing rejection **or** the documented silent-drop behavior. Journal must **stay exactly two** markers. Any missing positive-control marker, partial journal entry, extra marker, invalid signature, unexpected signal, generic XPC error or output forgery must fail closed.

This is **a test-scoped causal differential** for non-delivery to the guarded handler; it is NOT proof that the bad client has a trusted cryptographic identity. Neither the good peer nor the bad peer writes any journal data directly; fixed source code includes no write call to its test journal. Only the XPC service writes the marker over the passed FD.

## Adversarial boundaries

- Two differently named service bundles are controlled for identical sandbox entitlements and source; other service-launch differences or environmental errors remain potential residual alternative explanations. Replication on owner macOS 14.5 is essential.
- An ad-hoc signing **identifier string is spoofable by an independent attacker**. A production signer/Team ID, hardened runtime, secure update and revocation policy, independent compromised-client threat model, audit-token enforcement, TOCTOU-safe host mediation and resource broker remain NOT IMPLEMENTED.
- A file descriptor is transferred *only inside this fixed isolated laboratory* to record handler delivery; it is **not** granting NORDOI public/native host I/O to arbitrary `.noi` code. The reference model still denies `ResourceEffect::ExternalIo`.
- No system-wide launchd registration, sudo, extra production entitlements, changes to the original source tree, public syntax edits, main merge, install or release.
- 12 native admission gates remain **UNMET**.

## Acceptance and current state

- macOS 15 Apple Silicon: **R4L FIX1 5/5 native PASS**, R4K 8/8 PASS, strict Clippy PASS; Rustfmt follow-up submitted. This is only CI evidence for that exact source commit, not owner Sonoma.
- macOS 15 Intel: **PENDING latest head CI**.
- Windows and Linux: only portable model default-denial checks, no native XPC assertion.
- Owner macOS 14.5 Intel: **PENDING FIX1 exact-head temporary-clone replay**, not yet certified.
- Full R4A–R4L cumulative regular test target: **111 PASS, 0 FAIL, 3 intentionally ignored** for any future eligible owner PASS_WITH_CONTROLS.

```ini
R4K_FIX3_OWNER_MACOS_14_5=PASS_WITH_CONTROLS
R4L_ORIGINAL_OWNER_MACOS_14_5=REMEDIATE
R4L_FIX1=RESEARCH_CANDIDATE
TRUSTED_PRODUCTION_SIGNER=NOT_IMPLEMENTED
NATIVE_ADMISSION_GATES=12_UNMET
PR76=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```

## Primary API references

- [Apple XPC peer-signing requirement](https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement)
- [Apple XPC dictionary FD transfer](https://developer.apple.com/documentation/xpc/xpc_dictionary_set_fd)
- [Apple XPC FD duplication](https://developer.apple.com/documentation/xpc/xpc_dictionary_dup_fd)
