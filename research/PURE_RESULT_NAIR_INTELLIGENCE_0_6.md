# C0.6 Intelligence Note — Pure Result → NAIR

The design choice for C0.6 is deliberately conservative: NORDOI must not invent a dedicated return
instruction merely because the surface language has acquired a pure result. NAIR 0.6 already owns a
canonical pure value carrier: a register populated by `Const`.

The smallest correct operational representation is therefore:

```text
returns v  =>  Const r0, Int(v); Halt
no result  =>  Halt
```

This keeps the runtime instruction vocabulary unchanged and satisfies the zero-unused-cost rule: no
result means no register write.

The semantic fact "r0 is the program result" belongs to compiler provenance at this stage, not to
NAIR wire semantics. C0.6 therefore binds that relation in its witness. A future execution milestone
may expose a validated result without retroactively changing these bytes.

This split also preserves authority discipline. A register-local constant has no relationship to
host authority, capabilities, effect declarations, output streams, or process ABI. Result transport
is computation, not permission.
