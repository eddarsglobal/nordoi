#!/usr/bin/env bash
set -euo pipefail

echo "NORDOI release gate"
echo "1/4 cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "2/4 cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "3/4 cargo check --all-targets"
cargo check --all-targets

echo "4/4 cargo test --all-targets"
cargo test --all-targets

echo "NORDOI release gate: PASS"
