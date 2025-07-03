#!/bin/bash
# Run with valgrind to detect memory leaks

echo "🔍 Memory Leak Detection"
echo "======================="

# Build with debug symbols
cargo build --features full

# Run with valgrind
valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         --verbose \
         target/debug/petra configs/examples/simple-mqtt.yaml \
         --run-time 60
