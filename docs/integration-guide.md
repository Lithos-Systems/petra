# PETRA Production-Ready Code Integration & Constraint Guidelines

## Overview

PETRA is a highly modular industrial automation system built in Rust with over 80 feature flags. This document serves as the definitive guide for developing production-ready code, ensuring quality standards, and understanding where/how to integrate new functionality into the system.

## Table of Contents

1. [Pre-Release Checklist](#pre-release-checklist)
2. [Code Organization & Placement](#code-organization--placement)
3. [Core Architecture Constraints](#core-architecture-constraints)
4. [Production Readiness Standards](#production-readiness-standards)
5. [Testing Requirements](#testing-requirements)
6. [Performance Standards](#performance-standards)
7. [Security Requirements](#security-requirements)
8. [Documentation Standards](#documentation-standards)
9. [Common Integration Patterns](#common-integration-patterns)
10. [Debugging & Troubleshooting](#debugging--troubleshooting)

## Pre-Release Checklist

Before any code is considered production-ready, it must pass ALL of these checks:

### 🔍 Code Quality
- [ ] `cargo fmt --all -- --check` passes (use `cargo fmt` to fix)
- [ ] `cargo clippy --all-features -- -D warnings` shows no warnings
- [ ] `cargo clippy --all-features -- -W clippy::pedantic` reviewed
- [ ] Zero `unwrap()` calls in production code (grep for `.unwrap()`)
- [ ] Zero `expect()` calls without descriptive messages
- [ ] Zero `todo!()`, `unimplemented!()`, or `unreachable!()` macros
- [ ] All `println!` replaced with proper logging
- [ ] No hardcoded values - use configuration or constants

### 🏗️ Build Verification
- [ ] `cargo build --release --all-features` succeeds
- [ ] `cargo build --release --no-default-features` succeeds
- [ ] Binary size is reasonable for target deployment
- [ ] Build time is under 5 minutes on CI

### 🧪 Testing Coverage
- [ ] Unit tests cover all public APIs
- [ ] Integration tests verify component interactions
- [ ] Property-based tests for complex logic
- [ ] Error cases explicitly tested
- [ ] Tests pass with `--all-features` and `--no-default-features`
- [ ] No flaky tests (run 10x to verify)
- [ ] Test coverage > 80% for new code

### 🔒 Security Validation
- [ ] `cargo audit` shows no vulnerabilities
- [ ] All inputs validated and sanitized
- [ ] No SQL injection vulnerabilities
- [ ] No path traversal vulnerabilities
- [ ] Secrets never logged or exposed
- [ ] Rate limiting on all external APIs
- [ ] Authentication required for sensitive operations

### 📊 Performance Validation
- [ ] Benchmarks show no regression
- [ ] Memory usage profiled and acceptable
- [ ] No memory leaks (use `valgrind` or similar)
- [ ] CPU usage within targets
- [ ] Real-time constraints met (if applicable)

### 📚 Documentation
- [ ] All public APIs have doc comments with examples
- [ ] README updated if needed
- [ ] CHANGELOG entry added
- [ ] Configuration examples provided
- [ ] Migration guide for breaking changes

### 🔄 Async & Concurrency
- [ ] All async functions properly handle cancellation
- [ ] No blocking operations in async contexts
- [ ] Mutex locks held for minimal duration
- [ ] No potential deadlocks
- [ ] Proper timeout handling on all I/O operations

## Code Organization & Placement

### Directory Structure

```
petra/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── main.rs             # Binary entry point
│   ├── error.rs            # All error types
│   ├── value.rs            # Core value system
│   ├── signal.rs           # Signal bus core
│   ├── config.rs           # Configuration types
│   ├── engine.rs           # Scan engine
│   │
│   ├── blocks/             # Block implementations
│   │   ├── mod.rs          # Block trait & factory
│   │   ├── base.rs         # Core logic blocks
│   │   └── [category]/     # Grouped by function
│   │
│   ├── protocols/          # Protocol drivers
│   │   ├── mod.rs          # ProtocolDriver trait
│   │   ├── mqtt/           # MQTT implementation
│   │   ├── modbus/         # Modbus implementation
│   │   └── s7/             # S7 implementation
│   │
│   ├── storage/            # Data persistence
│   │   ├── mod.rs          # Storage traits
│   │   ├── parquet/        # Parquet storage
│   │   └── clickhouse/     # ClickHouse integration
│   │
│   ├── web/                # Web API & UI
│   │   ├── mod.rs          # Web server setup
│   │   ├── api/            # REST endpoints
│   │   └── ui/             # Web interface
│   │
│   └── bin/                # Utility binaries
│       └── petra.rs        # Main executable
│
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
├── configs/                # Configuration examples
│   ├── examples/           # Example configs
│   ├── production/         # Production templates
│   └── schemas/            # JSON schemas
└── docs/                   # Documentation
```

### Where to Place New Code

| Code Type | Location | Example |
|-----------|----------|---------|
| New block type | `src/blocks/[category]/` | `src/blocks/math/statistics.rs` |
| New protocol | `src/protocols/[name]/` | `src/protocols/opcua/` |
| New storage backend | `src/storage/[name]/` | `src/storage/influxdb/` |
| New validation rule | `src/validation/rules/` | `src/validation/rules/range.rs` |
| New alarm type | `src/alarms/types/` | `src/alarms/types/rate_of_change.rs` |
| Utility functions | `src/utils/[category].rs` | `src/utils/time.rs` |
| Web endpoints | `src/web/api/v1/` | `src/web/api/v1/signals.rs` |
| Binary tools | `src/bin/` | `src/bin/config_validator.rs` |

## Core Architecture Constraints

### 1. Type System Constraints

PETRA has a strict type system centered around the `Value` enum. All data flowing through the system must conform to these types:

```rust
pub enum Value {
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),      // Only with extended-types feature
    Binary(Vec<u8>),     // Only with extended-types feature
    Timestamp(DateTime), // Only with extended-types feature
    Array(Vec<Value>),   // Only with extended-types feature
    Object(HashMap<String, Value>), // Only with extended-types feature
}
```

**Production Requirements:**
- Never extend the base `Value` enum without the `extended-types` feature flag
- All values must be serializable/deserializable
- Type conversions must go through the official conversion methods
- Maintain backward compatibility when adding new types
- **Validate all numeric conversions for overflow/underflow**
- **Handle NaN and Infinity for floats explicitly**
- **Set reasonable size limits for String/Binary/Array types**

### 2. Error Handling Standards

All errors must flow through the centralized `PlcError` type:

```rust
pub enum PlcError {
    Config(String),
    Runtime(String),
    Protocol(String),
    Block(String),
    Signal(String),
    Validation(String),
    // Feature-gated variants...
}
```

**Production Requirements:**
- Always use `Result<T, PlcError>` for fallible operations
- Never panic in production code - convert to appropriate errors
- Provide descriptive error messages with context
- Use feature gates for error variants specific to optional features
- **Include error codes for machine parsing**
- **Add backtrace information in debug builds**
- **Log errors at appropriate levels (error/warn/info)**
- **Never expose internal implementation details in error messages**

### 3. Signal Bus Integration

The signal bus is the central nervous system of PETRA. All inter-component communication must go through it:

**Production Requirements:**
- Never bypass the signal bus for data exchange
- All signals must have unique names within their scope
- Use atomic operations via DashMap for thread safety
- Respect signal ownership and access patterns
- Signal names should follow the naming convention: `component.subcategory.signal_name`
- **Implement signal access control/permissions**
- **Add signal metadata (units, ranges, descriptions)**
- **Monitor signal bus performance metrics**
- **Implement dead signal detection and cleanup**

### 4. Resource Management

**Production Requirements:**
- **Set explicit resource limits:**
  - Max memory usage per component
  - Max file handles
  - Max network connections
  - Thread pool sizes
- **Implement backpressure mechanisms**
- **Add circuit breakers for external services**
- **Monitor and log resource usage**
- **Graceful degradation under load**

## Production Readiness Standards

### Memory Management

```rust
// ❌ Bad: Unbounded growth
let mut cache = HashMap::new();
cache.insert(key, value); // Grows forever

// ✅ Good: Bounded cache with eviction
use lru::LruCache;
let mut cache = LruCache::new(NonZeroUsize::new(1000).unwrap());
cache.put(key, value);
```

### Connection Management

```rust
// ❌ Bad: No connection pooling
let client = Client::connect(&url)?;

// ✅ Good: Connection pool with limits
let pool = Pool::builder()
    .max_connections(10)
    .connection_timeout(Duration::from_secs(30))
    .build(manager)?;
```

### Timeout Handling

```rust
// ❌ Bad: No timeout
let response = client.request().await?;

// ✅ Good: Explicit timeout
let response = tokio::time::timeout(
    Duration::from_secs(30),
    client.request()
).await??;
```

## Testing Requirements

### Unit Test Standards

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_happy_path() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
    
    #[test]
    fn test_error_condition() {
        // Test specific error scenarios
        let result = function_with_invalid_input();
        assert!(matches!(result, Err(PlcError::Validation(_))));
    }
    
    #[test]
    fn test_edge_cases() {
        // Test boundary conditions
        test_with_empty_input();
        test_with_max_values();
        test_with_min_values();
    }
}
```

### Integration Test Standards

```rust
#[tokio::test]
async fn test_component_integration() {
    // Setup test environment
    let bus = create_test_bus();
    let config = load_test_config();
    
    // Initialize components
    let component = Component::new(config, bus.clone())?;
    component.start().await?;
    
    // Execute test scenario
    bus.set("input", Value::Float(42.0))?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify results
    let output = bus.get("output")?;
    assert_eq!(output, Value::Float(84.0));
    
    // Cleanup
    component.stop().await?;
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_value_serialization(value in any::<Value>()) {
        let serialized = serde_json::to_string(&value)?;
        let deserialized: Value = serde_json::from_str(&serialized)?;
        prop_assert_eq!(value, deserialized);
    }
}
```

## Performance Standards

### Benchmarking Requirements

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_signal_bus(c: &mut Criterion) {
    let bus = SignalBus::new();
    
    c.bench_function("signal_bus_write", |b| {
        b.iter(|| {
            bus.set("test", black_box(Value::Float(42.0)))
        })
    });
    
    c.bench_function("signal_bus_read", |b| {
        bus.set("test", Value::Float(42.0)).unwrap();
        b.iter(|| {
            black_box(bus.get("test"))
        })
    });
}

criterion_group!(benches, benchmark_signal_bus);
criterion_main!(benches);
```

### Performance Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Signal read | < 100ns | 1µs |
| Signal write | < 500ns | 5µs |
| Block execution | < 10µs | 100µs |
| Config parse (1MB) | < 100ms | 1s |
| Startup time | < 1s | 5s |

## Security Requirements

### Input Validation

```rust
pub fn validate_signal_name(name: &str) -> Result<(), PlcError> {
    // Length check
    if name.is_empty() || name.len() > 255 {
        return Err(PlcError::Validation(
            "Signal name must be 1-255 characters".into()
        ));
    }
    
    // Character validation
    if !name.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_') {
        return Err(PlcError::Validation(
            "Signal name contains invalid characters".into()
        ));
    }
    
    // Path traversal prevention
    if name.contains("..") {
        return Err(PlcError::Validation(
            "Signal name cannot contain '..'".into()
        ));
    }
    
    Ok(())
}
```

### Authentication & Authorization

```rust
#[cfg(feature = "security")]
pub async fn authorized_endpoint(
    auth: BearerAuth,
    request: Request,
) -> Result<Response, PlcError> {
    // Verify token
    let claims = verify_jwt(&auth.token())?;
    
    // Check permissions
    if !claims.has_permission("signals:write") {
        return Err(PlcError::Authorization("Insufficient permissions".into()));
    }
    
    // Audit log
    audit_log(&claims.user_id, "signals:write", &request)?;
    
    // Process request
    process_authorized_request(request).await
}
```

## Documentation Standards

### API Documentation

```rust
/// Executes a block with the given inputs and writes outputs to the signal bus.
///
/// # Arguments
///
/// * `bus` - The signal bus for reading inputs and writing outputs
///
/// # Returns
///
/// Returns `Ok(())` if execution succeeds, or an error if:
/// - Required inputs are missing
/// - Input values have invalid types
/// - Block logic encounters an error
///
/// # Example
///
/// ```rust
/// # use petra::{SignalBus, Block, Value};
/// # let bus = SignalBus::new();
/// # let mut block = create_block();
/// bus.set("input1", Value::Float(10.0))?;
/// bus.set("input2", Value::Float(20.0))?;
/// 
/// block.execute(&bus)?;
/// 
/// let output = bus.get("output")?;
/// assert_eq!(output, Value::Float(30.0));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn execute(&mut self, bus: &SignalBus) -> Result<(), PlcError> {
    // Implementation
}
```

### Configuration Documentation

```yaml
# Example: PID Controller Block Configuration
blocks:
  - name: temperature_pid
    type: PID_CONTROLLER
    inputs:
      setpoint: operator.temperature_setpoint  # Target temperature (°C)
      process_value: sensors.tank_temperature  # Current temperature (°C)
    outputs:
      control_output: actuators.heater_power   # Heater power (0-100%)
    parameters:
      kp: 2.0        # Proportional gain
      ki: 0.1        # Integral gain  
      kd: 0.5        # Derivative gain
      min_output: 0.0     # Minimum control output
      max_output: 100.0   # Maximum control output
      sample_time_ms: 100 # Control loop period
```

## Common Integration Patterns

### Adding a New Block Type

1. **Define the block structure:**
```rust
// src/blocks/filters/exponential_filter.rs
pub struct ExponentialFilter {
    name: String,
    input: String,
    output: String,
    alpha: f64,
    last_value: Option<f64>,
}
```

2. **Implement the Block trait:**
```rust
impl Block for ExponentialFilter {
    fn execute(&mut self, bus: &SignalBus) -> Result<(), PlcError> {
        let input = bus.get_float(&self.input)?;
        
        let filtered = match self.last_value {
            Some(last) => self.alpha * input + (1.0 - self.alpha) * last,
            None => input,
        };
        
        self.last_value = Some(filtered);
        bus.set(&self.output, Value::Float(filtered))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn block_type(&self) -> &str {
        "EXPONENTIAL_FILTER"
    }
}
```

3. **Register in the factory:**
```rust
// src/blocks/mod.rs
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>, PlcError> {
    match config.block_type.as_str() {
        "EXPONENTIAL_FILTER" => {
            filters::create_exponential_filter(config)
        }
        // ... other blocks
    }
}
```

4. **Add tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exponential_filter() {
        let bus = create_test_bus();
        let config = BlockConfig {
            name: "test_filter".into(),
            block_type: "EXPONENTIAL_FILTER".into(),
            inputs: hashmap!{"in" => "input_signal"},
            outputs: hashmap!{"out" => "output_signal"},
            parameters: hashmap!{"alpha" => json!(0.5)},
        };
        
        let mut block = create_exponential_filter(&config)?;
        
        // Test filtering
        bus.set("input_signal", Value::Float(100.0))?;
        block.execute(&bus)?;
        
        let output = bus.get_float("output_signal")?;
        assert_eq!(output, 100.0); // First value passes through
    }
}
```

### Adding a New Protocol Driver

1. **Implement the ProtocolDriver trait:**
```rust
// src/protocols/dnp3/mod.rs
pub struct Dnp3Driver {
    config: Dnp3Config,
    connection: Option<Dnp3Connection>,
}

#[async_trait]
impl ProtocolDriver for Dnp3Driver {
    async fn connect(&mut self) -> Result<(), PlcError> {
        // Establish connection
        let conn = Dnp3Connection::new(&self.config).await
            .map_err(|e| PlcError::Protocol(format!("DNP3 connect failed: {}", e)))?;
        self.connection = Some(conn);
        Ok(())
    }
    
    async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>, PlcError> {
        let conn = self.connection.as_ref()
            .ok_or_else(|| PlcError::Protocol("Not connected".into()))?;
            
        // Read values from device
        let mut values = HashMap::new();
        for addr in addresses {
            let val = conn.read(addr).await?;
            values.insert(addr.clone(), convert_dnp3_value(val)?);
        }
        Ok(values)
    }
    
    // ... other trait methods
}
```

## Debugging & Troubleshooting

### Debug Builds

Add debug information to help troubleshooting:

```rust
#[cfg(debug_assertions)]
impl fmt::Debug for MyComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MyComponent")
            .field("state", &self.state)
            .field("buffer_size", &self.buffer.len())
            .field("last_error", &self.last_error)
            .finish()
    }
}
```

### Logging Best Practices

```rust
use log::{debug, error, info, warn};

// Component initialization
info!("Starting {} with config: {:?}", self.name, config);

// Normal operation
debug!("Processing signal: {} = {:?}", signal_name, value);

// Performance warnings
if elapsed > threshold {
    warn!("Slow operation: {} took {:?}", operation, elapsed);
}

// Errors with context
error!("Failed to connect to {}: {}", endpoint, err);
```

### Performance Profiling

```rust
#[cfg(feature = "metrics")]
fn execute_with_metrics(&mut self, bus: &SignalBus) -> Result<(), PlcError> {
    let start = Instant::now();
    
    let result = self.execute_inner(bus);
    
    let elapsed = start.elapsed();
    metrics::histogram!("block_execution_time", elapsed, "block_type" => self.block_type());
    
    if elapsed > Duration::from_millis(10) {
        warn!("Slow block execution: {} took {:?}", self.name(), elapsed);
    }
    
    result
}
```

## Async State Management

When implementing components that need mutable access in async contexts:

1. **Use Arc<Mutex<>> for shared mutable state**: 
   ```rust
   pub struct AsyncComponent {
       state: Arc<Mutex<ComponentState>>,
       // NOT: state: Mutex<ComponentState>  // This won't work across await points
   }
   ```

2. **Lock Duration**: Keep mutex locks as short as possible:
   ```rust
   // ❌ Bad: Long-held lock
   let mut state = self.state.lock().await;
   let result = expensive_operation().await; // Lock held during await!
   state.update(result);
   
   // ✅ Good: Short lock duration
   let result = expensive_operation().await;
   {
       let mut state = self.state.lock().await;
       state.update(result);
   } // Lock released here
   ```

3. **Recursive Async Functions**: Use `BoxFuture` for recursive async:
   ```rust
   use futures::future::BoxFuture;
   use futures::FutureExt;
   
   fn recursive_scan<'a>(&'a self) -> BoxFuture<'a, Result<(), PlcError>> {
       async move {
           // ... logic ...
           if condition {
               self.recursive_scan().await?;
           }
           Ok(())
       }.boxed()
   }
   ```

## Component Lifecycle Patterns

### Proper Shutdown Handling

```rust
pub struct Component {
    shutdown: tokio::sync::watch::Receiver<bool>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Component {
    pub async fn start(&mut self) -> Result<(), PlcError> {
        // Spawn background tasks with shutdown monitoring
        let shutdown_rx = self.shutdown.clone();
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Do work
                    }
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            info!("Shutting down background task");
                            break;
                        }
                    }
                }
            }
        });
        self.tasks.push(task);
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<(), PlcError> {
        // Signal shutdown
        self.shutdown.send(true)?;
        
        // Wait for tasks with timeout
        let timeout = Duration::from_secs(5);
        let start = Instant::now();
        
        for task in self.tasks.drain(..) {
            let remaining = timeout.saturating_sub(start.elapsed());
            match tokio::time::timeout(remaining, task).await {
                Ok(Ok(())) => debug!("Task shut down cleanly"),
                Ok(Err(e)) => error!("Task panicked: {:?}", e),
                Err(_) => {
                    error!("Task shutdown timeout, aborting");
                    task.abort();
                }
            }
        }
        Ok(())
    }
}
```

## Version Compatibility & Migration

### Maintaining Backward Compatibility

```rust
// Configuration versioning
#[derive(Deserialize)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1.0")]
    V1(ConfigV1),
    
    #[serde(rename = "2.0")]
    V2(ConfigV2),
}

impl Config {
    pub fn normalize(self) -> ConfigV2 {
        match self {
            Config::V1(v1) => migrate_v1_to_v2(v1),
            Config::V2(v2) => v2,
        }
    }
}
```

### Deprecation Patterns

```rust
#[deprecated(since = "0.5.0", note = "Use `new_method` instead")]
pub fn old_method(&self) -> Result<(), PlcError> {
    warn!("Called deprecated method old_method");
    self.new_method()
}
```

## Final Production Checklist

Before declaring code production-ready:

### 🚀 Deployment Readiness
- [ ] Dockerfile builds and runs correctly
- [ ] Docker image size is optimized (multi-stage build)
- [ ] Health check endpoint responds correctly
- [ ] Graceful shutdown implemented
- [ ] Signals (SIGTERM, SIGINT) handled properly
- [ ] Configuration can be provided via environment variables
- [ ] Secrets management implemented (no hardcoded credentials)

### 📈 Observability
- [ ] Structured logging implemented
- [ ] Metrics exposed (Prometheus format)
- [ ] Distributed tracing support (if applicable)
- [ ] Health metrics include subsystem status
- [ ] Performance metrics collected
- [ ] Error rates tracked

### 🔧 Operations
- [ ] Runbook documentation exists
- [ ] Common issues and solutions documented
- [ ] Monitoring alerts configured
- [ ] Backup/restore procedures documented
- [ ] Capacity planning guidelines provided
- [ ] Upgrade procedures tested

### 🎯 Quality Gates
- [ ] Code coverage > 80%
- [ ] Zero high/critical security vulnerabilities
- [ ] Performance benchmarks pass
- [ ] Load testing completed
- [ ] Chaos testing performed (if applicable)
- [ ] All CI/CD checks green

---

## Quick Reference

### Command Cheat Sheet

```bash
# Full validation suite
./scripts/petra-dev.sh test full

# Quick pre-commit check
./scripts/petra-dev.sh test quick

# Run specific feature tests
cargo test --features "mqtt,security"

# Check for unwraps
rg "\.unwrap\(\)" src/ --type rust --glob '!*/tests/*'

# Run security audit
cargo audit

# Generate test coverage
cargo tarpaulin --out Html --all-features

# Profile memory usage
valgrind --leak-check=full --show-leak-kinds=all ./target/release/petra

# Benchmark performance
cargo bench --features "optimized"
```

### Common Error Patterns to Avoid

```rust
// ❌ Bad patterns to avoid:
data.unwrap()                           // Use ? or expect with message
thread::sleep(Duration::from_secs(1))   // Use tokio::time::sleep in async
format!("Error: {}", e)                 // Include context
mutex.lock().unwrap()                   // Handle poisoned mutex
Vec::new()                              // Consider with_capacity
loop { /* no break */ }                 // Add timeout/cancellation
panic!("not implemented")               // Return error instead

// ✅ Good patterns to follow:
data.context("Failed to parse data")?
tokio::time::sleep(Duration::from_secs(1)).await
format!("Failed to connect to {}: {}", url, e)
mutex.lock().map_err(|e| PlcError::Runtime(e.to_string()))?
Vec::with_capacity(expected_size)
while !shutdown.load(Ordering::Relaxed) { /* ... */ }
Err(PlcError::Runtime("Not implemented".into()))
```

---

*This document is the authoritative guide for PETRA development. When in doubt, err on the side of safety, reliability, and maintainability. Your code will run in production environments where failures can have real-world consequences.*

*Last updated: December 2024*
