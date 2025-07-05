#!/bin/bash
# Enhanced benchmark runner with configurable signal counts

set -e

# Default values
DEFAULT_SIGNALS="100,1000,10000"
DEFAULT_BLOCKS="10,100,1000"
DEFAULT_FEATURES="--features standard-monitoring"
DEFAULT_OUTPUT_DIR="bench_results"

# Parse command line arguments
SIGNALS="${SIGNALS:-$DEFAULT_SIGNALS}"
BLOCKS="${BLOCKS:-$DEFAULT_BLOCKS}"
FEATURES="${FEATURES:-$DEFAULT_FEATURES}"
OUTPUT_DIR="${OUTPUT_DIR:-$DEFAULT_OUTPUT_DIR}"
BASELINE=""
COMPARE_BASELINE=""

print_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    --signals COUNTS        Comma-separated signal counts (default: $DEFAULT_SIGNALS)
    --blocks COUNTS         Comma-separated block counts (default: $DEFAULT_BLOCKS)
    --features FLAGS        Cargo feature flags (default: $DEFAULT_FEATURES)
    --output-dir DIR        Output directory (default: $DEFAULT_OUTPUT_DIR)
    --baseline NAME         Save baseline with name
    --compare BASELINE      Compare with existing baseline
    --help                  Show this help

Environment Variables:
    SIGNALS                 Override signal counts
    BLOCKS                  Override block counts
    FEATURES                Override feature flags
    OUTPUT_DIR              Override output directory

Examples:
    # Quick test with small signal counts
    $0 --signals "100,500" --blocks "10,50"
    
    # Stress test with large signal counts
    $0 --signals "10000,50000,100000" --blocks "1000,5000,10000"
    
    # Test with specific features
    $0 --features "--features enhanced-monitoring,optimized"
    
    # Save a baseline for comparison
    $0 --baseline "v1.0.0" --signals "1000,10000"
    
    # Compare with previous baseline
    $0 --compare "v1.0.0" --signals "1000,10000"

EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --signals)
            SIGNALS="$2"
            shift 2
            ;;
        --blocks)
            BLOCKS="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --baseline)
            BASELINE="$2"
            shift 2
            ;;
        --compare)
            COMPARE_BASELINE="$2"
            shift 2
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Create output directory
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="$OUTPUT_DIR/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Convert comma-separated values to arrays
IFS=',' read -ra SIGNAL_COUNTS <<< "$SIGNALS"
IFS=',' read -ra BLOCK_COUNTS <<< "$BLOCKS"

echo "🚀 Running PETRA benchmarks"
echo "=========================="
echo "Signal counts: ${SIGNAL_COUNTS[*]}"
echo "Block counts: ${BLOCK_COUNTS[*]}"
echo "Features: $FEATURES"
echo "Output: $RESULTS_DIR"
echo ""

# Set environment variables for the benchmark
export PETRA_BENCH_SIGNALS="$SIGNALS"
export PETRA_BENCH_BLOCKS="$BLOCKS"

# Build benchmark arguments
BENCH_ARGS="--bench engine_performance"
if [[ -n "$BASELINE" ]]; then
    BENCH_ARGS="$BENCH_ARGS -- --save-baseline $BASELINE"
elif [[ -n "$COMPARE_BASELINE" ]]; then
    BENCH_ARGS="$BENCH_ARGS -- --baseline $COMPARE_BASELINE"
fi

# Run the benchmark
echo "📊 Running benchmark with: cargo bench $FEATURES $BENCH_ARGS"
cargo bench $FEATURES $BENCH_ARGS 2>&1 | tee "$RESULTS_DIR/output.txt"

# Run feature-specific benchmarks if requested
if [[ "$FEATURES" != *"--no-default-features"* ]]; then
    echo ""
    echo "Running minimal feature benchmark for comparison..."
    if cargo bench --no-default-features --bench engine_performance 2>&1 | tee "$RESULTS_DIR/minimal.txt"; then
        echo "Minimal benchmark completed"
    else
        echo "WARNING: Minimal benchmark failed (likely due to missing required features)"
    fi
fi

# Generate summary report
cat > "$RESULTS_DIR/summary.md" << EOF
# Benchmark Results - $TIMESTAMP

## Configuration
- Signal counts: ${SIGNAL_COUNTS[*]}
- Block counts: ${BLOCK_COUNTS[*]}
- Features: $FEATURES
- Rust version: $(rustc --version)
- Platform: $(uname -a)

## Results Summary
```
$(grep -E "time:|throughput:|faster|slower" "$RESULTS_DIR/output.txt" 2>/dev/null | head -20 || echo "No performance data found")
```

## Performance Analysis
$(
if [[ -n "$COMPARE_BASELINE" ]]; then
    echo "### Comparison with $COMPARE_BASELINE"
    grep -E "change:|Performance" "$RESULTS_DIR/output.txt" 2>/dev/null || echo "No significant changes detected"
fi
)

## Raw Output
See \`output.txt\` for complete results.

## Criterion Reports
Open \`target/criterion/report/index.html\` for detailed HTML reports.
EOF

# Check for warnings or errors
if grep -qi "error\|failed\|panic" "$RESULTS_DIR/output.txt" 2>/dev/null; then
    echo "ERRORS detected in benchmark run"
    grep -i "error\|failed\|panic" "$RESULTS_DIR/output.txt" 2>/dev/null
fi

# Performance analysis with error handling
echo ""
echo "Performance Analysis"
echo "======================"

# Extract key metrics with error handling
if [ -f "$RESULTS_DIR/output.txt" ] && grep -q "scan_performance" "$RESULTS_DIR/output.txt" 2>/dev/null; then
    echo "Scan performance results found"
    grep -A5 -B2 "scan_performance" "$RESULTS_DIR/output.txt" 2>/dev/null | head -20
else
    echo "WARNING: No scan performance results found"
fi

echo ""
echo "Benchmark complete!"
echo "Results saved to: $RESULTS_DIR/"

# Check if HTML report exists before reporting it
if [ -f "target/criterion/report/index.html" ]; then
    echo "HTML report: target/criterion/report/index.html"
else
    echo "HTML report: Not generated (check for criterion errors)"
fi

echo "Summary: $RESULTS_DIR/summary.md"
