# NORDOI C0.5 — Pure Result Execution Plan Specification

Status: **candidate** until the NORDOI Testing & Release Law is completed.

## 1. Purpose

C0.5 is the first compiler planning boundary for the certified L0.6 pure-result semantics.
It turns a fully validated `NsirPureResultUnit` into a canonical compiler-owned execution plan
without changing C0.3, lowering to NAIR, invoking the runtime, performing I/O, dispatching effects,
looking up capabilities, or granting host authority.

The primary new source form carried into the plan is:

```noi
entry main returns 42;
```

C0.5 does not define additional source syntax. L0.6 remains the owner of the source-level pure-result
contract.

## 2. Certified inputs carried forward

C0.5 depends on the already-defined L0.6 boundary:

```text
SourceText
  -> L0.1/L0.2/L0.3/L0.4
  -> L0.6 PureResultUnit
  -> HIR validation
  -> NsirPureResultUnit
  -> C0.5 validation
  -> PureResultExecutionPlan
```

No C0.3 object is reused or reinterpreted for result-bearing source. The certified C0.3 plan remains
its own boundary and continues to accept only the certified L0.5 minimal body.

## 3. Plan forms

C0.5 publishes exactly these forms:

```text
EMPTY
ENTRY(name, result = NONE)
ENTRY(name, result = INT(i64))
```

The `INT` value is the exact `i64` value already validated by L0.6. C0.5 does not parse numeric text
again and does not introduce expression semantics.

## 4. Zero operational work

A pure literal result is semantic payload, not yet runtime work. Therefore every C0.5 plan currently
has:

```text
work items       = 0
required effects = 0
host authority   = NONE
NAIR lowering    = UNDEFINED
runtime invoked  = NO
```

This does not claim that future pure computation will always have zero implementation cost. It only
states that C0.5 introduces no operational instruction or runtime action for the already-known literal.

## 5. Effects and authority

Declared effects in the module do not become required effects of a pure-result entry. C0.5 requires
the entry's resolved effect set to be empty before publishing a plan.

A declared effect, resolved effect ID, result value, or plan identity never grants host authority.
Host authority remains outside source bytes, semantic witnesses and NAIR program identity.

## 6. Canonical witness

C0.5 adds:

```text
canonical_c05_bytes()
```

with domain:

```text
NORDOI-C0.5-PURE-RESULT-PLAN\0
```

The witness binds the exact L0.6 witness and then records the C0.5 plan form, entry name, optional
integer result, resolved required-effect IDs, zero-work count, and the no-authority bit.

The witness excludes source IDs, spans, whitespace, comments, filesystem paths, CLI state, host
authority, runtime state and timestamps.

C0.5 does not modify these earlier witnesses:

```text
C0.1 canonical_identity_bytes()
L0.4 canonical_semantic_bytes()
C0.2 canonical_c02_bytes()
L0.5 canonical_l05_bytes()
C0.3 canonical_c03_bytes()
C0.4 canonical_c04_bytes()
L0.6 canonical_l06_bytes()
V0.1 canonical_v01_receipt_bytes()
```

## 7. Validation

`validate_pure_result_execution_plan(...)` fails closed if an entry requires any semantic effect.
The public source boundary is:

```text
compile_pure_result_execution_plan_boundary(...)
```

which always compiles through the certified L0.6 source validation first.

## 8. CLI inspection

C0.5 adds:

```text
nordoi result-plan <path|->
```

The command reports the plan and its witnesses. It does not lower NAIR and does not invoke the runtime.
The certified `nordoi --version` output remains unchanged.

## 9. Compatibility law

C0.5 is additive. In particular:

```text
nordoi plan  result_source.noi  -> still fails at C0.3/L0.5
nordoi lower result_source.noi  -> still fails before C0.4 publication
nordoi run   result_source.noi  -> still fails before V0.1 runtime execution
```

No result-bearing source is silently accepted by an older boundary.

## 10. Explicit non-goals

C0.5 does not define result lowering to NAIR, a return-value opcode, registers for source results,
expressions, arithmetic, variables, calls, parameters, control flow, I/O, effects, capabilities,
authority, runtime result channels, ABI rules, or process exit-code semantics.

Those require later explicit milestones.
