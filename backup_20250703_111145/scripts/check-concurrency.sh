#!/bin/bash
# Test for race conditions and deadlocks

echo "🔍 Concurrency Testing"
echo "===================="

# Run with thread sanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo test --target x86_64-unknown-linux-gnu

# Run stress tests
cargo test --test stress_tests -- --test-threads=16
