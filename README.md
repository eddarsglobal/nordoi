# NORDOI C0.1 — Semantic Boundary / HIR-NSIR Foundation

C0.1 adds the first compiler-owned semantic boundary on top of the **certified K1.18 kernel**, the
**certified L0.1/L0.2/L0.3 frontend**, and the **certified T0.1 CLI**.

```text
.noi SourceText
    ↓
L0.1 lexer → L0.2 structural AST → L0.3 ModuleUnit
    ↓
C0.1 source-backed HIR
    ↓ validate-before-publication
C0.1 NSIR (module identity + diagnostic origin + UNLOWERED body)
    ↓
future semantic declarations / names / types / effects
    ↓
future explicit NSIR → NAIR lowering
    ↓
NAIR 0.6+ → NAM/runtime
```

C0.1 deliberately does **not** compile to NAIR. It creates a safe semantic seam first.

## What C0.1 adds

The new `compiler` module exposes:

```rust
SemanticName
SemanticPath
SemanticModuleIdentity
HirBodyState
HirUnit
NsirBodyState
NsirOrigin
NsirUnit
CompilerError
CompilerResult

lower_module_unit_to_hir(...)
validate_hir(...)
compile_semantic_boundary(...)
```

A source module such as `module alpha.beta;` receives compiler-owned canonical semantic identity.
Comments, whitespace, source IDs and spans do not alter its canonical identity bytes. Exact spans
remain available through `NsirOrigin` for diagnostics.

The body is intentionally published only as:

```text
UNLOWERED
```

C0.1 therefore does not pretend that the current structural AST already defines declarations,
expressions, types or effects.

The certified T0.1 tool gains:

```text
nordoi semantic <path|->
```

Normative candidate design: `docs/NOI_SEMANTIC_BOUNDARY_SPEC_0_1.md`.
Architecture record: `research/SEMANTIC_BOUNDARY_INTELLIGENCE_0_1.md`.

`tests/compiler_semantic.rs` adds 21 compiler-boundary tests, while the tooling suite gains three
real `semantic` command tests. All prior frontend and K1.18 regression tests remain mandatory.

## C0.1 does not freeze

C0.1 still does not define imports, packages, declarations, expressions, operator precedence,
symbol resolution, visibility, aliases, source-level types, effects/capability syntax, control flow,
executable NSIR instructions or NSIR → NAIR lowering.

## Certified floor remains unchanged

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

T0.1 adds the first executable developer-tooling boundary on top of the **certified K1.18 kernel** and the **certified L0.1 / L0.2 / L0.3 frontend stack**.

```text
K1.18 certified semantic floor
        ↓
L0.1 certified SourceText / SourceSpan / lossless Lexer
        ↓
L0.2 certified structural Parser / experimental AST
        ↓
L0.3 certified Names & Modules
        ↓
T0.1
inspection CLI: lex / parse / module
        ↓
future C0.x semantic compiler / HIR-NSIR
        ↓
NAIR 0.6+ → NAM/runtime
```


## What T0.1 adds

The repository now builds a `nordoi` executable with three deliberately narrow inspection commands:

```text
nordoi lex <path|->
nordoi parse <path|->
nordoi module <path|->
```

`-` reads UTF-8 source from standard input. The CLI does not compile or execute `.noi`; it exposes
the already-defined L0.1 lexer, L0.2 structural AST and L0.3 module analysis through one deterministic
tooling entry point.

```bash
cargo run --bin nordoi -- --help
cargo run --bin nordoi -- lex example.noi
cargo run --bin nordoi -- parse example.noi
cargo run --bin nordoi -- module example.noi
```

T0.1 reserves exit status `2` for usage errors, `3` for input/I/O failures and `4` for frontend
failures. Whole-file CLI input is bounded to 16 MiB. No network, shell, plugin loading or ambient
filesystem module inference is introduced.

Normative candidate design:

```text
docs/NOI_CLI_TOOLING_SPEC_0_1.md
```

Architecture/research record:

```text
research/CLI_TOOLING_INTELLIGENCE_0_1.md
```

`tests/tooling_cli.rs` exercises help/version, all three commands, standard input, deterministic
output and stable failure classes.

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

All L0.1 lexer tests, L0.2 parser tests, L0.3 module tests and K1.18 regression tests remain mandatory.

## Release Gate

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

T0.1 is a candidate until the local gate passes, GitHub CI is green for the exact T0.1 commit on all
required platforms and an annotated `t0.1` tag is pushed.

## Next architectural boundary

After T0.1 certification, the intended next milestone is **C0.1 — Typed Semantic IR / HIR-NSIR**.

C0.1 should introduce an explicit semantic representation between the experimental source surface
and NAIR without changing certified K1.18 semantics or prematurely binding source syntax directly to
NAIR instructions.
