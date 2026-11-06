# NORDOI G0.2 — First-use quickstart (current R2 CLI)

This guide uses **existing, implemented** `.noi` syntax and the G0.2 R2 NORDOI CLI. It does not define a new language standard or grant ambient authority.

## First program

The checked-in reference file is `examples/p2_1_console/hello.noi`:

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

The program **declares** its console effect. The CLI invocation must also **grant** that capability; declaration alone is not authorization.

## Run (macOS, Linux, Windows)

From the repository root, using Rust/Cargo:

```text
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

Expected program stdout is exactly:

```text
Hello from NORDOI P2.1!
```

The CLI writes the existing P2.1 execution receipt to stderr. The older, still supported spelling `console-run` remains available, but the simple `run` spelling above is the recommended first-use route.

## Explicit authority and denial

The `--grant-console` flag is an explicit CLI capability grant; the source declaration `effect ConsoleWrite;` is **not** permission.

Without the grant, plain `nordoi run <path|->` retains its original V0.1 HALT-only execution path. It does **not** silently gain P2.1 console capability.

To exercise the established P2.1 capability-denial boundary directly, run:

```text
cargo run --bin nordoi -- console-run examples/p2_1_console/hello.noi
```

This existing P2.1 command must fail closed with exit code `5` and no program stdout. The historical, ungranted `run` path may return a different frontend error code; do not conflate these two command contracts.

## Standard input

On macOS/Linux:

```sh
cat examples/p2_1_console/hello.noi | cargo run --bin nordoi -- run - --grant-console
```

The portable stdin path argument is `-`. On Windows, pipe the UTF-8 file contents into the same `cargo run --bin nordoi -- run - --grant-console` invocation.

## Limits

- The unified `run --grant-console` opt-in uses only the **existing P2.1** console path. It is not general-purpose execution for P2.2–P2.7 effects.
- No new `.noi` syntax, compiler, NAIR, kernel, dependency, execution authority or service behavior is introduced by this guide.
- No VPS, SSH, remote witness or anti-rollback service is needed for this ordinary first-use exercise.
- Performance superiority is not established by G0.2 or the G0.2 R3 measurements.
- G0.2 R2 was checked by local regression and exact-commit GitHub CI on macOS, Linux and Windows; this guide does not confer production certification.
- Production security certification and deployment remain separate and NOT AUTHORIZED.
