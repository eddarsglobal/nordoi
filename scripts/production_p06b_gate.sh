#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
P03="production/native-effect-p03/Cargo.toml"
P06="production/native-rollback-p06/Cargo.toml"
P05C="production/native-effect-profile-p05c/Cargo.toml"
echo "=== P0.6B per-dispatch anchor source boundary ==="
python3 scripts/production_p06b_boundary_check.py --repo "$PWD"
echo "=== P0.6B rustfmt ==="
cargo fmt --manifest-path "$P03" -- --check
cargo fmt --manifest-path "$P06" -- --check
cargo fmt --manifest-path "$P05C" -- --check
echo "=== P0.6B clippy ==="
cargo clippy --locked --manifest-path "$P03" --all-targets -- -D warnings
cargo clippy --locked --manifest-path "$P06" --all-targets -- -D warnings
cargo clippy --locked --manifest-path "$P05C" --all-targets -- -D warnings
echo "=== P0.6B cargo check ==="
cargo check --locked --manifest-path "$P03" --all-targets
cargo check --locked --manifest-path "$P06" --all-targets
cargo check --locked --manifest-path "$P05C" --all-targets
echo "=== P0.6B anchor ordering tests ==="
cargo test --locked --manifest-path "$P03" --test p06b_anchor
printf '%s\n' "NORDOI PRODUCTION P0.6B PER-DISPATCH ANCHOR BARRIER: LOCAL GATE PASS"
printf '%s\n' "P0.6B ANCHOR ORDERING TESTS: 6/6 PASS"
printf '%s\n' "INDEPENDENT ANCHOR BACKEND QUALITY: NOT CERTIFIED"
printf '%s\n' "KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED"
printf '%s\n' "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
