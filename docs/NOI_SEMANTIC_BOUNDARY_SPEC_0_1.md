# NORDOI Compiler Semantic Boundary Specification 0.1

Status: **C0.1 candidate**  
Kernel semantic floor: **K1.18 certified**  
Frontend prerequisites: **L0.1 / L0.2 / L0.3 certified**  
Tooling prerequisite: **T0.1 certified**  
NAIR: **0.6 unchanged**

## 1. Purpose

C0.1 creates the first explicit compiler boundary between experimental `.noi` source structure and
future executable semantics. It introduces a source-backed HIR stage and a validated NSIR stage.
C0.1 does **not** define declarations, expressions, source-level types, effects, imports, symbol
resolution or NAIR lowering.

The core pipeline is:

```text
SourceText
  → L0.1 lexer
  → L0.2 structural AST
  → L0.3 ModuleUnit
  → C0.1 HIR
  → validation
  → C0.1 NSIR
```

Publication of NSIR is fail-closed: no `NsirUnit` is produced until the entire C0.1 HIR unit has
passed its structural semantic checks.

## 2. Semantic identity

C0.1 owns semantic names instead of retaining pointers into source text:

```text
SemanticName
SemanticPath
SemanticModuleIdentity
```

A module is either:

```text
Anonymous
Named(SemanticPath)
```

Named paths inherit the current executable-name profile deliberately:

```text
ASCII identifier only
MAX_SEMANTIC_NAME_BYTES = 128
MAX_SEMANTIC_PATH_SEGMENTS = 64
```

These limits are compiler-boundary limits and are validated independently even though L0.3 uses the
same current budgets.

## 3. Canonical module-identity bytes

`SemanticModuleIdentity::canonical_bytes()` is a deterministic C0.1 identity witness. The encoding
uses an explicit domain:

```text
NORDOI-C0.1-MODULE-ID\0
```

followed by a variant byte and, for named modules, big-endian length-prefixed UTF-8 segments.

The identity bytes deliberately exclude:

- source IDs;
- source file names;
- source spans;
- whitespace;
- comments;
- filesystem paths;
- host authority;
- CLI state;
- runtime state.

Therefore these two headers have equal C0.1 semantic module identity:

```noi
module alpha.beta;
```

```noi
module alpha /* diagnostic-only source text */ . beta ;
```

The C0.1 identity witness is **not** yet a global package/module key and must not be used to infer
filesystem or package semantics.

## 4. HIR

C0.1 HIR is represented by `HirUnit`:

```text
module identity
file span
optional module-declaration span
body span
body state
```

The only C0.1 body state is:

```text
HirBodyState::Unlowered
```

This is intentional. A structural L0.2 body is not silently reinterpreted as declarations,
statements or expressions.

For a named module, the HIR body begins exactly at the end of the leading L0.3 module declaration.
For an anonymous module, the HIR body is the entire source file.

Source spans are diagnostic metadata. They are not semantic module identity.

## 5. NSIR

`validate_hir(...)` is the only public transition from HIR to `NsirUnit`.

C0.1 validates at minimum:

- file/module/body spans belong to the same source unit;
- module span, when present, is contained by the file span;
- body span is contained by the file span;
- body does not begin before a present module declaration ends;
- semantic names and paths satisfy C0.1 bounds before construction.

A validated NSIR unit contains:

```text
SemanticModuleIdentity
NsirOrigin       # diagnostic metadata only
NsirBodyState::Unlowered
```

`NsirUnit` has no public arbitrary constructor. This preserves the validate-before-publication
boundary.

## 6. Frontend adapter

C0.1 exposes:

```rust
lower_module_unit_to_hir(...)
compile_semantic_boundary(...)
validate_hir(...)
```

`compile_semantic_boundary` executes the complete C0.1 vertical slice:

```text
L0.3 ModuleUnit → HIR → validation → NSIR
```

Any inherited lexical, parser or module error prevents NSIR publication.

## 7. Tooling

The certified T0.1 binary gains one inspection command:

```text
nordoi semantic <path|->
```

It prints the validated C0.1 module identity, `UNLOWERED` body state, canonical identity bytes and
diagnostic origin spans. It still does not compile to or execute NAIR.

## 8. Security and resource rules

C0.1 follows fail-closed validation and bounded identity construction. It does not introduce:

- ambient filesystem module discovery;
- network access;
- shell execution;
- plugin loading;
- capabilities or host authority in source/program bytes;
- unsafe code;
- unbounded name/path growth.

No host authority is serialized into HIR/NSIR semantic identity.

## 9. Deliberately deferred

C0.1 does not freeze:

- import syntax;
- package syntax;
- declaration syntax;
- expression syntax;
- literals beyond lexical candidates;
- operator precedence;
- symbol tables;
- name resolution;
- visibility;
- aliases;
- language type syntax or type checking;
- effect/capability syntax;
- function ABI;
- control flow;
- HIR item kinds;
- NSIR executable instructions;
- lowering from NSIR to NAIR.

Those surfaces require later compiler/language milestones.

## 10. Kernel isolation

C0.1 changes no certified kernel semantic surface:

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The compiler module may consume frontend types, but C0.1 has no dependency on `crate::nair` and does
not translate or execute NAIR.

## 11. Tests

`tests/compiler_semantic.rs` covers semantic identity, source-independence of canonical identity,
HIR spans/body state, compiler limits, inherited frontend failures, deterministic publication and
adversarial invalid-span combinations.

`tests/tooling_cli.rs` also exercises the real `nordoi semantic` command.

All prior regression suites remain mandatory.
