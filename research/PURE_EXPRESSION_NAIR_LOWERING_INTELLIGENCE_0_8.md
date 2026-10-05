# C0.8 Pure Expression → NAIR Lowering Intelligence

C0.8 exists to make the transition from semantic calculation structure to operational IR explicit and inspectable.

The central design choice is **no hidden constant folding**. C0.7 already certifies both the postfix program and its validated value. C0.8 intentionally lowers the postfix program itself rather than replacing it with a single `CONST result` instruction. This preserves calculation identity and creates a trustworthy bridge to future expression execution.

N0.7 made this possible by adding only `ADD_INT_CHECKED` and by retaining minimal-required-minor canonical encoding. C0.8 therefore does not need to mutate the NAIR format again.

The register allocator is deliberately trivial and deterministic: one fresh SSA register per postfix node. This is not claimed to be an optimized allocation strategy. It is a semantic foundation whose priorities are canonicality, auditability, validation-before-publication, and zero ambiguity. Later optimization milestones may introduce proven transformations without rewriting this certified lowering law.

Important invariants:

- one postfix node -> one NAIR calculation instruction,
- no register overwrite,
- exact lhs/rhs order,
- final stack register -> result register,
- literal-only programs stay NAIR 0.6,
- addition programs require NAIR 0.7,
- compiler never invokes runtime in C0.8,
- no effect, capability, I/O or host authority is introduced.
