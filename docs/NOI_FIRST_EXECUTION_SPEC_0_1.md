# NORDOI V0.1 — First Executable `.noi` Program

Status: **candidate** until local Release Gate, exact-commit CI, and annotated tag certification.

## 1. Purpose

V0.1 closes the first complete NORDOI source-to-runtime path:

```text
.noi source
  ↓ L0.1–L0.5
validated minimal source semantics
  ↓ C0.1–C0.3
zero-work SemanticExecutionPlan
  ↓ C0.4
NAIR 0.6 [Halt]
  ↓ V0.1
closed AtomicRuntime execution
```

V0.1 does not add a new source construct, NAIR opcode, capability, effect, runtime primitive, or host
integration. It executes only the already-certified C0.4 HALT-only lowering.

## 2. Supported executable subset

V0.1 supports the same complete L0.5 bodies already accepted by C0.4:

```noi
// empty/trivia-only body
```

or:

```noi
entry main;
```

Both lower to the same operational NAIR 0.6 program:

```text
[Halt]
```

The entry name remains compiler provenance and is not serialized into NAIR.

## 3. Execution boundary

V0.1 introduces the top-level orchestration API:

```text
execute_source_v01(source)
validate_v01_execution(lowering, runtime)
SourceExecutionReport
SourceExecutionError
```

The orchestration layer is intentionally outside `compiler`. The compiler remains responsible for
source → semantic plan → NAIR. The existing runtime remains responsible for NAIR execution.

```text
compiler ≠ runtime
V0.1 orchestration = explicit bridge between already-separated boundaries
```

## 4. Canonical input

V0.1 always executes with:

```rust
InputBatch::default()
```

which is canonical empty input. No ambient keyboard, mouse, network, filesystem, clock, process,
environment, locale, random source, or host capability is consulted by the V0.1 execution bridge.

## 5. Required runtime outcome

After `run_closed()` returns, V0.1 validates all of the following before publishing a
`SourceExecutionReport`:

- lowered NAIR is exactly one `Instruction::Halt`;
- semantic work item count = 0;
- required semantic effects = 0;
- required host authority = NONE;
- consumed input events = 0;
- executed instructions = 1;
- created domains = 0;
- created atoms = 0;
- committed transactions = 0;
- rolled-back transactions = 0;
- scheduled work = 0;
- final atoms = 0;
- render bindings = 0;
- render frames = 0;
- input bridges = 0;
- input applications = 0;
- runtime is quiescent.

Any mismatch fails closed as a V0.1 execution invariant violation.

## 6. Execution receipt

V0.1 introduces:

```text
canonical_v01_receipt_bytes()
```

with domain separator:

```text
NORDOI-V0.1-EXECUTION-RECEIPT\0
```

The receipt commits to:

1. the exact C0.4 lowering witness;
2. canonical empty input bytes;
3. the deterministic `RuntimeReplayKey`;
4. the validated bounded execution summary.

The V0.1 receipt is **execution evidence**. It is not:

- source semantics;
- a source/program identity;
- the NAIR wire format;
- a runtime checkpoint;
- host authority;
- an effect attestation;
- a promise that future richer programs will have identical execution metadata.

## 7. Determinism and provenance

Two source programs whose C0.4 lowering produces identical `[Halt]` NAIR and identical empty input
have the same runtime replay key. For example, `entry main;` and `entry other;` are operationally
identical at V0.1.

Their V0.1 receipts remain distinct because the receipt also commits to the C0.4 compiler witness,
which preserves semantic provenance.

```text
same operational NAIR → same runtime replay identity
same semantics/provenance → same V0.1 receipt
changed semantic provenance → distinct V0.1 receipt
```

## 8. Authority and effects

V0.1 does not infer or grant authority from declarations.

```text
declared effect ≠ authority
resolved effect ≠ authority
semantic plan ≠ authority
NAIR bytes ≠ authority grant
successful V0.1 execution ≠ authority grant
```

The only executable NAIR instruction is `Halt`, so V0.1 performs no effect dispatch and requires no
host capability.

## 9. CLI

V0.1 adds:

```text
nordoi run <path|->
```

Successful output reports:

- source semantic form (`EMPTY` or `ENTRY`);
- C0.4 witness;
- V0.1 execution receipt;
- runtime replay key;
- executed instruction count;
- zero input/domain/atom/transaction/render/bridge/scheduled-work counts;
- quiescent HALTED outcome.

Compilation/frontend failure uses exit code `4`. Runtime or V0.1 execution-invariant failure uses
exit code `5`.

The certified T0.1/C0.2 `--version` output remains unchanged for compatibility.

## 10. Compatibility

V0.1 preserves unchanged:

- all C0.1/L0.4/C0.2/L0.5/C0.3/C0.4 witnesses;
- NAIR format 0.6 and all NAIR implementation files;
- runtime implementation and replay semantics;
- runtime checkpoint formats;
- K1.18 kernel semantic stability floor;
- CI workflow;
- local Release Gate;
- host authority model.

## 11. Non-goals

V0.1 does not define:

- output/printing from `.noi`;
- variables, expressions, statements, calls, or returns;
- state creation or mutation from source;
- effect invocation;
- capability acquisition;
- network/filesystem/time/process access;
- user input consumption;
- rendering;
- persistence or checkpoint creation;
- a new NAIR opcode or version;
- general-purpose executable NORDOI applications.

Those require later explicit language and execution milestones.

## 12. Security law

V0.1 is safe by omission and validation:

1. frontend/body parsing fails closed;
2. C0.3 requires a pure zero-work plan;
3. C0.4 validates exact HALT-only NAIR;
4. V0.1 supplies explicit empty input;
5. the existing closed runtime executes in fresh state;
6. V0.1 validates the entire expected zero-work runtime outcome before publication;
7. no authority is serialized or inferred.
