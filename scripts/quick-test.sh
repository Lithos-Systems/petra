#!/bin/bash
# Quick test to verify everything is working

set -e

echo "🧪 Running quick tests..."

# Format check
echo "Checking formatting..."
cargo fmt --all -- --check

# Clippy
echo "Running clippy..."
cargo clippy --all-features -- -D warnings

# Quick test subset
echo "Running unit tests..."
cargo test --lib

# Run one integration test
echo "Running integration test..."
cargo test --test integration test_simple_control_loop

# Quick benchmark
echo "Running quick benchmark..."
cargo bench --bench engine_performance -- --warm-up-time 1 --measurement-time 2

echo "✅ All quick tests passed!"
