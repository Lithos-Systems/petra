#!/bin/bash
# Run benchmarks and generate reports

set -e

check_feature_exists() {
    local feature="$1"
    grep -q "^${feature}\s*=" Cargo.toml
}

run_benchmark() {
    local label="$1"
    local feature_args="$2"
    local output_file="$3"

    echo "Running ${label} features..."
    if cargo bench ${feature_args} --bench engine_performance >"${output_file}" 2>&1; then
        echo "✓ ${label} benchmark completed"
    else
        echo "✗ ${label} benchmark failed (see ${output_file})"
    fi
}

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="bench_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

echo "🏃 Running benchmarks..."

# Run standard benchmarks
cargo bench --all-features -- --save-baseline "$TIMESTAMP" | tee "$RESULTS_DIR/output.txt"

# Run with different feature sets
run_benchmark "minimal" "--no-default-features" "$RESULTS_DIR/minimal.txt"

if check_feature_exists "enhanced-monitoring"; then
    run_benchmark "enhanced" "--features enhanced-monitoring,extended-types" "$RESULTS_DIR/enhanced.txt"
elif check_feature_exists "standard-monitoring"; then
    run_benchmark "standard" "--features standard-monitoring,extended-types" "$RESULTS_DIR/standard.txt"
fi

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
