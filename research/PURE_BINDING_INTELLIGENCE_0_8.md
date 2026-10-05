# L0.8 Intelligence Record — Pure Named Bindings

## Constitutional decision

The first named reusable value should not force NORDOI to inherit the historical assumption that a
"variable" is inherently a mutable memory location. `CONSTITUTION.md` is the governing authority.
Atomic Speed, Zero Legacy Debt, Impossible States First, Canonical Semantics and zero unused cost all
favor starting with immutable semantic names whose operational representation remains unfrozen.

## Why `const` instead of general variables

A general `let`/variable milestone would prematurely combine several independent questions:

- lexical scope;
- mutation;
- assignment;
- lifetime;
- storage class;
- ownership;
- capture;
- initialization ordering;
- runtime representation.

None is required to prove deterministic name binding. L0.8 therefore takes the smallest useful step:
a contextual module-body pure binding whose initializer is a canonical integer literal.

## Why literal-only initializers

Allowing:

```noi
const y = x + 2;
```

would immediately require laws for dependency graphs, forward references, cycles and evaluation
order. Those are valuable future problems, but combining them with first name resolution would make
the milestone broader and harder to certify.

## Why references keep semantic identity

Replacing every reference with its integer value during L0.8 would make:

```noi
const tax = 20;
```

and:

```noi
const width = 20;
```

indistinguishable inside expression semantics. That would erase information future tooling, AI/SI,
refactoring, diagnostics and optimization may need. L0.8 therefore encodes `BINDING(id)` in postfix
semantics and stores values in a canonical binding registry.

Operational lowering may later erase the identity when proven safe and useful.

## Canonical IDs

IDs are assigned by sorted semantic name, not source order. This makes harmless declaration
reordering witness-stable and provides deterministic lookup independent of parser container order.

## Performance posture

L0.8 introduces no runtime representation at all. This is stronger than promising that constants are
"usually optimized": the language boundary simply does not define a storage cost yet.

The future compiler is free to inline, materialize once, encode directly in an instruction, place in
a backend constant pool, or use another representation if and only if that representation preserves
certified semantics and wins under NORDOI's measured performance law.

## Security posture

Bindings resolve only inside the compiler-owned module binding registry. No ambient environment,
filesystem, host symbol table, dynamic plugin or runtime namespace participates.

Unknown names and duplicates fail closed. Resource counts are bounded.

## Why no NAIR change

C21 IR Before Surface Lock-In does not mean every surface addition must immediately change NAIR.
The correct boundary is to establish stable name semantics first. C0.9 can then specify a plan, and a
later compiler/NAIR milestone can decide whether binding references need any operational identity at
all.

## Rejected alternatives

### Mutable variable first

Rejected because it couples name binding to storage and mutation before their laws are understood.

### Runtime global constant table

Rejected because it imposes a permanent cost on a semantic abstraction that may require no runtime
object.

### Eager textual substitution

Rejected because it destroys semantic binding identity and weakens diagnostics/refactoring evidence.

### Binding initializer expressions now

Deferred to avoid prematurely freezing dependency and cycle laws.

## Next pressure point

Once L0.8 proves canonical named bindings, the next question is not syntax expansion. It is whether a
compiler execution plan can preserve those references while maintaining zero effects, zero authority
and zero unnecessary runtime work. That is the intended C0.9 boundary.
