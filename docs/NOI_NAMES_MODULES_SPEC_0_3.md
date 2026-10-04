# NORDOI Names & Modules Specification 0.3

**Milestone:** L0.3  
**Status:** Candidate until release-gate, exact-commit CI and annotated tag certification  
**Kernel floor:** K1.18 certified  
**Lexical floor:** L0.1 certified  
**Structural parser floor:** L0.2 certified  
**NAIR:** 0.6 unchanged

## 1. Purpose

L0.3 introduces the first deliberately narrow source meaning above the lossless L0.2 structural AST:

- bounded source names;
- an optional explicit module header;
- a bounded dotted module path;
- source-exact spans for names and module declarations.

L0.3 does not introduce a general declaration grammar. It does not define imports, packages,
filesystem mapping, symbol tables, lexical scopes, visibility, name lookup, type lookup, overload
resolution, HIR/NSIR or lowering to NAIR.

## 2. Contextual `module`

The text `module` remains an L0.1 `Identifier`. The lexer gains no keyword token.

L0.3 interprets `module` contextually only when it is the first significant top-level element in a
source file. Significant means non-whitespace and non-comment.

Therefore:

```noi
module alpha.beta;
```

has an L0.3 module header, while:

```noi
alpha module beta;
```

does not.

`Module`, `MODULE` and other spellings are not contextual module markers.

This decision prevents L0.3 from globally reserving a keyword before the final source grammar exists.

## 3. Module header grammar

Ignoring trivia between significant elements, the only L0.3 module-header shape is:

```text
module-header := "module" name ("." name)* ";"
name          := L0.1 Identifier subject to L0.3 name bounds
```

Trivia may appear between every significant element:

```noi
/* lead */ module /* a */ alpha /* b */ . /* c */ beta /* d */ ;
```

The canonical module identity of that header is:

```text
alpha.beta
```

The original source text remains unchanged and lossless in the L0.2 AST.

## 4. Anonymous compilation units

A source file without a leading contextual module header is valid in L0.3 and produces an anonymous
`ModuleUnit`.

L0.3 does not derive module identity from:

- file name;
- directory path;
- repository path;
- package metadata;
- operating-system path rules.

Those policies require a later project/package design and must not be silently inferred.

## 5. Names

`Name` is a source-backed span over an L0.1 `Identifier`.

The current executable identifier profile remains ASCII:

```text
start:    A-Z | a-z | _
continue: A-Z | a-z | 0-9 | _
```

L0.3 does not add Unicode identifiers. The future Unicode-security profile remains an explicit
language decision.

Names are case-sensitive and are not normalized in L0.3.

## 6. Bounds

L0.3 defines:

```text
MAX_NAME_BYTES       = 128
MAX_MODULE_SEGMENTS  = 64
```

A name longer than 128 UTF-8 bytes fails closed. Under the current ASCII executable-name profile,
this is also 128 characters.

A module path with more than 64 name segments fails closed before publication.

## 7. Source identity and spans

Every `Name`, `ModulePath` and `ModuleDecl` retains source identity through `SourceSpan`.

`ModulePath::span()` covers the original contiguous region from the first name segment start to the
last name segment end, including any intervening punctuation and trivia.

`ModulePath::canonical_text(source)` joins validated name segments with `.` and deliberately omits
formatting trivia.

`ModuleDecl::span()` begins at the contextual `module` identifier and ends after the required `;`.

## 8. Structural dependency

L0.3 first performs the complete L0.2 parse. A lexical or structural error anywhere in the source
prevents publication of `ModuleUnit`.

This means malformed structure is never hidden merely because a valid module header appears first.

## 9. Failure model

L0.3 fails closed for at least:

- missing name after contextual `module`;
- non-identifier module segment;
- missing `.` or `;` after a segment;
- missing segment after `.`;
- missing terminating `;`;
- overlong name;
- too many module segments;
- any inherited L0.1 lexical/security error;
- any inherited L0.2 structural error.

No partially valid module declaration is published on failure.

## 10. Contextual neutrality after the header

L0.3 only inspects the optional leading module header. A later token whose source text is `module`
is not globally reclassified by this layer.

This keeps the remainder of the source surface available for future grammar design.

## 11. Public frontend surface

L0.3 adds:

```rust
Name
NameError
NameResult
MAX_NAME_BYTES

ModulePath
ModuleDecl
ModuleUnit
ModuleAnalyzer
ModuleError
ModuleResult
MAX_MODULE_SEGMENTS
analyze_module_unit(...)
```

L0.1 and L0.2 public surfaces remain available.

## 12. Explicit non-goals

L0.3 does not define:

- `import`, `use`, `export` or dependency syntax;
- nested modules;
- package manifests;
- module-to-file mapping;
- relative module paths;
- aliasing;
- wildcard imports;
- symbol declarations;
- symbol scopes;
- duplicate symbol detection;
- cross-file name resolution;
- visibility;
- namespaces beyond the explicit module identity;
- Unicode identifier normalization/confusable policy;
- type/effect/capability syntax;
- HIR/NSIR;
- NAIR changes.

## 13. Kernel and NAIR compatibility

L0.3 changes no certified kernel semantics.

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The Cargo package therefore remains `nordoi_kernel` version `1.18.0`.

## 14. Certification tests

The L0.3 test suite must prove at least:

- anonymous unit behavior;
- exact contextual-keyword behavior;
- lexer keyword neutrality;
- simple and dotted module paths;
- trivia tolerance;
- exact source spans;
- canonical module identity;
- required terminator;
- invalid/missing segments;
- name byte bound;
- module segment bound;
- inherited lexical failure;
- inherited structural failure;
- deterministic equivalence of analyzer API paths;
- exact source-backed `Name` identity.

All prior L0.1, L0.2 and K1.18 regression tests remain mandatory.

## 15. Release gate

Certification requires:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

followed by green GitHub CI for the exact L0.3 commit on all required platforms and an annotated
`l0.3` tag.

## 16. Next boundary

After L0.3 certification, the intended next milestone is **T0.1 — CLI / Tooling Entry Point**.

The CLI may expose source/lex/parse/module inspection, but it must not silently invent semantic
compiler stages that do not yet exist.
