#!/usr/bin/env bash
set -euo pipefail

echo "NORDOI release gate"
echo "1/2 cargo check --all-targets"
cargo check --all-targets

echo "2/2 cargo test --all-targets"
cargo test --all-targets

echo "NORDOI release gate: PASS"
