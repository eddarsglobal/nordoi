#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
MANIFEST="production/native-effect-p03/Cargo.toml"

echo "=== P0.3 kernel EffectBackend adapter: rustfmt ==="
cargo fmt --manifest-path "$MANIFEST" -- --check

echo "=== P0.3 kernel EffectBackend adapter: clippy ==="
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo "=== P0.3 kernel EffectBackend adapter: check ==="
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo "=== P0.3 kernel EffectBackend adapter: tests ==="
cargo test --locked --manifest-path "$MANIFEST" --all-targets

echo "NORDOI PRODUCTION P0.3 GUARDED EFFECT ADAPTER: LOCAL GATE PASS"
echo "PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED"
