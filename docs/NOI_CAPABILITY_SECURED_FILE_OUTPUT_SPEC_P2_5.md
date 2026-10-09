# NOI Capability-Secured File Output Specification — P2.5

## Status

Candidate specification for NORDOI P2.5.

## Certified baseline

P2.5 is additive above immutable certified milestones:

- Production Profile 1 — `v1.7`;
- P2.1 Capability-Secured Observable I/O — `p2.1`;
- P2.2 Dynamic Capability-Secured Observable Output — `p2.2`;
- P2.3 Structured Dynamic Text Output — `p2.3`;
- P2.4 Multi-Segment Structured Output — `p2.4`.

P2.5 MUST NOT alter those semantics, commands, receipts, kernel behavior, runtime behavior or NAIR formats.

## Goal

P2.5 certifies the first bounded filesystem write surface. The language may request one new UTF-8 file only when both source and host authority agree on the exact target name.

## Surface

```noi
module app.main;

effect FileWrite;

entry main writes "report.txt" emits "Hello from NORDOI P2.5!\n";
```

Rules:

1. `effect FileWrite;` is mandatory and is the only effect accepted by the P2.5 command.
2. The entry form is exactly `entry <name> writes "<file>" emits "<text>";`.
3. The target is exactly one relative file name, not a path.
4. `/`, `\\`, `.`, `..`, control characters and overlong names are rejected before authority.
5. The file name is bounded to 128 UTF-8 bytes.
6. Decoded output is bounded to 4096 UTF-8 bytes.
7. P2.5 does not add dynamic strings or runtime expression evaluation.

## Host authority

CLI surface:

```text
nordoi file-write <path|-> [--grant-output-dir <dir>]
```

Without `--grant-output-dir`, authority is absent and execution MUST fail closed before any file is created.

With the flag, the host grants only:

```text
Capability::FileWrite(<exact source target name>)
```

The absolute output-directory path is host operational metadata. It is not source semantics and MUST NOT enter canonical plan or receipt identity.

## Sandboxing and path law

P2.5 permits only one file directly inside the explicitly granted host directory.

The implementation MUST:

- canonicalize the granted directory;
- require that it already exists and is a directory;
- join exactly one validated file name;
- open the target with create-new semantics;
- reject any existing file or symlink target;
- never overwrite;
- never create subdirectories;
- never resolve a source-supplied absolute path.

## Separation of phases

P2.5 MUST preserve:

```text
source
  -> compile deterministic file-output plan
  -> authorize exact FileWrite(target)
  -> materialize into host-granted directory
  -> emit deterministic receipt
```

Compilation and authority checks MUST be independently testable without touching the filesystem.

## Deterministic evidence

The P2.5 plan commits to:

- module identity;
- entry identity;
- `FileWrite` effect boundary;
- exact relative target file name;
- exact decoded output bytes;
- file-name and output bounds.

The P2.5 receipt commits to:

- plan SHA-256;
- module and entry identity;
- target file name;
- content SHA-256;
- explicit authority;
- zero ambient authority;
- overwrite denied.

The host output directory, source path, source ID, username, operating system and wall-clock time MUST NOT affect canonical identity.

## Failure law

Missing, unrelated, wrong-target or revoked authority MUST fail before materialization.

Existing targets MUST remain byte-for-byte unchanged.

Materialization failure MUST NOT be reported as successful output.

## Non-goals

P2.5 does not certify overwrite, append, random access, file read, directory creation, nested paths, symlink traversal, dynamic file names, dynamic file contents, filesystem enumeration, network output, process output, timestamps or ambient current-directory authority.
