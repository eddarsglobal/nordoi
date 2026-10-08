# NORDOI Production Profile 2 Progress

## Baseline

Production Profile 1 is certified and frozen at immutable tag `v1.7`.

## P2.1 — Capability-Secured Observable I/O — CERTIFIED

Certified at immutable annotated tag `p2.1`.

Proof set includes explicit `ConsoleWrite`, exact host capability grant, deny-by-default zero-output failure, bounded static UTF-8, deterministic receipts, zero ambient authority, unchanged Profile 1 encodings, Release Gate PASS, and exact-SHA cross-platform CI.

## P2.2 — Dynamic Capability-Secured Observable Output — CERTIFIED

Certified commit:

```text
2270161e1f5a816379754ff7cfd02cb31e1183c0
```

Certified exact-SHA CI run:

```text
37768061909
```

Immutable annotated tag: `p2.2`.

Proof set includes runtime-dependent `Int`/`Bool` output through certified V0.7 computation, exact `ConsoleWrite` authority, deterministic runtime/P2.2 receipt binding, zero ambient authority, fail-closed static/dynamic boundary, Release Gate PASS, 20/20 focused tests, and Linux/macOS/Windows CI.

## P2.3 — Structured Dynamic Text Output — CANDIDATE

P2.3 composes bounded decoded UTF-8 text with exactly one runtime-computed `Int` or `Bool` while preserving P2.1/P2.2 authority and evidence semantics.

Target proof set:

- required quoted UTF-8 prefix;
- one V0.7 runtime `Int`/`Bool` segment;
- optional quoted UTF-8 suffix;
- runtime segment must depend on explicit input;
- arithmetic remains V0.7 numeric arithmetic rather than implicit string coercion;
- 4096-byte total output bound proven before authority;
- exact `ConsoleWrite` host grant required before runtime execution;
- zero stdout bytes when grant is absent;
- deterministic template plan, V0.7 runtime receipt hash, and P2.3 receipt;
- source path/ID/host/time independent identities;
- Profile 1, P2.1, and P2.2 certified boundaries unchanged;
- Release Gate PASS;
- 20/20 focused tests;
- exact-SHA cross-platform CI before immutable `p2.3` tag.

Production Profile 2 remains incremental; no global completion percentage is claimed yet.
