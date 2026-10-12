#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export PYTHONDONTWRITEBYTECODE=1
MANIFEST="production/native-effect-profile-p05c/Cargo.toml"
echo '=== P0.5C closed build-graph policy ==='
python3 scripts/production_p05c_graph_check.py --repo "$PWD"
echo '=== P0.5C Rust format ==='
cargo fmt --manifest-path "$MANIFEST" -- --check
echo '=== P0.5C Rust clippy ==='
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings
echo '=== P0.5C Rust check ==='
cargo check --locked --manifest-path "$MANIFEST" --all-targets
echo '=== P0.5C negative compile / mutation tests ==='
python3 scripts/production_p05c_negative_compile.py
echo '=== P0.5C Rust tests ==='
cargo test --locked --manifest-path "$MANIFEST" --all-targets
echo 'NORDOI PRODUCTION P0.5C CLOSED BUILD GRAPH: LOCAL GATE PASS'
echo 'HISTORICAL KERNEL APIS: STILL PRESENT OUTSIDE SELECTED PROFILE'
echo 'KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED'
echo 'PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED'
