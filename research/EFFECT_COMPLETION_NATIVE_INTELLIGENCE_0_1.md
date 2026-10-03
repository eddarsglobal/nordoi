# Effect Completion Native Intelligence 0.1

K1.13 records the architectural decision to move completion projection declarations into canonical
NAIR only after K1.12 certified the governed re-entry laws beneath the IR.

The key separation is:

```text
program semantics = what a completion projects
host authority     = which external source/namespace may supply it
```

Serializing both would let a program claim external authority merely by loading bytes. Keeping the
route binding host-supplied preserves least privilege, replaceable deployment policy and deterministic
program representation.

K1.13 therefore follows the same layering pattern used for native reactions:

```text
semantic core first
    ↓
certify invariants
    ↓
encode native NAIR declaration
    ↓
leave source-language syntax for later
```
