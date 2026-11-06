# NORDOI G0.2 — First-use quickstart (candidate)

This guide uses **existing, already implemented** `.noi` syntax and the current NORDOI CLI. It does not define a new language standard or grant ambient authority.

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
cargo run --bin nordoi -- console-run examples/p2_1_console/hello.noi --grant-console
```

Expected program stdout is exactly:

```text
Hello from NORDOI P2.1!
```

The CLI writes execution receipt information to stderr. Running without `--grant-console` must fail closed, return code `5`, and emit no program output:

```text
cargo run --bin nordoi -- console-run examples/p2_1_console/hello.noi
```

To run a source from stdin on macOS/Linux:

```sh
cat examples/p2_1_console/hello.noi | cargo run --bin nordoi -- console-run - --grant-console
```

The portable CLI argument for stdin is `-`; Windows users can pipe the UTF-8 source into the same `cargo run --bin nordoi -- console-run - --grant-console` invocation.

## Limits

- This is the already implemented P2.1 console capability path, **not** general-purpose `nordoi run` output support.
- No new `.noi` syntax, compiler, NAIR, kernel, or service behavior is added.
- No VPS, SSH, remote witness, or anti-rollback service is necessary for this ordinary language exercise.
- No performance superiority is claimed without reproducible measurements.
- G0.2 is not certified until the full local release gate and exact-commit GitHub CI succeed.
- Production security certification and deployment remain separate and NOT AUTHORIZED.
