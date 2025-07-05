#!/bin/bash
# Feature Dependency Analyzer for PETRA
# Identifies which features require libclang/bindgen and suggests safe alternatives

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "error")   echo -e "${RED}ERROR: $message${NC}" ;;
        "success") echo -e "${GREEN}SUCCESS: $message${NC}" ;;
        "warning") echo -e "${YELLOW}WARNING: $message${NC}" ;;
        "info")    echo -e "${BLUE}INFO: $message${NC}" ;;
        "feature") echo -e "${MAGENTA}FEATURE: $message${NC}" ;;
    esac
}

# Dependencies that require libclang/bindgen
BINDGEN_DEPS=(
    "zstd"
    "rocksdb"
    "cranelift"
    "cranelift-module"
    "cranelift-jit"
    "rust-snap7"
)

# Features that may pull in bindgen dependencies
BINDGEN_FEATURES=(
    "compression"
    "advanced-storage"
    "jit-compilation"
    "s7-support"
    "enterprise-storage"
    "full"
    "enterprise"
)

# Safe features that definitely don't require bindgen
SAFE_FEATURES=(
    "mqtt"
    "standard-monitoring"
    "enhanced-monitoring"
    "optimized"
    "metrics"
    "realtime"
    "extended-types"
    "validation"
    "regex-validation"
    "schema-validation"
    "web"
    "health"
    "alarms"
    "email"
    "security"
    "basic-auth"
    "jwt-auth"
    "rbac"
    "audit"
    "history"
)

check_libclang_required() {
    local features=$1
    
    print_status "info" "Checking if features require libclang: $features"
    
    if cargo check --no-default-features --features "$features" 2>&1 | grep -q "clang-sys\|bindgen\|libclang"; then
        return 0
    else
        return 1
    fi
}

analyze_cargo_toml() {
    echo "Analyzing Cargo.toml for bindgen dependencies..."
    echo "=============================================="
    
    cd "$PROJECT_ROOT"
    
    print_status "info" "Checking for direct bindgen dependencies..."
    
    for dep in "${BINDGEN_DEPS[@]}"; do
        if grep -q "^$dep = " Cargo.toml; then
            print_status "warning" "Found bindgen dependency: $dep"
        fi
    done
    
    echo ""
    print_status "info" "Checking for optional bindgen dependencies..."
    
    for dep in "${BINDGEN_DEPS[@]}"; do
        if grep -A5 "^$dep = " Cargo.toml | grep -q "optional = true"; then
            print_status "success" "Bindgen dependency is optional: $dep"
        fi
    done
}

test_feature_combinations() {
    echo ""
    echo "Testing Feature Combinations"
    echo "============================"
    
    cd "$PROJECT_ROOT"
    
    print_status "info" "Testing minimal build (no features)..."
    if cargo check --no-default-features >/dev/null 2>&1; then
        print_status "success" "Minimal build works"
    else
        print_status "error" "Minimal build failed"
    fi
    
    print_status "info" "Testing safe feature combinations..."
    local safe_combo=$(IFS=,; echo "${SAFE_FEATURES[*]:0:5}")
    if check_libclang_required "$safe_combo"; then
        print_status "warning" "Safe features require libclang (unexpected)"
    else
        print_status "success" "Safe features don't require libclang: $safe_combo"
    fi
    
    print_status "info" "Testing potentially problematic features..."
    for feature in "${BINDGEN_FEATURES[@]}"; do
        if grep -q "^$feature = " Cargo.toml; then
            if check_libclang_required "$feature"; then
                print_status "warning" "Feature requires libclang: $feature"
            else
                print_status "success" "Feature is safe: $feature"
            fi
        fi
    done
}

generate_safe_feature_sets() {
    echo ""
    echo "Recommended Safe Feature Sets"
    echo "============================="
    
    echo "The following feature combinations should work without libclang:"
    echo ""
    
    echo "**Minimal (Edge Device):**"
    echo "  cargo build --no-default-features --features \"mqtt\""
    echo ""
    
    echo "**Basic Monitoring:**"
    echo "  cargo build --no-default-features --features \"standard-monitoring,mqtt,extended-types\""
    echo ""
    
    echo "**Standard Deployment:**"
    echo "  cargo build --no-default-features --features \"standard-monitoring,mqtt,extended-types,validation,web,health\""
    echo ""
    
    echo "**Enhanced Monitoring:**"
    echo "  cargo build --no-default-features --features \"enhanced-monitoring,mqtt,extended-types,validation,web,health,metrics\""
    echo ""
    
    echo "**With Security:**"
    echo "  cargo build --no-default-features --features \"enhanced-monitoring,mqtt,extended-types,security,basic-auth,web,health\""
    echo ""
    
    local all_safe=$(IFS=,; echo "${SAFE_FEATURES[*]}")
    echo "**All Safe Features:**"
    echo "  cargo build --no-default-features --features \"$all_safe\""
    echo ""
}

check_system_dependencies() {
    echo ""
    echo "System Dependencies Check"
    echo "========================="
    
    if command -v llvm-config >/dev/null 2>&1; then
        print_status "success" "llvm-config found: $(llvm-config --version)"
    else
        print_status "warning" "llvm-config not found"
    fi
    
    if [ -n "${LIBCLANG_PATH:-}" ]; then
        if [ -f "$LIBCLANG_PATH/libclang.so" ]; then
            print_status "success" "libclang found at: $LIBCLANG_PATH"
        else
            print_status "warning" "LIBCLANG_PATH set but libclang not found: $LIBCLANG_PATH"
        fi
    else
        local libclang_found=false
        for path in /usr/lib/llvm-*/lib /usr/lib/x86_64-linux-gnu /usr/lib; do
            if [ -f "$path/libclang.so" ] || [ -f "$path/libclang.so.1" ]; then
                print_status "success" "libclang found at: $path"
                libclang_found=true
                break
            fi
        done
        if [ "$libclang_found" = false ]; then
            print_status "warning" "libclang not found in common locations"
        fi
    fi
}

show_installation_instructions() {
    echo ""
    echo "Installation Instructions"
    echo "========================"
    
    echo "To install libclang dependencies:"
    echo ""
    
    echo "**Ubuntu/Debian:**"
    echo "  sudo apt update"
    echo "  sudo apt install -y llvm-dev libclang-dev"
    echo "  export LIBCLANG_PATH=/usr/lib/llvm-14/lib"
    echo ""
    
    echo "**CentOS/RHEL:**"
    echo "  sudo yum install -y llvm-devel clang-devel"
    echo "  export LIBCLANG_PATH=/usr/lib64"
    echo ""
    
    echo "**macOS:**"
    echo "  brew install llvm"
    echo "  export LIBCLANG_PATH=$(brew --prefix llvm)/lib"
    echo ""
    
    echo "**To make permanent, add to ~/.bashrc or ~/.zshrc:**"
    echo "  echo 'export LIBCLANG_PATH=/usr/lib/llvm-14/lib' >> ~/.bashrc"
    echo ""
}

main() {
    echo "PETRA Feature Dependency Analyzer"
    echo "================================="
    echo ""
    
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_status "error" "Cargo.toml not found. Run from PETRA project directory."
        exit 1
    fi
    
    analyze_cargo_toml
    test_feature_combinations
    check_system_dependencies
    generate_safe_feature_sets
    show_installation_instructions
    
    echo ""
    print_status "success" "Analysis complete!"
}

case "${1:-}" in
    "--check-features")
        shift
        if [ $# -eq 0 ]; then
            echo "Usage: $0 --check-features feature1,feature2,..."
            exit 1
        fi
        cd "$PROJECT_ROOT"
        check_libclang_required "$1"
        if [ $? -eq 0 ]; then
            echo "Features require libclang: $1"
            exit 1
        else
            echo "Features are safe: $1"
            exit 0
        fi
        ;;
    "--help"|"-h")
        echo "Usage: $0 [--check-features feature1,feature2,...]"
        echo ""
        echo "Options:"
        echo "  --check-features  Check if specific features require libclang"
        echo "  --help           Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0                                    # Full analysis"
        echo "  $0 --check-features mqtt,web,health  # Check specific features"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
