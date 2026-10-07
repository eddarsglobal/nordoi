# Real Modules & Imports Intelligence 1.3

V1.3 deliberately moves NORDOI away from a sequence of runtime-only milestones and toward a usable programming language. The key architectural choice is that modules are a compiler concern, not a runtime service.

A weaker design would carry module names into the VM, perform filesystem lookup during execution, or permit dynamic import. That would add ambient authority, nondeterministic dependency discovery, deployment coupling, and a much larger attack surface. V1.3 instead computes the reachable import graph before lowering and erases namespace mechanics into resolved internal symbols.

This gives NORDOI several useful properties simultaneously:

1. source remains human-scale and compositional;
2. module identities are explicit and canonical;
3. dependency discovery is bounded and deterministic;
4. import cycles fail before runtime;
5. the runtime needs no filesystem authority;
6. existing NAIR certification is reused unchanged;
7. multi-file language growth does not imply VM growth.

The first vertical slice intentionally keeps imported modules library-only. This is a constraint, not the final language model. It allows NORDOI to prove real cross-file pure function composition first, then later add richer exports, types, constants, capabilities, packaging, and diagnostics without invalidating the core static-resolution law.

A central V1.3 proof is therefore not a new opcode. It is the absence of one: a two-file NOI program should execute through the same NAIR 0.12 direct-call mechanism as the equivalent one-file source.
