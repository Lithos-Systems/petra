#!/bin/bash
# Pre-release validation script

set -e

echo "🚀 Pre-release checks for Petra"

# 1. Version check
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "Version: $VERSION"

# 2. Clean build
echo "Clean build check..."
cargo clean
cargo build --all-features --release

# 3. All tests
echo "Running all tests..."
cargo test --all-features

# 4. Documentation
echo "Building documentation..."
cargo doc --all-features --no-deps

# 5. Security audit
echo "Security audit..."
cargo audit

# 6. License headers
echo "Checking license headers..."
./scripts/add-license-headers.sh

# 7. Benchmarks
echo "Running benchmarks..."
./scripts/run-benchmarks.sh

# 8. Example configs
echo "Validating example configs..."
for config in configs/examples/basic/*.yaml; do
    echo "  Checking $config"
    cargo run --bin petra -- "$config" --validate-only || exit 1
done

echo "✅ All pre-release checks passed!"
echo "Ready to tag version $VERSION"
