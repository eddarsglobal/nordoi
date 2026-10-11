# P0.5B Source Lock: precise boundary and non-claims

**Status**: local candidate. No commit, CI certification or deployment approval.

## Allowed surface

The opt-in `FencedProductionEffects` facade owns a kernel `AtomicEventLoop`,
`GovernedFencedEffectJournal`, `GovernedEffectDispatcher` and P0.4
`ProductionEffectBackend`. This new gate checks their direct source wiring,
exactly seven public methods and no additional raw execution method in that
file. Legacy effect/fencing/persistence/audit and native observable I/O files
must remain byte-for-byte equal to the certified P0.5A Git baseline, as
verified by `git diff <base> -- <frozen paths>`.

## Evidence

Ten negative source mutations must be rejected, and the old P0.5A runtime
regression suite executes under macOS, Windows and Ubuntu CI. This is a
fail-closed pattern checker and a Git baseline comparison; a Rust AST / whole
program proof is **not provided**.

## Missing protections

- Historical kernel generic/public dispatch remains accessible elsewhere.
- Native observable I/O remains public to other consumers.
- No proof of a unique deployable host binary or complete provider isolation.
- Host-injected fence-store durability, lease recovery and linearizability
  remain unverified.
- External exactly-once execution, remote idempotency and separate monotonic
  anti-rollback anchoring remain unresolved.

Next: produce a genuinely single deployable host composition that refuses raw
kernel/observable I/O entrypoints and then integrate an independently trusted
rollback anchor. Do not claim global production safety from this source lock.
