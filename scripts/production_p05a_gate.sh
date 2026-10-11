#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
MANIFEST="production/native-effect-p03/Cargo.toml"
echo "=== P0.5A source boundary ==="
python3 scripts/production_p05a_boundary_check.py
echo "=== P0.5A rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check
echo "=== P0.5A clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings
echo "=== P0.5A check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets
echo "=== P0.5A tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets
echo "NORDOI PRODUCTION P0.5A FENCED HOST PROFILE: LOCAL GATE PASS"
echo "KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
