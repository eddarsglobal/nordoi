# NORDOI R0.2 — Deterministic Resource & Task Reference Model

**CANDIDATE — not yet certified.** Additive-only delta on **r0.1**, commit `5a95dad7c7e95498b8bf04319007635bd4048152`.

This package includes a pure reference Rust state machine, **27 Rust tests**, model-obligation mapping, architecture decision and a Python static validator. It does not export a production API or introduce concurrency runtime behavior.

```
python3 scripts/validate_r01_foundations.py
python3 scripts/validate_r02_reference.py
cargo fmt --all
cargo test --test resource_task_r02 -- --nocapture
./scripts/release_gate.sh
```

Do not tag `r0.2` until all local controls and the GitHub CI on the exact new commit succeed. Do not update the 58.35% planning estimate or the 96.27% G0.1 evidence figure based solely on these reference tests.
