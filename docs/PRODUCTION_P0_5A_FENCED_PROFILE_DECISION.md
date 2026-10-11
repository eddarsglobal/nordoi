# NORDOI Production P0.5A — Fenced host profile decision

**Status: local candidate; no production authorization.**

## Motivation

The read-only P0.5 inventory at `2c0c2e4fe5c558baff5ee2a34dc792bfb939adeb` reported
45 scanned files, 1 legacy context fallback, 10 generic-dispatch text matches,
7 external-effect entry matches and 3 raw execution call matches.
Counts are **regex matches, not proven exploitable vulnerabilities**.

## Boundary

`FencedProductionEffects` owns an actual `AtomicEventLoop`, a real `GovernedFencedEffectJournal`
and a P0.4 `ProductionEffectBackend` so that its own `dispatch_next` accepts
**no caller-selected arbitrary `EffectBackend`**. The journal supplies a
current fencing lease and delivery key, while P0.4 persists the identity
before the signed operation passes through P0.3. Its private event loop and backend, with no raw accessors, prevent accidentally
selecting unguarded dispatch methods **through this facade**.

## Explicitly unresolved

- The core `EffectBackend` trait and its default `execute_with_context → execute`
  adapter remain publicly available in the historical kernel.
- Other public dispatch entrypoints and `observable_io_p2*` APIs still exist.
- Users/hosts who call these other APIs independently are **not protected** by
  the P0.5A facade. Kernel-wide compulsory routing is not certified.
- The destination may commit an effect whose result becomes unknown;
  neither exactly-once external execution nor recipient-side idempotency is proven.
- The local journals can be rolled back together absent independent anchoring.
- Host-supplied fence stores may violate their contracts; production needs
  a vetted, durable implementation and adversarial testing.

## Next steps

P0.5B must audit and close other production call surfaces, including native
observable I/O. A later anchor gate must reject independently rolled-back state.
Do not merge this candidate as proof of production safety.
