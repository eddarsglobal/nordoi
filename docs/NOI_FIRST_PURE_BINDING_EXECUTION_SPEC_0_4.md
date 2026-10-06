# NORDOI V0.4 — First Executed Pure Binding Program

## Governance

`CONSTITUTION.md` is the supreme project authority. This specification is subordinate to it, to `laws/LAW_0001_NORDOI_MASTER_LAW.md`, and to all certified earlier boundaries through C0.10.

V0.4 does not redefine the Constitution or duplicate its laws. It applies them to one narrow execution boundary.

## Scope

V0.4 executes the exact C0.10 NAIR generated from L0.8/C0.9 immutable named bindings through the existing closed observed runtime.

Example:

```noi
const x = 20;
const y = 22;
entry main returns x + y;
```

C0.10 has already erased binding names before runtime:

```text
CONST r0 INT(20)
CONST r1 INT(22)
ADD_INT_CHECKED r2 r0 r1
HALT
```

V0.4 executes that exact program and validates:

```text
r0 = INT(20)
r1 = INT(22)
r2 = INT(42)
```

## Binding zero-runtime-cost law

V0.4 introduces no runtime binding representation.

A binding declaration or reference creates no:

- runtime binding table,
- binding lookup,
- atom,
- heap object,
- stack slot dedicated to the source name,
- domain,
- transaction,
- effect,
- capability,
- authority.

The runtime executes only the existing NAIR instructions produced by C0.10.

An unused binding must therefore contribute zero executed instructions. For:

```noi
const unused = 999;
entry main returns 42;
```

execution remains exactly:

```text
CONST r0 INT(42)
HALT
```

## Semantic identity versus operational identity

C0.10 may erase different source-level binding semantics to identical operational NAIR. In that case:

- the runtime replay key may be identical because the executed NAIR and canonical input are identical;
- the V0.4 receipt must remain distinct when the C0.10 semantic witness is distinct.

This preserves auditability without charging runtime cost for names.

## Closed execution invariants

A successful V0.4 execution requires:

- semantic work item count = 0;
- runtime binding storage = 0;
- required effects = 0;
- host authority = absent;
- canonical empty input;
- exact C0.10 instruction count executed;
- all expected transient SSA registers present with exact integer values;
- zero atoms;
- zero domains;
- zero transactions;
- zero render frames/state;
- zero input bridges/applications;
- zero scheduled residual work;
- quiescent completion.

Any mismatch fails closed.

## Receipt

The deterministic receipt domain is:

```text
NORDOI-V0.4-PURE-BINDING-EXECUTION-RECEIPT\0
```

The receipt commits to:

- exact C0.10 witness;
- canonical empty input;
- runtime replay key;
- execution counts;
- final transient registers;
- result register/value;
- zero binding runtime storage;
- zero binding runtime lookups;
- quiescence.

The receipt is evidence, not authority and not persistent program state.

## API

```text
PureBindingExecutionReport
PureBindingExecutionError
PureBindingExecutionResult
execute_pure_binding_source_v04(...)
validate_v04_execution(...)
canonical_v04_receipt_bytes()
```

## Tooling

```text
nordoi bindings-run <path|->
```

Compiler/frontend failures use exit code 4. Runtime/invariant failures use exit code 5.

## Frozen surfaces

V0.4 does not change:

- `CONSTITUTION.md`;
- `laws/`;
- NAIR instruction or wire format;
- runtime semantics/state format;
- runtime checkpoints;
- kernel semantics;
- C0.10 lowering semantics;
- earlier V0.1/V0.2/V0.3 execution boundaries;
- exact `--version` output;
- CI workflow;
- Release Gate.
