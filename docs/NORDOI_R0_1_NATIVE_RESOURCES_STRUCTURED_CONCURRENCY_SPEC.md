# NORDOI R0.1 — Native Resources & Structured Concurrency Foundations

**Status: CANDIDATE / SPECIFICATION ONLY / NOT CERTIFIED.**

## Certified baseline and non-claims

This is an additive research-and-governance milestone on certified **g0.1**, commit `3d1a648bbe7e4e1d7a89affa649277b734660009`, CI `37961081022`. It adds **no** runtime, compiler, NAIR, kernel, API, scheduling, concurrency, authorization or syntax behavior. No new principles become certified by this package.

## Architectural decision

Advance a NORDOI-native *scope* as the single structural lifetime frontier for owned resources and child tasks. Scope exit is not successful if live children, unresolved failure, borrowed ownership, or unauthorized effects remain. Exact lexical and language surface notation remains open until semantics pass dedicated counterexample review. The runtime must not assume that a host thread is the semantic task, or that a host descriptor is the semantic resource.

## Candidate semantic entities

- **Resource identity**: typed, nonforgeable resource token, tied to an explicit grant and ownership state; identity alone is not authority.
- **Resource grant**: unforgeable capability authorizing a narrowly scoped effect; authority transfer must be deliberate and auditable.
- **Task**: computation accepted within an owning scope; lifecycle is explicit (`declared`, `admitted`, `running`, `completed`, `failed`, `cancelled`, `joined`). This is a proposed model, not an implemented state machine.
- **Scope**: unique owning frontier for child tasks; closing requires terminal accounted outcomes for every admitted child.
- **Cancellation**: cooperative request with an eventual explicit outcome, not an assumption of instant preemption.
- **Result**: typed terminal outcome, including propagated failure/cancellation, never ambient exception swallowing.

## Design obligations

1. Preserve all six Future-Native rules as machine-readable decision evidence.
2. Enforce the sixteen invariants in `governance/r01_resource_task_invariants_v1.tsv` at implementation milestones.
3. Define deterministic ordering over accepted causal task events; do not assume host completion order is deterministic.
4. Enforce explicit budget/backpressure before work admission; avoid unbounded queue creation by default.
5. Specify transactions, irreversible external I/O, rollback limits and committed effects separately; never promise rollback of real-world effects without compensating mechanisms.
6. Preserve the certified kernel K1.18, NAIR 0.6, p2.7 resources and G0.1 conformance behavior until evidence-backed extension is approved.
7. Preserve zero-cost-by-omission, including no task scheduler startup for a program with no task effects.
8. Treat native, browser, mobile, distributed, GPU/NPU and embedded backends as potential targets, not as currently certified implementations.

## Negative cases the next implementation MUST reject

- child execution after scope closure;
- use-after-release, forged resource handles, implicit elevation of authority;
- hidden, unbounded task spawning;
- silent cancellation or silent child failure;
- nondeterministic join semantics without an explicit deterministic policy;
- falsely labeling a future distributed node crash as a local atomic rollback;
- treating task identity as permission to access a device or a file.

## R0.2 proposed acceptance criteria

A pure deterministic reference transition model with property tests and counterexamples; no OS threads, no ambient I/O, no scheduler runtime. Only after that: kernel/NAIR extension proposal backed by formalized invariants, negative tests, byte-level boundary diff and a dedicated release gate. No compiler/runtime functionality is claimed for R0.1.

## Validation

Run `python3 scripts/validate_r01_foundations.py` (stdlib only), then existing Rust release gate on your own toolchain. The validator checks G0.1 baseline sources and model integrity; **it cannot certify Rust semantics**.
