# NORDOI V0.3 — First Executed Pure Expression

Status: candidate specification pending Testing & Release Law certification.

## 1. Purpose

V0.3 is the first source-level execution boundary for the certified L0.7/C0.7/C0.8 pure-expression pipeline. It executes the exact C0.8 NAIR program through the existing closed AtomicRuntime and validates transient final registers as execution evidence.

V0.3 adds no new source syntax, NAIR opcode, persistent state, effect, capability, host authority, checkpoint field, or external I/O channel.

## 2. Certified input pipeline

```text
.noi source
  → L0.7 pure expression semantics
  → C0.7 pure expression execution plan
  → C0.8 pure expression NAIR lowering
  → existing closed runtime with transient register observation
  → V0.3 validation + deterministic receipt
```

For:

```noi
entry main returns 20 + 22;
```

C0.8 must already provide:

```text
CONST r0, INT(20)
CONST r1, INT(22)
ADD_INT_CHECKED r2, r0, r1
HALT
```

V0.3 executes that exact program and requires the final observed register map to contain exactly:

```text
r0 = INT(20)
r1 = INT(22)
r2 = INT(42)
```

The source-level result is `INT(42)` from `r2` only after the complete register observation has been validated.

## 3. Structural execution law

V0.3 validates every postfix node, not only the final result.

Given a certified postfix sequence, deterministic SSA numbering is the zero-based postfix node index:

- `INT(v)` at node `n` requires `CONST rn, INT(v)` and observed `rn = INT(v)`.
- `ADD` at node `n` consumes the two stack-top predecessor registers, requires `ADD_INT_CHECKED rn, lhs, rhs`, and observed `rn` must equal checked integer addition of the predecessor values.
- exactly one `HALT` follows the expression instructions.

The final postfix stack must contain exactly one register, which must equal `PureExpressionNairArtifact::result_register()`.

## 4. Minor-version compatibility

V0.3 does not rewrite C0.8 wire-format selection:

- expression-free execution: NAIR 0.6,
- literal-only expression: NAIR 0.6,
- expression using `ADD_INT_CHECKED`: NAIR 0.7.

No V0.3 receipt field is inserted into NAIR bytes.

## 5. Closed-runtime invariants

A successful V0.3 execution requires:

- semantic work count = 0,
- required semantic effects = 0,
- host authority = absent,
- canonical empty input,
- input events = 0,
- final atoms = 0,
- created domains = 0,
- created atoms = 0,
- transactions = 0,
- scheduled work = 0,
- render bindings/frames = 0,
- input bridges/applications = 0,
- runtime quiescent at completion,
- executed instruction count equals the exact C0.8 instruction count,
- transient final register map equals the deterministic postfix register/value map.

Any mismatch fails closed and publishes no V0.3 report.

## 6. Result-free forms

For `entry main;` and an empty body, C0.8 must remain exactly `[HALT]`, the result register must be absent, the runtime must observe zero registers, and V0.3 publishes `result=NONE`.

`returns 0` remains a real result and is not equivalent to no result.

## 7. Receipt

`PureExpressionExecutionReport::canonical_v03_receipt_bytes()` begins with:

```text
NORDOI-V0.3-PURE-EXPRESSION-EXECUTION-RECEIPT\0
```

It commits to:

1. exact C0.8 compiler witness,
2. canonical input bytes,
3. runtime replay key,
4. closed-runtime execution counters,
5. every transient final register id/value pair in canonical register order,
6. result-register/value binding,
7. quiescence.

Operational metadata that is not explicitly listed remains outside source semantic identity.

## 8. CLI

Additive command:

```text
nordoi expr-run <path|->
```

Compiler/frontend failure exits with code 4. Runtime or V0.3 invariant failure exits with code 5. Existing commands and the frozen `--version` string remain unchanged.

## 9. Frozen surfaces

V0.3 must not modify:

- C0.8 lowering semantics or witness,
- NAIR 0.6/0.7 wire format or opcodes,
- runtime execution semantics,
- runtime checkpoints,
- Kernel K1.18 semantics,
- effect/capability/authority models,
- V0.1 `run`,
- V0.2 `result-run`,
- CI and Release Gate law.
