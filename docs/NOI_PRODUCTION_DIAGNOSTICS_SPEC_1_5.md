# NOI Production Diagnostics Specification 1.5

## Status

Candidate specification for NORDOI V1.5 — Production Diagnostics & Developer Experience.

## Scope

V1.5 defines a deterministic project-check diagnostic boundary over the V1.4 project/build system. It adds no language authority, runtime I/O, package-network behavior, dynamic loading or new NAIR format.

## Command

```text
nordoi check <project-root> [--json]
```

`check` reads `NORDOI.toml`, resolves the V1.3 module graph, performs the V1.4 compile/lowering validation, and publishes no build artifact or lock file.

## Stable schema

Machine-readable output schema identifier:

```text
nordoi.diagnostic.v1
```

An error object contains exactly the semantic fields:

- `schema`;
- `status`;
- `code`;
- `stage`;
- `message`;
- `location` or `null`;
- `importTrace`.

Success contains project identity, module/import counts, NAIR minor/instruction count and an empty diagnostic array.

## Diagnostic code law

Codes are stable semantic identifiers. Human wording is not the compatibility boundary.

| Code | Meaning |
| --- | --- |
| NDX1001 | invalid project manifest |
| NDX2001 | project/source I/O failure |
| NDX2002 | source/lexical failure |
| NDX2003 | module declaration failure |
| NDX2004 | imported module unavailable |
| NDX2005 | cyclic import graph |
| NDX2101 | module graph resolution failure |
| NDX2201 | semantic failure after module composition |
| NDX3001 | lowering/NAIR validation failure |
| NDX4001 | package validation failure |

No code may silently change meaning after certification.

## Source location law

A diagnostic may claim `file:line:column` only when a real certified `SourceSpan` can be projected back to that exact `SourceText`. If no trustworthy span exists, location is absent or file-only. V1.5 forbids guessed source coordinates.

## Import trace law

Import traces are compiler evidence, not runtime state. Missing imports use the deterministic discovery parent chain. Cycles use a deterministic closed cycle trace. Adjacent duplicate trace elements are canonicalized away.

## Determinism

Equal canonical project sources must produce equal success JSON. Equal error inputs must produce stable code, stage and import-trace ordering. Absolute temporary paths may differ only where the user supplied filesystem path itself is diagnostic evidence.

## Authority

`check` performs compiler-side source reads only. It creates no project artifact, lock file, runtime filesystem capability, network dependency request or execution authority.

## Frozen boundaries

V1.5 preserves the public version string and all certified NAIR minors through 0.14.
