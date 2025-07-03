#!/bin/bash
# Detect performance regressions

set -e

echo "📊 Performance Regression Check"
echo "=============================="

# Save current branch
CURRENT_BRANCH=$(git branch --show-current)

# Run benchmarks on current branch
echo "Running benchmarks on $CURRENT_BRANCH..."
cargo bench --all-features -- --save-baseline current

# Checkout main branch
echo "Switching to main branch..."
git checkout main

# Run benchmarks on main
echo "Running benchmarks on main..."
cargo bench --all-features -- --save-baseline main

# Switch back
git checkout "$CURRENT_BRANCH"

# Compare results
echo ""
echo "Comparing results..."
cargo bench --all-features -- --baseline main | tee benchmark-comparison.txt

# Check for regressions
REGRESSIONS=$(grep -c "slower" benchmark-comparison.txt || true)

if [ "$REGRESSIONS" -gt 0 ]; then
    echo ""
    echo "⚠️  Found $REGRESSIONS performance regressions!"
    echo "Details:"
    grep "slower" benchmark-comparison.txt
    exit 1
else
    echo ""
    echo "✅ No performance regressions detected!"
fi
