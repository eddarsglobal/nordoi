# NORDOI L0.2 — Structural Parser & Experimental AST

L0.2 continues NORDOI's language track on top of the **certified K1.18 kernel** and the **certified
L0.1 lexical foundation**. It adds the first parser while deliberately refusing to freeze the final
`.noi` grammar.

```text
K1.18 certified semantic floor
        ↓
L0.1 certified SourceText / SourceSpan / lossless Lexer
        ↓
L0.2
iterative structural Parser → lossless experimental AST
        ↓
future L0.3 names / modules
        ↓
future C0.x semantic compiler / HIR-NSIR
        ↓
NAIR 0.6+ → NAM/runtime
```

## What L0.2 adds

The `frontend` module now additionally provides:

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

The parser recognizes only exact balanced grouping:

```text
(...)
[...]
{...}
```

Everything else remains represented by the existing L0.1 tokens.

## Lossless structural AST

A successful AST preserves every non-EOF token exactly once, including whitespace and comments.
Groups store their opening token, nested elements and closing token. A source-order traversal can
therefore reconstruct the accepted `.noi` source byte-for-byte.

The EOF token is retained explicitly on `AstFile`.

## What L0.2 deliberately does not freeze

L0.2 still does **not** define:

- final `.noi` grammar;
- keywords;
- declarations or statements;
- expression syntax;
- multi-character operators, precedence or associativity;
- modules/imports;
- name binding;
- numeric value/type semantics;
- final string escape semantics;
- Unicode identifier policy;
- HIR/NSIR;
- type/effect/capability source syntax;
- lowering to NAIR.

Text such as `if`, `let`, `fn`, `module`, `effect` and `capability` remains an ordinary
`Identifier`. Punctuation such as `->`, `=>`, `==` and `::` remains a sequence of single-character
tokens unless a character is one of the six group delimiters.

## Bounded iterative parsing

The structural parser does not recurse through the native call stack. It uses explicit parse frames
and enforces:

```text
MAX_PARSE_NESTING = 256
```

Depth 256 is accepted; depth 257 fails closed.

This prevents hostile or accidental deep nesting from turning the first `.noi` parser into a native
stack-exhaustion surface.

## Structural failures

L0.2 rejects:

```text
unexpected root closer
mismatched closer
unclosed opener
nesting above the limit
any inherited L0.1 lexical/security failure
```

No partial compiler AST is published after failure.

## Security inherited from L0.1

The L0.1 lexical profile remains intact:

- executable-code identifiers are ASCII-only for now;
- Unicode remains available in comments/quoted text;
- NUL is rejected everywhere;
- bidi controls are rejected everywhere;
- non-ASCII executable whitespace/identifier characters remain fail-closed.

Delimiter-looking characters inside comments or quoted text are opaque to L0.2 and cannot alter the
structural tree.

## Kernel compatibility

L0.2 changes no certified kernel semantics:

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The Cargo package remains `nordoi_kernel` version `1.18.0` because L0.2 is a frontend-track milestone,
not a K-series kernel semantic version bump.

## Design intelligence

The parser architecture tradeoffs are recorded in:

```text
research/STRUCTURAL_PARSER_INTELLIGENCE_0_2.md
```

The normative candidate design is:

```text
docs/NOI_STRUCTURAL_PARSER_SPEC_0_2.md
```

L0.1's lexical spec and intelligence record remain present and unchanged in role.

## Tests

`tests/frontend_parser.rs` covers lossless AST reconstruction, all delimiter families, nesting,
spans, trivia, keyword/operator neutrality, delimiter opacity inside strings/comments, structural
failure diagnostics, inherited lexer failures, depth bounds and deterministic parsing.

All L0.1 lexer tests and all prior K1.18 regression tests remain mandatory.

## Release Gate

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

L0.2 is a candidate until the local gate passes, GitHub CI is green for the exact L0.2 commit on all
required platforms and an annotated `l0.2` tag is pushed.

## Next architectural boundary

After L0.2 certification, the intended next language milestone is **L0.3 — Names & Modules**. L0.3
may begin assigning carefully scoped source meaning to identifiers and compilation units, while
K1.18 and NAIR 0.6 remain unchanged unless that work discovers a real certified-kernel blocker.
