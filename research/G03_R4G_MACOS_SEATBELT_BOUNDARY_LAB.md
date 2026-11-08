# NORDOI G0.3 R4G — macOS Seatbelt kernel-enforced file-access denial experiment

**RESEARCH ONLY · TEST FIXTURE · NO PUBLIC `.noi` SANDBOX · PRODUCTION NOT AUTHORIZED**

## Inputs and branch custody

Parent is exact G0.3 R4F FIX1 candidate PR #66 HEAD `df484a4b0677e5863faf63c8ee69b72ad15d77a0`. The owner's Mac Intel R4F FIX1 transcript shows `G03_R4F_FIX1_MAC_INTEL=PASS_PORTABLE_NON_LINUX`: 71/71 ordinary tests R4A–R4F PASS, 2 intentionally ignored child helper tests, Rustfmt PASS, Clippy strict PASS (rustc 1.98.1), exact Git/blob source pin integrity PASS, original `production/p0.1` branch `e2898bf7c511e53636ff28c9fd3efc93165a0215` and `main` sync unchanged. Remote `main` is `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6`. PRs #56 #58 #60 #62 #64 #66 remain draft/unmerged.

The R4G branch is **stacked on R4F**, not main. Adds exactly three files: `tests/g03_r4g_macos_seatbelt_boundary_lab.rs`, this document, and `.github/workflows/g03_r4g_macos_seatbelt_boundary.yml`. No edits to `src/`, compiler, NAIR, kernel, CLI, `.noi` grammar, host APIs, production runtime, package dependencies, or end-user UI.

## Why the mechanism is research-only

Apple's macOS App Sandbox is the **supported** entitlement-and-code-signing model for distributing restricted apps. `/usr/bin/sandbox-exec` can apply a Seatbelt sandbox profile to an arbitrary child, but its macOS man page marks it **DEPRECATED**. Therefore R4G uses it **only to test OS enforcement**, not as a chosen stable production API, signed App Sandbox equivalent, automatic fallback, or general code security guarantee.

Sources:
- Apple: https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox
- macOS manpage (DEPRECATED): https://manp.gs/mac/1/sandbox-exec
- GitHub supported Intel Mac runner label `macos-15-intel`: https://docs.github.com/en/actions/reference/runners/github-hosted-runners

## Real adversarial witness

A trusted parent Rust integration test creates a **new disposable directory** only (no arbitrary paths), canonicalizes it to avoid `/var` symlink alias ambiguity, and writes `R4G_PARENT_ONLY\n` to its sentinel. It spawns the *exact current test executable* in a **distinct OS process** with the ignored `r4g_child_entry` test (not a generic `.noi` task runner). For the purpose of a real adversarial path probe, the parent passes the **disposable target path** by environment; it does **not** pass a parent file descriptor, NAM object, grant or permission.

Two real child runs use the same current UID and the same direct Rust filesystem probes:
1. **Positive control (not sandboxed)**: child `fs::read(sentinel)`, `OpenOptions::write(true).open(sentinel)`, and `OpenOptions::create_new(true).open(new_path)` **all succeed**. Child removes only its own newly created temporary test file. If ordinary access fails, the test rejects the setup instead of attributing a POSIX/TCC denial to Seatbelt.
2. **Seatbelt profile**: parent invokes `/usr/bin/sandbox-exec -p` to run the same child with `(version 1)(allow default)(deny file-read-data (subpath "<canonical test directory>"))(deny file-write* (subpath "<canonical test directory>"))`. The parent requires that **the same three operations all fail** under actual kernel mediation, that the child exits successfully, that the transcript contains one exact result marker, and that the parent sentinel and create target are unchanged. `allow default` means the profile is **NOT a general sandbox** or global deny-by-default policy, only a narrow protected-directory denial experiment.

A disabled/missing `sandbox-exec`, invalid profile, forged/duplicate/malformed frame, timeout, excessive child output, child crash, or failed control MUST fail the corresponding positive test; no synthetic fallback `PASS`. A separate check confirms the parent remains free to append after the child has been restricted. The child waits for no host credentials and cannot mutate parent NAM via this fixed test path.

GitHub target platforms: **macos-15-intel** x86_64 and **macos-15** Apple Silicon prove the actual macOS-denial experiment if both job results PASS. Ubuntu/Windows may only prove compilation and that the macOS backend is explicitly absent and R0.2 `ExternalIo` remains denied, **not** Seatbelt enforcement. Additional version-specific CI can test strict Clippy on Rust 1.98, matching owner Mac compiler. Owner's macOS 14.5 Intel replay is separate and required before claiming this milestone MAC_INTEL_PASS.

## Limitations and remaining security gates

- This profile permits everything **outside one disposable test directory**. It does not prove isolation of arbitrary host files, network, process control, clocks, devices, inherited handles, symlinks/hardlinks, sandbox escape resistance, or a general untrusted-child runtime.
- The `sandbox-exec` interface is deprecated. It may be removed/disabled or behave differently on future macOS versions. Positive CI is an OS-policy witness only; it does not authorize relying on the deprecated interface in product architecture. A dedicated signed App Sandbox helper with entitlements is a separate future design.
- Other-process file descriptors inherited before policy installation, open-file handles, crash recovery, fsync, atomic filesystem+NAM commits, and real cancellation of blocking syscalls remain unproven.
- A current fixed Rust integration-test executable is used, not arbitrary user-supplied `.noi` code. Reference R0.2 `ResourceEffect::ExternalIo` remains explicitly unsupported.
- **All 12 R11 native admission gates G01–G12 remain UNMET.** No production, release, branch merge, or general security certification.

## Acceptance statement

```text
R4G=MACOS_SEATBELT_RESEARCH_ONLY
LINUX_LANDLOCK_R4F=SEPARATE_LINUX_CI_EVIDENCE
MACOS_POSITIVE_CONTROL_AND_KERNEL_DENIAL=CI_REQUIRED
WINDOWS_MACOS_SEATBELT_BACKEND=NOT_APPLICABLE
OWNER_MAC_INTEL=NOT_YET_TESTED
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```

**Constitutional design note:** extreme simplicity for a non-programmer and extreme execution performance remain design requirements. All sandbox/profile ceremony is internal/test-only. No outward syntax or permanent per-task overhead is authorized by this lab.
