# NORDOI CLI / Tooling Specification 0.1

Status: **T0.1 candidate**

This specification defines the first executable developer-tooling surface for `.noi` source. It is
an inspection boundary only. T0.1 does not define compilation, execution, HIR/NSIR, NAIR lowering,
package resolution, imports, or filesystem-derived module identity.

## 1. Command surface

The binary name is:

```text
nordoi
```

T0.1 defines exactly three source inspection commands:

```text
nordoi lex <path|->
nordoi parse <path|->
nordoi module <path|->
```

and two informational switches:

```text
nordoi --help
nordoi --version
```

`-` means UTF-8 source is read from standard input and is named `<stdin>` for diagnostics.

## 2. Layer ownership

The commands are intentionally thin adapters over already-defined frontend layers:

```text
lex     -> L0.1 SourceText + Lexer
parse   -> L0.2 structural Parser + experimental AST
module  -> L0.3 ModuleAnalyzer
```

The tool MUST NOT reinterpret tokens, invent keywords, infer module identity from a pathname, or
lower source to NAIR.

## 3. Determinism

For equal source bytes, equal command, and equal source display name, successful T0.1 output MUST
be byte-for-byte deterministic.

The tool emits no implicit color, timestamps, host paths beyond the user-supplied source name,
random identifiers, locale-dependent text, network data, or wall-clock data.

T0.1 textual output is a human/tooling inspection format, not a frozen serialization protocol.
Future tooling versions may version a machine-readable format separately rather than silently
turning this text into compiler ABI.

## 4. Input boundary

T0.1 accepts UTF-8 only.

The tooling boundary imposes:

```text
MAX_TOOL_INPUT_BYTES = 16 MiB
```

This limit is intentionally much smaller than the underlying `SourceText` representational maximum.
It bounds whole-file CLI allocation while the frontend remains experimental. A future streaming or
workspace frontend may define a different governed input strategy.

Files are never interpreted as module identity. Standard input behaves identically to a named file
except for its diagnostic display name.

## 5. Output

### 5.1 `lex`

`lex` reports source size, token count, token kind, exact byte span, physical line/column span and an
escaped lossless token slice. EOF is explicit.

### 5.2 `parse`

`parse` reports the L0.2 file span and recursively prints the lossless structural tree. Parenthesis,
bracket and brace groups remain structural only; the CLI does not label them as calls, indexing,
collections, blocks or other future semantics.

### 5.3 `module`

`module` reports either the L0.3 canonical dotted module identity or `<anonymous>`. It does not use
the filename, directory, repository or current working directory to create identity.

## 6. Diagnostics

Frontend diagnostics use this shape when a source span exists:

```text
error[stage]: "source-name":line:column: message
```

The source name and source fragments are escaped so control characters cannot forge terminal lines.
No partial successful AST/module output is published after a frontend failure.

## 7. Exit status

T0.1 reserves:

```text
0  success
2  command-line usage failure
3  input/output or UTF-8 loading failure
4  frontend/source analysis failure
```

These codes are part of the T0.1 command contract.

## 8. Security and robustness

T0.1 inherits all L0.1 lexical protections, including NUL and bidirectional-control rejection, all
L0.2 structural nesting bounds, and all L0.3 name/module bounds.

The tool additionally:

- bounds whole-input reads before frontend publication;
- performs no network access;
- performs no code execution;
- performs no plugin or dynamic-library loading;
- performs no ambient filesystem discovery;
- emits no ANSI control sequences by default;
- avoids shell invocation.

## 9. Compatibility floor

```text
Kernel baseline          K1.18
NAIR                     0.6
Lexical foundation       L0.1
Structural parser        L0.2
Names & modules          L0.3
Tooling                  T0.1 candidate
```

T0.1 MUST NOT modify certified K1.18/NAIR semantics.

## 10. Explicit non-goals

T0.1 does not define:

- compile/build/run commands;
- semantic validation beyond L0.3;
- imports or packages;
- name resolution;
- types or effects;
- HIR/NSIR;
- NAIR generation;
- formatter behavior;
- language server behavior;
- project/workspace files;
- stable JSON or binary tooling protocol.
