#!/bin/bash
# test.sh - Unified test runner

set -e

# Test levels
LEVEL="${1:-quick}"

case "$LEVEL" in
    "quick")
        echo "⚡ Quick tests (30 seconds)"
        cargo fmt --check
        cargo clippy -- -D warnings
        cargo test --lib
        ;;
    
    "standard")
        echo "🧪 Standard tests (2 minutes)"
        cargo fmt --check
        cargo clippy --all-features -- -D warnings
        cargo test --all-features
        ;;
    
    "full")
        echo "🔬 Full test suite (5+ minutes)"
        ./scripts/pre-release-check.sh
        ;;
    
    "bench")
        echo "📊 Running benchmarks"
        cargo bench --bench engine_performance
        ;;
esac

echo "✅ Tests passed!"
