# R0.2 council record: pure deterministic reference model

Baseline: R0.1 certified on commit `5a95dad7c7e95498b8bf04319007635bd4048152`, GitHub workflow `37965737669`.

Decision: model transitions and negative cases before specifying a scheduler, operating-system thread adapter, GPU work queue, browser Promise mapping, syntax construct or compiler lowering.

Alternative deferred: immediate integration into K1.18 and NAIR 0.6. Rejected at this phase because it would weaken the certified boundary and conflate a reference model with runtime proof.

FNG1 Constitutional driver: explicit effects, ownership and bounded task lifetimes.
FNG2 No feature-parity-only: no copying of POSIX threads, Python tasks, Rust async, or JS promises.
FNG3 Simplicity gain: one explicit scope controls children and resources; notation to be selected later.
FNG4 Security/provability gain: deny by default, negative transitions, checked budgets and trusted deterministic replay assumptions.
FNG5 Universal architecture: typed transitions independent of hosts; device/distributed execution unimplemented.
FNG6 Certified boundaries preserved: new reference and test files only, no edits to certified source.

Review limits: model clone-on-transition is intentionally not performance optimized; no scheduler fairness, no liveness proof, no cryptographic capability, no signed durable log, no real cancellation preemption, no external I/O rollback. Reference tests are examples, not formal proofs of all states.
