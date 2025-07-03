#!/bin/bash
# cleanup-project.sh - One-time cleanup script for Petra project reorganization

set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Petra Project Cleanup${NC}"
echo "====================="
echo ""

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "info") echo -e "${BLUE}ℹ ${NC}${message}" ;;
        "success") echo -e "${GREEN}✓${NC} ${message}" ;;
        "warning") echo -e "${YELLOW}⚠${NC} ${message}" ;;
        "error") echo -e "${RED}✗${NC} ${message}" ;;
    esac
}

# Create backup
print_status "info" "Creating backup of current state..."
BACKUP_DIR="backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r scripts configs "$BACKUP_DIR/" 2>/dev/null || true
print_status "success" "Backup created in $BACKUP_DIR"

# 1. Remove obsolete scripts
print_status "info" "Removing obsolete scripts..."
REMOVED_SCRIPTS=0

if [ -f "scripts/reorganize-configs.sh" ]; then
    rm -f "scripts/reorganize-configs.sh"
    print_status "success" "Removed reorganize-configs.sh"
    ((REMOVED_SCRIPTS++))
fi

if [ -f "scripts/quick-test.sh" ]; then
    rm -f "scripts/quick-test.sh"
    print_status "success" "Removed quick-test.sh"
    ((REMOVED_SCRIPTS++))
fi

if [ -f "scripts/test-harness.sh" ]; then
    rm -f "scripts/test-harness.sh"
    print_status "success" "Removed test-harness.sh"
    ((REMOVED_SCRIPTS++))
fi

if [ -f "scripts/pre-release-check.sh" ]; then
    rm -f "scripts/pre-release-check.sh"
    print_status "success" "Removed pre-release-check.sh (integrated into petra-dev.sh)"
    ((REMOVED_SCRIPTS++))
fi

if [ $REMOVED_SCRIPTS -eq 0 ]; then
    print_status "info" "No obsolete scripts found"
fi

# 2. Reorganize configs
print_status "info" "Reorganizing configuration files..."
mkdir -p configs/{examples,production,schemas}
mkdir -p configs/examples/{basic,advanced,industrial}

# Move basic examples
MOVED_CONFIGS=0
for pattern in "example-*.yaml" "simple-*.yaml" "demo-*.yaml"; do
    for file in configs/$pattern; do
        if [ -f "$file" ]; then
            mv "$file" configs/examples/basic/ 2>/dev/null && ((MOVED_CONFIGS++)) || true
        fi
    done
done

# Move advanced examples
for pattern in "*-clickhouse.yaml" "*-complex.yaml" "*-s7.yaml" "*-opcua.yaml"; do
    for file in configs/$pattern; do
        if [ -f "$file" ] && [ ! -f "configs/examples/advanced/$(basename "$file")" ]; then
            mv "$file" configs/examples/advanced/ 2>/dev/null && ((MOVED_CONFIGS++)) || true
        fi
    done
done

# Move industrial examples
for pattern in "industrial-*.yaml" "scada-*.yaml" "building-*.yaml" "edge-*.yaml"; do
    for file in configs/$pattern; do
        if [ -f "$file" ] && [ ! -f "configs/examples/industrial/$(basename "$file")" ]; then
            mv "$file" configs/examples/industrial/ 2>/dev/null && ((MOVED_CONFIGS++)) || true
        fi
    done
done

# Move production configs
for file in configs/production-*.yaml; do
    if [ -f "$file" ] && [ ! -f "configs/production/$(basename "$file")" ]; then
        mv "$file" configs/production/ 2>/dev/null && ((MOVED_CONFIGS++)) || true
    fi
done

# Move schemas if they exist in root of configs
if [ -f "configs/petra-config-v1.json" ]; then
    mv configs/*.json configs/schemas/ 2>/dev/null && ((MOVED_CONFIGS++)) || true
fi

if [ $MOVED_CONFIGS -gt 0 ]; then
    print_status "success" "Moved $MOVED_CONFIGS configuration files"
else
    print_status "info" "Configuration files already organized"
fi

# 3. Create tools directory structure
print_status "info" "Creating tools directory structure..."
mkdir -p tools/{bin,scripts,examples}

# Check if we should move bin files
if [ -d "src/bin" ] && [ "$(ls -A src/bin)" ]; then
    print_status "warning" "Found binaries in src/bin:"
    ls -la src/bin/
    echo ""
    read -p "Move binaries from src/bin to tools/bin? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        mv src/bin/* tools/bin/
        rmdir src/bin
        print_status "success" "Moved binaries to tools/bin"
        
        # Update Cargo.toml to point to new location
        if [ -f "Cargo.toml" ]; then
            print_status "info" "Note: You'll need to update [[bin]] sections in Cargo.toml to point to tools/bin/"
        fi
    fi
fi

# 4. Move and consolidate documentation
print_status "info" "Reorganizing documentation..."
mkdir -p docs

if [ -f "Petra Codebase Mind Map.md" ]; then
    mv "Petra Codebase Mind Map.md" docs/architecture.md
    print_status "success" "Moved mind map to docs/architecture.md"
fi

# 5. Create the new consolidated script
if [ ! -f "scripts/petra-dev.sh" ]; then
    print_status "warning" "petra-dev.sh not found. Please create it from the provided artifact."
fi

# 6. Set executable permissions
chmod +x scripts/*.sh 2>/dev/null || true

# 7. Clean up empty directories
find configs -type d -empty -delete 2>/dev/null || true

# Summary
echo ""
echo -e "${BLUE}Cleanup Summary${NC}"
echo "==============="
print_status "info" "Removed $REMOVED_SCRIPTS obsolete scripts"
print_status "info" "Reorganized $MOVED_CONFIGS configuration files"
print_status "info" "Created new directory structure"
echo ""
echo "Next steps:"
echo "1. Copy petra-dev.sh from the artifact to scripts/"
echo "2. Run: chmod +x scripts/petra-dev.sh"
echo "3. Update Cargo.toml if you moved binaries"
echo "4. Review and commit changes"
echo "5. Remove this cleanup script after confirming changes"
echo ""
print_status "success" "Cleanup complete! Backup saved in $BACKUP_DIR"
