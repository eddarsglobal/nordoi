# NORDOI G0.3 R4F — Real Linux kernel Landlock filesystem-denial witness

**RESEARCH_ONLY · TEST_ONLY · NOT A GENERAL SANDBOX · NO PRODUCTION AUTHORIZATION**

## Provenance and non-mutation boundary

Parent is exact R4E draft PR #64 HEAD `e6fbda5162be0c8762b3fd5904f17baa5884ce4f`, already validated on Intel Mac with 67/67 regular R4A–R4E tests PASS, one intentionally ignored R4E subprocess entry, Rustfmt and Clippy PASS. Current GitHub `main` remains `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6`. Owner Mac original `production/p0.1` `e2898bf7c511e53636ff28c9fd3efc93165a0215` was preserved in prior receipts. R4A/#56, R4B/#58, R4C/#60, R4D/#62, R4E/#64 are all DRAFT/UNMERGED.

R4F adds only a test-only Rust file, this research document, and CI workflow. No `src/` source edits, new dependencies, `.noi` syntax, CLI, compiler, NAIR, kernel, grant semantics, or public runtime executor.

## New evidence: actual Linux kernel mediation (not merely IPC)

The Linux test executable launches the exact same Rust integration test binary in a *distinct OS child process*, filtered to a fixed ignored `r4f_child_entry`. Parent creates exactly one new private temporary directory, writes a known sentinel file, and passes only its test path via environment **specifically to attempt hostile direct file access**. No File handle, NAM object, host grant or permission token is sent.

Inside the child (and **before** any test-controlled file access), test-only FFI calls:
- `landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION)` to check Landlock kernel ABI, **fail closed** on unsupported/disabled kernel;
- `landlock_create_ruleset(...)` with **the complete ABI v1 filesystem rights** in `handled_access_fs`, **no allow rules**, so read, write, dir, execute, create and remove rights are default-denied;
- `prctl(PR_SET_NO_NEW_PRIVS, 1, ...)`, then `landlock_restrict_self(fd, 0)`, and closes ruleset FD;
- direct child `std::fs::read`, `OpenOptions::write`, `OpenOptions::create_new` against the *known path*, each of which must get `PermissionDenied` from the kernel.
 
A positive Linux test **fails** if the kernel lacks Landlock, kernel mediation fails, any direct access succeeds, the child exits nonzero, or the 10-second deadline is exceeded. Parent confirms that sentinel content was untouched and no new child-created file appeared; a separate Linux test confirms parent can still read and append after child policy enforcement.

CI independently runs the Linux positive enforcement evidence on a Linux runner. **macOS and Windows do not implement Landlock**; they run explicit fail-closed unsupported tests, not a fake/skip-coded sandbox PASS. The 3-OS workflow tests correct platform behavior, but **only Linux** demonstrates this particular real OS kernel boundary.

## Failure semantics, limits and adversarial scope

Negative tests reject an unsupported kernel result, a missing/altered/duplicated Landlock success frame, or a timed-out subprocess. This is an OS filesystem-denial witness against the attempted paths, not an exhaustive hostile code exercise.

**Severe limitations that must stay visible:**
- Linux Landlock is a filesystem restriction. ABI v1 **does not** establish network, IPC, process spawning, CPU/memory ceilings, arbitrary device/IOCTL filtering, syscall filtering, system-call-level cancellation, existing inherited-FD revocation, crash recovery or cross-store NAM/disk atomicity.
- In particular, the child inherits some OS identity and pre-opened stdio FDs. Landlock restrictions do not retroactively revoke inherited opened file descriptions.
- This is not a generic .noi compiler-to-sandbox binding or a backend automatically selected at run time. The test child is a fixed trusted Rust fixture, not untrusted or arbitrary user code.
- Mac Intel **cannot perform the Linux positive test natively**. An owner Mac run verifies the expected macOS unsupported outcome and R4A–R4E regressions. Do not claim it demonstrates macOS sandboxing. Future Apple sandbox controls require an independent, supported, owner-consented macOS test.
- R0.2's `ResourceEffect::ExternalIo` remains intentionally unsupported, and there are no real nonforgeable OS host credentials, general task scheduler or robust crash recovery.

## Acceptance contract

GitHub exact-head R4F Linux positive tests must confirm kernel-enforced real file READ/WRITE/CREATE denial. Mac/Windows jobs must confirm unsupported/default-denied status. Rustfmt and Clippy -D warnings, main Test Gate, and exact-commit Mac Intel isolated source verification are required, but the Mac does not substitute for positive Linux enforcement. The full positive evidence is per-Linux job only. **All 12 R11-G01–G12 native-admission gates remain UNMET**, including full cross-OS native backend and independent review.

FNG C2/C3/C6/C7/C9/C14/C17/C18/C19/C20 aligned: explicit security proof without new complexity for public NORDOI; Linux-only specialization does not silently generalize to all OS. No automatic merge of stacked PR #56/#58/#60/#62/#64 or R4F; no production deployment.

```text
R4F=RESEARCH_ONLY
LINUX_LANDLOCK_OS_ENFORCEMENT=REQUIRES_CI_PASS
MACOS_WINDOWS_LANDLOCK=UNSUPPORTED_FAIL_CLOSED
OWNER_MAC_INTEL=NOT_YET_TESTED
NATIVE_ADMISSION_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```

## Upstream kernel reference

- https://docs.kernel.org/userspace-api/landlock.html
- https://www.kernel.org/doc/html/v6.6/userspace-api/landlock.html
