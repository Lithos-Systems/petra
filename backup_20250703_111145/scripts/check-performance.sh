#!/bin/bash
# Check for performance regressions

echo "📊 Performance Testing"
echo "===================="

# Run benchmarks and save results
cargo bench --bench engine_performance -- --save-baseline current

# Compare with previous baseline if exists
if [ -f target/criterion/baseline.json ]; then
    cargo bench --bench engine_performance -- --baseline previous
fi

# Check specific performance requirements
cargo test --test performance_requirements
