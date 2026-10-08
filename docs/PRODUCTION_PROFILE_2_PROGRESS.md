# NORDOI Production Profile 2 Progress

## Baseline

Production Profile 1 is certified and frozen at immutable tag `v1.7`.

## P2.1 — Capability-Secured Observable I/O — CERTIFIED

Certified at immutable annotated tag `p2.1`.

Certified proof set:

- explicit `effect ConsoleWrite;`;
- exact `Capability::ConsoleWrite` host grant;
- deny-by-default with zero stdout bytes on denial;
- bounded static UTF-8 output;
- deterministic plan and receipt SHA-256 identities;
- host/path/time-independent receipt;
- zero ambient authority;
- unchanged Profile 1 kernel/runtime/NAIR encodings;
- Release Gate PASS;
- exact-SHA Linux/macOS/Windows CI.

## P2.2 — Dynamic Capability-Secured Observable Output — CANDIDATE

P2.2 extends the observable boundary to one runtime-computed `Int` or `Bool` while preserving P2.1 authority semantics.

Target proof set:

- `entry <name> emits <dynamic-expression>;`;
- expression must depend on explicit runtime input;
- V0.7 dynamic computation reused rather than replaced;
- canonical Int/Bool rendering only;
- 64-byte rendered-output bound;
- exact `ConsoleWrite` grant required before runtime execution;
- zero stdout bytes when grant is absent;
- deterministic P2.2 plan, V0.7 runtime receipt hash, and P2.2 receipt;
- source path/ID/host/time independent identities;
- no string concatenation, filesystem, network, or new ambient authority;
- Profile 1 and P2.1 certified boundaries unchanged;
- Release Gate PASS;
- 20/20 focused tests;
- exact-SHA cross-platform CI before immutable `p2.2` tag.

Production Profile 2 remains incremental; no global completion percentage is claimed yet.
