# Petra Test Plan

---

## Unit Tests (in each module)

- [x] Value type conversions  
- [x] Signal-bus basic operations  
- [x] Block-logic execution  
- [x] Config parsing & validation  
- [ ] Validation-rules engine  
- [ ] Security authentication  
- [ ] Alarm conditions  

---

## Integration Tests

### 1. End-to-End Workflow&nbsp;(`tests/integration/e2e_test.rs`)

- [ ] Load config → start engine → process signals → verify outputs  
- [ ] MQTT publish / subscribe with real broker  
- [ ] S7 PLC simulation  
- [ ] Storage write and query  

### 2. Performance Tests&nbsp;(`tests/integration/performance_test.rs`)

- [ ] 10 000 signals at 50 ms scan time  
- [ ] Jitter &lt; 5 % of scan time  
- [ ] Memory usage &lt; 512 MB for 10 k signals  
- [ ] Storage throughput &gt; 100 k values / sec  

### 3. Resilience Tests&nbsp;(`tests/integration/resilience_test.rs`)

- [ ] MQTT-broker disconnect / reconnect  
- [ ] S7-PLC timeout handling  
- [ ] Storage fail-over (local → remote)  
- [ ] Signal-bus overflow handling  

### 4. Security Tests&nbsp;(`tests/integration/security_test.rs`)

- [ ] Authentication failures  
- [ ] Rate-limiting enforcement  
- [ ] Input validation (SQL-injection, path traversal)  
- [ ] Audit-log generation  

---

## Property-Based Tests

- [ ] Signal-bus concurrent operations  
- [ ] Config generation & validation  
- [ ] Block execution with random inputs  
- [ ] Storage compression / decompression  

---

## Benchmarks

- [ ] Scan-cycle performance  
- [ ] Signal-bus operations  
- [ ] Block execution  
- [ ] Storage write throughput  
- [ ] MQTT message throughput  

---

## Create Integration Test Framework

`tests/common/mod.rs`:

```rust
use petra::{Config, Engine, SignalBus, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, Instant};

pub struct TestHarness {
    pub engine: Engine,
    pub bus: Arc<SignalBus>,
}

impl TestHarness {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = Engine::new(config)?;
        let bus = engine.signal_bus();
        Ok(Self { engine, bus })
    }

    pub async fn run_for(&mut self, duration: Duration) {
        let handle = tokio::spawn({
            let mut engine = self.engine.clone();
            async move { engine.run().await }
        });

        sleep(duration).await;
        handle.abort();
    }

    pub fn set_signal(&self, name: &str, value: Value) -> petra::Result<()> {
        self.bus.write_signal(name, value)
    }

    pub fn get_signal(&self, name: &str) -> petra::Result<Value> {
        self.bus.read_signal(name)
    }

    pub async fn wait_for_condition<F>(
        &self,
        name: &str,
        condition: F,
        timeout: Duration,
    ) -> bool
    where
        F: Fn(&Value) -> bool,
    {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Ok(value) = self.get_signal(name) {
                if condition(&value) {
                    return true;
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
        false
    }
}

pub fn test_config() -> Config {
    Config {
        signals: vec![
            petra::config::SignalConfig {
                name: "test_input".into(),
                signal_type: "float".into(),
                initial: Some(petra::config::InitialValue::Float(0.0)),
                ..Default::default()
            },
            petra::config::SignalConfig {
                name: "test_output".into(),
                signal_type: "float".into(),
                initial: Some(petra::config::InitialValue::Float(0.0)),
                ..Default::default()
            },
        ],
        blocks: vec![],
        scan_time_ms: 50,
        ..Default::default()
    }
}
````

```
