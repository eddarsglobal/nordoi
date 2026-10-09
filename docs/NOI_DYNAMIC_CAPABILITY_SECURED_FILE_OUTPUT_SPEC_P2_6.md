# NORDOI P2.6 — Dynamic Capability-Secured File Output

## Status

Candidate for Production Profile 2 certification.

## Purpose

P2.6 composes already-certified dynamic structured rendering with already-certified bounded filesystem materialization. It does not introduce a general String runtime, dynamic paths, overwrite authority, directory creation, reads, network access or console output.

## Canonical form

```noi
module app.main;

effect FileWrite;

input key_code;
const bias = 1;

entry main writes "report.txt" emits "input=" + key_code + ", next=" + (key_code + bias) + ", accepted=" + (key_code >= 40);
```

The output expression is intentionally the P2.4 multi-segment structured subset: 2..8 runtime Int/Bool segments separated by bounded static UTF-8 text. One runtime segment remains outside P2.6 and is not silently widened.

## Authority law

The source must declare exactly `effect FileWrite;` and the host must grant exactly `Capability::FileWrite(target)` through an explicit output directory grant. `ConsoleWrite` is neither declared nor granted by P2.6.

Authority is checked before runtime evaluation and before any filesystem materialization.

## File law

P2.6 inherits P2.5 target constraints: one ordinary relative file name, maximum 128 UTF-8 bytes, no path separators, traversal, absolute paths, directory creation or overwrite. Materialization is create-new only.

## Output bound

The P2.4 render plan proves the worst-case output is no more than 4096 UTF-8 bytes before authority. Runtime materialization rechecks the actual rendered size as an invariant.

## Proof chain

P2.6 canonical identity binds:

1. the P2.5 target-policy plan identity;
2. the P2.4 structured render plan;
3. explicit runtime input;
4. ordered runtime results;
5. per-segment V0.7 runtime witness/input/replay/result proofs;
6. final rendered UTF-8 bytes;
7. the P2.5 file-materialization receipt.

The host absolute output directory is excluded from canonical identity.

## Frozen boundaries

P2.6 adds no NAIR opcode and modifies no certified P2.1-P2.5 implementation. Production Profile 1 remains closed and immutable.
