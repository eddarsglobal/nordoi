# NORDOI Lexical Foundation Intelligence 0.1

**Milestone:** L0.1  
**Purpose:** satisfy the Language Intelligence Charter before NORDOI adopts its first source lexer.

NORDOI must not copy a historical lexer merely because it is familiar. This note compares relevant
language families and Unicode guidance against NORDOI's goals: simple surface, deterministic
machine meaning, security by construction, AI/SI generation clarity, lightness and freedom from
legacy debt.

## 1. Problems a lexer must solve

A lexer must at minimum answer:

1. how source identity and byte ranges are represented;
2. what characters can form tokens;
3. how trivia/comments are preserved or discarded;
4. how strings and delimiters terminate;
5. how ambiguities are split into tokens;
6. how invalid/invisible characters fail;
7. how diagnostics point back to exact source;
8. how tooling can reconstruct source without guessing.

For NORDOI, the lexer must additionally avoid freezing semantic choices before the parser and
semantic compiler exist.

## 2. Rust family

Rust uses explicit lexical token classes, Unicode XID-style identifiers, line/block comments,
nested block comments, rich literal forms and punctuation/operator tokens. Its reference separates
lexical grammar from later language semantics.

### Strengths

- rigorous reference grammar;
- deterministic tokenization;
- nested block comments are tooling-friendly;
- source language is UTF-8;
- raw literal mechanisms reduce escaping pressure.

### Structural costs / debt risks for NORDOI

- a large literal/operator vocabulary would freeze too much surface too early;
- raw identifiers exist partly to coexist with keyword evolution;
- adopting Unicode identifiers immediately imports Unicode-version, normalization and confusable
  policy questions before NORDOI has designed them.

### NORDOI consequence

Reuse the ideas of precise token spans and nested comments, but do not copy Rust's keyword,
operator or literal inventory into L0.1.

Reference: https://doc.rust-lang.org/reference/tokens.html

## 3. Python family

Python tokenization includes names, numbers, strings, operators/delimiters and significant logical
newline/indentation tokens. Python accepts Unicode names and normalizes identifiers according to its
language rules.

### Strengths

- readable human surface;
- Unicode-aware names;
- lexical rules are well documented;
- indentation carries visible structural meaning.

### Structural costs / debt risks for NORDOI

- indentation-sensitive structure would be a major grammar decision, not a neutral lexer choice;
- lexical encoding compatibility mechanisms reflect historical source-file evolution;
- Unicode identifiers require explicit normalization and security policy.

### NORDOI consequence

Do not introduce INDENT/DEDENT or encoding declarations in L0.1. NORDOI source is already UTF-8 at
this boundary, and structural whitespace must remain an open language-design question.

Reference: https://docs.python.org/3/reference/lexical_analysis.html

## 4. Swift family

Swift uses longest-match tokenization, Unicode-rich identifiers, nested multiline comments, a broad
operator character space and sophisticated string literal forms.

### Strengths

- strong Unicode ergonomics;
- nested comments;
- expressive literal system;
- lexical rules are explicit and toolable.

### Structural costs / debt risks for NORDOI

- operator lexing is context-sensitive enough to carry substantial language complexity;
- permissive identifier forms expand visual ambiguity and security review surface;
- rich string forms are valuable but too large a design commitment for the first lexer.

### NORDOI consequence

Keep nested comments and exact source preservation, but do not adopt custom-operator or raw-identifier
complexity in L0.1.

Reference: https://docs.swift.org/latest/documentation/the-swift-programming-language/lexicalstructure/

## 5. Go family

Go uses a compact lexical vocabulary, Unicode letters in identifiers, line/general comments,
maximal-munch tokenization and a lexer-level semicolon insertion rule.

### Strengths

- deliberately small language surface;
- simple lexical specification;
- predictable tooling;
- UTF-8 and Unicode support.

### Structural costs / debt risks for NORDOI

- semicolon insertion makes line structure part of language semantics;
- non-nested block comments are less convenient for commenting out regions;
- a fixed keyword/operator set would again freeze choices before NORDOI has a parser.

### NORDOI consequence

Adopt compactness, not semicolon insertion. Keep line breaks as trivia until grammar design proves
that they require semantic status.

Reference: https://go.dev/ref/spec

## 6. Unicode UAX #31

Unicode Standard Annex #31 provides recommended profiles for Unicode identifiers and discusses
normalization. It is the right family of rules to consult before NORDOI enables non-ASCII
identifiers.

### Strengths

- standardized identifier properties;
- versioned Unicode basis;
- interoperable guidance for parsers and lexers.

### Cost for NORDOI

Using XID properties alone does not answer the whole security question. NORDOI would still need to
pin a Unicode version and define normalization, script mixing, diagnostics and migration behavior.

Reference: https://www.unicode.org/reports/tr31/

## 7. Unicode UTS #39

Unicode Technical Standard #39 defines security mechanisms for identifiers, including restriction
profiles and confusable detection. It explicitly addresses the fact that visually similar Unicode
strings can be difficult for humans to distinguish.

### Strengths

- security-focused identifier guidance;
- confusable and mixed-script analysis;
- suitable foundation for a future NORDOI Unicode identifier profile.

### Cost for NORDOI

A correct implementation requires versioned Unicode security data and policy. Pulling that machinery
into L0.1 would violate the goal of a small first vertical slice before the identifier policy itself
has been designed.

Reference: https://www.unicode.org/reports/tr39/

## 8. Historical compatibility debt observed

Across mature languages, lexical complexity often grows from:

- new keywords that must coexist with older identifiers;
- multiple string forms added after original escaping proved inconvenient;
- source encoding history;
- operator customization;
- context-sensitive token rules;
- syntax extensions that require lexer exceptions.

NORDOI should not pre-pay this debt before real `.noi` programs demonstrate a need.

## 9. Security consequences

The source layer is a security boundary because invisible or visually reordered characters can make
reviewed code appear different from parsed code.

L0.1 therefore chooses:

- exact UTF-8 byte spans;
- explicit source identity carried by every span;
- NUL rejection;
- bidi-control rejection everywhere;
- ASCII executable identifiers for now;
- only ASCII structural whitespace for now;
- Unicode preserved inside quoted text/comments when it cannot change token structure.

This is not a claim that ASCII-only identifiers are NORDOI's final design. It is a fail-closed
bootstrap policy until a Unicode identifier specification can be reviewed and versioned.

## 10. Performance consequences

A first lexer should remain linear, allocation-light and independent of semantic compilation.

L0.1:

- uses no third-party dependency;
- allocates token records but not lexeme strings;
- keeps line-start offsets for diagnostics;
- returns source spans that borrow lexemes from `SourceText`;
- does not normalize or copy token contents.

Future benchmarks should measure throughput and allocation count before any performance superiority
claim is made.

## 11. Cognitive cost for humans

A programmer should see source boundaries that match tool boundaries. Lossless spans make diagnostics
and IDE selections exact. Preserving comments/trivia avoids formatters having to reconstruct intent.

Deferring keywords/operators also prevents early examples from accidentally becoming permanent
language law.

## 12. Cost for AI/SI generation and verification

AI/SI benefits from:

- deterministic token boundaries;
- exact source spans;
- low lexical ambiguity;
- no context-dependent keyword classification in L0.1;
- explicit rejection of visually dangerous controls;
- token streams that can reconstruct source exactly.

AI/SI does not need an AI model at runtime to interpret these rules.

## 13. What L0.1 makes impossible by construction

L0.1 makes these states unrepresentable as successful lexical output:

- a token span that crosses source identity;
- a public span that splits a UTF-8 scalar;
- a successful token stream containing NUL;
- a successful token stream containing bidi formatting controls;
- accidental acceptance of an unspecified Unicode identifier profile;
- accidental keyword classification;
- accidental multi-character operator commitment;
- silent loss of comments/whitespace from the source token stream;
- raw newline inside the initial quoted-text form;
- unterminated block comment or quoted text accepted as valid tokens.

## 14. L0.1 decision

The first NORDOI lexer is intentionally a **lexical substrate**, not a miniature final language.

```text
exact source
  + exact spans
  + lossless trivia
  + narrow safe token forms
  + fail-closed Unicode code policy
  - no parser commitments
  - no keyword commitments
  - no operator commitments
  - no semantic literal commitments
```

This gives L0.2/C0.1 a deterministic source boundary while preserving NORDOI's freedom to design the
actual language from semantic law rather than historical habit.
