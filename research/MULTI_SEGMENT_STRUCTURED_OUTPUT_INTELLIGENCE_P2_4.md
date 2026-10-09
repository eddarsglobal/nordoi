# P2.4 Intelligence Note — Multi-Segment Structured Output

P2.4 expands practical observable programs without broadening ambient authority. The safest step after P2.3 is not a general String type or overloaded `+`; it is a bounded structural composition layer that repeats an already-certified P2.3 runtime slot.

The key architectural choice is reuse. Each dynamic slot is compiled and executed through P2.3, which itself is built on V0.7. P2.4 only owns parsing of the ordered template, global quota proof, ordering, and the aggregate receipt. This minimizes new trusted semantics.

A dedicated 2..8 segment boundary preserves milestone separation: one runtime slot remains P2.3. Static separators make dynamic slot boundaries explicit while arithmetic such as `(key_code + 1)` stays within V0.7 numeric semantics.

The aggregate receipt hashes every ordered P2.3 segment receipt. Reordering segments, changing static text, changing input, or changing any runtime computation changes the P2.4 identity. Host-specific metadata remains excluded.

P2.4 still intentionally avoids a general-purpose String runtime, mutable buffers, dynamic allocation APIs, filesystem/network I/O, interpolation loops, and unbounded emission. Those would require separate capability, type, memory and resource-governance milestones.
