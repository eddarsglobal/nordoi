# NAIR Pure Integer Arithmetic Intelligence 0.7

C0.7 cannot be lowered faithfully to NAIR 0.6 because NAIR 0.6 has `Const` but no arithmetic instruction. Constant-folding the complete expression in the compiler would erase the already-certified postfix evaluation structure and would make the first expression lowering dishonest.

N0.7 therefore introduces exactly one checked integer-add primitive. The new instruction is SSA-like: operands must already exist and the destination must be fresh. Validation propagates the known register values and rejects type mismatches and overflow before execution; execution repeats checked validation defensively.

A key compatibility decision is **not** to rewrite every old NAIR program with a 0.7 header. `NairProgram::required_format_minor()` selects 0.7 only when the new arithmetic opcode is present. Existing 0.6 program bytes remain byte-for-byte stable, preserving compiler witnesses and runtime replay keys that already incorporate those bytes.

Rejected alternatives:

- Hidden compiler constant folding: rejected because it destroys the C0.7 calculation structure.
- Adding `ADD` while leaving the header at 0.6: rejected because it mutates the certified wire format without versioning.
- Generic numeric add across ints/floats: rejected because it freezes coercion and float semantics too early.
- Wrapping integer addition: rejected because silent overflow conflicts with fail-closed semantics.
