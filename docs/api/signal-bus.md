# Signal Bus API

The **Signal Bus** is the core communication backbone of Petra.  
It provides **thread-safe, high-performance** signal routing between components.

---

## Overview

The Signal Bus relies on a `DashMap` for concurrent access and layers in several optimizations:

- **Lock-free reads** for hot paths  
- **Batch updates** to reduce contention  
- **Signal-metadata caching**  
- **Access-pattern tracking** for adaptive tuning  

---

## Basic Usage

```rust
use petra::{SignalBus, Value};

// Create a new signal bus
let bus = SignalBus::new();

// Write a signal
bus.write_signal("temperature", Value::Float(23.5))?;

// Read a signal
let value = bus.read_signal("temperature")?;

// Subscribe to changes
let mut receiver = bus.subscribe("temperature")?;
while let Some(change) = receiver.recv().await {
    println!("Temperature changed to: {:?}", change.value);
}
````

---

## Advanced Features

### Batch Operations

```rust
// Batch write for efficiency
let updates = vec![
    ("temp1",    Value::Float(23.5)),
    ("temp2",    Value::Float(24.1)),
    ("pressure", Value::Float(101.3)),
];
bus.write_batch(updates)?;

// Batch read
let signals = vec!["temp1", "temp2", "pressure"];
let values  = bus.read_batch(&signals)?;
```

### Access Patterns

```rust
// Get access statistics
let stats = bus.get_access_stats("temperature");
println!("Read count : {}", stats.read_count);
println!("Write count: {}", stats.write_count);
println!("Last access: {:?}", stats.last_access);
```

### Signal Metadata

```rust
// Set metadata
bus.set_metadata("temperature", SignalMetadata {
    unit:        Some("°C".into()),
    description: Some("Reactor core temperature".into()),
    min_value:   Some(Value::Float(0.0)),
    max_value:   Some(Value::Float(100.0)),
    source:      Some("S7-1200".into()),
})?;

// Get metadata
let metadata = bus.get_metadata("temperature")?;
```

---

## Performance Considerations

* **Hot-path optimization:** frequently accessed signals are cached.
* **Batch operations:** prefer batch reads/writes for multiple updates.
* **Subscription filtering:** use wildcards judiciously to avoid overhead.
* **Memory usage:** monitor signal count and prune stale signals.

---

## Error Handling

All Signal Bus functions return `Result<T, PlcError>`:

```rust
match bus.read_signal("temperature") {
    Ok(value) => println!("Temperature: {:?}", value),
    Err(PlcError::SignalNotFound(name)) => {
        println!("Signal {} not found", name)
    }
    Err(e) => println!("Error: {}", e),
}
```

---

## Thread Safety

The Signal Bus is fully thread-safe and can be shared across threads:

```rust
use std::sync::Arc;

let bus = Arc::new(SignalBus::new());

// Clone for another thread
let bus_clone = bus.clone();
tokio::spawn(async move {
    bus_clone.write_signal("status", Value::Bool(true)).unwrap();
});
```

```

