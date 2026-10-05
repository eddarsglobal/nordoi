# NORDOI L0.6 — Pure Result Foundation

L0.6 builds on certified **V0.1 First Executable `.noi` Program** and adds the first pure source-level
result without modifying the certified L0.5/C0.3/C0.4/V0.1 execution chain.

```noi
module demo.result;

type User;
effect Network;

entry main returns 42;
```

The new contextual `returns` form accepts only canonical non-negative decimal integers in
`0..=i64::MAX`. It defines no general expression grammar, signs, operators, variables, calls, I/O,
effects, capability grants or runtime work.

L0.6 adds:

```text
analyze_pure_result_unit(...)
compile_pure_result_boundary(...)
NsirPureResultUnit
canonical_l06_bytes()

nordoi result <path|->
```

The certified L0.5 boundary remains unchanged: `nordoi body` still rejects result-bearing entries, and
C0.3/C0.4/V0.1 do not silently plan/lower/run them. This makes the next result→execution transition an
explicit future compiler milestone.

Normative candidate design: `docs/NOI_PURE_RESULT_SPEC_0_6.md`.  
Architecture/research record: `research/PURE_RESULT_INTELLIGENCE_0_6.md`.

The compiler execution plan, NAIR 0.6, runtime, checkpoint formats, kernel semantics, CI and Release
Gate are unchanged.

## Certified V0.1 foundation carried forward

V0.1 builds on certified **C0.4 Semantic Plan → NAIR Lowering Foundation** and closes the first
complete `.noi` source-to-runtime path without broadening NAIR, runtime authority, or the source
language.

```text
.noi source
  ↓ L0.1–L0.5
validated minimal source semantics
  ↓ C0.1–C0.4
NAIR 0.6 [Halt]
  ↓ V0.1
closed AtomicRuntime execution with canonical empty input
```

V0.1 adds:

```text
execute_source_v01(...)
validate_v01_execution(...)
SourceExecutionReport
SourceExecutionError
canonical_v01_receipt_bytes()

nordoi run <path|->
```

The execution boundary validates a strict zero-work result: exactly one executed `Halt`, zero input,
zero domains, zero atoms, zero transactions, zero frames, zero input bridges, zero scheduled work,
and a quiescent runtime. No effect or host authority is granted or consumed.

The V0.1 receipt binds C0.4 compiler provenance to canonical empty input and the deterministic runtime
replay key. It is execution evidence, not source semantics, NAIR bytes, or a runtime checkpoint.

Normative candidate design: `docs/NOI_FIRST_EXECUTION_SPEC_0_1.md`.  
Architecture/research record: `research/FIRST_EXECUTION_INTELLIGENCE_0_1.md`.

The existing NAIR 0.6 implementation, runtime implementation, runtime checkpoint, K1.18 kernel
semantics, CI and Release Gate are unchanged.

## Certified C0.4 foundation carried forward

C0.4 builds on certified **C0.3 Executable Semantic Plan** and introduces the first explicit
compiler-owned lowering into existing **NAIR 0.6**, without changing NAIR and without invoking the
runtime.

```text
C0.3 SemanticExecutionPlan (zero work / zero effects / no authority)
  ↓ validate-before-lowering
C0.4 NairLoweringArtifact
  ├─ preserves C0.3 plan witness
  ├─ emits exact canonical NAIR 0.6 bytes
  └─ host authority: NONE
  ↓
NairProgram [Halt]
```

NAIR 0.6 requires a terminal `Halt`, so both current fully understood plans — `EMPTY` and pure
`ENTRY(name)` — lower to exactly one `Instruction::Halt`. Their raw NAIR bytes are intentionally
identical because they perform identical runtime work: none. C0.4 `canonical_c04_bytes()` preserves
the compiler provenance by binding the exact C0.3 witness to those exact NAIR bytes.

C0.4 adds:

```text
NairLoweringArtifact
lower_execution_plan_to_nair(...)
compile_nair_lowering_boundary(...)
canonical_c04_bytes()

nordoi lower <path|->
```

The lowering command validates and prints NAIR; it does **not** execute NAIR. Declared or resolved
effects remain non-authoritative, and no host authority is serialized into program bytes.

Normative candidate design: `docs/NOI_NAIR_LOWERING_SPEC_0_4.md`.  
Architecture/research record: `research/NAIR_LOWERING_INTELLIGENCE_0_4.md`.

The existing NAIR 0.6 implementation, runtime, runtime checkpoint, K1.18 kernel semantics, CI and
Release Gate are unchanged.

## Certified C0.3 foundation carried forward

C0.3 builds on the certified **L0.5 Minimal Body Semantics** and introduces the first compiler-owned
execution plan without changing NAIR or invoking the runtime.

```text
L0.5 NsirBodyUnit (EMPTY | pure ENTRY)
  ↓ validate-before-publication
C0.3 SemanticExecutionPlan
  ├─ form: EMPTY | ENTRY(name)
  ├─ work items: 0
  ├─ required effects: 0
  └─ host authority: NONE
  ↓
C0.4 explicit semantic-plan → NAIR lowering
```

C0.3 adds `compile_execution_plan_boundary()`, `validate_execution_plan()`,
`SemanticExecutionPlan`, `SemanticPlanForm`, `SemanticEntryPlan`, and the new canonical witness
`canonical_c03_bytes()`. It also adds the inspection command:

```text
nordoi plan <path|->
```

The plan is **not runtime execution**. It performs no host calls, capability lookup, effect dispatch,
NAIR generation, or work. `entry main;` remains a pure zero-work semantic entry.

All earlier witnesses remain unchanged, including L0.5 `canonical_l05_bytes()`. The certified C0.2
`--version` output also remains unchanged for compatibility.

Normative candidate design: `docs/NOI_EXECUTABLE_SEMANTIC_PLAN_SPEC_0_3.md`.  
Architecture/research record: `research/EXECUTABLE_SEMANTIC_PLAN_INTELLIGENCE_0_3.md`.

The kernel, NAIR 0.6, runtime, runtime checkpoint, host authority model, CI, and Release Gate are
unchanged.


## Certified L0.5 foundation carried forward

L0.5 builds on the **certified K1.18 kernel**, **certified L0.1/L0.2/L0.3/L0.4 frontend**,
**certified T0.1 CLI**, and **certified C0.1/C0.2 semantic compiler boundaries**.

It adds the first body form that NORDOI understands completely:

```noi
module demo.app;
type User;
effect Network;

entry main;
```

The L0.5 body is exactly either empty/trivia-only or one contextual `entry Name;`. The entry is a
**pure zero-work semantic entrypoint**: no parameters, no return value, no calls, no statements, no
effect operations, no host authority and no runtime execution. Any other significant body syntax
fails closed at the L0.5 boundary.

```text
L0.4 residual body
  ↓
L0.5 complete body recognition
  ↓
HirBodyUnit
  ↓ validate-before-publication
NsirBodyUnit (EMPTY | pure ENTRY)
  ↓
future executable semantic plan
  ↓
future explicit NSIR → NAIR lowering
```

The certified C0.2 `nordoi semantic` path remains unchanged and continues to report its residual body
as `UNLOWERED`. L0.5 adds a separate inspection command:

```text
nordoi body <path|->
```

L0.5 adds `canonical_l05_bytes()` while preserving C0.1 `canonical_identity_bytes()`, L0.4
`canonical_semantic_bytes()` and C0.2 `canonical_c02_bytes()` unchanged. Source spans, comments and
whitespace remain diagnostic metadata outside canonical identity.

Normative candidate design: `docs/NOI_MINIMAL_BODY_SPEC_0_5.md`.  
Architecture/research record: `research/MINIMAL_BODY_INTELLIGENCE_0_5.md`.

The kernel, NAIR 0.6, runtime, runtime checkpoint and K1.18 stability map are unchanged.

## Certified C0.2 foundation carried forward

C0.2 builds on the **certified K1.18 kernel**, **certified L0.1/L0.2/L0.3/L0.4 language foundations**,
**certified T0.1 CLI**, and **certified C0.1 semantic boundary**.

It does not add runtime behavior. It turns the already-valid L0.4 `type` / `effect` declarations into a
canonical compiler-owned registry with separate typed IDs:

```text
type UserId;       -> SemanticTypeId(1)
effect Network;    -> SemanticEffectId(1)
```

IDs are module-local, non-zero, deterministic, and assigned from canonical name order rather than
source declaration order. Type and effect namespaces remain distinct, so `type State; effect State;`
is unambiguous.

```text
.noi source
  ↓
L0.1/L0.2/L0.3/L0.4 frontend
  ↓
C0.1/L0.4 HIR validation
  ↓
C0.2 canonical SemanticRegistry
  ├─ NsirTypeSymbol + SemanticTypeId
  └─ NsirEffectSymbol + SemanticEffectId
  ↓
resolved semantic requirements
  ↓
future body semantics / explicit NSIR → NAIR lowering
```

`SemanticEffectSet` can now be resolved against the registry into `ResolvedEffectSet`. An undeclared
effect fails closed. Resolution is **not authority**: resolving `Network` proves only that the declared
semantic identity exists. Host capability checks remain outside program bytes and outside C0.2.

C0.2 preserves both older witnesses unchanged:

- C0.1 `canonical_identity_bytes()`
- L0.4 `canonical_semantic_bytes()`

and adds `canonical_c02_bytes()` for module + canonical symbol-registry identity.

The body remains explicitly `UNLOWERED`; C0.2 defines no functions, calls, expressions, handlers,
control flow, ABI, memory layout, capability grants, NAIR instructions, or runtime execution.

Normative candidate design: `docs/NOI_SEMANTIC_REGISTRY_SPEC_0_2.md`.  
Architecture/research record: `research/SEMANTIC_REGISTRY_INTELLIGENCE_0_2.md`.

The T0.1 `nordoi semantic` command reports the C0.2 registry witness and deterministic symbol IDs.

## Certified L0.4 foundation carried forward

L0.4 builds on the **certified K1.18 kernel**, **certified L0.1/L0.2/L0.3 frontend**,
**certified T0.1 CLI** and **certified C0.1 semantic boundary**.

The new source prelude is deliberately small:

```noi
module demo.core;

type UserId;
effect Network;

future_body_here
```

`type Name;` creates an opaque nominal type identity. `effect Name;` creates a named effect identity
only: it grants no capability, authority, host access or runtime execution. The residual program body
remains `UNLOWERED`.

```text
.noi SourceText
    ↓
L0.1 lexer → L0.2 structural AST → L0.3 ModuleUnit
    ↓
L0.4 contextual type/effect prelude
    ↓
C0.1 HIR + L0.4 declarations
    ↓ validate-before-publication
validated NSIR + canonical L0.4 semantic witness
    ↓
future function/type/effect-use semantics
    ↓
future explicit NSIR → NAIR lowering
```

L0.4 also adds compiler-owned `SemanticEffectSet`: empty means no required effects, members are
canonicalized, duplicate-free and bounded. It is not yet attached to functions because NORDOI has not
yet frozen function syntax or function types.

Normative candidate design: `docs/NOI_TYPES_EFFECTS_SPEC_0_4.md`.  
Architecture/research record: `research/TYPES_EFFECTS_INTELLIGENCE_0_4.md`.

The T0.1 `nordoi semantic` command now reports validated type/effect counts and the L0.4 semantic
witness while still performing no NAIR lowering or execution.

## Certified C0.1 foundation carried forward

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

C0.3 is a candidate until the local gate passes, GitHub CI is green for the exact C0.3 commit on all
required platforms and an annotated `c0.3` tag is pushed.

## Next architectural boundary

After C0.3 certification, the intended next compiler boundary is an explicit, separately governed
plan/NSIR → NAIR lowering milestone. C0.3 itself deliberately leaves that mapping undefined.
