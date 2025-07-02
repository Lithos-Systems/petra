#!/bin/bash
# Generate release notes from CHANGELOG

set -e

VERSION=${1:-$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)}

echo "📝 Generating release notes for v$VERSION"

# Extract section from CHANGELOG
awk "/## \[$VERSION\]/{flag=1;next}/## \[/{flag=0}flag" CHANGELOG.md > release-notes-$VERSION.md

# Add header
cat > release-notes-$VERSION-full.md << EOF
# Petra v$VERSION Release Notes

$(date +"%B %d, %Y")

## Overview

Petra v$VERSION is here! This release includes significant improvements to performance, security, and developer experience.

## What's Changed

$(cat release-notes-$VERSION.md)

## Installation

### From Crates.io
\`\`\`bash
cargo install petra --version $VERSION
\`\`\`

### Using Docker
\`\`\`bash
docker pull ghcr.io/your-org/petra:$VERSION
\`\`\`

### From Source
\`\`\`bash
git clone https://github.com/your-org/petra
cd petra
git checkout v$VERSION
cargo build --release
\`\`\`

## Acknowledgments

Thanks to all contributors who made this release possible!

---

**Full Changelog**: https://github.com/your-org/petra/compare/v${PREV_VERSION}...v$VERSION
EOF

echo "✅ Release notes saved to release-notes-$VERSION-full.md"
