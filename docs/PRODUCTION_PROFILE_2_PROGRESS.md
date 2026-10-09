# NORDOI Production Profile 2 Progress

## Baseline

Production Profile 1 is certified and frozen at immutable tag `v1.7`.

## P2.1 — Capability-Secured Observable I/O — CERTIFIED

Immutable annotated tag: `p2.1`.

## P2.2 — Dynamic Capability-Secured Observable Output — CERTIFIED

Certified commit `2270161e1f5a816379754ff7cfd02cb31e1183c0`, exact-SHA CI run `37768061909`, immutable annotated tag `p2.2`.

## P2.3 — Structured Dynamic Text Output — CERTIFIED

Certified commit `375b82cadf9f56f58fafee99342575cb06f1286d`, exact-SHA CI run `37772168709`, immutable annotated tag `p2.3`.

## P2.4 — Multi-Segment Structured Output — CERTIFIED

Certified commit `7522e3eae52d6e9deeaa7d7f7fb7861a550690a3`, exact-SHA CI run `37931625937`, immutable annotated tag `p2.4`.

## P2.5 — Capability-Secured File Output — CERTIFIED

Certified commit:

```text
85c65b50dbd1e11e9ca5c6f78e332d8cd9cdb2e6
```

Certified exact-SHA CI run:

```text
37937794556
```

Immutable annotated tag: `p2.5`.

Proof set includes exact `FileWrite(target)` authority, one bounded ordinary relative target, create-new-only materialization, overwrite denial, path-traversal rejection, host-path-independent receipts, Release Gate PASS, 20/20 focused tests and Linux/macOS/Windows exact-SHA CI.

## P2.6 — Dynamic Capability-Secured File Output — CANDIDATE

P2.6 composes the certified P2.4 structured runtime renderer with the certified P2.5 file boundary while keeping host authority restricted to exact `FileWrite(target)`.

Target proof set:

- exact `effect FileWrite;` declaration;
- P2.4-style 2..8 runtime Int/Bool segments;
- runtime expressions remain V0.7-certified computations;
- no fabricated or ambient `ConsoleWrite` authority;
- P2.5 target-name restrictions and 128-byte bound preserved;
- P2.4 4096-byte worst-case output proof preserved before authority;
- exact `Capability::FileWrite(target)` grant required before runtime evaluation;
- create-new-only P2.5 materialization and overwrite denial;
- deterministic per-segment runtime proof hashes;
- global P2.6 receipt binds input, ordered results, rendered bytes and P2.5 receipt;
- host output root excluded from canonical identity;
- no new NAIR opcode, runtime filesystem primitive or kernel authority;
- Profile 1 and P2.1-P2.5 unchanged;
- Release Gate PASS;
- 20/20 focused tests;
- exact-SHA Linux/macOS/Windows CI before immutable `p2.6` tag.

Production Profile 2 remains incremental; no global completion percentage is claimed yet.
