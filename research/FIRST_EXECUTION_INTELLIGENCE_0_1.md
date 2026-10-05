# V0.1 First Execution — Architecture Intelligence Record

## Decision

The first executable `.noi` milestone must prove the whole chain without broadening the language or
runtime surface. V0.1 therefore executes only the C0.4 output that is already known to be exact
NAIR 0.6 `[Halt]`.

## Why execution orchestration is not inside `compiler`

C0.4 deliberately ended at a validated `NairProgram`. Making the compiler call the runtime would
collapse two distinct authority and lifecycle boundaries. V0.1 instead introduces a top-level
`source_execution` orchestration layer:

```text
frontend/compiler → immutable lowering artifact
runtime           → execution of NairProgram
source_execution  → explicit composition + postcondition validation
```

This leaves future compiler targets free to emit NAIR without executing it, and leaves the runtime
free to execute NAIR that did not originate from `.noi` source.

## Why V0.1 uses empty input

Ambient input would make the first source execution depend on external state before the language has
source-level input semantics. Empty `InputBatch` is explicit, canonical and deterministic.

## Why the runtime replay key is not enough

The replay key identifies operational NAIR + input. At C0.4, different source provenance can lower
to the same `[Halt]`. V0.1 therefore adds an execution receipt that binds the C0.4 witness to the
runtime replay identity and verified result. This keeps operational identity and compiler provenance
separate instead of contaminating NAIR with source metadata.

## Why the receipt is not program semantics

Execution observations are evidence about one validated activation. They are not automatically
source semantics or a durable checkpoint. The receipt is domain-separated and explicitly scoped as
V0.1 evidence so later runtime evolution does not silently redefine program meaning.

## Next pressure point

After V0.1 certification, NORDOI has proved source → semantics → NAIR → runtime. The next language
step should add one observable but still pure source-level computation/result before adding effects,
state mutation or host authority. That keeps “No Work Without Effect” and least-authority reasoning
tractable while moving beyond a HALT-only executable language.
