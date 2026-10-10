# R0.6 — Independent finite-state conformance specification

**Authority boundary:** `r0.5` (`6cebd4051304c81901669303265c0ad7456b7e12`) is immutable.
The test-only R0.2 model is **system under test (SUT)**, not an approved NORDOI runtime.

## Decision: independently predict; compare; reject on divergence

The `Oracle` in `tests/resource_task_r06.rs` separately maintains a small set of
observables: read declaration, issued/revoked grant, cumulative resource/task
admission, task phase/cancellation/outcome, scope closed flag, and accepted
event count. It **must not** call the SUT to compute expected results.

For each operation, predict the exact Rust `Result<Signature, ModelError>` using
the oracle; execute the corresponding R0.2 operation; compare the two results.
Accepted operations increment both event counts and check the accepted model's
own causal replay. Rejected operations must preserve both oracle and SUT state;
no failed operation may consume event budget. Actual typed receipt identities
are checked for monotonic allocation within this bounded scenario.

The differential test exercises R0.2 `Model::apply` and the host-only
`Model::host_grant` separately, including correct error-check ordering at the
event-cap and wrong-host-witness boundary. Foreign identities are obtained
from a second, unrelated domain via the public typed reference API.

## Exhaustiveness **only within named finite sets**

1. 32 deterministic seeds × 96 generated attempted operations (3,072 comparisons);
   every seed produces a reproducible sequence with an explicitly fixed PRNG.
2. Every 5-event word over the explicit alphabet {declare Read, host grant,
   acquire, release, close}: 5^5 = 3,125 words. The enumeration includes
   invalid words and closes early when legal.
3. Fixed targeted examples for capability-vs-declaration, revoke/reissue,
   release, cancellation-failure precedence, failed/completed close, zero
   budgets, unsupported I/O, foreign-domain handles, and event exhaustion.
4. A synthetic mutation of the expected report must be detected by comparison;
   this is NOT a defect found in the R0.2 implementation.

Scope: exactly one root scope, **zero children**, zero or one task and zero or
one resource, Read effect only (plus deliberately unsupported ExternalIo),
bounded event log length and nonconcurrent, sequential reference transitions.
This scope intentionally excludes Write-capability matching, arbitrary nested
scope trees, general resource/task cardinality, execution scheduling, external
I/O, durable signed logs, and cryptographic attack resistance. R0.3–R0.5 cover
other finite profiles but do not remove these explicit limitations.

## Governance / Future-Native boundaries

- **FNG1**: no default ambient host authority; `HostPermit` only in the model harness.
- **FNG2**: no public language surface or runtime scheduling behavior.
- **FNG3**: explicit state/effect/ownership transitions are checked within profile.
- **FNG4**: failure and cancellation remain observable, not hidden success.
- **FNG5**: deterministic replay is an auxiliary SUT check, not the oracle.
- **FNG6**: frozen K1.18 / NAIR 0.6 / compiler and certified tags unmodified.

Nothing in this artifact grants new runtime authority. It is TEST-ONLY, not
formal proof, not a release of production structured concurrency, and not a
claim that software is impossible to compromise. Do not promote it into
`src/lib.rs` without explicit architectural governance and certification.

## Acceptance criteria

1. Static validator PASS with 15 test-to-governance witnesses.
2. `cargo test --test resource_task_r06 -- --nocapture`: all 15 Rust tests pass;
   the two `R06_WITNESS` summary records are visible.
3. `cargo fmt --all -- --check`, Clippy `-D warnings`, cargo check and complete
   cargo test suite PASS (full `scripts/release_gate.sh`).
4. `python3 scripts/validate_r02_manifest.py` reports 7/7 SHA-256 PASS.
5. `git diff --exit-code r0.5 -- .` shows no modifications of tracked sources.
6. Only following a matching five-job successful CI run may `r0.6` be tagged.
