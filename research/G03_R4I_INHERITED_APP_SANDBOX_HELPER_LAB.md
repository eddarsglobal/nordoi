# NORDOI G0.3 R4I — inherited App Sandbox across a real embedded helper process

**RESEARCH LAB ONLY · AD-HOC SIGNED · NOT XPC · NOT PRODUCTION · NO PUBLIC `.noi` EXECUTOR**

## Source custody and scope

This candidate is stacked on **R4H PR #70** exact HEAD `41f138884b684f6e89b102104824e5fe41d801fc`, not on `main`. Owner macOS 14.5 Intel R4H verified **84 regular R4A-H tests PASS**, 3 intentionally ignored child entries, strict clippy+rustfmt PASS; original `production/p0.1` `e2898bf7c511e53636ff28c9fd3efc93165a0215` and local `main` `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6` remain preserved. All upstream R4A-H PRs #56/#58/#60/#62/#64/#66/#68/#70 remain draft/unmerged.

Three additive research files ONLY: `tests/g03_r4i_inherited_app_sandbox_helper_lab.rs`, this report, and dedicated CI. No `src/`, user-facing `.noi` grammar, CLI, production runtime, kernel, NAIR, compiler, external crates, or real permissions are changed.

## Why R4I is a new experiment

R4H showed one executable's **own** ad-hoc signed App Sandbox protection. R4I tests **inheritance across a genuine parent → helper process boundary**. Apple states an embedded child tool that inherits from a sandboxed parent must carry exactly two entitlements: `com.apple.security.app-sandbox` and `com.apple.security.inherit`. XPC is preferred for stronger privilege separation; creating a child process by itself does not add the XPC security boundary.

Mac-only fixture constructs from fixed C source:
1. New disposable parent-owned directory and sentinel. Parent Rust test stays outside App Sandbox. It compiles one helper source and an App Sandbox parent source with system `cc`.
2. Parent app `R4IParent.app` has a real `Info.plist`, signed **ad hoc** using `/usr/bin/codesign` with **only `com.apple.security.app-sandbox=true`**.
3. Embedded tool at `R4IParent.app/Contents/Helpers/r4i-helper` is **separately code-signed** with **exactly two** entitlements, `com.apple.security.app-sandbox=true` and `com.apple.security.inherit=true`. Tool is signed **before** its containing app. The unsandboxed control is the very same compiled helper copied before signing and signed **without** sandbox entitlements.
4. Parent test inspects and verifies signatures plus **signed entitlements** read from actual binaries using `codesign --display --entitlements :-`. Missing/extra helper entitlement keys must fail, and tampering a signed copy must make code signature verification fail. No `get-task-allow` or network entitlements are allowed on the helper.
5. Rust test runs separately signed **unrestricted control** and requires real read/open-write/create against the identical test sentinel and sibling creation target **all succeed**.
6. Rust test runs real ad-hoc signed **sandboxed parent** (no `sandbox-exec`). Parent attempts read/open-write/create and must be denied by OS. It invokes `posix_spawn` of its separately signed embedded helper. The helper independently attempts the **same three operations** and must be denied. The parent waits on the actual child exit and forwards exactly one bounded helper receipt. No grant, file descriptor or model state is passed to the child. The full stdout must match exactly TWO frames, in order, containing both denied results; malformed, missing, duplicated or positive access results fail.
7. Rust test (unsandboxed driver) can still read/append to the parent-owned file after the sandbox processes exit; sentinel remains unmodified. Multiple fresh independent parent/child pairs exercised.

**Critical distinction:** if a sandboxed parent fails to launch its helper because its signature/entitlement is invalid, this is **FAIL**, not accepted as equivalent to a restricted child. If macOS declines to enforce inheritance, any successful file access is likewise **FAIL**. Only both processes completing cleanly with denied actual syscalls qualifies.

## Exact boundaries

- This is an **ad-hoc signed research fixture** inside a newly created private temporary directory, not distribution signing, Developer ID, Apple notarization, hardened runtime release, installation, App Store / Xcode archiving, or third-party malicious `.noi` code.
- Demonstrates inherited restrictions for specific direct filesystem probes to a disposable target, not exhaustive network, IPC, process controls, resource quotas, OS exploit resistance, privileged paths, inherited FDs or general nested process trees.
- Apple explicitly recommends **XPC** for stronger process isolation; R4I **does not** claim privilege separation simply from `posix_spawn`. A signed helper with inherit is expected to have the **same sandbox** as the parent, not a stricter sandbox by default.
- macOS 15 Intel and Apple Silicon GitHub jobs require actual positive OS evidence. Windows/Linux only verify their platform is unsupported and R0.2 `ExternalIo` remains denied. Owner macOS 14.5 Intel exact-source pinned isolated replay is a separate requirement.
- If signing/launch behavior differs across OS versions, a failed test remains a failed test: no permissive fallback, hardcoded success or relabelled gate.
- R0.2 `ResourceEffect::ExternalIo` remains unsupported, host I/O is not authorized, disk and NAM are not crash-atomic; **all 12 R11 native admission gates remain UNMET**. No release, merge, deployment, production or user-permission changes.

## Authoritative Apple guidance
- https://developer.apple.com/documentation/xcode/embedding-a-helper-tool-in-a-sandboxed-app
- https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html
- https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox

```text
R4I=RESEARCH_ONLY
APP_SANDBOX_HELPER_INHERITANCE=CI_AND_OWNER_EVIDENCE_REQUIRED
R4I_CHILD_EXECUTION=REAL_POSIX_SPAWN
SIGNED_ENTITLEMENTS=APP_SANDBOX_AND_INHERIT_ONLY
APPLE_DEVELOPER_ID=NOT_USED
NOTARIZATION=NONE
OS_SANDBOX_IS_NOT_XPC=TRUE
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
