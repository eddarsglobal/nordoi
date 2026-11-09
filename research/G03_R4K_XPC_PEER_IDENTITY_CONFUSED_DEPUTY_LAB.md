# NORDOI G0.3 R4K FIX1 — Kernel-enforced XPC peer requirements (research)

**DRAFT ONLY · FAIL CLOSED · TEST-ONLY AD-HOC SIGNATURES · NO PRODUCTION**

## Evidence custody
- Exact parent **R4J** commit: `5bb3eaeea558e25de82422e9566933493260c923`. Local owner macOS 14.5 Intel R4J: **98/98 regular tests PASS, 3 intentionally ignored**, strict format/Clippy PASS.
- 10 previous R4A–R4J draft PRs (#56/#58/#60/#62/#64/#66/#68/#70/#72/#74) remain unmerged. R4K FIX1 modifies **only the additive R4K research test, this report and its dedicated CI workflow** on draft PR #75.
- The first R4K attempt called `SecCodeCreateWithXPCMessage` then `SecCodeCheckValidity` **inside the separately sandboxed service**. macOS CI produced **100001** (`EPERM`, `Operation not permitted`) from `SecCodeCheckValidity`. The exact failing OS subsystem has not been independently proven. Initial native tests: 3 PASS, 4 FAIL. The full failure history is retained.
- R4K FIX1 switches to the **documented connection-level libxpc peer code-signing requirement**, set once on each inbound peer connection **before** `xpc_connection_resume`: `xpc_connection_set_peer_code_signing_requirement(peer, "identifier \\"org.eddarsglobal.nordoi.r4k.client\\"")`. Native XPC enforces this requirement on peer messages. Server does **not** trust a message-provided PID, grant, or identity.
- The genuine client also opens **a second actual XPC connection** whose peer-signature requirement intentionally names a nonexistent service. Success requires the **explicit** `XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT` result, not a timeout or generic disconnect; the regular connection must still complete the native launchd XPC round trip.
- Tests also reject unknown `writefile` commands, `grant` and `identity` fields in a `probe` dictionary, and preserve sentinel data against all service read/write/create attempts. A correct service PID must be distinct from the client PID.
- A separately signed wrong-client helper was investigated in an intermediate commit. It could not establish a distinguishable XPC-signature rejection versus a private service activation limitation (timeout); **this test was not counted as PASS** and was removed from acceptance. Independent wrong-signed client rejection is **still NOT VERIFIED**.

## Security claim boundaries
- An *ad-hoc* signing **identifier string can be chosen by an attacker**. An identifier-only requirement is useful for exercising OS signature enforcement, **not an unforgeable authentication anchor**. Production requires Developer ID / trusted Team ID or explicit pinning to a securely distributed signer or exact code hash, with a robust update and revocation policy.
- Client and XPC service are in a disposable `.app` with separately signed `.xpc` helper. Client intentionally **not sandboxed** (positive filesystem control); service App Sandbox has only `com.apple.security.app-sandbox=true`, no expanded securityd exceptions.
- The probe protocol accepts owner-generated test file paths for OS sandbox denial tests. It is **not** an arbitrary path grant, capability transfer, secure filesystem broker or safe untrusted `.noi` executor. Symlink and TOCTOU resilience, privileged file descriptors, authorization state, confused deputy beyond these two forged fields, and malware-grade substitution remain **UNIMPLEMENTED**.
- R0.2 `ResourceEffect::ExternalIo` remains default denied. All **12 R11 native gates UNMET**, no merge, release, install or production.

## Fail-closed acceptance matrix (pending final CI and owner Intel)
1. macOS 15 Apple Silicon: actual XPC with 8 regular R4K tests, explicit wrong-service XPC signature error, R4A–K cumulative regression, Rustfmt + Clippy **PENDING final head**.
2. macOS 15 Intel: same native suite **PENDING**.
3. Windows/Linux: only portable default-denial tests, no XPC claim **PENDING**.
4. Owner macOS 14.5 Intel exact-head temporary Git clone replay **PENDING**.
5. Independent wrong-signed client, authenticated permanent production trust anchor and full confused-deputy protection **NOT IMPLEMENTED / NOT VERIFIED**.

## Primary Apple references
- https://developer.apple.com/forums/thread/681053
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_code_signing_requirement
- https://developer.apple.com/documentation/technotes/tn3127-inside-code-signing-requirements
- https://developer.apple.com/documentation/xpc/xpc_connection_set_peer_lightweight_code_requirement

```ini
G03_R4K_FIX1=RESEARCH_CANDIDATE
PREVIOUS_R4J=PASS_WITH_CONTROLS
NATIVE_XPC_CODE_SIGNING_REQUIREMENT=CI_PENDING
CLIENT_SIDE_WRONG_SERVICE_REQUIREMENT=CI_PENDING
INDEPENDENT_WRONG_SIGNED_CLIENT=NOT_VERIFIED
AD_HOC_IDENTIFIER=NOT_PRODUCTION_TRUST_ANCHOR
OWNER_MAC_INTEL_14_5=PENDING
NATIVE_ADMISSION_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```


---

## FIX2 owner Sonoma incident (2026-11-09) — **REMEDIATE**

Owner Mac Intel `macOS 14.5` verified exact commit `899c716ebf6b388428ff5584913db43ad365d4a9`, with signed ZIP/script, pinned head and Git source blob. R4K native tests: **3 PASS / 5 FAIL**, exit 101; previously verified R4A–R4J 98 PASS, 3 intentionally ignored remained unchanged. Failure is **not** a missing response. The negative client-side XPC connection set a deliberately false service requirement, then actually obtained:

```text
R4K|WRONG_SERVICE_IDENTITY_NOT_EXPLICITLY_DENIED|TYPE=DICTIONARY|STATUS=probed
```

The test therefore **cannot claim that the outbound libxpc requirement denied the wrong service identity on macOS 14.5**. CI success on newer macOS 15 is evidence only for those OS environments. Exact kernel/libxpc root cause remains undetermined; owner original worktree and GitHub main remain untouched.

## FIX3 countermeasure candidate — *additional*, not less, verification

The unsandboxed client now checks **the actual XPC message sender** before processing an otherwise valid response via `SecCodeCreateWithXPCMessage`, `SecRequirementCreateWithString`, and `SecCodeCheckValidity` against the service's owner-generated signing identifier. A missing kernel-attributed sender, unparsable requirement, or non-valid legitimate sender **fails closed**.

The intentionally wrong-service-identity connection has two permitted and explicitly distinguished rejection witnesses:

1. `BACKEND=LIBXPC`: the OS itself returned `XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT`, with no service reply exposed.
2. `BACKEND=MESSAGE_SECURITY`: the OS delivered a reply *despite* the incorrect outgoing requirement; **on that same actual reply** Security.framework must successfully verify the real service identifier **and** reject the intentionally wrong identifier, while observing the `probed` status. If either lookup fails, if wrong ID matches, if the actual ID fails, or if an unexpected reply type/status arrives, fail.

These are **not interchangeable security claims**: `MESSAGE_SECURITY` is post-receipt sender validation, not proof of libxpc's inbound/outbound enforcement on macOS 14.5. Both use signed ad-hoc *identifier text*, which a hostile signer can reproduce; a trusted production identity anchor is still NOT IMPLEMENTED.

The genuine server-side peer requirement is unchanged and installed before resuming inbound connections; the kernel sandbox denial for the service remains active. No XPC service, user source, filesystem grant or public `.noi` language change is authorized.

### Latest exact-head verification scope

- Owner macOS 14.5 FIX2 = **REMEDIATE** (direct unexpected `STATUS=probed` proof).
- FIX3 Apple Silicon / macOS 15 = **CI PASS** for 106 regular tests and 3 intentionally ignored, plus format/Clippy, observed on code commit `463b008151825f82beeaa4769f5b4c6b38a6732e`.
- FIX3 macOS 15 Intel / portable / final doc HEAD = verify CI before claiming PASS.
- FIX3 owner macOS 14.5 Intel = **PENDING**, exact pinned owner-run isolated replay required.
- Independent wrong-signed client rejection, trusted signer, complete confused-deputy mediation and all 12 native admission gates remain **NOT VERIFIED / UNMET**.

```ini
G03_R4K_FIX2_OWNER_SONOMA14_5=REMEDIATE
G03_R4K_FIX3_OWNER_SONOMA14_5=PENDING
G03_R4K_FIX3=RESEARCH_ONLY
PRODUCTION_SIGNER_TRUST_ANCHOR=NONE
NATIVE_ADMISSION_GATES=12_UNMET
PR75=DRAFT_UNMERGED
PRODUCTION=NOT_AUTHORIZED
```
