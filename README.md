# NORDOI L0.1 — Source Text, Source Spans & Lexer

L0.1 opens NORDOI's language track on top of the **certified K1.18 kernel**. It is the first concrete
`.noi` frontend layer, but it intentionally stops before grammar and semantics.

```text
K1.18 certified semantic floor
        ↓
L0.1
SourceText → SourceSpan → lossless Lexer
        ↓
future L0.x parser / language surface
        ↓
future C0.x semantic compiler / HIR
        ↓
NAIR 0.6+ → NAM/runtime
```

## What L0.1 adds

The new `frontend` module provides:

```rust
SourceId
ByteOffset
SourceSpan
SourcePosition
SourceText
TokenKind
Token
Lexer
lex(...)
```

The token stream is **lossless**: every accepted source byte is covered exactly once by non-EOF token
spans, including whitespace and comments.

## What L0.1 deliberately does not freeze

L0.1 does **not** define:

- final `.noi` grammar;
- keywords;
- multi-character operators or precedence;
- numeric value/type semantics;
- final string escape semantics;
- AST;
- HIR/NSIR;
- type/effect/capability source syntax;
- lowering to NAIR.

The K1.18 stability manifest remains unchanged: `.noi` is still experimental and the compiler
frontend is still internal.

## Security-first lexical bootstrap

Executable-code identifiers are ASCII-only in L0.1. Unicode remains available inside comments and
quoted text, but NUL and Unicode bidi control characters are rejected everywhere.

This is a temporary fail-closed policy, not a declaration that final NORDOI identifiers will be
ASCII-only. Unicode identifiers will require a separately reviewed, versioned profile covering UAX
#31 identifier rules, normalization and UTS #39 security/confusable policy.

## Lossless trivia

L0.1 preserves:

```text
Whitespace
// line comments
/* nested block comments */
```

as tokens with exact source spans. Future parsers may ignore trivia while formatters, diagnostics and
IDEs can retain it without re-reading or guessing the source structure.

## Source positions

Spans use half-open UTF-8 byte offsets and are source-specific. Position lookup exposes one-based
physical line and Unicode-scalar column values. LF, CRLF and CR are recognized without source
normalization.

## Kernel compatibility

L0.1 changes no certified kernel semantics:

```text
Kernel baseline          K1.18
NAIR                     0.6
Runtime checkpoint       NDRTSM01 / 1.1
Semantic stability map   K1.18 identity unchanged
```

The Cargo package remains `nordoi_kernel` version `1.18.0` because L0.1 is a frontend-track milestone,
not a K-series kernel semantic version bump.

## Design intelligence

Before implementation, L0.1 records the lexer/identifier tradeoffs of Rust, Python, Swift, Go and
Unicode guidance in:

```text
research/LEXICAL_FOUNDATION_INTELLIGENCE_0_1.md
```

The normative candidate design is:

```text
docs/NOI_LEXICAL_FOUNDATION_SPEC_0_1.md
```

## Tests

`tests/frontend_lexer.rs` covers source identity, UTF-8 span safety, physical line mapping, lossless
reconstruction, keyword/operator neutrality, nested comments, quoted text, Unicode containment,
NUL rejection, bidi-control rejection and EOF identity.

All prior K1.18 regression tests remain present.

## Release Gate

Run:

```bash
cargo fmt --all
./scripts/release_gate.sh
```

L0.1 is a candidate until the local gate passes, GitHub CI is green on all required platforms and an
annotated `l0.1` tag is pushed.

## Next architectural boundary

After L0.1 certification, the next language work should remain vertical rather than returning to
runtime-only accumulation. The likely next slices are parser/green-tree work on the L track and the
semantic/HIR boundary on the C track, while K-series work resumes only when those layers discover a
real certified-kernel blocker.
