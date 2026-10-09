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

Certified commit:

```text
7522e3eae52d6e9deeaa7d7f7fb7861a550690a3
```

Certified exact-SHA CI run:

```text
37931625937
```

Immutable annotated tag: `p2.4`.

Proof set includes 2..8 ordered runtime Int/Bool segments, bounded static UTF-8 separators, exact `ConsoleWrite` authority, zero ambient authority, 4096-byte global proof, deterministic per-segment/global receipts, P2.3/P2.4 boundary preservation, Release Gate PASS, 20/20 focused tests and Linux/macOS/Windows exact-SHA CI.

## P2.5 — Capability-Secured File Output — CANDIDATE

P2.5 opens one bounded create-new filesystem output while preserving every certified P2.x boundary.

Target proof set:

- exact `effect FileWrite;` declaration;
- exactly one ordinary relative target file name;
- target-name bound of 128 UTF-8 bytes;
- output bound of 4096 UTF-8 bytes;
- no path separators, traversal, absolute source path, directory creation or dynamic target;
- exact `Capability::FileWrite(target)` authority;
- explicit host `--grant-output-dir` required before materialization;
- host output root excluded from canonical identity;
- create-new only and overwrite denied;
- existing target remains byte-for-byte unchanged;
- deterministic plan/content/receipt SHA-256 identities;
- source path/ID/host/time independent identities;
- no new NAIR opcode, runtime filesystem primitive or kernel authority;
- Profile 1 and P2.1-P2.4 unchanged;
- Release Gate PASS;
- 20/20 focused tests;
- exact-SHA Linux/macOS/Windows CI before immutable `p2.5` tag.

Production Profile 2 remains incremental; no global completion percentage is claimed yet.
