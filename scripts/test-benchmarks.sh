#!/bin/bash
echo "Testing engine benchmark..."
if cargo bench --no-default-features --bench engine --no-run; then
    echo "\u2713 engine benchmark compiles"
else
    echo "\u2717 engine benchmark failed"
    exit 1
fi

echo "Testing engine_performance benchmark..."
if cargo bench --bench engine_performance --no-run; then
    echo "\u2713 engine_performance benchmark compiles"
else
    echo "\u2717 engine_performance benchmark failed"
fi
