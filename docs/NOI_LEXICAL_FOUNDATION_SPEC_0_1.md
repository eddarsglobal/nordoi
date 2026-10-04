# NORDOI `.noi` Lexical Foundation 0.1

**Milestone:** L0.1  
**Status:** Candidate until local Release Gate + GitHub CI + annotated `l0.1` tag  
**Certified kernel baseline:** K1.18  
**NAIR:** 0.6 unchanged  
**Runtime checkpoint:** `NDRTSM01/1.1` unchanged

L0.1 is the first language-track milestone above the certified K1.18 semantic floor. It introduces
source identity, byte-accurate spans, physical source positions and a lossless experimental lexer.
It intentionally does **not** define parser, AST, HIR, name resolution, type semantics, effect
semantics, capability semantics or lowering to NAIR.

## 1. Boundary

```text
UTF-8 `.noi` source
      ↓
SourceText + SourceId
      ↓
ByteOffset + SourceSpan + SourcePosition
      ↓
lossless L0.1 lexer
      ↓
experimental token stream

NOT YET:
parser → AST → semantic analysis → HIR/NSIR → NAIR
```

The K1.18 stability manifest remains unchanged. `language.noi_surface` remains experimental and
`tooling.compiler_frontend` remains internal. L0.1 therefore cannot silently promote source syntax
into certified kernel law.

## 2. Source identity

Every source unit has an explicit compiler/caller-issued `SourceId`. IDs must be unique within one
frontend session. `SourceText::slice` rejects a span whose `SourceId` differs from its own. L0.1 does
not yet define a multi-file `SourceMap`; that registry belongs to a later frontend milestone.

`SourceText` preserves source bytes exactly. It does not normalize line endings or Unicode text.
Rust `String`/`str` guarantees valid UTF-8 before the L0.1 source layer sees the text.

The maximum accepted source size is `u32::MAX - 1` bytes. This keeps both byte offsets and the worst-case physical-line count representable as `u32`.

## 3. Span law

A `SourceSpan` is a half-open byte interval:

```text
[start, end)
```

It carries:

```text
source identity
start byte offset
end byte offset
```

Public span creation validates:

- source offsets are within the source;
- `start <= end`;
- both offsets are UTF-8 scalar boundaries.

A token span therefore always slices the exact original source text.

## 4. Source positions

`SourcePosition` exposes:

- source identity;
- byte offset;
- one-based physical line;
- one-based Unicode-scalar column.

Physical line recognition accepts LF, CRLF and CR without rewriting the source. CRLF counts as one
line boundary.

L0.1 deliberately defines scalar columns rather than terminal/display columns. Grapheme and visual
column policy belongs to future diagnostics/tooling work.

## 5. Lossless token classes

L0.1 emits only these experimental lexical classes:

```text
Identifier
NumericCandidate
QuotedText
Punctuation(char)
Whitespace
LineComment
BlockComment
Eof
```

The non-EOF token spans cover every accepted source byte exactly once. Concatenating their source
slices reconstructs the original source byte-for-byte.

### 5.1 No keyword freeze

L0.1 has no keyword table. Text such as `if`, `let`, `effect` or `capability` is still an
`Identifier`.

Keywords are parser/language-design concerns and must not be frozen by the first lexer.

### 5.2 No operator freeze

Visible ASCII punctuation is emitted one character at a time. For example:

```text
->   =>   ==   &&
```

remain sequences of `Punctuation(char)` tokens. L0.1 therefore does not choose a final operator set,
precedence table or maximal-munch operator law.

### 5.3 Numeric candidates are not numeric semantics

A numeric candidate begins with an ASCII decimal digit and continues through ASCII letters, decimal
digits or `_`.

This allows future literal design to inspect forms such as:

```text
42
0xff
12_345
1e9
```

without L0.1 claiming their value, base, type, overflow behavior or validity.

### 5.4 Quoted text is not string semantics

Double quotes delimit `QuotedText`. Backslash protects the immediately following scalar from acting
as the closing quote. L0.1 preserves the raw spelling and does not interpret escape meaning.

Raw CR/LF inside quoted text is rejected. Multiline strings, interpolation, raw strings and escape
semantics remain future language work.

## 6. Comments and trivia

L0.1 recognizes:

```text
// line comment
/* block comment */
```

Block comments may nest. Whitespace and comments are preserved as trivia tokens rather than erased,
so future formatters, diagnostics, IDEs and source-to-source tools can remain lossless.

Line comments exclude their terminating line break; the line break remains part of a following
`Whitespace` token.

## 7. Security-first character policy

L0.1 uses a deliberately narrow executable-code character profile.

Outside quoted text and comments:

- identifiers are ASCII letters/underscore followed by ASCII letters/digits/underscore;
- whitespace is only space, tab, LF and CR;
- punctuation is visible ASCII punctuation;
- non-ASCII identifier characters are rejected;
- non-ASCII whitespace is rejected;
- other non-ASCII code characters are rejected.

Unicode remains allowed in quoted text and comments, subject to the global exclusions below.

### 7.1 Global exclusions

The following are rejected everywhere, including comments and quoted text:

- NUL (`U+0000`);
- Arabic Letter Mark (`U+061C`);
- Left-to-Right Mark / Right-to-Left Mark (`U+200E`, `U+200F`);
- bidi embeddings/overrides (`U+202A`..`U+202E`);
- bidi isolates (`U+2066`..`U+2069`).

This prevents the first `.noi` lexer from silently accepting source-order controls that can make
visible code differ from logical code order.

### 7.2 Why Unicode identifiers are deferred

NORDOI is not declaring ASCII identifiers as the final language policy. Unicode identifiers require
an explicit, versioned profile covering at least:

- Unicode version;
- XID start/continue rules;
- normalization policy;
- script-mixing policy;
- confusable detection;
- bidi handling;
- migration when Unicode data changes.

Until that profile exists, L0.1 fails closed rather than adopting a permissive rule accidentally.

## 8. Determinism and lightness

The lexer:

- has no external crate dependency;
- does not allocate lexeme strings;
- stores token identity as kind + source span;
- operates linearly over source text;
- preserves source spelling instead of canonicalizing it;
- performs no semantic, type, effect or capability work.

`SourceText` stores one `u32` line-start offset per physical line to support deterministic diagnostics
without rescanning the entire source for every position lookup.

## 9. L0.1 non-goals

L0.1 does not define:

- final `.noi` grammar;
- keyword set;
- operator set or precedence;
- numeric value semantics;
- string escape semantics;
- parser or AST;
- HIR/NSIR;
- symbol/name resolution;
- type system;
- ownership/effect/capability source syntax;
- macros;
- modules/imports;
- compiler-to-NAIR lowering;
- Unicode identifier acceptance.

## 10. Required tests

L0.1 must test at least:

- source identity and cross-source rejection;
- UTF-8-boundary-safe spans;
- LF/CRLF/CR line positions;
- Unicode scalar columns;
- exact lossless source reconstruction from token spans;
- keyword neutrality;
- single-character punctuation neutrality;
- nested comments;
- unterminated comment/string rejection;
- Unicode acceptance inside comments/quoted text;
- non-ASCII identifier rejection;
- non-ASCII whitespace rejection;
- NUL rejection;
- bidi-control rejection inside code, comments and quoted text;
- exact EOF span.

All previous K1.18 regression tests remain mandatory.

## 11. Certification

L0.1 is accepted only after:

```bash
./scripts/release_gate.sh
```

passes locally, GitHub CI is green on all required jobs, and an annotated `l0.1` tag is pushed.

No L0.2 milestone may be treated as certified before that gate completes.
