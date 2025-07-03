#!/bin/bash
# Find and help fix unwrap() calls in the codebase

set -e

echo "🔍 Finding unwrap() calls in production code..."

# Create a report file
REPORT="unwrap_report_$(date +%Y%m%d_%H%M%S).txt"

# Find all unwrap() calls, excluding tests and examples
echo "=== UNWRAP CALLS TO FIX ===" > "$REPORT"
rg "\.unwrap\(\)" src/ --type rust \
    --glob '!*/tests/*' \
    --glob '!*/test.rs' \
    --glob '!*/bin/*test*.rs' \
    -n >> "$REPORT" || true

echo "" >> "$REPORT"
echo "=== EXPECT CALLS TO REVIEW ===" >> "$REPORT"
rg "\.expect\(" src/ --type rust \
    --glob '!*/tests/*' \
    --glob '!*/test.rs' \
    --glob '!*/bin/*test*.rs' \
    -n >> "$REPORT" || true

# Count occurrences
UNWRAP_COUNT=$(grep -c "unwrap()" "$REPORT" || echo "0")
EXPECT_COUNT=$(grep -c "expect(" "$REPORT" || echo "0")

echo "
📊 Summary:
- unwrap() calls: $UNWRAP_COUNT
- expect() calls: $EXPECT_COUNT
- Report saved to: $REPORT

Next steps:
1. Review the report
2. Run: cargo fix --edition-idioms
3. Manually fix remaining cases
"

# Show first 10 unwraps
echo "
First 10 unwrap() calls to fix:"
head -n 10 "$REPORT"
