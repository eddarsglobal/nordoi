#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
MANIFEST="production/native-effect-p03/Cargo.toml"

echo "=== P0.4 durable in-doubt host boundary: rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check

echo "=== P0.4 durable in-doubt host boundary: clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo "=== P0.4 durable in-doubt host boundary: check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.4 durable in-doubt host boundary: tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets

echo "NORDOI PRODUCTION P0.4 IN-DOUBT HOST BOUNDARY: LOCAL GATE PASS"
echo "KERNEL-WIDE MANDATORY ENFORCEMENT: NOT CERTIFIED"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
