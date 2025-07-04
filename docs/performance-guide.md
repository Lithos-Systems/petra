# PETRA Performance Optimization Guide

## Overview

PETRA includes several advanced performance features that can significantly improve execution speed for demanding applications.

## Feature Flags

### `parallel-execution`
Enables parallel execution of independent blocks. Analyzes dependencies and executes blocks in parallel groups.

**Usage:**
```toml
[dependencies]
petra = { version = "0.1", features = ["parallel-execution"] }
```

**Configuration:**
```yaml
engine:
  parallel_execution: true
  thread_pool_size: 8  # Optional, defaults to CPU count
```

### `simd-math`
Enables SIMD-optimized math operations for array processing.

**Supported operations:**
- Array addition, subtraction, multiplication, division
- Vector dot products
- Moving averages
- FFT operations (with additional dependencies)

### `zero-copy-protocols`
Optimizes protocol drivers to minimize memory copying.

### `memory-pools`
Pre-allocates memory for common value types to reduce allocation overhead.

### `cache-optimization`
Optimizes data layout for CPU cache efficiency.

## Performance Tips

1. **Enable the high-performance bundle:**
   ```toml
   petra = { version = "0.1", features = ["high-performance"] }
   ```

2. **Build with native CPU optimizations:**
   ```bash
   RUSTFLAGS="-C target-cpu=native" cargo build --release
   ```

3. **Use the performance profile:**
   ```bash
   cargo build --profile performance
   ```

4. **Configure for your workload:**
   - Many independent blocks → `parallel-execution`
   - Heavy array math → `simd-math`
   - High-frequency trading → `zero-copy-protocols`
   - Millions of signals → `memory-pools`

## Benchmarking

Run performance benchmarks:
```bash
cargo bench --features high-performance
```

## Monitoring

Enable performance metrics:
```yaml
monitoring:
  performance_tracking: true
  block_execution_times: true
  memory_usage_tracking: true
```
```

### 10. **Example Configuration**

**Create: `configs/high-performance.yaml`**

```yaml
# High-performance PETRA configuration
engine:
  scan_time_ms: 10
  parallel_execution: true
  thread_pool_size: 8
  simd_enabled: true
  memory_pools:
    enabled: true
    prewarm_size: 10000
  cache_optimization: true

monitoring:
  enhanced: true
  performance_tracking: true
  block_execution_times: true

# Example parallel block configuration
blocks:
  # These blocks can run in parallel (no dependencies)
  - name: "input_filter_1"
    type: "EXPONENTIAL_FILTER"
    priority: 100
    parallelizable: true
    inputs:
      in: "sensor_1"
    outputs:
      out: "filtered_1"
      
  - name: "input_filter_2"
    type: "EXPONENTIAL_FILTER"
    priority: 100
    parallelizable: true
    inputs:
      in: "sensor_2"
    outputs:
      out: "filtered_2"
      
  # This block depends on both filters (runs after)
  - name: "combine_values"
    type: "ADD"
    priority: 50
    inputs:
      a: "filtered_1"
      b: "filtered_2"
    outputs:
      out: "combined_output"

  # SIMD-optimized array operations
  - name: "array_processor"
    type: "SIMD_ARRAY_ADD"
    inputs:
      a: "array_input_1"
      b: "array_input_2"
    outputs:
      out: "array_result"
```
