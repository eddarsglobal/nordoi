# NORDOI G0.3 R4D — Single-critical-section native host adapter laboratory

**TEST_ONLY · RESEARCH_ONLY · NO NEW PUBLIC NORDOI LANGUAGE/RUNTIME · PRODUCTION_NOT_AUTHORIZED**

## Exact provenance and branch custody

The owner provided an isolated Mac Intel `G03_R4C_MAC_INTEL=PASS` log with R4A 10 + R4B 15 + R4C 13 = **38/38 PASS**, `rustfmt` and `clippy -D warnings` PASS and the previously synchronized local `main` preserved. The original local `production/p0.1` branch and its HEAD `e2898bf7c511e53636ff28c9fd3efc93165a0215` remained untouched. GitHub baseline main is `277d9a1af9aeed4dbd4afb1306e2931bc4439cb6`, R4A PR #56, R4B PR #58 and R4C PR #60 are draft/unmerged.

R4D is an additional *stacked* candidate, branched from R4C exact HEAD `73b1ffc12de4f3b23ff92ccede5cef02fbb96323`, not `main` or `production/p0.1`. The only new files are the lab test, this non-normative report and a dedicated CI workflow. No public syntax, compiler, NAIR, kernel, runtime, CLI, production source, or external authority is edited.

## Scope and actual witness

`tests/g03_r4d_governed_host_adapter_lab.rs` implements a tiny, **trusted Rust test-only** host adapter. It creates one actual OS File using `OpenOptions::create_new(true)` inside a newly created disposable temporary directory. There are no user-supplied paths and no network, process, device or environment privilege. The parent owns the source of truth: the unmodified R0.2 `Model`, modeled `Write` grant, scope/resource/task identities, the OS File, and an internal byte quota. Workers receive only an `Arc` to the trusted adapter plus a local test Ticket, never a raw native File, filesystem path, NAM kernel or untrusted guest program.

**Single lock invariant:** all modeled `Command::Use` checks and actual `File::write_all` run inside the **same `Mutex<State>`** as model grant revocation, cooperative cancellation and closing of the OS file. A revoke cannot be inserted in this *in-process test harness* between grant check and the native write. Therefore, if the parent revokes before worker entry, no new bytes are written; if a previously authorized worker write wins the lock, its bytes persist and all later writes fail closed. No timing guesses: test order is controlled by a zero-capacity channel and barrier. These are **ordering witnesses**, not general scheduler fairness or linearizability proofs of arbitrary OS operations.

Tickets are scope-bound and generation-checked; wrong scope, missing, stale or tampered token, revoked grant, closed handle, byte quota and overflow are tested. The tickets are NOT cryptographic, nonforgeable, nor secure against arbitrary hostile Rust in the same address space. Only a trusted host test invokes issuance. Zero-length payloads do no OS writes. File close precedes modeled `Release` and `Close` at test completion. Following successful modeled closure the parent can separately mutate the existing NAM atom using its normal ownership rules; failed/cancelled completion blocks NAM mutation.

**Critical cross-store caveat:** real File::write_all may fail after partial bytes have been written; the R0.2 modeled `Use` event may already be logged even when actual native IO fails. No journal, rollback, durability fsync, crash recovery, real syscall cancellation or transaction atomicity between filesystem and NAM is established. A test deliberately proves that already written file bytes persist despite a failed subsequent task and no NAM commit.

**Critical effect-model caveat:** the R0.2 reference model does not authorize `ResourceEffect::ExternalIo`; it rejects that effect. A modeled `Write` permission in this test is a **research fixture convention**, not production authorization to perform real filesystem I/O. The future internal runtime must define and enforce a real host authority boundary before any R11 admission flag can change.

## Fifteen targeted tests and acceptance

- Successful `Use` plus actual OS write, cleanup before scope close, then parent-only NAM commit.
- No ticket default-denied, wrong model scope, mutated/stale ticket denial.
- Byte quota and zero-byte no-work path.
- Actual concurrent thread revoke-before-write: zero native bytes, zero NAM mutation.
- Actual concurrent write-before-revoke: preexisting bytes persist, subsequent IO denied.
- Duplicate revoke and new ticket after revoke denied.
- Duplicate cancel and late success rejected; close/use-after-close rejected.
- Worker panic joined, modeled failure and OS file cleanup, no NAM commit.
- Separate disk write persists after task failure, proving no disk+NAM rollback.
- ExternalIo remains unsupported in R0.2.
- Drop removes only the newly created OS file fixture.

Acceptance must check 15/15 tests on each GitHub-hosted macOS/Linux/Windows runner with `cargo test --offline --release --test g03_r4d_governed_host_adapter_lab` and `cargo fmt --all -- --check`, plus general Test Gate (Check, Format/Clippy and 3-OS full Rust tests), then pinned owner Mac Intel run. Report exact failures transparently; do not declare PASS on first attempt unless CI shows it.

## Governance and next bounded phase

FNG1: C2 C3 C6 C7 C9 C14 C17 C18 C19 C20 are relevant but no global completion is claimed. FNG2: actual effect serialization, not a mock-only trace. FNG3: no user-facing thread/handle/token ceremony. FNG4: fail-closed local lab and negative authority tests without trusting malicious code. FNG5: std Rust sync and fs on 3 OS; no hardware vendor specialization. FNG6: certified boundaries untouched.

**All 12 R11 native readiness gates remain UNMET.** R4D does not provide a production-grade unforgeable host permission, process sandbox, real `.noi` scheduler, public I/O adapter, native durable transaction, crash recovery, real CPU/memory stress benchmark, independent external security review or production release authorization. Any future R4E must first address one of these genuine gaps with new evidence, not repackage these tests as native certification. No owner instruction has authorized merging PR #56, #58, #60 or R4D.

```text
R4D=RESEARCH_ONLY
OWNER_MAC_INTEL=PENDING
PR56_PR58_PR60=UNMERGED
R11_NATIVE_GATES=12_UNMET
MERGE_AUTHORIZATION=NONE
PRODUCTION=NOT_AUTHORIZED
```
