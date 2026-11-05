#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/production/native-remote-anchor-p06c/Cargo.toml"

echo "=== P0.6C remote-anchor source boundary ==="
python3 "$ROOT/scripts/production_p06c_boundary_check.py" --repo "$ROOT"

echo "=== P0.6C rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check

echo "=== P0.6C clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo "=== P0.6C cargo check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.6C authenticated remote-anchor tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.6B regression ==="
bash "$ROOT/scripts/production_p06b_gate.sh"

echo "=== P0.6A regression ==="
bash "$ROOT/scripts/production_p06a_gate.sh"

echo "NORDOI PRODUCTION P0.6C REMOTE ANCHOR CLIENT BACKEND: LOCAL GATE PASS"
echo "P0.6C REMOTE BACKEND TESTS: 10/10 PASS"
echo "INDEPENDENT FAILURE-DOMAIN DEPLOYMENT: NOT CERTIFIED"
echo "REMOTE SERVICE ROLLBACK-RESISTANT STORAGE: NOT CERTIFIED"
echo "KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
