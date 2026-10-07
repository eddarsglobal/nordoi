# Research — Acyclic Runtime Call Graph Intelligence 1.1

V1.1 deliberately avoids conventional VM call mechanics. A finite acyclic call graph can be proven before execution and represented as nested canonical pure expressions. This gives NORDOI compositional runtime functions without introducing dynamic target lookup, recursion management, arbitrary stack growth, or hidden control authority.

The essential intelligence rule is: **prove the graph first, execute only the finite graph second**.

Benefits:

- deterministic call targets;
- compile-time cycle rejection;
- statically bounded maximum depth;
- canonical replay identity;
- no indirect dispatch surface;
- no general-purpose call stack;
- static programs still erase to minimal NAIR.

V1.1 is therefore a controlled bridge between single-frame runtime calls and richer compositional language semantics.
