#!/bin/bash
# Validate runtime configurations and dependencies

echo "🔍 Runtime Validation"
echo "===================="

# Check for required environment variables
check_env() {
    if [ -z "${!1}" ]; then
        echo "❌ Missing required env var: $1"
        return 1
    fi
    echo "✅ $1 is set"
}

# Validate all example configs
echo "Validating configurations..."
for config in configs/examples/*.yaml; do
    echo -n "  $config... "
    if cargo run --release -- "$config" --validate-only > /dev/null 2>&1; then
        echo "✅"
    else
        echo "❌"
    fi
done

# Check system dependencies
echo -e "\nChecking system dependencies..."
command -v mosquitto_pub >/dev/null 2>&1 && echo "✅ MQTT tools" || echo "⚠️  MQTT tools missing"
command -v parquet-tools >/dev/null 2>&1 && echo "✅ Parquet tools" || echo "⚠️  Parquet tools missing"

# Test database connections if configured
if [ -n "$DATABASE_URL" ]; then
    echo -e "\nTesting database connection..."
    # Add database connection test
fi
