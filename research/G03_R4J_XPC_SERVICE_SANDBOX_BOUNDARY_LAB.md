# NORDOI G0.3 R4J — Genuine bundled XPC sandbox-service boundary (research)

**P0 RESEARCH ONLY · FAIL CLOSED · NOT PRODUCTION · NO `.noi` SURFACE CHANGE**

## Parent and custody
R4J stacks on R4I draft PR #72 exact HEAD `a3cc9ee41b55f34ceab6f052964cc6929d2309de`, owner Mac Intel 14.5 receipt: **92/92 regular R4A–R4I tests PASS**, 3 helper entries intentionally ignored, Clippy `-D warnings` and Rustfmt PASS; original `production/p0.1` at `e2898bf7c511e53636ff28c9fd3efc93165a0215` and `main` at `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6` unchanged. R4A–R4I PRs #56/#58/#60/#62/#64/#66/#68/#70/#72 draft/unmerged.

Additive-only three files: `tests/g03_r4j_xpc_service_sandbox_boundary_lab.rs`, this research report and `.github/workflows/g03_r4j_xpc_service_sandbox_boundary.yml`. No production code/CLI/kernel/NAIR/compiler, public grammar, .noi syntax or external dependencies modified.

## Actual native low-level XPC test (not fake sockets, pipes or ordinary child inheritance)

Apple's XPC docs describe a **private launchd-managed .xpc service embedded within a host .app**, communicating via libxpc and running in a separate restricted environment. R4I only proved that `posix_spawn` helper inherits the same App Sandbox; R4J targets **differing privileges** between an unsigned-entitlement client and a separately App-Sandbox-entitled service.

The macOS Rust integration test constructs a fresh temporary owner-controlled directory, one sentinel `R4J_PARENT_ONLY\n`, a same-directory nonexistent creation target, and native fixed C source (not arbitrary user input). It compiles two binaries using macOS system cc: the **client** links libxpc and runs as a bundled .app executable, and the **service** links libxpc and lives in `Contents/XPCServices/<unique-id>.xpc/Contents/MacOS/r4j-service`. Both bundles have real Info.plist, and the .xpc declares `CFBundlePackageType=XPC!`, `XPCService.RunLoopType=dispatch_main`. Both are ad-hoc signed (inner XPC service first, then outer client .app). The service carries **only** `com.apple.security.app-sandbox=true`; client has **no App Sandbox entitlement**, establishing an intentionally asymmetrical research policy. The Rust parent checks actual signed entitlements and verifies both signed bundles.

The client proves read/open-write/create **ALLOWED** against sentinel/creation target using real C OS calls, then contacts its private bundled XPC service via `xpc_connection_create(service-name, NULL)`, sends `probe` using a real XPC dictionary and `xpc_connection_send_message_with_reply_sync`. Only a **real distinct service PID** and direct service read/open-write/create **DENIED** counts as positive evidence. A second XPC request `writefile` MUST receive `status=denied`. The exact output contains three frames and may not be forged or duplicated. Both client and service must exit / reply cleanly, with bounded client stdout/stderr and deadline; a missing/unregistered private XPC service fails and **never gets a synthetic fallback**. Test driver checks parent-owned sentinel unchanged and child-created path absent. No launchd installation, global Mach service registration, sudo, security-entitlement exception or network permissions are authorized.

## What this does *not* prove

- A separately sandboxed service is not automatically authenticated by an immutable identity or peer audit token here. The test checks a real different PID and the correct OS filesystem-denial outcome, but does not establish protocol robustness against an adversarial installed client or service substitution in a compromised environment.
- The simple `probe` protocol takes a disposable test path; **do not generalize** to user path grants. A production service must validate client peer identities, audit tokens and authorization of each operation, protect against confused-deputy path abuse and avoid credentials/host handles leaking across IPC.
- The client in this lab is deliberately **not sandboxed** as a positive control to demonstrate privilege asymmetry; a complete deployed architecture with a sandboxed client, user-consented access, XPC service's separate resource authority and complex helpers is a distinct project.
- Ad-hoc signatures are not Apple Developer ID, notarization, production signing, Apple-approved distribution or installation. No user code is executed. App Sandbox protects the specific attempted directory; there is no claim of full filesystem/network/process/IPC/device isolation, denial of preexisting inherited handles, or adversarial escape resistance.
- R0.2 `ResourceEffect::ExternalIo` still unsupported. Parent NAM writes and host filesystem are not atomic. **All 12 R11 native admission gates remain UNMET**. No merge, release, deployment or other production authorization.

## Fail-closed acceptance

Positive macOS 15 Intel and macOS 15 Apple Silicon GitHub jobs must actually compile, sign, verify, use launchd-managed XPC, and see XPC service denial of all 3 actual OS calls with separate PID, plus rejection of unrecognized write operation. If host CI does not register the XPC service for a test app directly launched from a temporary directory, mark `R4J_MACOS_XPC=REMEDIATE`, NOT PASS. Other OS (Windows/Linux) may only report absent macOS XPC and persistent ExternalIo denial; they do not prove XPC. Strict rustfmt and Clippy Rust 1.98.1 and general Test Gate required. Owner macOS 14.5 Intel pinned-clone replay is mandatory for final local approval.

## Apple primary references
- https://developer.apple.com/documentation/xpc/creating-xpc-services
- https://developer.apple.com/documentation/xpc
- https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingXPCServices.html

```text
G03_R4J=RESEARCH_ONLY
LAUNCHD_PRIVATE_XPC_SERVICE=REQUIRES_LIVE_CI_EVIDENCE
XPC_CLIENT_TO_SERVICE_DIFFERENT_PRIVILEGES=TEST_SCOPE_ONLY
XPC_CLIENT_IDENTITY_AUTHENTICATION=NOT_IMPLEMENTED
OWNER_MAC_INTEL=PENDING
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
