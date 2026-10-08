# NORDOI V1.5 — Production Diagnostics Intelligence

Production readiness is not only execution correctness. A compiler that fails safely but cannot identify where, why and through which dependency path it failed is expensive to operate and difficult to integrate with editors and CI.

V1.5 therefore treats diagnostics as a governed compatibility surface.

## Principles

1. Machine identity is separated from human prose through stable `NDX` codes.
2. Source positions are evidence-backed; guessed spans are forbidden.
3. Multi-file failures preserve deterministic import provenance.
4. JSON output is a versioned schema, not an ad-hoc log scrape.
5. `check` is side-effect-minimal: it validates without producing packages or locks.
6. Diagnostics remain outside kernel semantics and do not expand runtime authority.

## Why no broad refactor

Retrofitting every historical CLI path at once would enlarge the certification surface and risk regressions. V1.5 instead establishes one clean project-level diagnostic boundary over the already-certified V1.3/V1.4 project pipeline. Older commands remain compatible while future IDE/CI work can target the V1.5 schema.

## Future evolution

Later releases may add warning classes, related spans, fix hints, LSP transport and richer package diagnostics. Those additions should remain additive under the `NDX` code law and must not turn diagnostics into an authority-bearing execution path.
