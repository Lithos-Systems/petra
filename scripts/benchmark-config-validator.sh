#!/bin/bash
# benchmark-config-validator.sh - validate benchmark feature configuration

set -e

CARGO_FILE="Cargo.toml"

echo "\ud83d\udd0d Benchmark Configuration Validator"

echo "=============================="

if [ ! -f "$CARGO_FILE" ]; then
    echo "Error: Cargo.toml not found" >&2
    exit 1
fi

# Extract feature list from Cargo.toml
list_features() {
    awk '/\[features\]/, /\[/{ if($0 ~ /^[a-zA-Z0-9_-]+\s*=\s*\[/) { gsub(/\s*=.*$/, "", $1); print $1 } }' "$CARGO_FILE"
}

FEATURES="$(list_features)"

echo "Available features:"
echo "$FEATURES" | sort

echo ""

check_feature() {
    local f="$1"
    echo "$FEATURES" | grep -q "^${f}$"
}

status=0
for f in enhanced-monitoring standard-monitoring extended-types; do
    if check_feature "$f"; then
        echo "\u2713 $f present"
    else
        echo "\u2717 $f missing"
        status=1
    fi
done

cat <<'EOF'

Suggested benchmark feature combinations:
 - Minimal: --no-default-features
 - Optimized: optimized,realtime,parallel-execution,simd-math
 - Production: production
 - Monitoring: enhanced-monitoring,metrics
EOF

exit $status
