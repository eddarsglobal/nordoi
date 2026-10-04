# NORDOI Structural Parser Intelligence 0.2

**Milestone:** L0.2  
**Purpose:** record parser architecture tradeoffs before NORDOI commits to source grammar.

This note follows `LANGUAGE_INTELLIGENCE_CHARTER.md`: mechanisms are evaluated for the problem they
solve, their historical debt, security/performance consequences, human cognitive cost and AI
code-generation cost. The goal is not to imitate another language.

## 1. Problem being solved now

L0.1 can identify exact source spans and lossless lexical tokens, but it cannot answer even the most
basic structural questions:

```text
which closing delimiter belongs to which opener?
is the source structurally balanced?
which tokens are nested inside which group?
```

A conventional language project often answers these questions while simultaneously choosing final
keywords, declarations, statements, expressions and operator precedence. For NORDOI that would be a
premature coupling: the language has not yet designed names/modules, typed semantic IR or effect
syntax.

L0.2 therefore needs a parser that creates useful structure **without accidentally becoming the final
language grammar**.

## 2. Recursive-descent parsers

### Original problem

Hand-written recursive descent maps grammar productions directly to functions. It is common in
systems languages because it is explicit, debuggable and supports precise diagnostics.

### Strengths

- straightforward control flow;
- excellent local diagnostics;
- no parser-generator runtime;
- easy contextual grammar decisions;
- often low overhead.

### Structural weakness for NORDOI L0.2

A recursive function per nested syntactic construct can consume the native call stack on adversarial
input. More importantly, building a full recursive-descent grammar now would force NORDOI to choose
productions that are not ready to be frozen.

### NORDOI decision

Use the explicitness of hand-written parsing, but **not recursive call-stack nesting** and not a full
language grammar at L0.2.

## 3. Parser generators (LR/LALR/PEG families)

### Original problem

Parser generators turn a declarative grammar into a recognizer, reducing manual parser mechanics and
often providing formal conflict analysis.

### Strengths

- grammar is centralized;
- mature theory exists for ambiguity/conflict detection;
- can be efficient and deterministic;
- useful when the grammar is mature.

### Structural weaknesses at NORDOI's current stage

- a grammar file creates pressure to freeze syntax early;
- generated artifacts/tooling add build dependencies;
- error recovery can be opaque;
- context-sensitive evolution can become generator-specific debt;
- AI agents may optimize for making the grammar compile rather than questioning whether the grammar
  itself is correct.

### NORDOI decision

Do not introduce a parser generator in L0.2. Re-evaluate only after source grammar has demonstrated
stability and there is a measurable reason to replace the hand-written frontend.

## 4. Pratt / precedence-climbing expression parsers

### Original problem

Pratt parsing and precedence climbing make expression parsing compact and extensible across many
operators.

### Strengths

- small implementation;
- flexible prefix/infix/postfix handling;
- good performance;
- easy to extend an established operator table.

### Historical debt risk

The technique makes it technically easy to add operators, which can hide the harder language-design
question: whether those operators and their precedence should exist at all. Existing languages carry
substantial cognitive debt from precedence tables, overloaded punctuation and context-dependent
operators.

### NORDOI decision

L0.2 has **no expression parser and no precedence table**. Expression design belongs with semantic
IR, types and effects so surface convenience cannot silently dictate semantic architecture.

## 5. Lossless concrete syntax / green-tree approaches

### Original problem

Modern IDE-oriented frontends need exact source preservation, resilient trees, incremental updates and
stable node identity. Lossless concrete syntax trees preserve trivia and punctuation rather than
throwing them away after parsing.

### Strengths

- formatting and comments survive exactly;
- diagnostics can refer to original spelling;
- refactoring/source-to-source tooling is possible;
- incremental parsing can be layered later;
- syntax and semantics remain separable.

### Costs

- more nodes/tokens than a semantic-only AST;
- green/red tree architectures can add implementation complexity;
- interning and incremental identity policies can become a subsystem of their own.

### NORDOI decision

Adopt the **lossless principle now**, but not a full green/red incremental-tree engine. L0.2 stores
every token exactly once inside a minimal group tree. This keeps the path open to a future green tree
without paying its complexity before tooling requires it.

## 6. Token-tree approaches

### Original problem

Token trees preserve balanced delimiter structure while deferring most grammar interpretation. This
is particularly useful in macro systems and staged parsing.

### Strengths

- delimiter balance is explicit;
- syntax remains largely uninterpreted;
- future phases can assign meaning contextually;
- representation is naturally lossless when trivia/tokens are retained.

### Weaknesses

- not sufficient by itself for name resolution, types or executable semantics;
- unrestricted macro-like token manipulation can create hygiene and security complexity.

### NORDOI decision

The L0.2 AST deliberately resembles the safe structural subset of a token-tree model: tokens plus
balanced groups. **No macro execution or token rewriting is introduced.**

## 7. Python-style indentation sensitivity

### Original problem

Indentation-sensitive grammars reduce punctuation and make visual block structure executable.

### Strengths

- concise source;
- visual structure and parser structure can align;
- fewer braces.

### Risks for NORDOI

- tabs/spaces and editor normalization become semantic concerns;
- copy/paste and generated code must preserve invisible layout precisely;
- formatters have less freedom;
- AI output failures can become visually subtle.

### NORDOI decision

L0.2 does not make whitespace structurally semantic. This does not permanently prohibit an
indentation-aware NORDOI surface, but such a decision needs dedicated evidence and security analysis.

## 8. Rust/Swift/Go-style brace structure

Many mainstream compiled languages use braces as explicit block delimiters and recursive-descent or
hybrid handwritten parsers. Explicit delimiters are robust under formatting and machine generation,
but those languages also attach a large amount of declaration/expression meaning to punctuation and
keywords.

NORDOI takes only the low-level lesson that explicit delimiters are easy to validate. L0.2 does not
inherit their block, statement, semicolon or expression rules.

## 9. Security analysis

### Stack exhaustion

A naive recursive parser can overflow its native call stack on deeply nested source. L0.2 uses an
explicit frame vector and rejects depth above 256.

### Delimiter smuggling

Delimiter-looking characters in comments or strings must not alter executable structure. L0.1 emits
those regions as single tokens, so L0.2 never scans their contents for delimiters.

### Partial publication

Malformed structure returns an error and no `AstFile`. L0.2 does not expose a partially accepted
compiler AST after mismatch/unclosed failures.

### Bidi/NUL

L0.2 inherits L0.1's global rejection before structural parsing begins.

### Dependency/supply-chain surface

No parser-generator or third-party parser crate is introduced.

## 10. Performance analysis

The parser performs one pass over the L0.1 tokens. Opening delimiters push one frame; closing
delimiters pop one frame. Every accepted token enters the AST exactly once.

```text
Time: O(tokens)
Parser auxiliary space: O(nesting), bounded at 256
AST space: O(tokens)
```

No lexeme strings are copied by the parser. Existing tokens carry spans into the source text.

## 11. Human cognitive cost

A full expression/declaration grammar would give humans more immediately writable examples, but that
convenience would be purchased with early compatibility debt. L0.2 instead makes one concept obvious:
**structural grouping**.

This is intentionally less exciting but easier to reason about, test and later replace.

## 12. AI generation cost

AI systems are especially sensitive to large implicit syntax rule sets: precedence, context-sensitive
keywords, optional terminators and overloaded punctuation create many superficially plausible but
invalid combinations.

A structural parser with explicit delimiter balance gives later AI-facing tooling a deterministic
first validation layer. It also avoids training future NORDOI agents to assume that familiar syntax
from Rust/JavaScript/Python already has NORDOI meaning.

## 13. What NORDOI makes impossible by construction in L0.2

Within the L0.2 boundary:

- a parsed group cannot have the wrong closing delimiter;
- a root-level closer cannot be silently ignored;
- an unclosed opener cannot produce a successful AST;
- group nesting cannot consume unbounded native call stack;
- comments/quoted text cannot inject structural delimiter nodes;
- keywords cannot accidentally be frozen by parser classification;
- multi-character operators cannot accidentally be frozen by parser matching;
- malformed lexical input cannot bypass the L0.1 security profile;
- a successful AST cannot lose accepted source bytes.

## 14. Deferred questions

L0.2 intentionally leaves open:

- final declaration syntax;
- contextual vs reserved keywords;
- expression model;
- whether NORDOI should have traditional operators at all;
- statement terminators;
- module/import syntax;
- attributes/metadata syntax;
- error-recovery trees for IDEs;
- incremental/green-tree representation;
- macro/metaprogramming model;
- Unicode identifier profile.

Those questions must be answered by later milestones with their own intelligence records rather than
being smuggled into a delimiter parser.

## 15. Decision

L0.2 adopts a **lossless, iterative, bounded structural parser** and an **experimental group AST**.

It is the smallest parser layer that gives NORDOI real syntax structure while preserving freedom for
L0.3 names/modules and the later typed semantic IR track.
