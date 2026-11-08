# NORDOI G0.3 R4E — Bounded child process with parent-only host effects

**RESEARCH_ONLY | DRAFT_CANDIDATE | NO PUBLIC .noi PROCESS EXECUTOR | PRODUCTION_NOT_AUTHORIZED**

## Source and custody
R4E is stacked on exact R4D draft PR #62 head `7fb707145ac9354b3d6e8c88c9fc80da7e1008c5`. R4A/B/C/D PRs #56/#58/#60/#62 remain DRAFT and UNMERGED. Main is `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6`. The owner previously validated R4A–R4D 53/53 Rust tests on Mac Intel in a **disposable clone** with the original `production/p0.1` worktree unchanged; that receipt is not a new R4E validation.

Exactly three additive research files: `tests/g03_r4e_cross_process_host_boundary_lab.rs`, this research report and `.github/workflows/g03_r4e_cross_process_boundary.yml`. All existing source, kernel, NAIR, compiler, runtime, CLI and user-facing language remain unchanged.

## New evidence, distinct from R4D

An actual **separate OS child process** runs the fixed Rust integration-test executable with precisely selected ignored test `r4e_child_entry`. The parent launches its exact current test executable, not an arbitrary shell command, and uses **stdin and stderr pipes** to exchange a tiny fixed protocol. The child performs only checked integer addition from a fixed test message, returns an explicit READY frame and one completion frame, or terminates abnormally. The subprocess **does not receive the parent File object, its path, the NAM kernel, resource identifiers or model permits in arguments or protocol**. This establishes only a disciplined parent/child implementation interface, not safe execution of malicious or arbitrary code.

The trusted parent retains:
- R0.2 model with explicit *modeled* Write declaration/grant and resource acquisition, task transitions and reference replay;
- an actual host OS File created with `OpenOptions::create_new(true)` in one new disposable directory;
- all actual filesystem writes, which occur only after a successful child exit, exactly one valid READY and RESULT frame, a non-cancelled task and an explicit modeled grant;
- all NAM mutation, which is separately conditioned on successful modeled scope close and reference replay.

The reader enforces a bounded stderr transcript; timeouts kill and reap the actual child and join the read thread. Correlation is a fixed test-run marker to detect stale/malformed frames: **not cryptographic authentication**. The child has no parent grant itself. Cancellation is initiated only **after actual child READY** and acknowledged by a return frame after parent sends CANCEL. There is no preemption of an arbitrary blocking syscall or general interrupt delivery.

## Tests and fault surfaces

The focused integration tests include successful real child computation/parent-only file+NAM; success response with no grant refused; live child ready/parent cancellation acknowledged; forged success after cancel refused; unrequested host-write request denied; forged correlation; duplicate success frames; checked arithmetic overflow; real child panic and nonzero exit; child early exit; stalled child killed/reaped; excessive child output limited; R0.2 ExternalIo explicitly unsupported; disposable parent file cleanup.

The single ignored test function is the child entrypoint: test-runner summaries may show N regular tests passing plus one ignored child helper. Distinguish functions from actually executed tests.

## Critical security and engineering limitations

**This is NOT an OS security sandbox**. The child inherits the invoking user's ambient privileges and could call arbitrary OS APIs if hostile or compromised. Passing no File or path is **not** revoking its OS capabilities; same-user child processes are not isolated from the entire filesystem by this fixture. Exact executable identity within this test is not code signing. A fixed correlation string on pipes is not nonforgeable authentication. Real process termination alone does not prove absence of child descendants or arbitrary cleanup. No process/resource quotas beyond fixture transcript and deadlines, no syscalls denied or filtered, no platform sandbox integration, no user .noi process executor or general scheduler.

R0.2 **still rejects ResourceEffect::ExternalIo**; a modeled Write grant is *not* a legitimate production permission to write a real host file. Disk writes can be partial, and disk/NAM are not atomic; no crash durability or rollback guaranteed.

R4E must NEVER mark R11-G04 or G06 met because the actual OS authority is still ambient to the child. In fact **all 12 R11-G01…G12 remain UNMET**. R4E can demonstrate a genuinely different process / protocol boundary with failure handling, not a security boundary capable of running untrusted code.

## Acceptance and next truthful step

- Exact R4E test target compiles and runs in CI macOS, Windows and Ubuntu (fixed real process behavior); format check; general NORDOI Test Gate 5/5.
- Owner Mac Intel exact-commit isolated clone, pinned 5 test-source blobs, combined R4A–R4E tests and `cargo fmt --check`/`clippy -D warnings`. Do not modify local `production/p0.1`.
- If accepted, R4F should investigate a **real OS-enforced lower-privilege child** and privileged host isolation with platform-specific adversarial evidence, or explicitly say support is not yet feasible. Before that, do not claim nonforgeable grants or admission.

```text
R4E=RESEARCH_ONLY
OWNER_MAC_INTEL=PENDING
PR56_PR58_PR60_PR62=UNMERGED
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
