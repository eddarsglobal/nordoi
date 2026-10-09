# NORDOI P2.7 — Bounded Atomic Multi-File Bundle Output

**Status:** Candidate
**Profile:** Production Profile 2
**Baseline:** P2.6 certified and immutable

## Purpose

P2.7 composes multiple already-bounded P2.6 dynamic file outputs into one commit unit without introducing ambient filesystem authority or a general filesystem API.

The final observable unit is a **new bundle directory** containing 2..8 ordinary files. The final bundle namespace remains absent while all dynamic values are evaluated and all file bytes are prepared. Publication occurs through one same-root directory rename.

P2.7 does **not** claim crash durability, hostile concurrent-host mutation resistance, distributed transactions, overwrite, append, file read, arbitrary directories, symlink traversal, networking, time, randomness or process authority.

## Source form

```noi
module app.main;

effect FileWrite;

input key_code;
const bias = 1;

entry main writes "report-bundle" {
    "report.txt" emits "input=" + key_code + ", next=" + (key_code + bias);
    "summary.txt" emits "accepted=" + (key_code >= 40) + ", key=" + key_code;
};
```

Every file body is independently required to satisfy the certified P2.6 structured dynamic-file contract: 2..8 runtime Int/Bool segments, P2.4/P2.6 output limits and explicit runtime input dependence.

## Bounds

- bundle files: 2..8;
- bundle name: one ordinary relative name, <= 128 UTF-8 bytes;
- file name: certified P2.5/P2.6 target policy, <= 128 UTF-8 bytes;
- each file: <= 4096 UTF-8 bytes under P2.6 worst-case proof;
- global bundle worst-case: <= 16384 UTF-8 bytes;
- duplicate target names: rejected before authority;
- file plans: canonicalized lexicographically by target name.

## Authority

A host grant never becomes general directory authority. For bundle `report-bundle` with `report.txt` and `summary.txt`, P2.7 requires exactly:

```text
FileWrite("report-bundle/report.txt")
FileWrite("report-bundle/summary.txt")
```

All required capabilities must be present before runtime evaluation or materialization. `ConsoleWrite` cannot substitute for any file capability.

## Transaction phases

```text
compile/validate all targets and quotas
        ↓
authorize every exact FileWrite(bundle/file)
        ↓
evaluate every P2.6 dynamic output in memory
        ↓
verify actual global quota
        ↓
verify final bundle absent
        ↓
write every file into private same-root staging
        ↓
re-check final bundle absence
        ↓
one directory rename publishes the final bundle
        ↓
deterministic global receipt
```

No final bundle file is published individually. If evaluation, staging or publication fails before the rename, the final bundle remains absent and staging is removed on the handled failure path.

## Atomicity claim boundary

P2.7 certifies **no partial final bundle state through the NORDOI publication path** and uses one same-filesystem directory rename as its commit point.

It explicitly does not claim:

- power-loss durability;
- journal/fsync durability;
- atomic transactions across filesystems;
- atomic transactions across machines;
- protection against a hostile external actor mutating the granted host directory concurrently with the commit race window.

Those require separate future contracts.

## Canonical proof

The P2.7 plan binds:

- module, entry and explicit input identity;
- `FileWrite` effect;
- bundle name;
- canonical lexicographic target order;
- every complete P2.6 file plan;
- per-file worst-case byte bound;
- global quota and file-count bounds.

The receipt binds:

- the complete P2.7 plan;
- runtime key-code input;
- ordered per-file proof hashes;
- actual total output bytes;
- explicit authority state;
- overwrite denial;
- publication mode `DIRECTORY-RENAME`;
- crash durability state `UNCLAIMED`.

Host absolute paths, staging names and source host filenames are excluded from canonical identity.

## Frozen boundaries

P2.7 adds no NAIR opcode and does not modify certified P2.1–P2.6 semantics, kernel semantics, runtime filesystem primitives or Profile 1.
