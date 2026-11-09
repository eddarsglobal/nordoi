# NORDOI G0.3 R4L — Independently signed inbound XPC peers

**Research only · strict admission witness · REMEDIATE · no product deployment**

## Parent custody

Exactly stacked on R4K FIX3 `5c2fa93cd3dbc729e0d09195ef1a0084a18f92f3` (owner macOS 14.5 Intel: 106/106 regular R4A–R4K tests PASS, 3 intentionally ignored, format and Clippy strict PASS, original source untouched). R4A–R4K draft PRs #56, #58, #60, #62, #64, #66, #68, #70, #72, #74, #75 remain unmerged. R4L PR #76 adds only one new test, this report, and a dedicated CI workflow; no public `.noi` syntax or kernel changes.

## Experiment

A disposable, sandboxed, launchd-managed embedded XPC service installs `xpc_connection_set_peer_code_signing_requirement` before resuming inbound peer connections. A signed coordinator launches **two standalone sibling processes**, built from the same C source and executable directory, signed *separately*:

- **Good client**: ad-hoc identifier `org.eddarsglobal.nordoi.r4l.client`, valid `codesign --verify --strict`, gets genuine `pong` from its private service.
- **Wrong-signed client**: ad-hoc identifier `org.eddarsglobal.nordoi.r4l.impostor`, also validly signed, must be rejected without a `pong` and **only** the explicit `XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT` is currently sufficient for strong positive proof. An indefinite wait, `SIGALRM`, generic connection error, or negative PID alone is **NOT** accepted as cryptographic identity rejection.

All signed code and service files are created in a fresh owner-owned temporary fixture. The original repository, user files, launchd configuration, security settings and network remain untouched. The reference NORDOI model still denies `ResourceEffect::ExternalIo`. The service has only the App Sandbox entitlement.

## Actual CI witness and interpretation — 2026-11-09

- Original candidate [CI #42588113715](https://github.com/eddarsglobal/nordoi/actions/runs/42588113715): macOS 15 Apple Silicon **3/5 tests PASS, 2 FAIL**, baseline signatures PASS; portable Linux/Windows PASS. Initial failed `GOOD_COUNT` assumption was invalid.
- The accepted-ping counter is **per embedded XPC service process**, not globally shared. Apple documents **one XPC service process per client**. The coordinator's counter remains 0 after a *separately executed authorized sibling* receives `pong`. Thus an unchanged coordinator counter **cannot prove non-delivery to the adversary's separate service**. R4L code treats this counter only as a coordinator-instance consistency check; it must not be cited as successful bad-peer refusal.
- Tightened [CI #42588647100](https://github.com/eddarsglobal/nordoi/actions/runs/42588647100): macOS 15 Intel **3/5 PASS, 2 FAIL** and Apple Silicon **3/5 PASS, 2 FAIL**. The good signed sibling receives `pong`; the bad sibling **hangs until `SIGALRM` (signal 14)**. This is **NOT** the exact expected XPC peer-signing error. We reject rather than reinterpret silence as success. Portable Windows and Linux default-denial tests PASS.
- Strict rustfmt initially failed because the new Rust test was unformatted. Source was later formatted using pinned CI Rust 1.98.1 without modifying the original Git tree; final head formatting/checks must be revalidated by CI.
- macOS owner Intel 14.5: **not yet replayed**. No claim that Sonoma behaves like macOS 15.

## Scope and next controlled investigation

The XPC signing requirement and the mismatch from the two `codesign` identities are real research controls. However, Apple's documented behavior for an untrusted **initiating** client is **silent non-delivery** to a server, not necessarily an explicit error visible to that client. Existing R4L observations cannot distinguish denial by code-signing enforcement from an alternate private-service access restriction. A future experiment must provide a **trusted, same-service-instance sideband** for counting actual handler delivery, or a trusted OS rejection receipt. Do not introduce an untrusted response as proof, assume a shared service instance, create a global launchd daemon, or grant arbitrary host I/O to force PASS.

An ad-hoc signing identifier is attacker-choosable; Developer ID / trusted Team ID or properly pinned code identity, adversarial client verification and full confused-deputy hardening are still NOT IMPLEMENTED. No merge, release, install, or production authority; native admission gates 12 UNMET.

```ini
G03_R4K_FIX3=PASS_WITH_CONTROLS
G03_R4L=REMEDIATE
R4L_NATIVE_MACOS_15_INTEL=3_PASS_2_FAIL
R4L_NATIVE_MACOS_15_ARM=3_PASS_2_FAIL
SIGNED_GOOD_SIBLING_REACHABILITY=PASS
SIGNED_BAD_SIBLING_SPECIFIC_REJECTION=NOT_VERIFIED
BAD_SIBLING_TIMEOUT=REJECTED_AS_PROOF
COORDINATOR_COUNTER=NOT_SHARED_BETWEEN_CLIENT_INSTANCES
OWNER_MACOS_14_5=PENDING
PRODUCTION_TRUSTED_SIGNER=NONE
NATIVE_ADMISSION_GATES=12_UNMET
PR76=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```

## Sources
- [Apple XPC process model](https://developer.apple.com/documentation/xpc)
- [Peer code-signing requirement API](https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement)
- [Team identity requirement](https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_team_identity_requirement)
