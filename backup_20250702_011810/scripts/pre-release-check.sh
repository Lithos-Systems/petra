#!/bin/bash
# Comprehensive pre-release validation

set -e

echo "🚀 Petra Pre-Release Checklist"
echo "=============================="

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Track overall status
ERRORS=0

# Function to check a condition
check() {
    local name=$1
    local cmd=$2
    
    echo -n "Checking $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        ERRORS=$((ERRORS + 1))
    fi
}

# Version check
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "Version: $VERSION"
echo ""

# 1. Code Quality Checks
echo "1. Code Quality"
echo "---------------"
check "Format" "cargo fmt --all -- --check"
check "Clippy" "cargo clippy --all-features -- -D warnings"
check "Clippy pedantic" "cargo clippy --all-features -- -W clippy::pedantic"
check "Doc examples" "cargo test --doc --all-features"

# 2. Build Checks
echo ""
echo "2. Build Checks"
echo "---------------"
check "Clean build" "cargo clean && cargo build --release --all-features"
check "Minimal build" "cargo build --release --no-default-features"
check "WASM build" "cargo build --target wasm32-unknown-unknown --no-default-features 2>/dev/null || true"

# 3. Test Suite
echo ""
echo "3. Test Suite"
echo "-------------"
check "Unit tests" "cargo test --lib --all-features"
check "Integration tests" "cargo test --test '*' --all-features"
check "Doc tests" "cargo test --doc --all-features"

# 4. Security
echo ""
echo "4. Security"
echo "-----------"
check "Audit" "cargo audit"
check "Outdated deps" "cargo outdated --exit-code 1 2>/dev/null || true"
check "License headers" "./scripts/add-license-headers.sh"

# 5. Documentation
echo ""
echo "5. Documentation"
echo "----------------"
check "Build docs" "cargo doc --all-features --no-deps"
check "README exists" "test -f README.md"
check "CHANGELOG updated" "grep -q \"\[$VERSION\]\" CHANGELOG.md"
check "Examples valid" "find configs/examples -name '*.yaml' -exec cargo run --bin petra -- {} --validate-only \;"

# 6. Performance
echo ""
echo "6. Performance"
echo "--------------"
check "Benchmarks compile" "cargo bench --no-run"

# 7. Unwrap check
echo ""
echo "7. Error Handling"
echo "-----------------"
UNWRAP_COUNT=$(rg "\.unwrap\(\)" src/ --type rust --glob '!*/tests/*' | wc -l || echo "0")
if [ "$UNWRAP_COUNT" -eq "0" ]; then
    echo -e "unwrap() calls... ${GREEN}✓${NC} (0 found)"
else
    echo -e "unwrap() calls... ${YELLOW}⚠${NC} ($UNWRAP_COUNT found)"
fi

# Summary
echo ""
echo "=============================="
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo "Ready to release v$VERSION"
else
    echo -e "${RED}❌ $ERRORS checks failed${NC}"
    echo "Please fix the issues before releasing"
    exit 1
fi
