#!/bin/bash
# Update version numbers across the project

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

NEW_VERSION=$1

echo "🔄 Updating version to $NEW_VERSION"

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" petra-designer/package.json

# Update README
sed -i.bak "s/Petra v[0-9]\+\.[0-9]\+\.[0-9]\+/Petra v$NEW_VERSION/g" README.md

# Update CHANGELOG
if ! grep -q "\[$NEW_VERSION\]" CHANGELOG.md; then
    DATE=$(date +%Y-%m-%d)
    sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $DATE/" CHANGELOG.md
fi

# Clean up backup files
find . -name "*.bak" -type f -delete

echo "Version updated to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "1. Review the changes: git diff"
echo "2. Commit: git commit -am 'chore: bump version to $NEW_VERSION'"
echo "3. Tag: git tag -s v$NEW_VERSION -m 'Release v$NEW_VERSION'"
