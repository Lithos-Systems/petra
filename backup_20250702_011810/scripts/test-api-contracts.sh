#!/bin/bash
# Validate API contracts and interfaces

echo "🔍 API Contract Testing"
echo "======================"

# Generate and validate JSON schemas
cargo run --bin generate_schema --features json-schema > schema.json
jsonschema-validate schema.json

# Test backward compatibility
echo "Testing config compatibility..."
for version in configs/versions/*.yaml; do
    cargo run -- "$version" --validate-only || echo "❌ $version failed"
done
