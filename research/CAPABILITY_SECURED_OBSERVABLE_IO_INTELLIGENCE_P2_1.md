# Capability-Secured Observable I/O Intelligence P2.1

The first observable operation is a security boundary, not a convenience `print` function.

A conventional print primitive often inherits process stdout implicitly. That would violate NORDOI's least-authority constitution because source execution would gain an ambient host channel merely by being run. P2.1 instead separates four facts that must all be independently true:

1. the source declares observable intent;
2. the requested channel is known and bounded;
3. the host grants the exact capability;
4. a deterministic receipt can commit to what was authorized and emitted.

P2.1 intentionally does not extend the frozen Profile 1 `Effect` enum. That enum is serialized by NAIR and effect durability subsystems. Adding a new variant there would create a hidden compatibility migration inside a milestone whose goal is only to open a new profile. A local observable effect plus the existing `CapabilitySet` gives P2.1 a strict new authority boundary without rewriting prior certified encodings.

The stdout write belongs to tooling after receipt construction. The language layer itself returns the authorized bytes and receipt. This keeps the effect decision deterministic and independently testable while allowing the CLI to materialize the result only after the grant passes.

The first slice is static text because it isolates authority from expression evaluation. Once this boundary is certified, later Production Profile 2 milestones can compose dynamic values, multiple bounded emissions, richer channels, and eventually capability-aware project manifests without weakening the initial deny-by-default rule.
