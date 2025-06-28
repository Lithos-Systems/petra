#!/bin/bash
# scripts/reorganize-configs.sh

# Create new structure
mkdir -p configs/{examples,production,schemas}
mkdir -p configs/examples/{basic,advanced}

# Move existing configs
if [ -d "configs" ]; then
    # Move example configs
    mv configs/example-*.yaml configs/examples/basic/ 2>/dev/null || true
    mv configs/simple-*.yaml configs/examples/basic/ 2>/dev/null || true
    mv configs/demo-*.yaml configs/examples/basic/ 2>/dev/null || true
    
    # Move advanced examples
    mv configs/*-clickhouse.yaml configs/examples/advanced/ 2>/dev/null || true
    mv configs/*-complex.yaml configs/examples/advanced/ 2>/dev/null || true
    
    # Move production configs
    mv configs/production-*.yaml configs/production/ 2>/dev/null || true
fi

echo "✅ Configuration files reorganized"
