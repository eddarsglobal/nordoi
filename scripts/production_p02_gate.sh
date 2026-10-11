#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
MANIFEST="production/native-trust-p01/Cargo.toml"

echo "=== P0.2 durable journal: rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check

echo "=== P0.2 durable journal: clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo "=== P0.2 durable journal: cargo check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.2 durable journal: P0.1 + P0.2 tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets

echo "NORDOI PRODUCTION P0.2 DURABLE TRUST: LOCAL GATE PASS"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
