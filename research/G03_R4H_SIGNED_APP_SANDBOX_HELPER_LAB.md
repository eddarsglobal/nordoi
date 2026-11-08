# NORDOI G0.3 R4H — Ad-hoc signed App Sandbox helper (macOS-only research)

**RESEARCH ONLY · NOT PRODUCTION SIGNING · NOT A GENERAL UNTRUSTED .noi EXECUTOR**

## Provenance and immutable source custody

R4H stacks on R4G draft PR #68 exact commit `b3164d51fb7613e5b0a2c5b9767760f053348d4f`. Owner Mac Intel 14.5 R4G result: R4A–R4G **76/76 PASS**, Rustfmt and Clippy 1.98.1 strict PASS, original `production/p0.1` HEAD `e2898bf7c511e53636ff28c9fd3efc93165a0215` untouched, local/remote `main` `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6` synchronized. R4G GitHub CI 4/4 macOS Intel/ARM/Linux/Windows correct-per-platform behavior, general Test Gate 5/5 PASS. PRs #56/#58/#60/#62/#64/#66/#68 all DRAFT/UNMERGED.

R4H has **only three additive files**: `tests/g03_r4h_signed_app_sandbox_helper_lab.rs`, this report, and `.github/workflows/g03_r4h_signed_app_sandbox_helper.yml`. It does not alter NORDOI source/kernel/runtime/CLI/compiler/NAIR, public .noi syntax, admission gates, or original owner repo.

## Why this is different from R4G

Apple documents **App Sandbox** as an OS-enforced, signed-entitlement technology. Unlike R4G (deprecated `/usr/bin/sandbox-exec -p`), R4H attempts actual entitlements on an executable itself. Apple recommends Xcode target Signing & Capabilities for real products, including appropriate Developer signing and helper inheritance; **R4H only proves research behavior for disposable local *ad-hoc signed* helper executables**.

The test Rust parent (existing NORDOI integration-test binary) creates one new temporary directory, one parent-owned sentinel with `R4H_PARENT_ONLY\n`, and one fixed C helper source. It uses the macOS system `/usr/bin/cc` to compile a simple, non-untrusted helper and makes an identical copy.

Both helper copies are signed with `/usr/bin/codesign --force --sign - --identifier org.eddarsglobal.nordoi.r4h.fixture`, i.e. **ad-hoc signing only**. The test signs ONLY the constrained helper with a minimal plist containing exactly `com.apple.security.app-sandbox = true`; the identically compiled and separately signed control copy deliberately has **no** App Sandbox entitlement. Parent requires `codesign --verify --strict --verbose=2` SUCCESS and inspects **actual signed entitlements** through `codesign --display --entitlements :-`. A separate test mutates one byte in a copied signed executable and requires code-signature verification FAIL.

Both children run as the same OS user and attempt direct C system calls against the parent sentinel (read one byte and open for write, without altering its contents), and the child creation target (`open(O_CREAT|O_EXCL)`). Exact stdout `R4H|OBS|READ=1|WRITE=1|CREATE=1` is mandatory for the signed non-entitled control; `R4H|OBS|READ=0|WRITE=0|CREATE=0` is mandatory for the signed App Sandbox helper. Files and temporary source created for this bounded test are deleted after the test. The parent verifies sentinel unchanged and child target absent, retains ability to append, and rejects forged or duplicate stdout. Successful tests therefore provide a narrow **actual OS-backed App Sandbox entitlement witness**; no claimed behavior is inferred from a signature or source plist alone.

If ad-hoc signatures do not actually activate App Sandbox on a given macOS version or in CI, the positive test MUST fail rather than labeling a phantom PASS. No fallback to deprecated `sandbox-exec`.

## Acceptance and exact scope

- macOS 15 Intel and Apple Silicon GitHub native CI jobs must actually build, ad-hoc sign, verify the signatures/entitlement and observe direct read/write/create denied with signed control allowed. Strict Clippy `-D warnings` and rustfmt with 1.98.1.
- Linux/Windows can only test reference-model `ExternalIo` default-denial and that App Sandbox is not supported, NEVER claim signed macOS isolation.
- NORDOI general Test Gate 5/5 PASS. Owner macOS 14.5 Intel isolated exact-Git-SHA and seven-plus-one blob pins, Rust tests and strict Clippy; owner original repository untouched.

## Serious limits and next real gap

Ad-hoc signing in CI needs no Developer account but **is not Apple Developer ID signing, notarization, distribution trust or guarantee of the production Xcode/App Sandbox helper model**. The research helper is a trivial fixed C executable compiled inside the test. It is **not** a cross-user, arbitrary untrusted Rust/.noi process, not a fully protected OS file/network/IPC/device surface, not a user-selected entitlement adapter, not security against sandbox escape and not production-ready. Existing/open-inherited FDs, signed nested helpers, hardened runtime, XPC inheritance, process quotas, memory budgets, user prompts, audit logging and cross-store disk/NAM atomicity remain unproven. A successful restricted test folder does not certify the entire host filesystem; confirm that OS enforcement is real against that target by having the same-user non-entitled control successfully access it.

Reference R0.2 `ResourceEffect::ExternalIo` remains intentionally unsupported, and parent modeled `Write` is not a production host-I/O authorization. **All 12 native readiness admission gates remain UNMET.** Nothing in this work authorizes native .noi execution, production deployment, publication or PR merging. The simplicity and speed goals of public NORDOI syntax remain protected: sandbox rules are an internal runtime concept, not user syntax.

## Apple primary docs

- App Sandbox configuration: https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox
- Protected user data and verification: https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox
- Embedded helper tool constraints: https://developer.apple.com/documentation/security/app-sandbox

```text
R4H=RESEARCH_ONLY
MACOS_APP_SANDBOX=AD_HOC_SIGNED_HELPER_LAB
DEVELOPER_ID=NOT_USED
NOTARIZATION=NOT_DONE
MACOS_SIGNED_ENTITLEMENT_ENFORCEMENT=NEEDS_CI_AND_OWNER_EVIDENCE
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
