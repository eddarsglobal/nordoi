# V0.6 Intelligence Note — Core Computation & Functions

The production objective is no longer to expose one operator per milestone. V0.6 batches the first practical pure-computation language core while preserving the constitutional rule that unused machinery should cost nothing.

The key architectural choice is proof-driven erasure. Because the V0.6 source surface has no dynamic input, every function call and conditional is resolvable before runtime. A conventional implementation would still build call frames, branch opcodes and arithmetic runtime work. NORDOI instead retains the complete semantic structure in its canonical witness and publishes only the minimal executable result.

This separates auditability from operational cost: changing an unused function changes the V0.6 semantic witness but does not change the two-instruction NAIR program when the observable result is unchanged.

Recursion is intentionally rejected. Adding a runtime call stack solely to support recursion would create permanent machinery before dynamic functions are otherwise necessary. A later production batch may introduce governed runtime calls when dynamic inputs and reusable runtime activation make that mechanism justified by observable work.
