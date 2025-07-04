## Core Files (Absolutely Essential)

### 1. **Build Configuration**
- `Cargo.toml` - Project manifest with dependencies and features

### 2. **Library Entry Point**
- `src/lib.rs` - Module exports and feature initialization

### 3. **Main Binary**
- `src/main.rs` - CLI interface and runtime entry point

### 4. **Core Type System**
- `src/error.rs` - Error handling (`PlcError` enum)
- `src/value.rs` - Value types (`Value` enum - Bool, Integer, Float)

### 5. **Signal Bus**
- `src/signal.rs` - Thread-safe signal storage and communication

### 6. **Configuration**
- `src/config.rs` - YAML configuration loading and validation

### 7. **Execution Engine**
- `src/engine.rs` - Scan cycle execution and block orchestration

### 8. **Block System**
- `src/blocks/mod.rs` - Block trait definition and factory
- `src/blocks/base.rs` - Essential logic blocks (AND, OR, NOT, GT, LT, EQ)
- `src/blocks/timer.rs` - Timer blocks (ON_DELAY, OFF_DELAY, PULSE)
- `src/blocks/math.rs` - Math blocks (ADD, SUB, MUL, DIV)

## Minimal Feature Set

For the absolute minimum runtime, you would compile with:

```bash
cargo build --release --no-default-features
```

This gives you:
- Core engine functionality
- Base blocks (logic, timers, math)
- YAML configuration
- Signal bus
- Basic error handling

## Minimal Configuration Example

```yaml
# minimal.yaml
version: "1.0"
engine:
  scan_time_ms: 100
  
blocks:
  - name: input_signal
    type: DATA_GENERATOR
    outputs:
      value: signals.input
    parameters:
      mode: sine
      frequency: 1.0
      
  - name: threshold_check
    type: GT
    inputs:
      in1: signals.input
      in2: signals.threshold
    outputs:
      out: signals.alarm
      
  - name: alarm_timer
    type: ON_DELAY
    inputs:
      in: signals.alarm
    outputs:
      out: signals.delayed_alarm
    parameters:
      delay_ms: 5000

signals:
  signals.threshold:
    initial_value: 0.5
```

## Total File Count

**14 files** are the absolute minimum:
1. `Cargo.toml`
2. `src/lib.rs`
3. `src/main.rs`
4. `src/error.rs`
5. `src/value.rs`
6. `src/signal.rs`
7. `src/config.rs`
8. `src/engine.rs`
9. `src/blocks/mod.rs`
10. `src/blocks/base.rs`
11. `src/blocks/timer.rs`
12. `src/blocks/math.rs`
13. `src/blocks/data.rs` (for DATA_GENERATOR)
14. `build.rs` (if it exists, for build-time feature validation)

## Dependencies

The minimal dependencies from `Cargo.toml` would be:
- `tokio` (async runtime)
- `serde`, `serde_yaml` (configuration)
- `dashmap` (signal bus)
- `chrono` (timestamps)
- `thiserror`, `anyhow` (error handling)
- `log`, `env_logger` (logging)

This minimal setup provides a fully functional PETRA instance capable of:
- Reading configuration
- Running scan cycles
- Executing logic, timer, and math blocks
- Managing signals
- Basic data generation for testing

For production use, you'd typically add:
- At least one protocol driver (e.g., MQTT)
- History/storage capability
- Monitoring/metrics
- Security features
