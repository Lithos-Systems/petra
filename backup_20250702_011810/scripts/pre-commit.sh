#!/bin/bash
# Install as: ln -s ../../scripts/pre-commit.sh .git/hooks/pre-commit

echo "🔍 Pre-commit checks..."

# Format check
cargo fmt --check || {
    echo "❌ Code needs formatting. Run: cargo fmt"
    exit 1
}

# Quick clippy
cargo clippy --tests -- -D warnings || {
    echo "❌ Clippy warnings found"
    exit 1
}

# Check for TODOs
if git diff --cached --name-only | xargs grep -i "TODO\|FIXME\|XXX" 2>/dev/null; then
    echo "⚠️  Found TODO/FIXME markers in staged files"
fi

echo "✅ Pre-commit checks passed"
