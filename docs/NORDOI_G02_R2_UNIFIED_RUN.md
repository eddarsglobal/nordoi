# NORDOI G0.2 R2 — Simpler first `run` (candidate)

## User benefit

The existing public first-use example can run through the already-familiar `run` verb:

```text
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

Expected program stdout:

```text
Hello from NORDOI P2.1!
```

This explicitly opts in to the existing `ConsoleWrite` authority. Without the grant, the plain `run` command continues using its pre-existing V0.1 HALT-only path. An `effect ConsoleWrite` declaration alone is not authority.

## Design contract

- No new `.noi` syntax or effect/declaration system.
- No new NAIR opcode, kernel, runtime execution engine, dependency, or device authority.
- The new CLI option delegates to the **existing P2.1** handler, preserving receipt and denial behavior.
- Plain `nordoi run <path|->` does not perform new work and preserves V0.1 semantics.
- This milestone intentionally does **not** unify dynamic/structured P2.2–P2.7 execution or grant additional capabilities.
- Not a production authorization, release tag, or performance superiority certification.

## Evidence required

1. Five targeted CLI tests (delegation parity, old `run`, stdin, undeclared effect, invalid option/help).
2. P2.1 legacy regression, Rustfmt, cargo check, Clippy with zero warnings, full Rust test suite.
3. Exact candidate GitHub commit CI on Ubuntu/macOS/Windows before any merge.
4. Zero changes to `production/p0.1` and P0.6E.

The user-facing language simplicity and minimal runtime cost principles are design-time gates.
