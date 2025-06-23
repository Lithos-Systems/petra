# PETRA

**Programmable Engine for Telemetry, Runtime, and Automation**
A high-performance, modular automation engine built in **Rust**.

---

## Project Structure

This repository is a single-crate Cargo workspace:

```text
Cargo.toml        – crate manifest  
configs/          – example YAML configurations  
src/              – library and binary sources  
tests/            – integration tests  
```

---

## Core Modules

The main library (via `src/lib.rs`) re-exports these modules:

```rust
pub mod error;
pub mod value;
pub mod signal;
pub mod block;
pub mod config;
pub mod engine;
pub mod mqtt;
pub mod twilio;
pub mod twilio_block;

#[cfg(feature = "history")]
pub mod history;

#[cfg(feature = "s7-support")]
pub mod s7;

#[cfg(feature = "advanced-storage")]
pub mod storage;
```

### Key Components

| Module       | Description                                                                                                       |
| ------------ | ----------------------------------------------------------------------------------------------------------------- |
| `value.rs`   | Unified enum for `Bool`, `Int`, and `Float` values, with type-safe conversion methods.                            |
| `signal.rs`  | The **SignalBus** — a thread-safe store (using `dashmap`) for signal state.                                       |
| `block.rs`   | The `Block` trait and factory — defines reusable logic blocks like gates, timers, comparisons, and `TwilioBlock`. |
| `engine.rs`  | PLC-style scan engine — initializes from config, executes blocks, manages state.                                  |
| `mqtt.rs`    | MQTT I/O handler — publishes signal changes and accepts commands.                                                 |
| `twilio.rs`  | Async interface to Twilio API — sends SMS or makes calls from logic.                                              |
| `s7.rs`      | Communicates with **Siemens S7 PLCs** using `rust-snap7`.                                                         |
| `history.rs` | \[Feature: `history`] Logs signals to **Parquet**, supports downsampling/retention.                               |
| `storage/`   | \[Feature: `advanced-storage`] RocksDB, ClickHouse, S3 backends, WAL manager.                                     |

---

## Binary Targets

| Binary           | Description                                                                   |
| ---------------- | ----------------------------------------------------------------------------- |
| `petra`          | Main runtime: loads YAML config, launches scan engine, MQTT, Twilio, S7, etc. |
| `s7_test`        | Tool for testing S7 connectivity.                                             |
| `simple_s7_test` | Minimal S7 interaction demo.                                                  |
| `twilio_test`    | Tool for testing Twilio credentials and simulated SMS/call triggers.          |

---

## Configuration Overview

Petra is configured via YAML. Example:

```yaml
signals:
  - name: "signal_name"
    type: "bool"
    initial: false

blocks:
  - name: "block_name"
    type: "AND"
    inputs:
      in1: "signal1"
    outputs:
      out: "result"

scan_time_ms: 100

mqtt:
  broker_host: "mqtt.lithos.systems"
  broker_port: 1883
  client_id: "petra-01"
  topic_prefix: "petra/plc"
  qos: 1
  publish_on_change: true

s7:
  ip: "192.168.1.100"
  rack: 0
  slot: 2
  poll_interval_ms: 100
  timeout_ms: 5000
  mappings: []

twilio:
  from_number: "+1234567890"
  poll_interval_ms: 1000
  actions: []
```

Environment variables may be used for credentials:

* `MQTT_USERNAME`, `MQTT_PASSWORD`
* `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_FROM_NUMBER`

---

## Build & Run

```bash
# Build the release binary
cargo build --release

# Run with an example configuration
cargo run --release configs/example-mqtt.yaml

# Test S7 connectivity
cargo run --bin s7_test -- --ip 192.168.1.100 connect

# Read a value from S7 PLC
cargo run --bin s7_test -- --ip 192.168.1.100 read -a DB -d 100 -A 0 -t bool -b 0
```

---

## Unique Features

* **PLC Control**: Direct integration with Siemens S7 (via Snap7).
* **MQTT Messaging**: Bi-directional MQTT interface for IoT/SCADA.
* **Twilio Alerts**: Voice and SMS actions triggered by signals.
* **Parquet History Logging**: Optional long-term signal retention.
* **Modular Blocks**: Extend the logic system by implementing the `Block` trait.
* **Prometheus Metrics**: Exposes `/metrics` on port **9090** when running.

---

## Advanced Tools

```bash
# Test Twilio functionality
cargo run --bin twilio_test

# Use twilio_test to simulate signal-driven SMS/call alerts

# Test S7 read/write
cargo run --bin s7_test -- --help
```
