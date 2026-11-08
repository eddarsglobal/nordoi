# NORDOI G0.3 R4C — Real OS file handle, scoped ownership and cancel/complete ordering

**RESEARCH_ONLY · FIXED-PROGRAM_TESTS_ONLY · PUBLIC_NORDOI_EXECUTOR=ABSENT · PRODUCTION=NOT_AUTHORIZED**

## Input custody and Git topology

Local FIX1 sync succeeded: `main` advanced by fast-forward to `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6` without a checkout; `production/p0.1` remained the active local branch at `e2898bf7c511e53636ff28c9fd3efc93165a0215`, with no uncommitted changes. Remote R4A and R4B refs fetched at `5b500e5c5cb2c67910ea56a34e81967995b73320` and `eb73c232da6f24ac30ce66e139cb668fd79acf93`. The owner did not authorize merging PR #56 or #58; these remain drafts.

R4C is a third **stacked, additive** test branch on R4B HEAD. It must not be merged unless separately approved. R4C tests verify behavior on actual OS file handles in a disposable test directory, not merely the R0.2 in-memory resource model.

## Actual experiment

Test file: `tests/g03_r4c_native_file_resource_lab.rs`. It imports unmodified R0.2 `src/resource_task_r02.rs` via a test-only path and uses the already existing `nordoi_kernel` K1.18 NAM. Production source files, public .noi syntax, NAIR, existing kernel, CLI, and host-permission boundary are unchanged.

**Trusted parent** creates a directory `$TMPDIR/nordoi-r4c-test-<pid>-<sequence>` with `fs::create_dir`, and a single OS file `bounded.txt` with `OpenOptions::create_new(true)`. It retains the native `File` handle; zero paths or handles reach the child. On scope exit, the test removes only the newly created disposable directory. No remote machine, real user files, actual network, privileged device or uncontrolled filesystem path may be opened.

An R0.2 root scope declares `ResourceEffect::Write`, gets an explicit **modeled** host grant, acquires and later releases a modeled resource. A separately implemented, minimal host-file guard requires an exact test lease, its scope identity, live state and non-revoked flag for any actual OS write. The parent explicitly revokes this guard when it revokes its modeled grant in the test. This is a **paired test witness**, not a product boundary where arbitrary Rust or .noi code can request grants.

A real Rust scoped thread reaches a zero-capacity-channel checkpoint; the parent either requests cancellation and sets an atomic Release bit **before** opening a Barrier, or allows completion and tests that subsequent cancellation is rejected by the R0.2 lifecycle. The child checks that bit using Acquire after the barrier. All native threads are joined before the parent writes the file or NAM, and the parent closes the OS file handle and releases the modeled resource before closing the scope. A synthetic worker panic/failure leaves file bytes empty and NAM version unchanged.

**Critical cross-store limitation:** a filesystem write is NOT an atomic NAM transaction and is NOT rolled back on an OS error. The test uses a successful, tiny, controlled write and verifies it, but does not certify crash consistency, all-or-nothing disk+NAM, or a real runtime trust boundary. The modeled `ResourceEffect::Write` is not equivalent to a certified real `ExternalIo` effect; R0.2 deliberately rejects `ExternalIo`. An actual host adaptor must resolve that semantic mismatch before R11-G04/G06 can be evaluated as met.

## Adversarial tests

The 13 tests exercise:
1. Cancellation after a **real child checkpoint**, before release, yields a Cancelled report and zero OS file bytes/NAM writes.
2. Child completion followed by late cancellation is rejected; only the parent writes the one-byte-bounded payload `42\n` after the join.
3. Failure and 4. actual Rust worker panic do not write native file bytes or NAM state.
5. Cross-scope ownership denial and 6. invalid (tampered) lease denial before OS write.
7. Explicit paired modeled + native revocation prevents further writes.
8. Duplicate close and use-after-close denied.
9. Model scope close blocked while resources/tasks outstanding.
10. R0.2 denies ExternalIo host grants.
11. Drop cleans the newly created private test directory.
12. Native `create_new` refuses replacement of an existing file.
13. Multiple cancellation-first trials without deadlines or OS scheduling assumptions.

CI targeted test and Rustfmt are required to PASS on macOS, Linux, Windows; full Test Gate must pass. Owner Mac Intel may reproduce by pinned isolated clone without touching the current local `production/p0.1` branch.

## Future-native constitutional gate

- FNG1: gives evidence toward C2, C3, C6, C7, C9, C14, C17, C18, C19, C20, not a general runtime certificate.
- FNG2: moves from purely synthetic resource transcripts to a **real**, bounded OS file-handle effect.
- FNG3: public .noi syntax remains unchanged; native handle/lease/Join ceremony exists only in the Rust laboratory.
- FNG4: wrong owner/lease, revoked authority, late cancel, double close and partial effects denied in the tested paths. **No** cross-store atomicity or malicious Rust sandbox.
- FNG5: platform-independent std Rust File and thread::scope, validated independently on three OS CI jobs.
- FNG6: no edits to source, compiler, kernel, NAIR, user language, old baseline, main or production branch.

## Unresolved gates

**ALL 12 native readiness admission gates R11-G01…G12 remain `UNMET`**. This narrow lab does not demonstrate a secure general-purpose scheduler, nonforgeable ambient authority, revocation from untrusted Rust, ownership of arbitrary native handles, dynamic user input, interruption during blocking syscalls, cancellation fairness, reliable rollback of external IO, crash-restart recovery or production certification.

Next R4D should only proceed after the owner Mac Intel run and an engineering review that prioritizes a minimal **internal host adapter** contract before general .noi concurrency. Do not turn this tiny host-test API into public syntax. This milestone does not require a human sign-off to run research tests, but has NO authorization to deploy.

```text
R4C=RESEARCH_ONLY
R4A_PR56=DRAFT_UNMERGED
R4B_PR58=DRAFT_UNMERGED
R4C_PR=DRAFT_UNMERGED
NATIVE_ADMISSION_GATES=12_UNMET
PRODUCTION=NOT_AUTHORIZED
```
