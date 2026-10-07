# Dynamic Control Flow Intelligence 0.8

V0.7 established the first legitimate runtime computation: a value exists at runtime because canonical external input was intentionally unknown at compile time.

V0.8 asks the next question: when that dynamic value controls a choice, how much control-flow machinery is actually necessary?

A conventional VM answer would introduce instruction pointers, branch offsets, arbitrary jumps, basic blocks, phi nodes, call frames, and eventually a stack machine or register-machine scheduler. NORDOI does not need to pay that cost yet.

The V0.8 answer is a bounded structured branch:

```text
dynamic BOOL
    |
BRANCH_VALUE
   /      \
value A  value B
   \      /
    SSA result
```

This is a real runtime decision: the result does not exist until the condition is observed. But the decision remains canonical, typed, bounded, authority-free, and directly observable through `runtime_branches`.

The compiler continues to erase every branch whose condition is static. Therefore the semantic distinction is now explicit:

```text
static condition  -> select at compile time -> zero runtime branch
runtime condition -> retain BRANCH_VALUE     -> one runtime branch
```

The V0.8 restriction that dynamic branch arms be statically reducible is intentional. It prevents the milestone from smuggling in a general control-flow engine before its invariants are designed. Later milestones can extend structured branches to dynamic arm computation or richer CFG forms only when their determinism, validation, replay identity, and security laws are explicit.

This keeps NORDOI aligned with the Atomic Machine principle: add runtime mechanism only when the semantics prove it is necessary.
