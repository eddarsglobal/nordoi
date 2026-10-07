# Project Build & Package Intelligence — V1.4

V1.4 deliberately moves NORDOI from "a source root that the CLI can execute" toward "a project that can be built, locked, transported and inspected" without weakening the constitutional boundaries established by K1.18 through V1.3.

The key architectural choice is that packaging is **not** a runtime module loader. The compiler resolves the complete reachable V1.3 module graph, proves its static import structure, lowers it through the existing certified semantic chain, then serializes the resulting canonical NAIR together with build identity metadata. Runtime import authority stays zero.

A strict manifest is preferable to a permissive general TOML implementation in this stage. It minimizes parser surface, avoids a new third-party dependency, and makes unsupported configuration fail closed. Future manifest sections can be added additively under explicit certification.

`NORDOI.lock` is not a dependency registry lock in V1.4 because external packages are intentionally absent. It is a canonical build-graph lock. This is still valuable: it makes semantic/module drift observable and gives CI a deterministic `--locked` policy before NORDOI acquires any future package ecosystem.

The `.npkg` package commits to project metadata, module graph identity, a V1.4 build witness, and canonical NAIR. It is intentionally small and does not embed source paths from the host machine. Reproducibility is semantic and path-independent.

The package inspection boundary validates canonical NAIR without requiring source files. That creates the first distribution-oriented proof point while keeping execution of packaged applications for a later explicitly governed milestone.

Security posture:

- no dependency network;
- no build scripts;
- no arbitrary manifest commands;
- no dynamic plugins;
- no runtime filesystem loader;
- source-root traversal rejected;
- project/package names restricted to portable safe alphabets;
- bounded manifest/package sizes;
- corrupted/trailing package bytes rejected;
- `--locked` drift fails closed.

This milestone should reduce the remaining Production Profile 1 gap primarily in build reproducibility, packaging, CI policy, and distribution metadata rather than by adding more runtime semantics.
