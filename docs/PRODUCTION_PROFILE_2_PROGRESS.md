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

Certified commit `85c65b50dbd1e11e9ca5c6f78e332d8cd9cdb2e6`, exact-SHA CI run `37937794556`, immutable annotated tag `p2.5`.

## P2.6 — Dynamic Capability-Secured File Output — CERTIFIED

Certified commit `e7ce55c24876636b5098f3d954a382087020617b`, exact-SHA CI run `37943416761`, immutable annotated tag `p2.6`.

Proof set includes P2.4/P2.5 composition, exact `FileWrite(target)` authority, no fabricated `ConsoleWrite`, deterministic input-sensitive receipts, create-new-only materialization, host-path-independent identity, Release Gate PASS, 20/20 focused tests and Linux/macOS/Windows exact-SHA CI.

## P2.7 — Bounded Atomic Multi-File Bundle Output — CANDIDATE

P2.7 composes 2..8 P2.6-style dynamic outputs into one create-new bundle commit unit.

Target proof set:

- exact `effect FileWrite;` declaration;
- 2..8 ordinary unique file targets;
- every file independently satisfies the certified P2.6 2..8-runtime-segment contract;
- each file remains <= 4096 UTF-8 bytes by P2.6 worst-case proof;
- global worst-case bundle output <= 16384 UTF-8 bytes before authority;
- bundle and file names are ordinary relative names with traversal forbidden;
- exact `FileWrite(bundle/file)` authority required for every target before runtime evaluation;
- missing or revoked one-target authority fails before any final bundle publication;
- all dynamic file bytes are computed before filesystem staging;
- final bundle is create-new only and existing targets are not overwritten;
- final namespace publication uses one same-root directory rename;
- handled pre-commit failures remove private staging and leave the final bundle absent;
- target order is canonicalized lexicographically, eliminating incidental declaration-order identity;
- canonical receipt excludes host absolute root and staging name;
- crash durability, hostile concurrent host mutation, distributed transactions and cross-filesystem atomicity remain explicitly unclaimed;
- no new NAIR opcode, general filesystem primitive, ConsoleWrite, network, time, random or process authority;
- Profile 1 and P2.1-P2.6 remain unchanged;
- Release Gate, 20 focused tests and exact-SHA Linux/macOS/Windows CI required before immutable `p2.7` tag.

After P2.7 certification, the Councils recommend a Constitutional Conformance Matrix before opening the next capability family.

Production Profile 2 remains incremental; no global completion percentage is claimed yet.
