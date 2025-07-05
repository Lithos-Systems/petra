# Benchmarks & Performance Guide

The repository contains Criterion-based benchmarks that measure the engine's scan-cycle performance with **configurable signal and block counts**.

## Quick Start

### Using Presets (Recommended)

```bash
# Quick development test (< 30 seconds)
./scripts/run-benchmark-preset.sh quick

# Standard CI/CD test (2-3 minutes)
./scripts/run-benchmark-preset.sh standard

# Stress test (10+ minutes)
./scripts/run-benchmark-preset.sh stress
```

### Custom Signal Counts

```bash
# Test specific signal/block combinations
./scripts/run-benchmarks-enhanced.sh --signals "1000,5000" --blocks "100,500"

# Test with specific features
./scripts/run-benchmarks-enhanced.sh --signals "1000" --blocks "100" --features "--features optimized"

# Environment variable approach
PETRA_BENCH_SIGNALS="100,1000" PETRA_BENCH_BLOCKS="10,100" cargo bench --bench engine_performance
```

## Available Presets

|Preset    |Signals |Blocks|Features          |Use Case              |
|----------|--------|------|------------------|----------------------|
|`quick`   |100-500 |10-50 |minimal           |Development testing   |
|`standard`|100-5k  |10-500|optimized         |CI/CD validation      |
|`stress`  |1k-50k  |100-5k|enhanced          |Performance validation|
|`memory`  |10k-100k|1k-10k|optimized,realtime|Memory testing        |
|`edge`    |50-500  |5-50  |minimal           |Edge devices          |

## Baseline Management

```bash
# Save a baseline for comparison
./scripts/run-benchmark-preset.sh standard --baseline "v1.0.0"

# Compare current performance with baseline
./scripts/run-benchmark-preset.sh standard --compare "v1.0.0"

# Compare different feature sets
./scripts/run-benchmarks-enhanced.sh --signals "1000" --blocks "100" --features "--no-default-features" --baseline "minimal"
./scripts/run-benchmarks-enhanced.sh --signals "1000" --blocks "100" --features "--all-features" --compare "minimal"
```

## Configuration

The benchmarks can be configured in multiple ways:

### 1. Environment Variables

```bash
export PETRA_BENCH_SIGNALS="100,1000,10000"
export PETRA_BENCH_BLOCKS="10,100,1000"
cargo bench --bench engine_performance
```

### 2. Script Arguments

```bash
./scripts/run-benchmarks-enhanced.sh --signals "1000,5000,10000" --blocks "100,500,1000"
```

### 3. Preset Configurations

```bash
./scripts/run-benchmark-preset.sh stress  # Uses predefined stress test configuration
```

## Understanding Results

The benchmark generates several performance metrics:

### Core Benchmarks

- **simple_test**: Basic signal bus operation (~190 ps)
- **feature_diagnostic**: Feature availability check (~170 ps)
- **value_creation**: Value type creation (~2.7 ns)
- **signal_operations**: Signal bus read/write operations (~135 ns)

### Scalability Benchmarks

- **scan_performance**: Full engine scan cycle with configurable signal/block counts
- **signal_bus_operations**: Signal bus performance at scale
- **block_execution**: Block execution performance

### Performance Analysis

Look for these patterns in results:

- **Linear scaling**: Performance should scale linearly with signal count
- **Logarithmic block scaling**: Block execution should scale sub-linearly
- **Memory efficiency**: Large signal counts should not cause excessive memory usage

## Tips for Performance Testing

### Development Testing

```bash
# Quick validation during development
./scripts/run-benchmark-preset.sh quick
```

### CI/CD Integration

```bash
# Standard test for CI pipelines
./scripts/run-benchmark-preset.sh standard --baseline "main"
```

### Performance Regression Detection

```bash
# Save current main branch baseline
git checkout main
./scripts/run-benchmark-preset.sh standard --baseline "main"

# Test feature branch
git checkout feature-branch
./scripts/run-benchmark-preset.sh standard --compare "main"
```

### Memory and Resource Testing

```bash
# Test with large signal counts
./scripts/run-benchmark-preset.sh memory

# Monitor system resources during test
htop &
./scripts/run-benchmark-preset.sh stress
```

## Interpreting Results

### Good Performance Indicators

- Scan cycles < 1ms for 1000 signals/100 blocks
- Linear signal bus scaling
- Sub-linear block execution scaling
- Consistent throughput across feature sets

### Performance Regression Signs

- Scan time increases > 20% for same configuration
- Memory usage grows non-linearly
- Timeout or panic errors during benchmarks

## Troubleshooting

### Common Issues

1. **Out of memory**: Reduce signal counts or use `edge` preset
1. **Long test times**: Use `quick` preset for development
1. **Inconsistent results**: Ensure system is idle during testing

### Debugging

```bash
# Run with verbose output
RUST_LOG=petra=debug ./scripts/run-benchmark-preset.sh quick

# Check system resources
free -h
cat /proc/meminfo | grep Available
```
