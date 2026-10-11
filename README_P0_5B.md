# NORDOI P0.5B — Narrow Production Surface Lock (candidate)

Baseline: `5da685370ed9bfb5f791554a2ea8442de2fa2ba6` on `production/p0.1`.
Preserved research tag: `r0.23` = `0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5`.

This package adds **7 files only**; it does not modify Rust code or the historical kernel.
The lock prevents unreviewed changes to the selected `FencedProductionEffects`
source boundary, its public methods, its owned P0.4 backend, and the historical
source files serving as a baseline. Ten negative mutation tests demonstrate that
specified bypass patterns fail the check. The existing P0.5A Rust integration
tests run unchanged on Ubuntu, macOS and Windows.

This is a **source-level regression barrier**, NOT kernel-wide mandatory
routing. It cannot detect clever code generation, aliases, procedural macros,
external native calls, other crates or malicious changes to the checker itself.
The historical `EffectBackend` fallback and observable I/O entrypoints still exist.
Host-controlled fence stores are not certified as persistent/linearizable;
independent anti-rollback anchoring is absent. No production deployment permission.

## Install

Merge the contents into the certified P0.5A checkout, without overwriting
existing files. Before copying, verify branch, HEAD, origin HEAD and clean tree.

## Run

```sh
python3 scripts/production_p05b_surface_check.py --repo "$PWD"
python3 scripts/production_p05b_negative_tests.py
bash scripts/production_p05b_gate.sh
bash scripts/production_p05a_gate.sh
bash scripts/production_p04_gate.sh
bash scripts/production_p03_gate.sh
bash scripts/production_p02_gate.sh
bash scripts/production_p01_gate.sh
bash scripts/release_gate.sh
```

Do not commit until the local gates all pass. After committing, require
P0.1/P0.2/P0.3/P0.4/P0.5A/P0.5B CI workflows, 6×5 jobs, on the same SHA.
