# NORDOI Constitutional Conformance Matrix — G0.1

## Purpose

This governance milestone audits the current certified NORDOI baseline against the constitutional corpus without changing kernel, NAIR, runtime, effects, capabilities or host authority.

It intentionally separates **constitutional evidence coverage** from **full NORDOI v1 delivery coverage**. The former asks whether a written principle already has certified evidence in its current scope. The latter asks how much of the complete universal language/architecture mission has actually been delivered.

## Certified baseline

- Production Profile 1: certified and frozen at `v1.7`.
- Production Profile 2 observable-I/O sequence: `p2.1` through `p2.7` certified.
- Latest certified baseline: `p2.7` / commit `0a634d6cab52074e18b42d1ffea00ce02cdba359` / CI `37956122196`.
- Public version remains frozen: `nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)`.

## Counter A — Constitutional evidence coverage

Principles: **322**  
Certified in current explicit scope: **310**  
Partial/global-future principles: **12**  
Not started as written principle: **0**

Evidence coverage = **96.27%**.

This number MUST NOT be interpreted as project completion. Most C18-C322 principles specify already-certified kernel/runtime invariants. A small number of global principles carry much larger future delivery scope.

## Counter B — Full NORDOI v1 delivery planning model

Weighted delivery estimate: **58.35% complete / 41.65% remaining**.

This is a transparent planning estimate, not a semantic certification claim. Weights and percentages are stored in `governance/nordoi_v1_delivery_model_v1.tsv`.

## Future-Native Gate

Every major milestone after G0.1 must satisfy all six rules in `governance/future_native_gate_v1.tsv` before implementation. The gate exists to prevent NORDOI from degenerating into feature-parity replication of historical languages.

## Current recommendation

After G0.1 certification, the next major capability family should be selected from the largest constitutionally important deficits rather than from filesystem feature parity. Highest-value candidates are: NORDOI-native resource/task semantics and structured concurrency; proof-driven heterogeneous CPU/GPU/NPU compute; unified spatial 2D/3D/XR execution; distributed execution; and universal backends.

## Machine-readable artifacts

- `governance/constitutional_conformance_v1.tsv`
- `governance/nordoi_v1_delivery_model_v1.tsv`
- `governance/future_native_gate_v1.tsv`

## Non-goals

G0.1 introduces no new `.noi` syntax, NAIR opcode, NAM semantics, runtime authority, filesystem authority, network authority, process authority, time authority, random authority or device authority.
