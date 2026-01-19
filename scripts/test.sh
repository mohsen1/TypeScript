#!/bin/bash
# Test script for the Zang TypeScript compiler

set -e

echo "Running Rust tests..."

# Run unit tests
cargo test --workspace

# Run with all features
cargo test --workspace --all-features

# Run doc tests
cargo test --workspace --doc

echo "Running clippy..."
cargo clippy --workspace -- -D warnings

echo "Checking formatting..."
cargo fmt --all -- --check

echo "All tests passed!"
