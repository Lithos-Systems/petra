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
            run_security_checks
            cargo test --test security_tests || { print_status "error" "Security tests failed"; exit 1; }
            print_status "success" "Security tests passed!"
            ;;
            
        "stress")
            print_status "running" "Stress testing"
            
            # Check if stress test infrastructure is available
            if docker-compose -f docker/compose/docker-compose.test.yml config > /dev/null 2>&1; then
                print_status "info" "Starting test infrastructure..."
                docker-compose -f docker/compose/docker-compose.test.yml up -d
                
                # Wait for services
                print_status "info" "Waiting for services to be ready..."
                timeout 30 bash -c 'until nc -z localhost 1883; do sleep 1; done' || {
                    print_status "error" "Test services failed to start"
                    docker-compose -f docker/compose/docker-compose.test.yml down
                    exit 1
                }
                
                # Run stress tests
                export MQTT_HOST=localhost
                export CLICKHOUSE_HOST=localhost
                cargo test --test stress_tests -- --test-threads=1 || {
                    print_status "error" "Stress tests failed"
                    docker-compose -f docker/compose/docker-compose.test.yml down
                    exit 1
                }
                
                # Cleanup
                docker-compose -f docker/compose/docker-compose.test.yml down
            else
                print_status "warning" "Test infrastructure not available, running basic stress tests"
                cargo test --test stress_tests -- --test-threads=1 || { 
                    print_status "error" "Stress tests failed"
                    exit 1
                }
            fi
            
            print_status "success" "Stress tests completed!"
            ;;
            
        "coverage")
            print_status "running" "Coverage analysis"
            if command -v cargo-tarpaulin &> /dev/null; then
                cargo tarpaulin --out Html --all-features || { 
                    print_status "error" "Coverage analysis failed"
                    exit 1
                }
                print_status "success" "Coverage report generated at tarpaulin-report.html"
            else
                print_status "warning" "cargo-tarpaulin not installed"
                print_status "info" "Install with: cargo install cargo-tarpaulin"
                exit 1
            fi
            ;;
            
        *)
            print_status "error" "Unknown test level: $LEVEL"
            exit 1
            ;;
    esac
}

# Run pre-release checks
run_pre_release_check() {
    print_status "running" "Petra Pre-Release Checklist"
    echo "=============================="
    
    local ERRORS=0
    
    # Version check
    VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
    print_status "info" "Version: $VERSION"
    echo ""
    
    # Check function
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
    
    # 5. Documentation
    echo ""
    echo "5. Documentation"
    echo "----------------"
    check "Build docs" "cargo doc --all-features --no-deps"
    check "README exists" "test -f README.md"
    check "CHANGELOG updated" "grep -q \"\[$VERSION\]\" CHANGELOG.md"
    
    # 6. Performance
    echo ""
    echo "6. Performance"
    echo "--------------"
    check "Benchmarks compile" "cargo bench --no-run"
    
    # 7. Error Handling
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
        print_status "success" "All checks passed! Ready to release v$VERSION"
    else
        print_status "error" "$ERRORS checks failed. Please fix the issues before releasing"
        exit 1
    fi
}

# Run benchmarks
run_benchmarks() {
    print_status "running" "Running benchmarks"
    
    if [ "$VERBOSE" = true ]; then
        cargo bench --bench engine_performance
    else
        cargo bench --bench engine_performance -- --warm-up-time 1 --measurement-time 2
    fi
    
    print_status "success" "Benchmarks completed!"
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
    
    # Update petra-designer package.json if it exists
    if [ -f "petra-designer/package.json" ]; then
        sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" petra-designer/package.json
    fi
    
    # Update README
    sed -i.bak "s/Petra v[0-9]\+\.[0-9]\+\.[0-9]\+/Petra v$NEW_VERSION/g" README.md
    
    # Update CHANGELOG
    if ! grep -q "\[$NEW_VERSION\]" CHANGELOG.md 2>/dev/null; then
        DATE=$(date +%Y-%m-%d)
        if [ -f "CHANGELOG.md" ]; then
            sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $DATE/" CHANGELOG.md
        else
            print_status "warning" "CHANGELOG.md not found"
        fi
    fi
    
    # Clean up backup files
    find . -name "*.bak" -type f -delete
    
    print_status "success" "Version updated to $NEW_VERSION"
    echo ""
    echo "Next steps:"
    echo "1. Review the changes: git diff"
    echo "2. Commit: git commit -am 'chore: bump version to $NEW_VERSION'"
    echo "3. Tag: git tag -s v$NEW_VERSION -m 'Release v$NEW_VERSION'"
}

# Run security checks
run_security_checks() {
    print_status "info" "Running security audit..."
    cargo audit || { print_status "warning" "Security vulnerabilities found"; }
    
    print_status "info" "Checking for outdated dependencies..."
    if command -v cargo-outdated &> /dev/null; then
        cargo outdated || true
    else
        print_status "warning" "cargo-outdated not installed"
        print_status "info" "Install with: cargo install cargo-outdated"
    fi
    
    # Check for common security patterns
    print_status "info" "Checking for unsafe patterns..."
    local UNSAFE_COUNT=$(rg "unsafe \{" src/ --type rust | wc -l || echo "0")
    if [ "$UNSAFE_COUNT" -eq "0" ]; then
        print_status "success" "No unsafe blocks found"
    else
        print_status "warning" "$UNSAFE_COUNT unsafe blocks found"
    fi
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
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
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

# Execute the appropriate command
case $COMMAND in
    test)
        run_tests
        ;;
    check)
        if [ "$LEVEL" = "pre-release" ] || [ "$LEVEL" = "full" ]; then
            run_pre_release_check
        else
            run_tests
        fi
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
        run_security_checks
        ;;
    *)
        print_status "error" "No command specified"
        usage
        ;;
esac
