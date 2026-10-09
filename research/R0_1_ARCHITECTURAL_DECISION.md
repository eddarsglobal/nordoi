# R0.1 council decision record

**Decision:** resources + structured concurrency semantic foundations first; implementation deferred pending deterministic reference model.

**Alternatives:** heterogeneous compute, spatial/XR, distributed execution, generic syntax expansion. All remain in the roadmap and no percentage has been changed.

**Reasoning:** ownership-scoped tasks and explicit resource authority are shared prerequisites for safe distributed, spatial and device compute workloads; designing these before backend-specific schedulers reduces lock-in. This is an architectural prioritization, not a claim that task semantics are already certified.

**Unresolved:** cancellation at irreversible I/O boundaries; deterministic ordering with external effects; inter-scope borrowing; real-time deadlines; fair scheduling; heterogeneous device memory; distributed partial failure; efficient ergonomics.

**Gate:** FNG1–FNG6 design checks documented; none of these checks alone prove soundness or performance.
