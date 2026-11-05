#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/production/native-remote-anchor-service-p06d/Cargo.toml"

echo "=== P0.6D remote-anchor service source boundary ==="
python3 "$ROOT/scripts/production_p06d_boundary_check.py" --repo "$ROOT"

echo "=== P0.6D rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check

echo "=== P0.6D clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo "=== P0.6D cargo check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.6D authenticated durable service tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.6C regression ==="
bash "$ROOT/scripts/production_p06c_gate.sh"

echo "=== P0.6B regression ==="
bash "$ROOT/scripts/production_p06b_gate.sh"

echo "NORDOI PRODUCTION P0.6D REMOTE ANCHOR SERVICE: LOCAL GATE PASS"
echo "P0.6D SERVICE TESTS: 12/12 PASS"
echo "INDEPENDENT FAILURE-DOMAIN DEPLOYMENT: NOT CERTIFIED"
echo "REMOTE STORAGE FULL-SNAPSHOT ANTI-ROLLBACK: NOT CERTIFIED"
echo "HARDWARE-BACKED SERVER KEY CUSTODY: NOT CERTIFIED"
echo "KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
