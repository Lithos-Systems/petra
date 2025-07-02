# PETRA Code Integration & Constraint Guidelines

## Overview

PETRA is a highly modular industrial automation system built in Rust with over 80 feature flags. This document defines the constraints, patterns, and best practices for incorporating new code into the system without breaking existing functionality.

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

**Constraints:**
- Never extend the base `Value` enum without the `extended-types` feature flag
- All values must be serializable/deserializable
- Type conversions must go through the official conversion methods
- Maintain backward compatibility when adding new types

### 2. Error Handling Constraints

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

**Constraints:**
- Always use `Result<T, PlcError>` for fallible operations
- Never panic in production code - convert to appropriate errors
- Provide descriptive error messages with context
- Use feature gates for error variants specific to optional features

### 3. Signal Bus Integration

The signal bus is the central nervous system of PETRA. All inter-component communication must go through it:

**Constraints:**
- Never bypass the signal bus for data exchange
- All signals must have unique names within their scope
- Use atomic operations via DashMap for thread safety
- Respect signal ownership and access patterns
- Signal names should follow the naming convention: `component.subcategory.signal_name`

## Feature Flag Constraints

### Feature Dependencies

PETRA uses a hierarchical feature dependency system. When adding features:

1. **Declare dependencies explicitly:**
```toml
[features]
my-feature = ["dependency-feature"]
```

2. **Respect the dependency hierarchy:**
- `jwt-auth` requires `security`
- `twilio` requires both `alarms` and `web`
- `engineering-types` requires `extended-types`
- All storage features require `history` as base

3. **Check mutually exclusive features:**
- Only one monitoring level can be active: `standard-monitoring` OR `enhanced-monitoring`
- Platform-specific features must be properly gated (e.g., `realtime` is Linux-only)

### Feature Organization

Features are organized into logical groups:

1. **Core Features**: `standard-monitoring`, `enhanced-monitoring`, `optimized`, `metrics`, `realtime`
2. **Protocol Features**: `mqtt`, `s7-support`, `modbus-support`, `opcua-support`
3. **Storage Features**: `history`, `advanced-storage`, `compression`, `wal`
4. **Security Features**: `security`, `basic-auth`, `jwt-auth`, `rbac`, `audit`
5. **Type Features**: `extended-types`, `engineering-types`, `quality-codes`, `value-arithmetic`
6. **Validation Features**: `validation`, `regex-validation`, `schema-validation`, `composite-validation`

## Block System Constraints

### Implementing New Blocks

All blocks must implement the core `Block` trait:

```rust
pub trait Block: Send + Sync {
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    fn name(&self) -> &str;
    fn block_type(&self) -> &str;
    
    // Optional methods with default implementations
    fn category(&self) -> &str { "core" }
    fn validate_config(config: &BlockConfig) -> Result<()> { Ok(()) }
    fn initialize(&mut self, config: &BlockConfig) -> Result<()> { Ok(()) }
    fn reset(&mut self) -> Result<()> { Ok(()) }
}
```

**Constraints:**
1. **Thread Safety**: Blocks must be `Send + Sync`
2. **Stateless Execution**: Avoid hidden state between executions
3. **Deterministic Behavior**: Same inputs must produce same outputs
4. **Performance**: Execute method should complete within microseconds
5. **Error Recovery**: Blocks must handle errors gracefully without corrupting state

### Block Factory Registration

New blocks must be registered in the factory system:

```rust
// In src/blocks/mod.rs
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        "MY_BLOCK" => my_module::create_my_block(config),
        // ... other blocks
    }
}
```

### Block Configuration

Blocks are configured via YAML with strict validation:

```yaml
blocks:
  - name: my_block_instance
    type: MY_BLOCK
    inputs:
      in1: signal.path.input1
      in2: signal.path.input2
    outputs:
      out: signal.path.output
    parameters:
      threshold: 10.0
      mode: "fast"
```

**Constraints:**
- Input/output signal paths must exist
- Parameters must be validated in `validate_config`
- Provide sensible defaults for optional parameters
- Document all parameters in block metadata

## Protocol Driver Constraints

### Implementing Protocol Drivers

All protocol drivers must implement the `ProtocolDriver` trait:

```rust
#[async_trait]
pub trait ProtocolDriver: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>>;
    async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()>;
    
    fn is_connected(&self) -> bool;
    fn protocol_name(&self) -> &'static str;
}
```

**Constraints:**
1. **Async Safety**: Use async-trait for async methods
2. **Connection Management**: Handle reconnection logic internally
3. **Error Propagation**: Convert protocol-specific errors to `PlcError::Protocol`
4. **Value Mapping**: Convert protocol data types to PETRA `Value` types
5. **Thread Safety**: Multiple threads may call methods concurrently

### Protocol Integration

Protocols integrate through the `ProtocolManager`:

```rust
let mut manager = ProtocolManager::new(signal_bus);
manager.add_driver("my_protocol".to_string(), Box::new(MyDriver::new(config)));
```

## Validation Framework Constraints

### Custom Validators

Validators must implement the `Validator` trait:

```rust
pub trait Validator: Send + Sync {
    fn validate(&self, value: &Value) -> ValidationResult;
    fn name(&self) -> &str;
}
```

**Validation Rules:**
1. Validators must be stateless
2. Return detailed error messages with paths
3. Support warning-level issues with appropriate feature flag
4. Chain validators using `CompositeValidator` for complex rules

## Module Integration Patterns

### 1. Configuration Integration

New modules must integrate with the YAML configuration system:

```rust
#[derive(Deserialize, Serialize)]
pub struct MyModuleConfig {
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    
    // Feature-gated fields
    #[cfg(feature = "my-feature")]
    pub advanced_option: String,
}
```

### 2. Signal Bus Patterns

Standard patterns for signal bus interaction:

```rust
// Reading signals
let value = bus.get("signal.name")?;
let bool_value = bus.get_bool("signal.name")?;

// Writing signals
bus.set("signal.name", Value::Float(42.0))?;

// Atomic updates
bus.update("counter", |old| {
    match old {
        Some(Value::Integer(n)) => Value::Integer(n + 1),
        _ => Value::Integer(1),
    }
})?;
```

### 3. Lifecycle Management

Components should follow the standard lifecycle:

```rust
pub trait Component {
    fn initialize(&mut self, config: &Config) -> Result<()>;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
}
```

## Testing Constraints

### Unit Testing

All new code must include comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Test happy path
    }
    
    #[test]
    fn test_error_conditions() {
        // Test error handling
    }
    
    #[cfg(feature = "my-feature")]
    #[test]
    fn test_feature_specific() {
        // Test feature-gated functionality
    }
}
```

### Integration Testing

Integration tests should use the test utilities:

```rust
use petra::test_utils::{create_test_bus, MockBlock};

#[tokio::test]
async fn test_integration() {
    let bus = create_test_bus();
    // Test component integration
}
```

## Performance Constraints

### Real-time Considerations

When `realtime` feature is enabled:

1. **Avoid Allocations**: Pre-allocate buffers
2. **Bounded Execution Time**: Operations must complete in predictable time
3. **Lock-free Algorithms**: Prefer atomic operations over mutexes
4. **Minimize Syscalls**: Batch operations where possible

### Monitoring Integration

With `enhanced-monitoring` feature:

```rust
#[cfg(feature = "enhanced-monitoring")]
fn last_execution_time(&self) -> Option<Duration> {
    self.last_execution
}

#[cfg(feature = "enhanced-monitoring")]
fn state(&self) -> HashMap<String, Value> {
    // Return current state for debugging
}
```

## Security Constraints

### Authentication & Authorization

When security features are enabled:

1. **Always validate permissions** before executing sensitive operations
2. **Audit log all security events** when audit feature is enabled
3. **Use constant-time comparisons** for security-sensitive data
4. **Validate all inputs** before processing

### Data Validation

With validation features:

```rust
let validator = RangeValidator::new("temperature")
    .min(-50.0)
    .max(150.0);

let result = validator.validate(&value)?;
if !result.valid {
    // Handle validation errors
}
```

## Version Compatibility

### Breaking Changes

Avoid breaking changes by:

1. **Never removing public APIs** - deprecate instead
2. **Maintaining serialization compatibility** for stored data
3. **Preserving configuration schema** - only add optional fields
4. **Supporting old protocol versions** during transitions

### Migration Support

Provide migration utilities for breaking changes:

```rust
pub fn migrate_v1_to_v2(old_config: &ConfigV1) -> Result<ConfigV2> {
    // Migration logic
}
```

## Checklist for New Code Integration

Before submitting new code, ensure:

- [ ] Code follows Rust idioms and clippy lints pass
- [ ] All public APIs are documented with examples
- [ ] Feature flags are properly configured with dependencies
- [ ] Unit tests cover happy path and error cases
- [ ] Integration tests verify component interaction
- [ ] Performance impact is measured and acceptable
- [ ] Security implications are considered and addressed
- [ ] Configuration changes are backward compatible
- [ ] Error messages are descriptive and actionable
- [ ] Code is thread-safe and handles concurrent access
- [ ] Documentation is updated (README, API docs, examples)
- [ ] Example configurations demonstrate new features

## Common Pitfalls to Avoid

1. **Direct Signal Access**: Never access signals outside the signal bus
2. **Blocking Operations**: Use async for I/O operations
3. **Panic in Production**: Always return errors instead
4. **Feature Flag Leakage**: Keep feature-specific code properly gated
5. **Global State**: Use dependency injection instead
6. **Tight Coupling**: Design components to be loosely coupled
7. **Resource Leaks**: Implement proper cleanup in destructors
8. **Race Conditions**: Use appropriate synchronization primitives
9. **Configuration Validation**: Validate at load time, not runtime
10. **Undocumented Behavior**: Document all non-obvious behavior

By following these constraints and guidelines, new code can be safely integrated into PETRA while maintaining system stability, performance, and maintainability.
