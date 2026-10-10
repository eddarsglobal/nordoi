# R0.4 design decision — adversarial generated witnesses

Decision: build a **test-only, additive** deterministic sequence generator over the frozen R0.2 resource/task reference model after R0.3 certification.

Alternatives rejected for this milestone: introducing tokio or a real worker pool (unjustified authority/semantic changes), invoking operating-system threads (non-deterministic reference behavior), editing a certified `src/resource_task_r02.rs` (breaks prior tag boundary), introducing third-party proptest/quickcheck dependencies (unnecessary install/supply-chain weight for a first minimal experiment), or promising complete formal verification.

Candidate probes: mixed valid/invalid operations, transactional rejection, event count, causal replay, scoped cleanup despite revocation, failure dominating cancellation, child closure ordering, foreign-domain refusal and closed-scope reports. A synthetic delta deletion reducer proves only its own shrinking behavior, not a flaw in NORDOI.

Risk register: generator blind spots, insufficient accepted-path coverage, debug-format instability, compiler-lint regressions, accidentally overwriting certified files, and mistaking synthetic shrinking for genuine counterexample discovery.

Mitigations: fixed seed set/operation bound as a versioned contract; explicit assertions; independent governance matrix; source-only separate test module; enforce existing source integrity; freeze immutable r0.3. The CI release gate must succeed on maintainer machine/GitHub before any certification.

Deferred: actual mutation testing of the model, model-checker integration, stronger coverage metrics, shrinking **genuine** invariant violations, liveness, fairness, host authority hardening, production scheduling or `.noi` source syntax. These require separate evidence and gates.
