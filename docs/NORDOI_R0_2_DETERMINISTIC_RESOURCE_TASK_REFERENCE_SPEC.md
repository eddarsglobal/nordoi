# NORDOI R0.2 — Deterministic Resource & Task Reference Model

**STATUS: CANDIDATE — REFERENCE MODEL ONLY — NOT RUNTIME CERTIFIED.**

## Frozen baseline

- Source milestone: `r0.1`, commit `5a95dad7c7e95498b8bf04319007635bd4048152`, CI `37965737669`.
- `g0.1` and `p2.7` semantic baselines remain frozen.
- This candidate is strictly **additive-only**, with no edits to existing tracked files.
- `src/resource_task_r02.rs` is **not added to `src/lib.rs`** and is compiled only through `tests/resource_task_r02.rs`.
- No changes to `Cargo.toml`, the CLI, .noi syntax, NAIR, kernel K1.18, runtime, scheduler, authority, effects or capabilities.

## Reference entities and semantics

1. **Scope:** deterministic identity tagged by an embedding-domain identifier, explicit child ownership, finite per-scope budgets and a terminal closure report. The root scope is created at bootstrap.
2. **Grant:** explicit host-issued typed effect token, tied to the scope and domain. A resource identity is never itself permission. Grant revocation blocks subsequent use; cleanup/release remains possible.
3. **Resource:** scope-owned object with immutable effect kind and a one-way live → released transition. Repeated release and use-after-release are rejected.
4. **Task:** declared → admitted → running → terminal → joined. An admitted task may finish without a running step (pure completion abstraction), but a declared task may not finish. A failure has a numeric code; cancellation requires an explicit request and acknowledgement.
5. **Close:** rejects any unjoined task, live resource or unclosed nested scope. A successful close returns a deterministic outcome; failure takes precedence over cancellation, which takes precedence over success. All per-task and per-child outcomes are included, sorted by semantic identity.
6. **Budget:** task count, resource acquisition count, child-scope count and global accepted-event count are bounded. Resource release does not refund the acquisition budget: this is a bounded-reference policy, not the final runtime admission policy.
7. **Atomicity:** an accepted command is evaluated on a cloned candidate state and published only on success. This proves fail-closed transitions **inside the pure model only**. It makes no claim about rollback of external side effects.
8. **Deterministic replay:** the trusted in-memory recorded sequence of commands and host grants can be replayed from an identical bootstrap domain, root budget and event budget. Reports do not depend on completion event ordering when the same task IDs have the same final outcomes. The event log is *not* a durable encoding or security attestation.

## Security and non-claims

The embedding-domain identifier is a **collision-avoidance namespace**, not a cryptographic secret. `HostPermit` is a Rust API witness used in this test harness; it is **not** a production security boundary. The reference module never accepts raw host I/O and explicitly rejects the `ExternalIo` effect. Authority enforcement in real backends, task transfer, fairness, preemption, transactions involving irreversible effects, deadlock freedom, resource borrowing and performance are **not implemented**.

Existing governance obligations `R01-I01` through `R01-I16` have model-level test witnesses recorded in `governance/r02_reference_obligations_v1.tsv`. The word **MODEL_TESTED** means only that bounded examples and counterexamples are encoded; it does *not* mean universally proven or production certified.

## Acceptance gate

From a clean `r0.1` checkout, add the files in this package and run:

```bash
python3 scripts/validate_r01_foundations.py
python3 scripts/validate_r02_reference.py
cargo fmt --all
cargo test --test resource_task_r02 -- --nocapture
./scripts/release_gate.sh
```

Before committing, verify: `git diff --exit-code r0.1 -- src/lib.rs src/kernel.rs src/nair src/runtime src/scheduler.rs src/capability.rs src/effect.rs src/observable_io_p21.rs src/observable_io_p22.rs src/observable_io_p23.rs src/observable_io_p24.rs src/observable_io_p25.rs src/observable_io_p26.rs src/observable_io_p27.rs Cargo.toml VERSION`. Also inspect `git status --short` for unexpected changes.

## Follow-on milestone

R0.3 must audit model limitations and only then propose a separate opt-in bridge toward compiler/NAIR/runtime semantics. An R0.2 certification is certification of a **pure reference model**, not certification of concurrent execution in NORDOI.
