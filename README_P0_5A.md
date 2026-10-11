# NORDOI P0.5A — Fenced host profile (candidate)

Base commit: `2c0c2e4fe5c558baff5ee2a34dc792bfb939adeb` on `production/p0.1`.
Research tag `r0.23` must remain `0428e74a0cd69b40a9fe12e9ae0806cbbeb03fd5`.

## Contents

Eight files: seven new, one modified `production/native-effect-p03/src/lib.rs`.
`src/` of the historical kernel remains untouched.

## Verification

```bash
cargo fmt --manifest-path production/native-effect-p03/Cargo.toml
bash scripts/production_p05a_gate.sh
bash scripts/production_p04_gate.sh
bash scripts/production_p03_gate.sh
bash scripts/production_p02_gate.sh
bash scripts/production_p01_gate.sh
bash scripts/release_gate.sh
```

**Rust compilation and testing have not been executed in the packaging environment.**
New integration tests include a signed fenced event-loop delivery and a missing-signature denial.
No commit until these gates pass locally. No production deployment authorization.
