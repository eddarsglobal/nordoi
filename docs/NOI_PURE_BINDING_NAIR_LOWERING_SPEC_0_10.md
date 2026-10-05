# NORDOI C0.10 — Pure Binding → NAIR Lowering Foundation

Status: candidate specification.

Governance: `CONSTITUTION.md` is the supreme NORDOI authority. This specification is subordinate to it, to `laws/LAW_0001_NORDOI_MASTER_LAW.md`, and to all certified earlier boundaries through C0.9.

## 1. Purpose

C0.10 defines the first lowering of certified L0.8/C0.9 immutable pure bindings into existing NAIR.

Example source:

```noi
const x = 20;
const y = 22;

entry main returns x + y;
```

C0.9 preserves semantic binding identity:

```text
bindings=[#1:x=INT(20),#2:y=INT(22)]
ops=[BINDING(1),BINDING(2),ADD]
value=INT(42)
```

C0.10 lowers this to existing NAIR only:

```text
CONST r0 INT(20)
CONST r1 INT(22)
ADD_INT_CHECKED r2 r0 r1
HALT
```

No runtime binding object exists.

## 2. Constitutional lowering rule

An immutable pure binding known completely at compile time SHALL NOT require runtime binding lookup or storage merely because the programmer gave a value a name.

For C0.10:

- each `INT(v)` emits the same `Const` as C0.8;
- each `BINDING(id)` resolves its certified immutable L0.8 value during compilation and emits the same `Const` as the equivalent literal;
- each `ADD` emits the existing N0.7 `IntAddChecked`;
- unused bindings emit zero NAIR instructions;
- there is no `LOAD_BINDING` instruction;
- there is no runtime binding table;
- there is no atom, heap object, stack slot, global variable, capability, effect or authority created for a pure binding.

This realizes the applicable constitutional obligations without modifying the Constitution itself.

## 3. Semantic identity versus operational representation

Binding erasure does not erase source semantic identity.

These programs may have identical canonical NAIR bytes:

```noi
const x = 42;
entry main returns x;
```

```noi
const x = 42;
entry main returns 42;
```

Their C0.9 plans are distinct, therefore their C0.10 compiler witnesses remain distinct.

The C0.10 witness domain is:

```text
NORDOI-C0.10-PURE-BINDING-NAIR\0
```

The witness commits to:

1. exact C0.9 canonical plan bytes;
2. optional final result-register metadata;
3. exact canonical NAIR bytes.

Source paths, source IDs, spans, comments, whitespace, CLI state, host environment and runtime authority are not promoted into semantic identity.

## 4. Register allocation

C0.10 uses deterministic monotonically fresh SSA registers exactly as C0.8 does for equivalent literal postfix structure.

For every operand (`INT` or resolved `BINDING`) one fresh `Const` destination is emitted. For every `ADD`, rhs and lhs are popped from the postfix register stack and one fresh `IntAddChecked` destination is emitted.

After a value-producing expression, the stack SHALL contain exactly one result register. Invalid internal postfix state fails closed.

## 5. NAIR compatibility

C0.10 adds no NAIR opcode and does not change NAIR canonical encoding.

- no expression → `[HALT]` → NAIR 0.6;
- one literal/binding result → `[CONST, HALT]` → NAIR 0.6;
- addition → existing `IntAddChecked` → NAIR 0.7.

The certified CLI version string remains unchanged:

```text
nordoi T0.1 (compiler C0.2, kernel K1.18, NAIR 0.6)
```

That string is a frozen historical tooling contract and does not claim that later optional NAIR 0.7 programs are unsupported.

## 6. Zero-unused-cost requirement

A binding that is not referenced by the entry expression SHALL contribute:

```text
0 NAIR instructions
0 runtime storage items
0 runtime lookups
0 effects
0 authority
```

Therefore:

```noi
const unused = 999;
entry main returns 42;
```

must have the same operational NAIR as:

```noi
entry main returns 42;
```

while their higher-level compiler witnesses may remain distinct.

## 7. Validation

Before publication, C0.10 requires:

- C0.9 work item count = 0;
- C0.9 runtime storage item count = 0;
- resolved effects empty;
- host authority not required;
- every binding ID resolves exactly to the certified registry entry;
- valid postfix register stack;
- valid NAIR program;
- canonical NAIR serialization succeeds.

Failure is closed.

## 8. Explicit non-goals

C0.10 does not define:

- runtime execution of binding programs;
- mutable variables;
- assignment;
- runtime binding storage;
- closures or captures;
- functions/calls;
- imports;
- new arithmetic operators;
- new NAIR instructions;
- effects, I/O or authority.

A later V milestone may execute C0.10 output. Mutation requires a separate future semantic law and SHALL NOT be inferred from `const`.

## 9. Tooling boundary

Additive command:

```text
nordoi bindings-lower <path|->
```

It prints L0.8/C0.9 identity, C0.10 witness, canonical NAIR, result-register metadata and explicit zero runtime binding storage/lookup. It never invokes the runtime.

Certified earlier commands remain unchanged.
