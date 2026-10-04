# NORDOI Names & Modules Intelligence 0.3

**Milestone:** L0.3  
**Status:** Architecture record

## 1. Question

What is the smallest useful notion of identity NORDOI can add after a lossless lexer and structural
parser without prematurely freezing declarations, imports, packages or filesystem conventions?

## 2. Decision

L0.3 adopts:

- source-backed bounded `Name`;
- one optional, contextual, leading `module` header;
- dotted bounded module identity;
- no implicit module identity;
- no import or symbol-resolution system yet.

## 3. Why `module` is contextual

A conventional lexer often classifies keywords early. That is attractive once a grammar is mature,
but NORDOI's source grammar is still intentionally experimental.

Globally reserving `module` now would create compatibility debt for every future context. L0.3
instead leaves the lexical token as `Identifier` and applies meaning only at one precisely defined
location: the first significant top-level element.

This preserves L0.1/L0.2 neutrality while permitting the language track to begin expressing
compilation-unit identity.

## 4. Why an explicit header

Several mature ecosystems infer module/package identity partly from filesystem structure. That can be
convenient, but it makes semantic identity dependent on build-system and host conventions.

NORDOI's first module identity is explicit in source. A source without the header is anonymous rather
than guessed.

Later tooling may map files to modules or require project-level consistency, but that must be a
separate governed policy.

## 5. Why dotted paths

A dotted sequence is intentionally small:

```text
alpha.beta.gamma
```

It is readable, requires only an already-existing single-character punctuation token, and does not
require L0.3 to introduce a multi-character operator.

The dot has meaning only inside this contextual header. L0.3 does not define member access or
expression semantics for `.`.

## 6. Why no imports yet

Imports immediately force decisions about:

- project roots;
- package identities;
- dependency graphs;
- cycles;
- aliases;
- visibility;
- relative vs absolute lookup;
- filesystem and network resolution;
- reproducibility and supply-chain policy.

Those decisions are materially larger than source module identity. Conflating them into L0.3 would
make the milestone too broad and would violate NORDOI's preference for explicit semantic boundaries.

## 7. Why names remain source-backed

L0.3 `Name` stores a span, not a normalized/interned global symbol.

Advantages:

- exact diagnostic location survives;
- no hidden normalization;
- no interner lifetime/global-state policy yet;
- source identity remains explicit;
- later semantic layers can choose their own canonical symbol representation.

A future HIR/NSIR may intern or canonicalize names after policy is defined.

## 8. ASCII remains temporary executable policy

L0.1 deliberately postponed executable Unicode identifiers until a security profile can cover
normalization and confusables.

L0.3 does not weaken that boundary. Unicode remains available in comments and quoted text, while
executable names stay within the existing ASCII profile.

This is a conservative staging decision, not a claim that final NORDOI identifiers must be ASCII.

## 9. Bounded names and module paths

Names and module paths are attacker-controlled compiler inputs. L0.3 therefore introduces explicit
small bounds:

```text
128 bytes per name
64 module segments
```

This limits pathological allocation/work while being far beyond normal module naming needs.

## 10. No hidden file-to-module mapping

The analyzer does not inspect `SourceText::name()` to derive semantic identity.

Two identical source texts with different host file names therefore receive identical L0.3 module
meaning unless an explicit module header differs.

That property keeps host metadata outside program semantics at this stage.

## 11. Failure before publication

L0.3 parses the entire structural source before module analysis. A malformed group late in the file
therefore prevents a seemingly valid leading module declaration from being published.

This aligns with NORDOI's fail-closed compiler boundary: no "partly accepted" compilation unit is
exposed as valid.

## 12. Alternatives rejected

### Global `module` keyword token

Rejected for L0.3 because it freezes lexical reservation globally.

### Implicit module from file path

Rejected because host/build layout would silently enter source meaning.

### Add imports together with modules

Rejected because dependency resolution is a separate architecture problem.

### Intern all names now

Rejected because interning/canonicalization policy belongs closer to semantic IR and multi-file
compilation.

### Permit arbitrary Unicode names now

Rejected until the Unicode security profile is explicit and tested.

## 13. What L0.3 makes impossible by construction

Within its boundary:

- a module path cannot be empty;
- a module header cannot omit `;`;
- a module segment cannot be a number, string or group;
- a module path cannot exceed the segment bound;
- a source-backed module name cannot exceed the name bound;
- a host file name cannot silently become module semantics;
- `module` cannot become a global lexer keyword by accident;
- malformed L0.2 structure cannot yield a published L0.3 unit.

## 14. What remains intentionally open

L0.3 leaves unresolved:

- import/dependency syntax;
- nested modules;
- project/package model;
- symbol namespaces;
- name lookup;
- visibility;
- aliasing;
- Unicode executable identifiers;
- semantic declarations;
- HIR/NSIR.

## 15. Next milestone rationale

With source text, lossless lexical structure, balanced structural AST and explicit compilation-unit
identity available, the next practical step is **T0.1 — CLI / Tooling Entry Point**.

A CLI can make these foundations observable (`lex`, `parse`, `module inspect`) before the semantic
compiler layers grow. This provides a stable human/debugging interface without pretending that
NORDOI already has types, effects or executable `.noi` lowering.
