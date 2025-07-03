#!/bin/bash
# Run benchmarks and generate reports

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="bench_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

echo "🏃 Running benchmarks..."

# Run standard benchmarks
cargo bench --all-features -- --save-baseline "$TIMESTAMP" | tee "$RESULTS_DIR/output.txt"

# Run with different feature sets
echo "Running minimal features..."
cargo bench --no-default-features --bench engine_performance | tee "$RESULTS_DIR/minimal.txt"

echo "Running enhanced features..."
cargo bench --features enhanced --bench engine_performance | tee "$RESULTS_DIR/enhanced.txt"

# Compare with previous baseline if exists
if [ -f "target/criterion/scan_performance/signals_and_blocks/1000-100/base/estimates.json" ]; then
    echo "📊 Comparing with baseline..."
    cargo bench -- --baseline base | tee "$RESULTS_DIR/comparison.txt"
fi

# Generate summary
cat > "$RESULTS_DIR/summary.md" << EOF
# Benchmark Results - $TIMESTAMP

## Configuration
- Rust version: $(rustc --version)
- Features: all
- Platform: $(uname -a)

## Key Metrics
$(grep -E "time:|throughput:" "$RESULTS_DIR/output.txt" | head -20)

## Comparison with Baseline
$(grep -E "change:|Performance" "$RESULTS_DIR/comparison.txt" 2>/dev/null || echo "No baseline available")
EOF

echo "✅ Results saved to $RESULTS_DIR/"
echo "📈 View detailed reports at target/criterion/report/index.html"
