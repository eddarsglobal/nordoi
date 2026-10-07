# V1.2 Intelligence Note — Structured Runtime Control Without a General VM

The V1.2 design deliberately avoids lowering function-body control to arbitrary jumps.

The key observation is that NORDOI already owns two bounded structures:

1. a statically known acyclic direct-call graph;
2. canonical pure call-expression trees.

Therefore a dynamic function-body decision can remain a tree operation instead of becoming a program-counter operation. NAIR 0.14 embeds a selective `IfElse` node inside the bounded `CALL_EVAL` expression tree.

This preserves important properties:

- execution cost is proportional only to the selected path;
- no general CFG is required;
- no arbitrary instruction pointer is required;
- call depth remains statically bounded;
- branch choice is observable and deterministic for equal canonical input;
- unselected work is semantically absent at runtime;
- authority remains unchanged.

The next milestones should continue this strategy: add expressive power by introducing structured, statically bounded forms before considering any general control-machine mechanism.
