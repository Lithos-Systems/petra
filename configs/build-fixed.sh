#!/bin/bash
# build-fixed.sh - Build Petra with corrected feature configuration

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🔧 Building Petra with Fixed Configuration${NC}"
echo "============================================="

# Clean previous builds to avoid feature conflicts
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cargo clean

echo -e "${YELLOW}Available build options:${NC}"
echo "1) Minimal (core features only)"
echo "2) Edge (IoT focused, single monitoring level)"
echo "3) Custom (specify exact features)"
echo "4) Debug current features"

read -p "Enter choice [1-4]: " choice

case $choice in
    1)
        FEATURES="metrics,health"
        BUILD_NAME="Minimal"
        echo -e "${YELLOW}Building minimal configuration...${NC}"
        ;;
    2)
        FEATURES="mqtt,basic-storage,standard-monitoring,metrics,health"
        BUILD_NAME="Edge"
        echo -e "${YELLOW}Building edge configuration...${NC}"
        ;;
    3)
        echo -e "${YELLOW}Current available features:${NC}"
        echo "Core: metrics, health, optimized"
        echo "Monitoring: standard-monitoring (choose only one)"
        echo "Storage: history, basic-storage"
        echo "Protocols: mqtt"
        echo "Security: security, basic-auth"
        echo ""
        echo "Enter features (comma-separated, no spaces):"
        read -r FEATURES
        BUILD_NAME="Custom"
        ;;
    4)
        echo -e "${YELLOW}Checking current feature configuration...${NC}"
        echo -e "${YELLOW}Default features from Cargo.toml:${NC}"
        grep -A 2 'default =' Cargo.toml || echo "No default found"
        echo
        echo -e "${YELLOW}Production bundle features:${NC}"
        grep -A 1 'production =' Cargo.toml || echo "No production bundle found"
        echo
        echo -e "${YELLOW}Standard monitoring definition:${NC}"
        grep -A 1 'standard-monitoring =' Cargo.toml || echo "No standard-monitoring found"
        echo
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

# Show what we're building
echo -e "${GREEN}Building: $BUILD_NAME${NC}"
echo -e "${YELLOW}Features: $FEATURES${NC}"
echo

# Set build optimizations
export RUSTFLAGS="-C target-cpu=native"

# Build with explicit feature set
echo -e "${YELLOW}Starting build...${NC}"
if cargo build --release --no-default-features --features "$FEATURES"; then
    echo -e "${GREEN}✅ Build completed successfully!${NC}"
    
    # Show binary info
    BINARY_PATH="target/release/petra"
    if [ -f "$BINARY_PATH" ]; then
        SIZE=$(du -h "$BINARY_PATH" | cut -f1)
        echo -e "${GREEN}Binary location:${NC} $BINARY_PATH"
        echo -e "${GREEN}Binary size:${NC} $SIZE"
        
        # Test the binary
        echo -e "${YELLOW}Testing binary...${NC}"
        if $BINARY_PATH --version; then
            echo -e "${GREEN}✅ Binary works correctly${NC}"
        else
            echo -e "${RED}❌ Binary test failed${NC}"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}Build Summary:${NC}"
    echo "Configuration: $BUILD_NAME"
    echo "Features: $FEATURES"
    echo "No default features used (avoiding conflicts)"
    
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo -e "${YELLOW}Try using option 4 to debug feature configuration${NC}"
    exit 1
fi

echo
echo -e "${GREEN}Next steps:${NC}"
echo "1. Test: ./target/release/petra --version"
echo "2. Run: ./run-petra-fixed.sh"
echo "3. Check features: ./target/release/petra --features" 
echo
echo -e "${YELLOW}Note: This build explicitly avoids default features to prevent conflicts${NC}"
