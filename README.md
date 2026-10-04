# NORDOI L0.3 — Names & Modules

L0.3 continues NORDOI's language track on top of the **certified K1.18 kernel**, **certified L0.1
lexical foundation** and **certified L0.2 structural parser**.

```text
K1.18 certified semantic floor
        ↓
L0.1 certified SourceText / SourceSpan / lossless Lexer
        ↓
L0.2 certified structural Parser / experimental AST
        ↓
L0.3
bounded Name + optional contextual module header
        ↓
T0.1 CLI / tooling entry point
        ↓
future C0.x semantic compiler / HIR-NSIR
        ↓
NAIR 0.6+ → NAM/runtime
```

## What L0.3 adds

The `frontend` module now additionally provides:

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

The first source-level module form is deliberately narrow:

```noi
module alpha.beta;
```

Whitespace and comments may appear between the significant header elements.

## Contextual, not lexical

`module` remains an ordinary L0.1 `Identifier` token.

L0.3 interprets it only when it is the **first significant top-level element** of the source. This
avoids globally freezing a keyword while the general `.noi` grammar is still experimental.

A later occurrence of `module` is not reclassified by L0.3.

## Anonymous units

A source file without a leading module header remains valid:

```noi
alpha
```

Its `ModuleUnit` is anonymous. NORDOI does **not** infer module identity from the file name,
directory, repository or operating-system path.

## Bounded names

L0.3 keeps the certified L0.1 executable identifier profile and adds:

```text
MAX_NAME_BYTES      = 128
MAX_MODULE_SEGMENTS = 64
```

Names remain ASCII-only for executable source until NORDOI defines and certifies a Unicode
identifier/confusable policy. Unicode remains available in comments and quoted text as before.

## Source-backed identity

`Name`, `ModulePath` and `ModuleDecl` retain exact source spans.

For:

```noi
module alpha /* x */ . beta;
```

the canonical module identity is:

```text
alpha.beta
```

while the lossless L0.2 AST still preserves every original byte and comment.

## What L0.3 deliberately does not freeze

L0.3 still does **not** define:

- imports / `use`;
- packages;
- filesystem-to-module mapping;
- nested modules;
- declarations or statements;
- expression syntax;
- operator precedence;
- symbol tables or name lookup;
- visibility;
- aliases;
- Unicode executable-name policy;
- types;
- effects/capability source syntax;
- HIR/NSIR;
- lowering to NAIR.

## Failure model

L0.3 fails closed for malformed module headers, including:

```text
module;
module 42;
module alpha
module alpha beta;
module alpha.;
```

It also inherits every L0.1 lexical/security failure and every L0.2 structural failure. The entire
L0.2 structural parse completes before a `ModuleUnit` is published.

## Kernel compatibility

L0.3 changes no certified kernel semantics:

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The Cargo package remains `nordoi_kernel` version `1.18.0` because L0.3 is a frontend-track
milestone, not a K-series kernel semantic version bump.

## Design records

Normative candidate design:

```text
docs/NOI_NAMES_MODULES_SPEC_0_3.md
```

Architecture/research record:

```text
research/NAMES_MODULES_INTELLIGENCE_0_3.md
```

The certified L0.1/L0.2 specs and research records remain present.

## Tests

`tests/frontend_modules.rs` covers contextual behavior, anonymous units, bounded names and paths,
source spans, canonical identity, malformed headers, inherited lexer/parser failures and API
determinism.

All L0.1 lexer tests, L0.2 parser tests and K1.18 regression tests remain mandatory.

## Release Gate

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

L0.3 is a candidate until the local gate passes, GitHub CI is green for the exact L0.3 commit on all
required platforms and an annotated `l0.3` tag is pushed.

## Next architectural boundary

After L0.3 certification, the intended next milestone is **T0.1 — CLI / Tooling Entry Point**.

The CLI should expose the language foundation for inspection without pretending that semantic
compilation or `.noi` → NAIR lowering already exists.
