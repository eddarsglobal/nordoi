# NORDOI G0.3 R3-R1 — Build once, run directly (teaching candidate)

**Status: NON-NORMATIVE DOCUMENTATION EXPERIMENT.** This alternative is not the official first-use guide, not a new public `.noi` syntax, not an AI/human usability result, and not a production release. The original R2 study cards A/B remain frozen.

## The unchanged first program

The existing tracked source file `examples/p2_1_console/hello.noi` is:

```noi
module app.main;

effect ConsoleWrite;

entry main emits "Hello from NORDOI P2.1!\n";
```

- `effect ConsoleWrite;` declares the console effect; it **does not grant permission**.
- `--grant-console` is an **explicit and separate** narrow execution authorization.
- This tutorial uses the existing P2.1 console implementation. It does not extend `run` to P2.2–P2.7 effects.

## Option A (current official path): build and run via Cargo

From the **NORDOI repository root**:

```text
cargo run --bin nordoi -- run examples/p2_1_console/hello.noi --grant-console
```

The result is program stdout exactly `Hello from NORDOI P2.1!` with a newline. Execution receipt information is printed on stderr.

## Option B (teaching experiment): compile once, then run the binary

### First, build the same already-implemented NORDOI binary

On macOS, Linux, or Windows, with the Rust/Cargo toolchain installed, from the **NORDOI repository root**:

```text
cargo build --release --bin nordoi
```

This compilation must succeed first. It is not free or skipped, and the resulting binary is specific to the host platform. The examples below assume Cargo's default `target` directory; a custom `CARGO_TARGET_DIR` changes the location.

### Then run on macOS or Linux (shell)

```sh
./target/release/nordoi run examples/p2_1_console/hello.noi --grant-console
```

### Or run on Windows (PowerShell)

```powershell
.\target\release\nordoi.exe run examples/p2_1_console/hello.noi --grant-console
```

The existing `.noi` program and explicit console authorization are **identical** in A and B. Only the **tool invocation and compilation timing** differ. The `--grant-console` argument must not be silently inserted by a wrapper or inferred from the source.

## Denial is part of the lesson

If no console authority is provided, the **pre-existing P2.1 `console-run` command** refuses the program (exit code 5, no program stdout):

```sh
./target/release/nordoi console-run examples/p2_1_console/hello.noi
```

On Windows, replace the path with `.\target\release\nordoi.exe`. Do not confuse this contract with **plain ungranted `run`**, which retains the historical V0.1 HALT-only frontend path and can report a different error code.

## What this candidate can and cannot demonstrate

- **Can** demonstrate a known existing, direct executable command after a successful release build, with no new NORDOI runtime code path.
- **Cannot** prove that a first-time human is faster or more accurate using B. It could move perceived complexity from one command into the initial build step; this is unmeasured.
- **Cannot** prove faster NORDOI execution merely because `cargo run` is not invoked for each later launch. A controlled comparison must specify whether it measures rebuild/check overhead, binary start-up, or language runtime execution.
- **Cannot** assume a prebuilt binary exists, make operating-system-neutral executable path claims, bypass permissions, or claim the public `.noi` surface was simplified.
- A future beginner study must independently measure installation/setup, first successful program, source comprehension, command mistakes, explicit-grant comprehension, hints and completion time, without mixing AI simulations into participant data.

**No changes are authorized to `.noi` grammar, source examples, compiler, NAIR, NAM, kernel, runtime authority, production, or the R2 study materials.**
