#!/bin/bash
# Automated release process

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

echo "🚀 Releasing Petra v$VERSION"
echo "========================="

# 1. Run all checks
echo "Running pre-release checks..."
./scripts/pre-release-check.sh || exit 1

# 2. Update version
echo "Updating version..."
./scripts/update-version.sh "$VERSION"

# 3. Generate release notes
echo "Generating release notes..."
./scripts/generate-release-notes.sh "$VERSION"

# 4. Commit changes
echo "Committing changes..."
git add -A
git commit -m "chore: release v$VERSION

- Updated version to $VERSION
- Generated release notes
- Updated CHANGELOG"

# 5. Create tag
echo "Creating tag..."
git tag -s "v$VERSION" -m "Release v$VERSION"

# 6. Build release artifacts
echo "Building release artifacts..."
mkdir -p "release/v$VERSION"

# Build for multiple targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."
    if cargo build --release --target "$target" --all-features 2>/dev/null; then
        cp "target/$target/release/petra" "release/v$VERSION/petra-$target" 2>/dev/null || \
        cp "target/$target/release/petra.exe" "release/v$VERSION/petra-$target.exe" 2>/dev/null || true
    fi
done

# 7. Create tarball
echo "Creating release tarball..."
cd release
tar -czf "petra-v$VERSION.tar.gz" "v$VERSION"
cd ..

# 8. Summary
echo ""
echo "✅ Release v$VERSION prepared!"
echo ""
echo "Next steps:"
echo "1. Review changes: git show"
echo "2. Push to GitHub: git push origin main --tags"
echo "3. Create GitHub release with release/petra-v$VERSION.tar.gz"
echo "4. Publish to crates.io: cargo publish"
echo ""
echo "Release notes: release-notes-$VERSION-full.md"
