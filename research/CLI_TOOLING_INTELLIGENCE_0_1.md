# CLI Tooling Intelligence 0.1

Status: research record for **NORDOI T0.1**.

## Question

What is the smallest command-line boundary that makes the new `.noi` frontend observable and useful
without prematurely turning experimental syntax into compiler semantics?

## Decision

T0.1 is an inspection-only executable with three vertical slices:

```text
source -> lex
source -> parse
source -> module
```

The binary deliberately mirrors the certified frontend layering rather than adding a fourth parser,
a command-specific grammar, or hidden source transformations.

## Why not a compiler command yet?

A `compile` or `run` command would imply a semantic lowering contract that does not yet exist. NORDOI
has not introduced HIR/NSIR, typed semantics, effect syntax, source capabilities, imports, or the
source-to-NAIR bridge. Pretending otherwise would couple the experimental surface directly to NAIR
0.6 and make later semantic corrections unnecessarily expensive.

## Why text inspection first?

The first tool should make invariants visible:

- exact token boundaries;
- lossless trivia retention;
- structural grouping;
- source spans;
- contextual module identity;
- deterministic diagnostics.

These are useful for language development, tests, editors and future tooling without claiming
execution semantics.

## Deterministic output

T0.1 avoids color, time, locale, network and ambient repository information. Equal inputs produce
equal output. This is useful for golden testing while still explicitly refusing to freeze the
human-readable output as a long-term serialization ABI.

A future structured protocol should be separately versioned instead of relying on accidental parsing
of T0.1 console text.

## Input governance

Although `SourceText` can represent much larger units, a whole-file CLI should not casually allocate
multi-gigabyte inputs. T0.1 therefore uses a 16 MiB tool boundary. This is a tooling policy, not a
language-source semantic limit.

A future workspace compiler can replace this with mapped, incremental or streaming source storage
under an explicit resource-governance design.

## Exit codes

The first CLI distinguishes four operational classes:

```text
0 success
2 usage
3 loading / I/O
4 frontend analysis
```

This is enough for scripts and CI without inventing dozens of unstable semantic categories.

## Path independence

The `module` command proves a central L0.3 invariant: a path is an input locator, not language
identity. `foo/bar.noi` does not become `foo.bar`; only source syntax can declare a module.

This prevents build-system convention from silently entering core language meaning.

## Security posture

The CLI is intentionally boring:

- no network;
- no shell;
- no dynamic plugins;
- no code execution;
- bounded reads;
- UTF-8 validation;
- escaped output fragments;
- inherited lexer/parser/module fail-closed behavior.

That makes T0.1 a narrow observer over the language frontend instead of a new authority boundary.

## Comparative lessons

Mature language ecosystems usually separate frontend inspection from execution through tools such as
token dumps, parser dumps, compiler `--emit` modes or syntax-tree inspection. The transferable lesson
is not their exact command spelling; it is that observability should precede a large build system and
that machine protocols should be explicitly versioned.

NORDOI keeps this lesson while avoiding dependency on any specific compiler framework or provider.

## Forward path

T0.1 enables future work to consume one executable frontend entry point. The next semantic milestone
can introduce a typed semantic representation without changing K1.18 or pretending the CLI itself is
the language.

Expected trajectory:

```text
T0.1 inspection CLI
  -> C0.1 typed semantic IR / HIR-NSIR
  -> L0.4 types + effects
  -> C0.2 semantic IR -> NAIR
  -> V0.1 first executable .noi program
```
