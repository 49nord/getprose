#!/bin/sh -e

echo "running tests"

cargo fmt -- --check

cargo clippy --all-targets --all-features --tests --examples -- -D warnings

cargo build --all-targets --all-features
cargo test --all-targets --all-features

echo "✓ all good"
