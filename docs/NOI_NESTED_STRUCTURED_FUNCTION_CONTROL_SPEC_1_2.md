# NOI V1.2 — Nested Structured Runtime Control in Function Bodies

## Goal

Allow pure bounded runtime functions to make one or more structured `if/else` decisions while retaining the V1.1 acyclic bounded direct-call model.

## Surface

```noi
fn bias(x) returns if x > 40 {
    x + 100
} else {
    x + 200
};

input key_code;
entry main returns bias(key_code);
```

## Required properties

- function-body `if` condition must be BOOL;
- both arms must have the same value kind;
- only the selected arm executes;
- direct calls may appear in selected arms;
- call graph is resolved before lowering;
- direct and indirect recursion remain rejected;
- maximum runtime call depth remains 8;
- function parameters remain INT-only in this vertical slice;
- no structured `if` is admitted directly at entry level in V1.2;
- no structured `if` or nested call is admitted inside a call argument in V1.2;
- static function control must erase before runtime when all inputs are known.

## Runtime proof target

For `key_code = 41`:

```text
result=INT(141)
runtime-calls=1
runtime-branches=1
call-body-instructions=7
max-call-depth=1
certified-max-call-depth=1
nair-instructions=3
nair-minor=0.14
authority=NONE
input-boundary=CANONICAL
```

For `key_code = 39`:

```text
result=INT(239)
runtime-calls=1
runtime-branches=1
nair-minor=0.14
```

## Fail-closed requirements

Compilation or validation must reject:

- non-BOOL conditions;
- mismatched arm kinds;
- unknown functions;
- arity mismatch;
- direct recursion;
- indirect cycles;
- certified call depth overflow.

Selected runtime arithmetic overflow fails closed. An equivalent overflow in an unselected arm must not execute.
