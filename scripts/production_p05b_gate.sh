#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
echo '=== P0.5B narrow source lock ==='
python3 scripts/production_p05b_surface_check.py --repo "$PWD"
echo '=== P0.5B negative mutation tests ==='
python3 scripts/production_p05b_negative_tests.py
echo '=== P0.5B Rust format ==='
cargo fmt --manifest-path production/native-effect-p03/Cargo.toml -- --check
echo '=== P0.5B Rust clippy ==='
cargo clippy --locked --manifest-path production/native-effect-p03/Cargo.toml --all-targets -- -D warnings
echo '=== P0.5B Rust check ==='
cargo check --locked --manifest-path production/native-effect-p03/Cargo.toml --all-targets
echo '=== P0.5B Rust tests ==='
cargo test --locked --manifest-path production/native-effect-p03/Cargo.toml --all-targets
echo 'NORDOI PRODUCTION P0.5B SURFACE LOCK: LOCAL GATE PASS'
echo 'KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED'
echo 'PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED'
