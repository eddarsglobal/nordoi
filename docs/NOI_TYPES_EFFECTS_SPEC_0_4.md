# NORDOI Types & Effects Foundation Specification 0.4

Status: **L0.4 candidate**  
Kernel semantic floor: **K1.18 certified**  
Frontend prerequisites: **L0.1 / L0.2 / L0.3 certified**  
Tooling prerequisite: **T0.1 certified**  
Compiler prerequisite: **C0.1 certified**  
NAIR: **0.6 unchanged**

## 1. Purpose

L0.4 introduces the smallest source-level type/effect foundation that can be validated without
pretending NORDOI already has functions, expressions, values, handlers or executable effect
operations.

The only new source declarations are:

```noi
type UserId;
effect Network;
```

`type Name;` declares an **opaque nominal type identity**. It defines no representation, fields,
layout, constructors, conversions, nullability, generic parameters or ABI.

`effect Name;` declares a **named effect identity**. It defines no operation, handler, capability,
host authority, dispatch behavior or runtime permission.

This is intentional: identity is established before behavior.

## 2. Contextual declaration prelude

`type` and `effect` remain ordinary L0.1 identifier tokens. L0.4 interprets them contextually only in
a leading top-level declaration prelude after the optional L0.3 module header.

Example:

```noi
module demo.core;

type UserId;
effect Network;

future_body_here
```

The declaration prelude ends at the first significant top-level element that is not a valid `type`
or `effect` declaration start. Once the residual body has begun, later occurrences of `type` and
`effect` are not reinterpreted by L0.4.

This prevents L0.4 from claiming future expression or declaration grammar.

Trivia may appear between declaration components:

```noi
type /* diagnostic-only */ UserId ;
effect // diagnostic-only
Network ;
```

## 3. Fail-closed grammar

Within the leading declaration prelude, a recognized contextual keyword commits to its declaration
shape:

```text
type   Identifier ;
effect Identifier ;
```

Therefore malformed leading declarations fail closed:

```text
type
type 42;
type UserId
effect;
effect (Network);
effect Network type Other;
```

The current executable-name profile remains the L0.1/L0.3 ASCII identifier profile. Unicode
executable identifiers remain deferred until a dedicated Unicode security profile is specified and
certified.

## 4. Bounded resources

L0.4 sets:

```text
MAX_TYPE_EFFECT_DECLARATIONS = 1024
MAX_SEMANTIC_DECLARATIONS    = 1024
MAX_SEMANTIC_EFFECT_REQUIREMENTS = 256
```

Existing name bounds remain:

```text
MAX_NAME_BYTES          = 128
MAX_SEMANTIC_NAME_BYTES = 128
```

The frontend rejects declaration count overflow before unbounded declaration growth. The compiler
revalidates its own independent semantic bound so manually constructed HIR cannot bypass the
frontend budget.

## 5. Surface representation

L0.4 adds:

```text
SurfaceDeclarationKind::{Type, Effect}
SurfaceDeclaration
TypeEffectUnit
TypeEffectAnalyzer
analyze_type_effect_unit(...)
```

A `SurfaceDeclaration` retains:

```text
kind
full declaration span
contextual keyword token
name
terminator token
```

`TypeEffectUnit` retains the L0.3 `ModuleUnit`, the declaration prelude and the residual body span.
The residual body remains structurally available but semantically opaque.

## 6. HIR representation

C0.1 `HirUnit` is extended compatibly with a declaration list:

```text
HirDeclaration {
    kind: Type | Effect
    name: SemanticName
    span: diagnostic source span
}
```

The certified C0.1 constructor remains available and creates a HIR unit with zero declarations.
L0.4 adds `HirUnit::new_with_declarations(...)` and:

```text
lower_type_effect_unit_to_hir(...)
```

The residual body state remains exactly:

```text
HirBodyState::Unlowered
```

L0.4 therefore does not claim expression, statement, function or control-flow semantics.

## 7. NSIR publication

`validate_hir(...)` remains the validate-before-publication gate and now additionally checks:

- semantic declaration count bounds;
- declaration spans belong to the same source unit;
- declaration spans are contained by the file span;
- declarations do not begin before the post-module prelude boundary;
- declaration spans are ordered and non-overlapping;
- declarations do not overlap the residual unlowered body;
- duplicate declarations of the same kind/name are rejected.

`NsirUnit` gains validated `NsirDeclaration` entries. Source spans remain diagnostic metadata.

The two declaration kinds use separate semantic namespaces in L0.4, so this is valid:

```noi
type State;
effect State;
```

but repeating either declaration in its own namespace is invalid.

## 8. Canonical semantic witness

C0.1 `NsirUnit::canonical_identity_bytes()` remains unchanged and continues to represent only the
C0.1 module identity witness.

L0.4 adds:

```text
NsirUnit::canonical_semantic_bytes()
```

with domain:

```text
NORDOI-L0.4-SEMANTIC\0
```

The witness includes:

- canonical C0.1 module identity;
- declaration kind;
- declaration semantic name.

It excludes:

- source IDs and file names;
- source spans;
- comments and whitespace;
- declaration source order (opaque declarations are order-independent in L0.4);
- filesystem paths;
- host authority;
- runtime state;
- CLI state.

Thus equivalent opaque declaration sets receive equal L0.4 semantic witnesses even if formatting,
source IDs or declaration order differ.

## 9. Type law at L0.4

An L0.4 type declaration is nominal and opaque:

```noi
type UserId;
```

means only that a semantic type identity named `UserId` exists in the module's type namespace.

L0.4 deliberately defines no:

- primitive type syntax;
- aliases;
- records/structs;
- variants/enums;
- generics;
- subtyping;
- nullability;
- ownership qualifiers;
- layout or ABI;
- value construction;
- type inference;
- conversions.

Those mechanisms require later evidence and separate law.

## 10. Effect law at L0.4

An L0.4 effect declaration:

```noi
effect Network;
```

means only that a semantic effect identity named `Network` exists in the module's effect namespace.

The declaration is **intent vocabulary, not authority**.

It does not:

- grant network access;
- grant any capability;
- call a host API;
- serialize host authority;
- schedule or dispatch a runtime effect;
- define an effect operation;
- define a handler or continuation;
- change NAIR or runtime semantics.

This preserves the NORDOI law:

```text
declared intent/effect
        +
explicit narrow authority
        +
policy validation
```

## 11. Semantic effect sets

L0.4 adds compiler-owned `SemanticEffectSet` as a future signature primitive. It is not yet attached
to source functions because functions are not defined.

Rules:

- the empty set is explicitly pure/no-required-effects;
- members are sorted canonically;
- duplicate requirements are rejected;
- the set is bounded by `MAX_SEMANTIC_EFFECT_REQUIREMENTS = 256`;
- it contains effect identities only, never capabilities or authority.

Canonical effect-set bytes use domain:

```text
NORDOI-L0.4-EFFECT-SET\0
```

## 12. C0.1 compatibility

The certified C0.1 entry point remains:

```text
compile_semantic_boundary(...)
```

and stays module-only. It continues to treat the full post-module source as `UNLOWERED`, preserving
its certified contract.

L0.4 adds a new entry point:

```text
compile_type_effect_boundary(...)
```

which executes:

```text
SourceText
  → L0.1 lexer
  → L0.2 structural AST
  → L0.3 ModuleUnit
  → L0.4 TypeEffectUnit
  → C0.1 HIR + L0.4 declarations
  → validate_hir
  → validated NSIR
```

## 13. Tooling

The existing T0.1 command remains:

```text
nordoi semantic <path|->
```

It now uses the L0.4 boundary and reports type/effect declaration counts plus the L0.4 canonical
semantic witness. It still does not lower to or execute NAIR.

The T0.1 tool version and C0.1 compiler version strings remain unchanged; L0.4 is a language-track
milestone rather than a new tooling/compiler major boundary.

## 14. Kernel isolation

L0.4 changes no certified kernel semantic surface:

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The L0.4 compiler/frontend additions do not depend on `crate::nair`, `crate::runtime` or
`crate::kernel`.

## 15. Deliberately deferred

L0.4 does not freeze:

- function syntax;
- function types;
- parameter/return syntax;
- effect-use annotations;
- effect inference;
- effect operations;
- handlers or resumable continuations;
- capability source syntax;
- type representations;
- generic types;
- algebraic data types;
- type inference;
- expression syntax;
- symbol resolution;
- imports/packages;
- executable NSIR instructions;
- NSIR → NAIR lowering.

## 16. Tests

`tests/frontend_types_effects.rs` covers contextual parsing, bounds, malformed declarations,
trivia, body cutover, inherited lexer/parser security failures and deterministic publication.

`tests/compiler_types_effects.rs` covers semantic declaration validation, canonical identity,
C0.1 compatibility, duplicate rejection, adversarial spans, bounded effect sets and explicit
intent/authority separation.

`tests/tooling_types_effects.rs` exercises the real `nordoi semantic` command.

Every earlier regression suite remains mandatory.
