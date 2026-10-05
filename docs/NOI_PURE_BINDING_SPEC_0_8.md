# NORDOI L0.8 — Pure Named Bindings Foundation

Status: candidate specification pending Testing & Release Law certification.

## 0. Governance

This milestone is governed first by `CONSTITUTION.md`, then by
`laws/LAW_0001_NORDOI_MASTER_LAW.md`, then by all previously certified semantic contracts.
Nothing in L0.8 may weaken those authorities. In particular, C2 Atomic Speed, C3 Atomic Security,
C4 Zero Legacy Debt, C5 Impossible States First, C7 What You Do Not Use Must Cost Nothing,
C10 Canonical Semantics, C14 Evidence Before Evolution, C21 IR Before Surface Lock-In, C22 Validate
Before Execute and C23 Safe By Omission govern this boundary.

L0.8 does not copy those laws. It conforms to them.

## 1. Purpose

L0.8 introduces the first source-level reusable named value without defining mutable storage,
runtime memory, assignment, a stack frame, an atom, a capability, an effect or a NAIR operation.

The contextual form is:

```noi
const x = 20;
const y = 22;

entry main returns x + y;
```

The result is semantically `INT(42)`.

`const` is contextual only in the leading L0.8 body binding prelude. L0.8 does not globally reserve
all future uses of the token text `const`.

## 2. Binding declaration grammar

```text
PureBindingDecl := "const" Name "=" CanonicalNonNegativeI64 ";"
```

A body may contain zero or more leading pure binding declarations, followed by zero or one existing
entry form.

The current maximum is:

```text
MAX_PURE_BINDINGS = 256
```

The initializer is deliberately a literal only. Binding-to-binding initialization and general
initializer expressions remain undefined so that dependency ordering, cycles and evaluation timing
are not frozen accidentally.

## 3. Expression grammar

L0.8 extends the L0.7 operand set only:

```text
PureBindingExpression := Operand ("+" Operand)*
Operand               := CanonicalNonNegativeI64
                       | BindingName
                       | "(" PureBindingExpression ")"
```

No new operator is introduced. L0.7 checked-addition behavior remains the only arithmetic behavior.

The existing `MAX_PURE_EXPRESSION_NODES = 1024` bound remains in force.

## 4. Binding identity

Each valid binding receives a compiler-owned `SemanticPureBindingId`.

IDs are:

- module-local;
- non-zero;
- deterministic;
- assigned by canonical binding-name order, not declaration order;
- separate from type IDs and effect IDs.

For:

```noi
const z = 3;
const a = 1;
const m = 2;
```

canonical IDs are:

```text
#1 a
#2 m
#3 z
```

Changing source declaration order without changing names or values must not change canonical L0.8
meaning.

## 5. Namespace law

Pure binding names form their own L0.8 namespace. Duplicate binding names in one module fail closed.
A binding name may currently coincide with a type or effect name because those namespaces are
semantically distinct.

A reference that does not resolve in the pure-binding registry fails closed.

No implicit lookup from filesystem, environment, host, package manager or runtime state is allowed.

## 6. Immutable value law

Every L0.8 binding is an immutable compile-time semantic association:

```text
SemanticPureBindingId → canonical i64 value
```

L0.8 defines no:

- assignment;
- mutation;
- address;
- pointer;
- storage slot;
- atom;
- stack/local frame;
- heap allocation;
- lifetime;
- ownership transfer;
- runtime initialization.

A future lowering may choose the cheapest correct representation. L0.8 intentionally does not force
one.

## 7. Canonical postfix meaning

Expression semantics preserve evaluation structure as postfix operations:

```text
INT(v)
BINDING(id)
ADD
```

Example:

```noi
const x = 20;
const y = 22;
entry main returns x + y;
```

becomes semantically:

```text
[BINDING(1), BINDING(2), ADD]
value = 42
```

A reference preserves binding identity; it is not silently rewritten to `INT(value)` in the L0.8
semantic witness.

## 8. Checked arithmetic

`ADD` evaluates using checked signed `i64` addition. Overflow fails before publication of L0.8 NSIR.
Literal syntax remains canonical non-negative decimal, but a future arithmetic milestone may
separately expand the computed integer domain.

## 9. Canonical witness

`NsirPureBindingUnit::canonical_l08_bytes()` begins with:

```text
NORDOI-L0.8-PURE-BINDINGS\0
```

It commits to:

1. exact C0.2 semantic witness;
2. canonical binding registry;
3. entry identity;
4. empty resolved effect requirements;
5. exact postfix operations using binding IDs;
6. validated result value.

It excludes:

- source ID;
- filename/path;
- comments;
- whitespace;
- source spans;
- binding declaration order;
- CLI metadata;
- runtime state;
- host authority;
- capabilities;
- filesystem/environment state.

## 10. Zero-cost boundary

L0.8 performs semantic analysis only. It defines zero runtime work and zero runtime storage.

A binding declaration does **not** imply:

- an allocation;
- a load instruction;
- a global variable;
- an atom;
- a runtime lookup.

This preserves the constitutional requirement that an abstraction must not acquire runtime cost
before such cost is semantically necessary and separately certified.

## 11. Effects and authority

Effect declarations in the module do not become binding requirements. L0.8 pure entries require an
empty resolved effect set.

Binding declarations and references grant no host authority or capability.

## 12. CLI inspection boundary

Additive command:

```text
nordoi bindings <path|->
```

The command reports canonical binding IDs/values, postfix reference operations, result and L0.8
witness. It must report:

```text
planning=UNDEFINED nair=UNCHANGED runtime=NOT_INVOKED storage=NONE authority=NONE
```

The certified `--version` string remains unchanged.

## 13. Frozen earlier boundaries

L0.8 must not rewrite L0.7/C0.7/C0.8/V0.3. In particular, existing commands `expr`, `expr-plan`,
`expr-lower` and `expr-run` must reject a source that uses the new `const` prelude until a later
explicit milestone extends those boundaries.

## 14. Explicitly deferred

L0.8 does not define:

- mutable variables;
- local block scope;
- assignment;
- binding initializers that reference other bindings;
- general constant folding;
- subtraction/multiplication/division;
- booleans/comparisons;
- functions/calls/parameters;
- control flow;
- imports/packages;
- source-level effect use;
- I/O;
- NAIR lowering;
- runtime execution.

Those require separate governed milestones.

## 15. Next boundary

After L0.8 certification, the intended next compiler boundary is **C0.9 — Pure Binding Execution
Plan**, which may preserve binding-reference semantics in a validated zero-authority plan without yet
choosing an operational NAIR representation.
