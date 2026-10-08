# NORDOI Capability-Secured Observable I/O Specification P2.1

## Status

Candidate specification for Production Profile 2 milestone P2.1. Production Profile 1 remains frozen at certified tag `v1.7`.

## Purpose

P2.1 introduces the first deliberately observable NORDOI operation without introducing ambient authority or modifying certified Profile 1 byte formats.

## Surface

A P2.1 observable source contains an optional module declaration, an explicit `effect ConsoleWrite;` declaration, and exactly one body:

```noi
entry <name> emits "<quoted-utf8>";
```

Only one emission is permitted in P2.1.

## Quotas

- decoded output maximum: 4096 UTF-8 bytes;
- entry name maximum: 128 bytes;
- one output operation per P2.1 program;
- no dynamic interpolation;
- no filesystem/network/process or other I/O channel.

## Quoted output

The lexer-provided quoted text token is decoded with exactly these escapes:

- `\\`
- `\"`
- `\n`
- `\r`
- `\t`

Any other escape fails closed before authority is checked.

## Effect declaration

The source must declare exactly the P2.1 observable effect `ConsoleWrite`. Other effect declarations are outside P2.1 and fail closed. Type declarations may coexist but do not grant authority.

The P2.1 effect is local to the observable I/O layer. It is not inserted into the frozen Profile 1 `Effect` enum, because that enum participates in certified NAIR/effect-audit/persistence encodings.

## Authority

`P21ObservableEffect::ConsoleWrite` maps exactly to `Capability::ConsoleWrite`.

The host must explicitly add that capability to `CapabilitySet`. No grant is synthesized from source code. No other capability implies it.

Execution order is:

1. parse and validate source;
2. enforce effect declaration;
3. decode output and enforce quota;
4. construct canonical plan;
5. check exact host capability;
6. construct deterministic receipt;
7. only then may tooling materialize output bytes.

A denied execution must produce zero program-output bytes.

## Canonical plan and receipt

Plan domain:

```text
NORDOI-P2.1-CAPABILITY-SECURED-OBSERVABLE-IO-PLAN\0
```

Receipt domain:

```text
NORDOI-P2.1-CAPABILITY-SECURED-OBSERVABLE-IO-RECEIPT\0
```

Canonical identities include module identity, entry identity, exact effect/capability name, exact decoded output bytes, and explicit authority state. They exclude path, source ID, host, OS, time, environment, and process identity.

## Tooling

```bash
nordoi console-run <path|-> [--grant-console]
```

Without `--grant-console`, authority is absent and the command fails closed. With the grant, exact program output is written to stdout and deterministic receipt metadata to stderr.

## Non-goals

P2.1 does not add:

- dynamic output expressions;
- multiple emissions;
- input/output streams;
- filesystem or network I/O;
- asynchronous effects;
- new NAIR instructions;
- new runtime authority;
- a new package or release format;
- mutation of Production Profile 1 certification artifacts.
