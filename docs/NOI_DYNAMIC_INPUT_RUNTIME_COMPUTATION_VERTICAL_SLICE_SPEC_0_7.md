# NORDOI V0.7 — Dynamic Input & Runtime Computation Vertical Slice

Status: candidate specification for V0.7 certification.

## Purpose

V0.7 introduces the first NORDOI value that is deliberately unknown at compile time and therefore must survive into runtime computation.

The boundary is additive. Certified V0.6 `core-run` semantics remain unchanged.

## Surface

V0.7 adds the `dynamic-run` vertical slice with this minimal body grammar:

```noi
input <name>;
const <name> = <static-expression>;
entry <name> returns <expression>;
```

The input declaration is optional, unique when present, and denotes the key-code of canonical input event 0. Constants are immutable and must be compile-time evaluable. Entry expressions support non-negative integer literals, names, parentheses, checked `+`, and integer comparisons `== != < <= > >=`.

Example:

```noi
module demo.dynamic;

input key_code;
const bias = 2;

entry main returns key_code + bias;
```

With canonical keyboard key-code `40`, the result is `INT(42)`.

## Canonical input law

The program never reads an OS keyboard, device, environment variable, file, socket, or host callback directly. Runtime data arrives only through an `InputBatch` that is canonicalized by the existing Atomic Input boundary.

Same source + same canonical input batch MUST produce the same NAIR program, runtime replay key, result, witness, and V0.7 receipt.

## NAIR 0.9

V0.7 adds:

```text
READ_INPUT_KEY_CODE dst,event_index
```

Canonical opcode: `0x43`.

The instruction reads the key-code from a keyboard-key payload at the requested canonical input event index and publishes it as an integer SSA register.

Programs containing `READ_INPUT_KEY_CODE` require NAIR format minor `0.9`. The certified NAIR base minor remains `0.6`; integer arithmetic remains `0.7`; integer comparisons remain `0.8`.

## Runtime computation law

NAIR validation now distinguishes register type from compile-time register value. Therefore checked integer addition and integer comparison may consume a dynamic integer register while still rejecting invalid types before execution.

Static operands continue to be folded. A completely static V0.7-subset program MUST still lower to:

```text
CONST r0 <result>
HALT
```

and therefore remain NAIR `0.6`.

A dynamic `input + constant` expression lowers minimally to:

```text
READ_INPUT_KEY_CODE r0 event=0
CONST r1 INT(<constant>)
ADD_INT_CHECKED r2 r0 r1
HALT
```

No runtime call stack and no runtime branch machinery are introduced in V0.7.

## Fail-closed rules

V0.7 rejects:

- more than one input declaration;
- duplicate input/constant names;
- constants depending on runtime input;
- unknown names;
- unsupported expression forms;
- chained comparisons;
- missing canonical input event 0 when the result depends on input;
- an event 0 payload that is not a keyboard-key event;
- checked integer overflow during runtime addition.

## Compatibility

`nordoi --version` remains intentionally frozen at the certified public baseline:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

The `dynamic-run` report exposes the actual required artifact minor (`nair-minor=0.9`) without rewriting prior certified version assertions.

## Certification gate

V0.7 may be tagged only after:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo check --all-targets`
4. `cargo test --all-targets`
5. V0.7 focused tests
6. smoke proof with key-code 40 => `INT(42)`
7. exact-SHA multi-platform GitHub CI
8. annotated `v0.7` tag
