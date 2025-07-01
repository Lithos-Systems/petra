#!/bin/bash
# build-production.sh - Production build script for Petra

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Building Petra for Production${NC}"
echo "======================================"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required tools
echo -e "${YELLOW}Checking build dependencies...${NC}"

if ! command_exists cargo; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command_exists pkg-config; then
    echo -e "${RED}❌ pkg-config not found. Install with: sudo apt-get install pkg-config${NC}"
    exit 1
fi

# Check for optional but recommended tools
if command_exists clang; then
    echo -e "${GREEN}✅ Clang found - using fast linker${NC}"
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
else
    echo -e "${YELLOW}⚠️  Clang not found - builds will be slower${NC}"
fi

# Clean previous builds
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cargo clean

# Build configurations
echo -e "${GREEN}Select build configuration:${NC}"
echo "1) Production (optimized + metrics + security)"
echo "2) Production Full (all enterprise features)"
echo "3) Edge (minimal for edge devices)"
echo "4) SCADA (industrial automation)"
echo "5) Custom"

read -p "Enter choice [1-5]: " choice

case $choice in
    1)
        FEATURES="production"
        BUILD_NAME="Production"
        ;;
    2)
        FEATURES="production-full"
        BUILD_NAME="Production Full"
        ;;
    3)
        FEATURES="edge"
        BUILD_NAME="Edge"
        ;;
    4)
        FEATURES="scada"
        BUILD_NAME="SCADA"
        ;;
    5)
        echo "Enter features (space-separated):"
        read -r FEATURES
        BUILD_NAME="Custom"
        ;;
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

# Start build
echo -e "${GREEN}Building Petra - $BUILD_NAME configuration${NC}"
echo "Features: $FEATURES"
echo "======================================"

# Set optimizations for native CPU
export RUSTFLAGS="${RUSTFLAGS} -C target-cpu=native"

# Build with selected features
if cargo build --release --features "$FEATURES"; then
    echo -e "${GREEN}✅ Build completed successfully!${NC}"
    
    # Display binary info
    BINARY_PATH="target/release/petra"
    if [ -f "$BINARY_PATH" ]; then
        SIZE=$(du -h "$BINARY_PATH" | cut -f1)
        echo -e "${GREEN}Binary location:${NC} $BINARY_PATH"
        echo -e "${GREEN}Binary size:${NC} $SIZE"
        
        # Strip symbols for smaller binary (optional)
        if command_exists strip; then
            echo -e "${YELLOW}Stripping debug symbols...${NC}"
            strip "$BINARY_PATH"
            NEW_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
            echo -e "${GREEN}Stripped size:${NC} $NEW_SIZE"
        fi
    fi
    
    # Show feature summary
    echo -e "\n${GREEN}Build Summary:${NC}"
    echo "Configuration: $BUILD_NAME"
    echo "Features: $FEATURES"
    echo "Optimization: Release mode with native CPU targeting"
    
else
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
fi

echo -e "\n${GREEN}Next steps:${NC}"
echo "1. Test the binary: ./target/release/petra --version"
echo "2. Run with config: ./target/release/petra config.yaml"
echo "3. Build Docker image: docker build -t petra ."
