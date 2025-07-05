#!/bin/bash
# Run benchmarks and generate reports
# Updated to support configurable signal counts

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="bench_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Default configuration (can be overridden by environment variables)
export PETRA_BENCH_SIGNALS="${PETRA_BENCH_SIGNALS:-100,1000,10000}"
export PETRA_BENCH_BLOCKS="${PETRA_BENCH_BLOCKS:-10,100,1000}"

echo "🏃 Running benchmarks..."
echo "Configuration: ${PETRA_BENCH_SIGNALS} signals, ${PETRA_BENCH_BLOCKS} blocks"

# Run standard benchmarks
cargo bench --all-features -- --save-baseline "$TIMESTAMP" | tee "$RESULTS_DIR/output.txt"

echo "Running minimal features..."
cargo bench --no-default-features --bench engine_performance | tee "$RESULTS_DIR/minimal.txt"

echo "Running enhanced features..."
cargo bench --features enhanced --bench engine_performance | tee "$RESULTS_DIR/enhanced.txt"

if [ -f "target/criterion/scan_performance/signals_and_blocks/1000-100/base/estimates.json" ]; then
    echo "📊 Comparing with baseline..."
    cargo bench -- --baseline base | tee "$RESULTS_DIR/comparison.txt"
fi

cat > "$RESULTS_DIR/summary.md" << EOM
# Benchmark Results - $TIMESTAMP

## Configuration
- Signal configuration: ${PETRA_BENCH_SIGNALS}
- Block configuration: ${PETRA_BENCH_BLOCKS}
- Rust version: $(rustc --version)
- Features: all
- Platform: $(uname -a)

## Key Metrics
$(grep -E "time:|throughput:" "$RESULTS_DIR/output.txt" | head -20)

## Comparison with Baseline
$(grep -E "change:|Performance" "$RESULTS_DIR/comparison.txt" 2>/dev/null || echo "No baseline available")
EOM

echo "✅ Results saved to $RESULTS_DIR/"
echo "📈 View detailed reports at target/criterion/report/index.html"
