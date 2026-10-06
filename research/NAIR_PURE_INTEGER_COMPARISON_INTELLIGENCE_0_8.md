# NAIR Pure Boolean & Integer Comparison Intelligence 0.8

L0.9 and C0.11 can represent and plan pure boolean conditions, but NAIR 0.7 cannot faithfully execute integer comparisons. Boolean literals need no new instruction because `CONST` already serializes `Value::Bool`; only integer comparison is missing.

N0.8 therefore introduces six typed SSA comparison instructions and nothing else. A generic `COMPARE` instruction with an operator tag was rejected for this milestone because it adds one encoded byte to every comparison and a second semantic dispatch layer. Six dedicated opcodes keep the wire representation minimal and the operation explicit.

The comparison instructions follow the same validate-before-execute discipline as `ADD_INT_CHECKED`: both operands must already be defined integer registers and the destination must be fresh. Validation computes the resulting boolean in its known-register map, while execution repeats the type check defensively.

Minimal-required-minor encoding remains mandatory. `CONST BOOL` programs stay at 0.6, checked-add programs stay at 0.7, and only programs containing N0.8 comparisons use 0.8. This protects all certified historical NAIR bytes and compiler/runtime witnesses.

Rejected alternatives:

- Compiler constant folding of every comparison: rejected because it would erase certified condition structure.
- A single generic comparison opcode plus tag: rejected because it is larger per instruction and freezes a generic comparison model prematurely.
- Generic equality across all `Value` variants: rejected because L0.9 has only integer comparison semantics.
- Branch instructions in the same milestone: rejected because comparison and control flow are separate semantic boundaries.
