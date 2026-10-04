# NORDOI `.noi` Structural Parser & Experimental AST 0.2

**Milestone:** L0.2  
**Status:** Candidate until local Release Gate + GitHub CI + annotated `l0.2` tag  
**Certified kernel baseline:** K1.18 unchanged  
**Lexical foundation:** L0.1 certified  
**NAIR:** 0.6 unchanged  
**Runtime checkpoint:** `NDRTSM01/1.1` unchanged

L0.2 adds the first parser layer above the certified L0.1 source/lexer foundation. Its job is
intentionally narrow: convert the lossless token stream into a deterministic, lossless structural
tree whose only grammar commitment is balanced grouping with `()`, `[]` and `{}`.

L0.2 does **not** define declaration syntax, keyword meaning, operator meaning, precedence, module
syntax, names, types, effects, capabilities, HIR/NSIR or lowering to NAIR.

## 1. Boundary

```text
UTF-8 `.noi` source
      ↓
L0.1 SourceText / SourceSpan
      ↓
L0.1 lossless lexer
      ↓
L0.2 structural parser
      ↓
AstFile
  ├─ Token
  └─ AstGroup
       ├─ opening token
       ├─ nested elements
       └─ closing token

NOT YET:
keywords / declarations / expressions / operators / names / modules
      ↓
HIR / NSIR / types / effects / capabilities
      ↓
NAIR lowering
```

The K1.18 semantic stability manifest remains unchanged. `language.noi_surface` remains
**experimental** and `tooling.compiler_frontend` remains **internal**.

## 2. Structural grammar

The complete L0.2 structural grammar is deliberately small:

```text
File      := Element* EOF
Element   := non-delimiter Token | Group
Group     := '(' Element* ')'
          | '[' Element* ']'
          | '{' Element* '}'
```

Whitespace and comments are ordinary lossless token elements. Quoted text is one L0.1 token, so
characters that look like delimiters inside quoted text do not participate in L0.2 grouping.
Likewise, delimiter characters inside comments are opaque to the parser.

## 3. No semantic delimiter meaning

L0.2 assigns only a delimiter family:

```text
Parenthesis
Bracket
Brace
```

It does **not** claim that:

- `name(...)` is a function call;
- `name[...]` is indexing or a collection;
- `{...}` is a block, object, map or scope;
- `(...)` is a tuple;
- `[...]` is an array;
- any delimiter creates ownership, lifetime, transaction or capability scope.

Those meanings require later language design and semantic analysis.

## 4. No keyword freeze

Every L0.1 `Identifier` remains an identifier in L0.2. Text such as:

```text
if let fn module import effect capability async match
```

receives no special parser classification.

L0.3 or later may introduce contextual/soft syntax after names and modules are designed. L0.2 must
not force that future choice by embedding a keyword table.

## 5. No operator or precedence freeze

L0.1 punctuation remains single-character punctuation unless the character is one of the six group
delimiters. Therefore:

```text
->  =>  ==  !=  &&  ||  ::  ??
```

remain separate punctuation tokens. L0.2 has no Pratt parser, precedence table, associativity table
or maximal-munch operator table.

This intentionally defers expression grammar until NORDOI can design it together with type/effect
semantics rather than inheriting precedence debt from existing languages.

## 6. Lossless AST law

`AstFile` contains every non-EOF L0.1 token exactly once.

For a group, the AST stores:

```text
opening token
nested elements
closing token
```

The file stores the exact EOF token separately. A depth-first source-order traversal over AST token
leaves reconstructs the original accepted source byte-for-byte.

L0.2 therefore does not require a second source scan to preserve comments or formatting.

## 7. Span law

- `AstFile::span()` is exactly `SourceText::full_span()`.
- `AstGroup::span()` begins at the opening delimiter and ends immediately after the matching closing
  delimiter.
- `AstElement::span()` is either the exact token span or exact group span.
- all spans retain the original `SourceId`.

No L0.2 node synthesizes source positions or normalized text.

## 8. Deterministic grouping

Matching is exact:

```text
( ↔ )
[ ↔ ]
{ ↔ }
```

The parser fails closed on:

- a closing delimiter at root level;
- a mismatched closer;
- an opener left unclosed at EOF;
- lexical failure from L0.1;
- structural nesting deeper than the certified L0.2 limit.

No recovery AST is published after these failures in L0.2. Error recovery for IDE/incremental use is
a later tooling concern and must not weaken the fail-closed compiler path.

## 9. Iterative parser and nesting bound

L0.2 uses an explicit parser frame stack rather than recursive descent for delimiter grouping.

The maximum open-group depth is:

```text
MAX_PARSE_NESTING = 256
```

Depth 256 is accepted; the 257th simultaneously open group is rejected before it can cause
unbounded native call-stack growth.

This limit is a frontend resource-safety rule, not a semantic rule of the certified K1.18 kernel.
A later compiler profile may version it explicitly if required.

## 10. Complexity

For `n` L0.1 tokens and nesting depth `d`:

```text
time              O(n)
parser frame space O(d), d <= 256
AST storage        O(n)
external crates    0
```

The parser performs no identifier interning, string decoding, numeric conversion, name resolution,
type inference, effect inference or NAIR construction.

## 11. Error model

L0.2 introduces `ParseError` with explicit structural failures:

```text
Lex(...)
UnexpectedClosingDelimiter
MismatchedClosingDelimiter
UnclosedDelimiter
NestingLimitExceeded
MissingEof
MultipleEof
TokenAfterEof
```

The final three protect parser/token-stream invariants even though the public `Parser::new` path
currently obtains tokens directly from the certified L0.1 lexer.

Each structural error exposes a primary source span. Mismatch diagnostics additionally preserve the
opening and closing spans, allowing later tooling to render two-location diagnostics without changing
parser semantics.

## 12. Experimental AST API

L0.2 exposes:

```rust
Delimiter
AstFile
AstElement
AstGroup
Parser
parse(...)
ParseError
ParseResult
MAX_PARSE_NESTING
```

The AST is explicitly experimental. Its Rust API is not a certified kernel surface and may evolve
before a public compiler/tooling API is frozen.

## 13. Security properties

L0.2 inherits all L0.1 lexical exclusions, including NUL and bidi-control rejection. In addition:

- delimiter nesting cannot recurse the native parser stack;
- nesting is explicitly bounded;
- comments and quoted text cannot smuggle executable structural delimiters;
- malformed grouping publishes no partial AST;
- the parser does not execute macros, imports, filesystem access, network access or ambient effects;
- the parser introduces no new dependencies or code-generation hooks.

## 14. Non-goals

L0.2 does not define:

- final `.noi` grammar;
- declaration forms;
- expressions;
- statements;
- keyword set;
- operator set/precedence/associativity;
- modules/imports;
- name binding or symbol identity;
- Unicode identifier expansion;
- numeric or string value semantics;
- macros;
- attributes/annotations;
- AST error recovery;
- incremental parsing;
- green/red syntax trees;
- HIR/NSIR;
- type/effect/capability source semantics;
- compiler-to-NAIR lowering.

## 15. Required tests

L0.2 must test at least:

- empty source + exact EOF;
- all three delimiter families;
- nested group structure;
- exact group spans;
- byte-for-byte AST reconstruction;
- trivia preservation;
- keyword neutrality;
- operator neutrality;
- comments and quoted text hiding delimiter-looking characters;
- unexpected closing delimiter rejection;
- mismatched delimiter rejection;
- unclosed delimiter rejection;
- deepest unclosed group reporting;
- propagation of L0.1 lexical failures;
- acceptance at depth 256;
- rejection at depth 257;
- deterministic repeated parsing;
- equivalence of `Parser` and `parse(...)` entry points.

All L0.1 lexer tests and all K1.18 regression tests remain mandatory.

## 16. Certification

L0.2 is accepted only after:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

passes locally, GitHub CI is green for the exact L0.2 commit on all required jobs, and an annotated
`l0.2` tag is pushed.

No L0.3 milestone may be treated as certified before that gate completes.
