# V0.5 Conditional Core — Architecture Record

## Decision

NORDOI moves from micro-milestones to larger vertical production batches where the participating layers can be certified together without weakening constitutional gates.

For the first batch, boolean/comparison execution and pure `if/else` are closed end-to-end in one release.

## Why no runtime branch yet

Every condition representable by this V0.5 surface is decidable from canonical literals and immutable compile-time bindings. Emitting a runtime jump would therefore perform work whose outcome is already known. That would conflict with the NORDOI performance law.

V0.5 instead validates both branches, resolves the condition, preserves both branch semantics in its witness, and lowers only the selected branch.

This separates three concerns cleanly:

- **semantic completeness:** both branches are valid and auditable;
- **operational minimality:** dead branch cost is zero;
- **future extensibility:** dynamic conditions can later introduce governed runtime control-flow primitives without contaminating this static case.

## Why execute direct conditions too

N0.8 comparison opcodes must not remain isolated IR primitives. `condition-run` closes L0.9 → C0.11 → N0.8 → runtime in this same batch, proving that boolean values and all six integer comparators are executable machine meaning.

## Security

No host authority is added. Both execution paths use canonical empty input and the existing closed runtime. Failures before execution publish no successful report. Dead branches are validated rather than trusted or ignored.

## Performance evidence encoded as invariants

The batch tests structural zero-cost claims that are deterministic across CI runners:

- literal bool requires NAIR 0.6 only;
- comparison requires NAIR 0.8 only;
- static `if` has runtime branch count zero;
- dead branch instruction count is zero;
- changing only a dead branch does not change selected operational NAIR;
- the complete V0.5 witness still changes, preserving semantic auditability.

These are stronger and less flaky than wall-clock thresholds on shared CI runners.
