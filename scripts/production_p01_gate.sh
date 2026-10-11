#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
MANIFEST="production/native-trust-p01/Cargo.toml"

test -f production/native-trust-p01/Cargo.lock || {
  echo "ABORT: generate and commit Cargo.lock first" >&2
  exit 1
}

echo '=== P0.1 native trust: rustfmt ==='
cargo fmt --manifest-path "$MANIFEST" -- --check

echo '=== P0.1 native trust: clippy ==='
cargo clippy --locked --manifest-path "$MANIFEST" --all-targets -- -D warnings

echo '=== P0.1 native trust: check ==='
cargo check --locked --manifest-path "$MANIFEST" --all-targets

echo '=== P0.1 native trust: tests ==='
cargo test --locked --manifest-path "$MANIFEST" --all-targets -- --nocapture

echo 'NORDOI PRODUCTION P0.1 CRYPTO COMPONENT: LOCAL GATE PASS'
echo 'PRODUCTION DEPLOYMENT AUTHORIZATION: NOT GRANTED'
