#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PYTHONDONTWRITEBYTECODE=1
P01="production/native-trust-p01/Cargo.toml"
P03="production/native-effect-p03/Cargo.toml"
P06="production/native-rollback-p06/Cargo.toml"

echo '=== P0.6A witness/anchor source boundary ==='
python3 scripts/production_p06a_boundary_check.py --repo "$PWD"

echo '=== P0.6A rustfmt ==='
cargo fmt --manifest-path "$P01" -- --check
cargo fmt --manifest-path "$P03" -- --check
cargo fmt --manifest-path "$P06" -- --check

echo '=== P0.6A clippy ==='
cargo clippy --locked --manifest-path "$P01" --all-targets -- -D warnings
cargo clippy --locked --manifest-path "$P03" --all-targets -- -D warnings
cargo clippy --locked --manifest-path "$P06" --all-targets -- -D warnings

echo '=== P0.6A cargo check ==='
cargo check --locked --manifest-path "$P01" --all-targets
cargo check --locked --manifest-path "$P03" --all-targets
cargo check --locked --manifest-path "$P06" --all-targets

echo '=== P0.6A rollback tests ==='
cargo test --locked --manifest-path "$P06" --all-targets

echo 'NORDOI PRODUCTION P0.6A ROLLBACK WITNESS CONTRACT: LOCAL GATE PASS'
echo 'P0.6A ROLLBACK TESTS: 10/10 PASS'
echo 'INDEPENDENT MONOTONIC ANTI-ROLLBACK: NOT CERTIFIED'
echo 'PER-DISPATCH ANCHOR ADVANCE: NOT YET INTEGRATED (P0.6B REQUIRED)'
echo 'PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED'
