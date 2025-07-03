#!/bin/bash
# petra-dev.sh - Unified development tool for Petra
#
# This consolidates all development scripts into one comprehensive tool

set -e

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
COMMAND=""
LEVEL="standard"
VERBOSE=false

# Print usage information
usage() {
    cat << EOF
${BLUE}Petra Development Tool${NC}

Usage: $0 <command> [options]

Commands:
    ${GREEN}test${NC}      Run tests at various levels
    ${GREEN}check${NC}     Run pre-release checks
    ${GREEN}bench${NC}     Run benchmarks
    ${GREEN}clean${NC}     Clean build artifacts and reorganize
    ${GREEN}version${NC}   Update version across the project
    ${GREEN}security${NC}  Run security audits and checks

Options:
    --level <level>    Test/check level (quick|standard|full|security|stress|coverage)
    --verbose         Show detailed output
    --help           Show this help message

Examples:
    $0 test --level quick
    $0 check --level pre-release
    $0 bench
    $0 version 0.2.0

EOF
    exit 0
}

# Print colored status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "info") echo -e "${BLUE}ℹ ${NC}${message}" ;;
        "success") echo -e "${GREEN}✓${NC} ${message}" ;;
        "warning") echo -e "${YELLOW}⚠${NC} ${message}" ;;
        "error") echo -e "${RED}✗${NC} ${message}" ;;
        "running") echo -e "${BLUE}►${NC} ${message}" ;;
    esac
}

# Run tests at different levels
run_tests() {
    case "$LEVEL" in
        "quick")
            print_status "running" "Quick tests (30 seconds)"
            print_status "info" "Running format check..."
            cargo fmt --check || { print_status "error" "Format check failed"; exit 1; }
            
            print_status "info" "Running clippy..."
            cargo clippy -- -D warnings || { print_status "error" "Clippy failed"; exit 1; }
            
            print_status "info" "Running unit tests..."
            cargo test --lib || { print_status "error" "Unit tests failed"; exit 1; }
            
            print_status "success" "Quick tests passed!"
            ;;
            
        "standard")
            print_status "running" "Standard tests (2 minutes)"
            print_status "info" "Running format check..."
            cargo fmt --check || { print_status "error" "Format check failed"; exit 1; }
            
            print_status "info" "Running clippy with all features..."
            cargo clippy --all-features -- -D warnings || { print_status "error" "Clippy failed"; exit 1; }
            
            print_status "info" "Running all tests..."
            cargo test --all-features || { print_status "error" "Tests failed"; exit 1; }
            
            print_status "success" "Standard tests passed!"
            ;;
            
        "full")
            print_status "running" "Full test suite (5+ minutes)"
            run_pre_release_check
            ;;
            
        "security")
            print_status "running" "Security tests"
            print_status "info" "Running cargo audit..."
            cargo audit || { print_status "error" "Security audit failed"; exit 1; }
            
            print_status "info" "Running security tests..."
            cargo test --test security_tests || { print_status "error" "Security tests failed"; exit 1; }
            
            print_status "success" "Security tests passed!"
            ;;
            
        "stress")
            print_status "running" "Stress testing"
            cargo test --test stress_tests -- --test-threads=1 || { print_status "error" "Stress tests failed"; exit 1; }
            print_status "success" "Stress tests passed!"
            ;;
            
        "coverage")
            print_status "running" "Coverage analysis"
            cargo tarpaulin --out Html --all-features || { print_status "error" "Coverage analysis failed"; exit 1; }
            print_status "success" "Coverage report generated!"
            ;;
            
        *)
            print_status "error" "Unknown test level: $LEVEL"
            exit 1
            ;;
    esac
}

# Run pre-release checks
run_pre_release_check() {
    print_status "running" "Pre-Release Checklist"
    echo "=============================="
    
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
    
    # Get version
    VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Version: $VERSION"
    echo ""
    
    # Code Quality
    echo "1. Code Quality"
    echo "---------------"
    check "Format" "cargo fmt --all -- --check"
    check "Clippy" "cargo clippy --all-features -- -D warnings"
    check "Doc examples" "cargo test --doc --all-features"
    
    # Build Checks
    echo ""
    echo "2. Build Checks"
    echo "---------------"
    check "Clean build" "cargo clean && cargo build --release --all-features"
    check "Minimal build" "cargo build --release --no-default-features"
    
    # Test Suite
    echo ""
    echo "3. Test Suite"
    echo "-------------"
    check "Unit tests" "cargo test --lib --all-features"
    check "Integration tests" "cargo test --test '*' --all-features"
    check "Doc tests" "cargo test --doc --all-features"
    
    # Security
    echo ""
    echo "4. Security"
    echo "-----------"
    check "Audit" "cargo audit"
    
    # Documentation
    echo ""
    echo "5. Documentation"
    echo "----------------"
    check "Build docs" "cargo doc --all-features --no-deps"
    check "README exists" "test -f README.md"
    check "CHANGELOG updated" "grep -q \"\[$VERSION\]\" CHANGELOG.md"
    
    # Performance
    echo ""
    echo "6. Performance"
    echo "--------------"
    check "Benchmarks compile" "cargo bench --no-run"
    
    # Summary
    echo ""
    echo "=============================="
    if [ $ERRORS -eq 0 ]; then
        print_status "success" "All checks passed! Ready to release v$VERSION"
    else
        print_status "error" "$ERRORS checks failed"
        exit 1
    fi
}

# Run benchmarks
run_benchmarks() {
    print_status "running" "Running benchmarks"
    cargo bench --bench engine_performance
    print_status "success" "Benchmarks complete!"
}

# Clean and reorganize project
run_clean() {
    print_status "running" "Cleaning project"
    
    # Clean build artifacts
    print_status "info" "Removing build artifacts..."
    cargo clean
    
    # Clean old backup files
    print_status "info" "Removing backup files..."
    find . -name "*.bak" -type f -delete
    
    # Reorganize configs if needed
    if [ -f "scripts/reorganize-configs.sh" ]; then
        print_status "info" "Reorganizing configuration files..."
        
        # Create new structure
        mkdir -p configs/{examples,production,schemas}
        mkdir -p configs/examples/{basic,advanced}
        
        # Move existing configs (if they exist in old locations)
        if ls configs/example-*.yaml 2>/dev/null || ls configs/simple-*.yaml 2>/dev/null || ls configs/demo-*.yaml 2>/dev/null; then
            print_status "info" "Moving example configs..."
            mv configs/example-*.yaml configs/examples/basic/ 2>/dev/null || true
            mv configs/simple-*.yaml configs/examples/basic/ 2>/dev/null || true
            mv configs/demo-*.yaml configs/examples/basic/ 2>/dev/null || true
        fi
        
        if ls configs/*-clickhouse.yaml 2>/dev/null || ls configs/*-complex.yaml 2>/dev/null; then
            print_status "info" "Moving advanced examples..."
            mv configs/*-clickhouse.yaml configs/examples/advanced/ 2>/dev/null || true
            mv configs/*-complex.yaml configs/examples/advanced/ 2>/dev/null || true
        fi
        
        if ls configs/production-*.yaml 2>/dev/null; then
            print_status "info" "Moving production configs..."
            mv configs/production-*.yaml configs/production/ 2>/dev/null || true
        fi
        
        # Remove the reorganize script as it's now integrated
        rm -f scripts/reorganize-configs.sh
        print_status "success" "Configuration files reorganized"
    fi
    
    # Move mind map if it exists
    if [ -f "Petra Codebase Mind Map.md" ]; then
        print_status "info" "Moving architecture documentation..."
        mkdir -p docs
        mv "Petra Codebase Mind Map.md" docs/architecture.md
        print_status "success" "Architecture documentation moved to docs/"
    fi
    
    # Remove obsolete test scripts if they exist
    if [ -f "scripts/quick-test.sh" ] || [ -f "scripts/test-harness.sh" ]; then
        print_status "info" "Removing obsolete test scripts..."
        rm -f scripts/quick-test.sh
        rm -f scripts/test-harness.sh
        print_status "success" "Obsolete scripts removed"
    fi
    
    print_status "success" "Project cleaned and reorganized!"
}

# Update version
update_version() {
    local NEW_VERSION=$1
    
    if [ -z "$NEW_VERSION" ]; then
        print_status "error" "Version number required"
        echo "Usage: $0 version <new_version>"
        exit 1
    fi
    
    print_status "running" "Updating version to $NEW_VERSION"
    
    # Update Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    
    # Update README if it contains version
    if grep -q "version:" README.md 2>/dev/null; then
        sed -i.bak "s/version: .*/version: $NEW_VERSION/" README.md
    fi
    
    # Update CHANGELOG
    if [ -f "CHANGELOG.md" ]; then
        echo "## [$NEW_VERSION] - $(date +%Y-%m-%d)" >> CHANGELOG.md
        echo "" >> CHANGELOG.md
        echo "### Added" >> CHANGELOG.md
        echo "### Changed" >> CHANGELOG.md
        echo "### Fixed" >> CHANGELOG.md
        echo "" >> CHANGELOG.md
    fi
    
    # Clean up backup files
    rm -f *.bak
    
    print_status "success" "Version updated to $NEW_VERSION"
}

# Run security checks
run_security() {
    print_status "running" "Security audit"
    
    # Run cargo audit
    print_status "info" "Checking for vulnerabilities..."
    cargo audit || { print_status "error" "Security vulnerabilities found"; exit 1; }
    
    # Check for outdated dependencies
    print_status "info" "Checking for outdated dependencies..."
    if command -v cargo-outdated &> /dev/null; then
        cargo outdated
    else
        print_status "warning" "cargo-outdated not installed. Install with: cargo install cargo-outdated"
    fi
    
    # Check for sensitive data
    print_status "info" "Checking for hardcoded secrets..."
    if grep -r "password\|secret\|key" --include="*.rs" src/ | grep -v "test\|example"; then
        print_status "warning" "Potential hardcoded secrets found"
    fi
    
    print_status "success" "Security checks complete!"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        test|check|bench|clean|version|security)
            COMMAND=$1
            shift
            ;;
        --level)
            LEVEL="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            usage
            ;;
        *)
            if [ "$COMMAND" = "version" ] && [ -z "$NEW_VERSION" ]; then
                NEW_VERSION=$1
                shift
            else
                print_status "error" "Unknown option: $1"
                usage
            fi
            ;;
    esac
done

# Execute command
case $COMMAND in
    test)
        run_tests
        ;;
    check)
        run_pre_release_check
        ;;
    bench)
        run_benchmarks
        ;;
    clean)
        run_clean
        ;;
    version)
        update_version "$NEW_VERSION"
        ;;
    security)
        run_security
        ;;
    *)
        print_status "error" "No command specified"
        usage
        ;;
esac
