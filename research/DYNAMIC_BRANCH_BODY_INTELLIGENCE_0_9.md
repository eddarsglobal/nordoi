# Dynamic Branch Body Intelligence 0.9

V0.8 proved that NORDOI can make a real runtime decision without adopting a conventional jump-oriented VM. V0.9 extends that proof: useful computation may now live behind the decision boundary while remaining selectively evaluated.

The key architectural choice is `BRANCH_EVAL`, not generic `JUMP`, `JUMP_IF`, basic blocks, or an exposed instruction pointer. Each branch body is a bounded pure expression tree. This keeps the new cost proportional to genuinely dynamic work and preserves the Atomic Machine principle that static work disappears before runtime.

Validation and evaluation are intentionally distinct. Both branch expressions are structurally and type validated before execution. Runtime then evaluates only the selected expression. This separates safety from speculative work: a non-selected dynamic expression can be valid yet expensive or capable of runtime overflow for a different input, and it still performs zero work for the current activation.

The execution report exposes `selected_branch_instructions` and `discarded_branch_instructions`. The V0.9 invariant requires the latter to remain zero. These counters turn lazy branch semantics into explicit certification evidence rather than an optimization assumption.

V0.9 remains deliberately narrow: no nested dynamic branches inside branch bodies, no arbitrary jumps, no runtime call stack, no recursion, no effects, and no implicit authority. Later milestones can widen structured computation only after this selective boundary is certified.
