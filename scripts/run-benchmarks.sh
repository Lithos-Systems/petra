#!/bin/bash
# Run benchmarks and generate reports
# Fixed version that avoids dependencies requiring libclang/bindgen

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="bench_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

echo "Running benchmarks..."
echo "Note: Avoiding features that require libclang (zstd, rocksdb, cranelift)"

# Function to check if feature exists
check_feature_exists() {
    local feature=$1
    if grep -q "^$feature = " Cargo.toml; then
        return 0
    else
        return 1
    fi
}

# Define safe feature sets that don't require libclang
SAFE_CORE_FEATURES="standard-monitoring,enhanced-monitoring,optimized,metrics,realtime"
SAFE_PROTOCOL_FEATURES="mqtt"
SAFE_BASIC_FEATURES="extended-types,validation,regex-validation"
SAFE_WEB_FEATURES="web,health,alarms,email"

# Build safe feature combinations
MINIMAL_SAFE_FEATURES="standard-monitoring"
STANDARD_SAFE_FEATURES="$SAFE_CORE_FEATURES,$SAFE_PROTOCOL_FEATURES,$SAFE_BASIC_FEATURES"
ENHANCED_SAFE_FEATURES="$STANDARD_SAFE_FEATURES,$SAFE_WEB_FEATURES"

# Function to run benchmark with error handling
run_benchmark() {
    local name=$1
    local features=$2
    local output_file=$3
    
    echo "Running $name features..."
    echo "Benchmark features:" | tee "$output_file"
    
    # Show which features are enabled
    if [[ "$features" == "--no-default-features" ]]; then
        echo "Mode: Minimal (no default features)" | tee -a "$output_file"
        echo "Extended types: false" | tee -a "$output_file"
        echo "Enhanced monitoring: false" | tee -a "$output_file"
    else
        echo "Mode: $name" | tee -a "$output_file"
        echo "Features: $features" | tee -a "$output_file"
        echo "Extended types: $(echo "$features" | grep -q 'extended-types' && echo true || echo false)" | tee -a "$output_file"
        echo "Enhanced monitoring: $(echo "$features" | grep -q 'enhanced-monitoring' && echo true || echo false)" | tee -a "$output_file"
    fi
    
    # Run the benchmark
    if [[ "$features" == "--no-default-features" ]]; then
        echo "Running: cargo bench --no-default-features --bench engine_performance" | tee -a "$output_file"
        cargo bench --no-default-features --bench engine_performance 2>&1 | tee -a "$output_file"
    else
        echo "Running: cargo bench --no-default-features --features \"$features\" --bench engine_performance" | tee -a "$output_file"
        cargo bench --no-default-features --features "$features" --bench engine_performance 2>&1 | tee -a "$output_file"
    fi
    
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo "Benchmark failed with exit code: $exit_code" | tee -a "$output_file"
        return $exit_code
    else
        echo "Benchmark completed successfully" | tee -a "$output_file"
        return 0
    fi
}

# Start with minimal benchmark (baseline)
echo "=== Running Minimal Benchmark (Baseline) ==="
if run_benchmark "minimal" "--no-default-features" "$RESULTS_DIR/minimal.txt"; then
    echo "Minimal benchmark completed"
else
    echo "Minimal benchmark failed - this indicates a fundamental build issue"
fi

echo ""
echo "=== Running Safe Feature Combinations ==="

# Run benchmarks with progressively more features (but avoiding bindgen deps)
if run_benchmark "basic-monitoring" "$MINIMAL_SAFE_FEATURES" "$RESULTS_DIR/basic.txt"; then
    echo "Basic monitoring benchmark completed"
else
    echo "Basic monitoring benchmark failed"
fi

if run_benchmark "standard-safe" "$STANDARD_SAFE_FEATURES" "$RESULTS_DIR/standard-safe.txt"; then
    echo "Standard safe features benchmark completed"
else
    echo "Standard safe features benchmark failed"
fi

if run_benchmark "enhanced-safe" "$ENHANCED_SAFE_FEATURES" "$RESULTS_DIR/enhanced-safe.txt"; then
    echo "Enhanced safe features benchmark completed"
else
    echo "Enhanced safe features benchmark failed"
fi

# Test individual feature bundles if they exist (and are safe)
echo ""
echo "=== Testing Feature Bundles ==="

if check_feature_exists "edge"; then
    if run_benchmark "edge-bundle" "edge" "$RESULTS_DIR/edge.txt"; then
        echo "Edge bundle benchmark completed"
    else
        echo "Edge bundle benchmark failed"
    fi
fi

# Test production bundle but exclude problematic dependencies
if check_feature_exists "production"; then
    echo "Checking if production bundle requires bindgen dependencies..."
    if cargo check --no-default-features --features "production" 2>&1 | grep -q "clang-sys\|bindgen"; then
        echo "Production bundle requires libclang - skipping"
        echo "Production bundle requires libclang dependencies - skipped" > "$RESULTS_DIR/production-skipped.txt"
    else
        if run_benchmark "production" "production" "$RESULTS_DIR/production.txt"; then
            echo "Production bundle benchmark completed"
        else
            echo "Production bundle benchmark failed"
        fi
    fi
fi

# Save baseline for future comparison
echo ""
echo "Saving baseline for future comparisons..."
if [ -f "target/criterion/*/report/index.html" ]; then
    cargo bench --no-default-features --features "$MINIMAL_SAFE_FEATURES" -- --save-baseline "safe-baseline-$TIMESTAMP" 2>&1 | tee "$RESULTS_DIR/baseline.txt"
fi

# Generate comprehensive summary
cat > "$RESULTS_DIR/summary.md" << EOF
# PETRA Benchmark Results - $TIMESTAMP

## Configuration
- **Rust version**: \$(rustc --version)
- **Cargo version**: \$(cargo --version)
- **Platform**: \$(uname -a)
- **Benchmark Strategy**: Avoiding bindgen dependencies (zstd, rocksdb, cranelift)

## Feature Sets Tested

### Minimal (Baseline)
- **Features**: None (--no-default-features)
- **Purpose**: Absolute baseline performance
- **Result**: \([ -f "$RESULTS_DIR/minimal.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/minimal.txt" && echo "Success" || echo "Failed"\)

### Basic Monitoring
- **Features**: \`$MINIMAL_SAFE_FEATURES\`
- **Purpose**: Basic monitoring overhead measurement
- **Result**: \([ -f "$RESULTS_DIR/basic.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/basic.txt" && echo "Success" || echo "Failed"\)

### Standard Safe Features
- **Features**: \`$STANDARD_SAFE_FEATURES\`
- **Purpose**: Standard deployment without heavy dependencies
- **Result**: \([ -f "$RESULTS_DIR/standard-safe.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/standard-safe.txt" && echo "Success" || echo "Failed"\)

### Enhanced Safe Features
- **Features**: \`$ENHANCED_SAFE_FEATURES\`
- **Purpose**: Full-featured deployment without bindgen dependencies
- **Result**: \([ -f "$RESULTS_DIR/enhanced-safe.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/enhanced-safe.txt" && echo "Success" || echo "Failed"\)

## Performance Metrics

### Key Timing Results
\$(if [ -f "$RESULTS_DIR/minimal.txt" ]; then
    echo "**Minimal Features:**"
    grep -E "time:|simple_test|value_creation|signal_operations|feature_diagnostic" "$RESULTS_DIR/minimal.txt" | head -10 | sed 's/^/- /'
fi)

\$(if [ -f "$RESULTS_DIR/enhanced-safe.txt" ]; then
    echo ""
    echo "**Enhanced Safe Features:**"
    grep -E "time:|simple_test|value_creation|signal_operations|feature_diagnostic" "$RESULTS_DIR/enhanced-safe.txt" | head -10 | sed 's/^/- /'
fi)

## Dependencies Avoided

The following dependencies require libclang/bindgen and were excluded:
- **zstd** (compression) - requires zstd-sys with bindgen
- **rocksdb** (database) - requires librocksdb-sys with bindgen  
- **cranelift** family (JIT compilation) - requires LLVM bindings
- **advanced-storage** bundle - includes rocksdb
- **compression** features - includes zstd
- **jit-compilation** features - includes cranelift

## Recommendations

### For CI/CD Pipelines
1. Use **minimal** and **standard-safe** feature sets for regular benchmarks
2. Test **enhanced-safe** features for comprehensive performance analysis
3. Set up separate environment with libclang for full feature testing

### For Development
- Use \`cargo build --no-default-features --features "$STANDARD_SAFE_FEATURES"\` for fast builds
- Install libclang dependencies only when testing storage/compression features

### For Production Deployment
- **Edge devices**: Use minimal or basic monitoring features
- **Standard servers**: Use standard-safe feature set
- **Enterprise**: Install libclang and use full feature set including storage

## Files Generated
\$(ls -la "$RESULTS_DIR"/*.txt 2>/dev/null | awk '{print "- " $9}' || echo "- No benchmark files generated")

EOF

echo ""
echo "Results saved to $RESULTS_DIR/"
echo "View detailed reports at target/criterion/report/index.html"
echo "View summary at $RESULTS_DIR/summary.md"

# Display quick summary
echo ""
echo "Quick Summary:"
echo "=============="

declare -A results
if [ -f "$RESULTS_DIR/minimal.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/minimal.txt"; then
    results["Minimal"]="Success"
else
    results["Minimal"]="Failed"
fi

if [ -f "$RESULTS_DIR/standard-safe.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/standard-safe.txt"; then
    results["Standard"]="Success"
else
    results["Standard"]="Failed"
fi

if [ -f "$RESULTS_DIR/enhanced-safe.txt" ] && grep -q "completed successfully" "$RESULTS_DIR/enhanced-safe.txt"; then
    results["Enhanced"]="Success"
else
    results["Enhanced"]="Failed"
fi

for config in "${!results[@]}"; do
    echo "$config: ${results[$config]}"
done

echo ""
echo "To run benchmarks with ALL features (requires libclang):"
echo "   sudo apt install -y llvm-dev libclang-dev"
echo "   export LIBCLANG_PATH=/usr/lib/llvm-14/lib"
echo "   cargo bench --all-features"
echo ""
echo "Open target/criterion/report/index.html for detailed analysis"
