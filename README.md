# PETRA
#### Programmable Engine for Telemetry, Runtime, and Automation

Petra is a programmable automation engine implemented in Rust. The project is structured as a Cargo workspace with a single crate (petra). Key directories:

Cargo.toml        – crate manifest
configs/          – example YAML configurations
src/              – library and binary sources
tests/            – integration tests

## Core Modules

The main library exposes functionality through src/lib.rs, which re-exports the major modules:

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

## Important components:

    Value (src/value.rs) – enum wrapper for Bool, Int, and Float signals with conversion helpers.

    SignalBus (src/signal.rs) – thread‑safe store for signal values using dashmap.

    Blocks (src/block.rs) – trait Block plus built‑in blocks (logic gates, timers, latches, comparisons) and a factory to create them from YAML config. A notable block is TwilioBlock for SMS/call actions.

    Engine (src/engine.rs) – drives the PLC scan cycle: initializes signals from configuration, executes blocks each cycle, and reports statistics.

    MQTT Handler (src/mqtt.rs) – publishes signal updates and accepts commands via MQTT.

    Twilio Connector (src/twilio.rs) – asynchronous integration with the Twilio API, enabling message/voice actions triggered by signals.

    S7 Connector (src/s7.rs) – communicates with Siemens S7 PLCs via rust-snap7.

    History Manager (src/history.rs) – optional feature to log signal values to Parquet files with downsampling and retention.

    Advanced Storage (src/storage/) – optional modules for write‑ahead logging (RocksDB), local Parquet storage, remote backends (ClickHouse, S3), and a manager orchestrating them.

## Binary Targets

The crate defines multiple binaries:

    petra (src/main.rs) – main application. Loads a YAML configuration, starts the engine, MQTT handler, Twilio connector, optional S7 connector, and history manager.

    s7_test, simple_s7_test – utilities for testing S7 PLC connectivity.

    twilio_test – CLI tool for sending test SMS/voice calls or running signal-based triggers.

## Configuration

Configurations are YAML files (see configs/). Example snippet from README.md shows typical fields:

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

s7:  # optional
  ip: "192.168.1.100"
  rack: 0
  slot: 2
  poll_interval_ms: 100
  timeout_ms: 5000
  mappings: []

twilio:  # optional
  from_number: "+1234567890"
  poll_interval_ms: 1000
  actions: []


# Build Petra
cargo build --release

# Run with example MQTT configuration
cargo run --release configs/example-mqtt.yaml

# Test S7 connection
cargo run --bin s7_test -- --ip 192.168.1.100 connect

# Read a value
cargo run --bin s7_test -- --ip 192.168.1.100 read -a DB -d 100 -A 0 -t bool -b 0

## Uniqueness

Petra combines multiple industrial automation features in a single Rust-based engine:

    Direct PLC Control – communicates with Siemens S7 PLCs via rust-snap7.

    MQTT Integration – exposes a command/status interface for IoT and SCADA use.

    Twilio Actions – send SMS or voice notifications triggered by logic blocks or connector rules.

    Historical Data Logging – optional Parquet writer with configurable downsampling and retention.

    Extensibility – new block types can be implemented by defining a struct that implements the Block trait and adding a case in create_block().

## How to Use

    Build and run using one of the example configuration files:

cargo build --release
cargo run --release configs/example-mqtt.yaml

Configure signals, logic blocks, MQTT, Twilio, and optional S7 settings in a YAML file similar to those in configs/.

Environment variables may supply credentials for MQTT or Twilio (MQTT_USERNAME, MQTT_PASSWORD, TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_FROM_NUMBER).

## Advanced tools:

    Use cargo run --bin twilio_test to verify Twilio credentials or simulate signal triggers.

    Use cargo run --bin s7_test to connect to and test an S7 PLC.

## Metrics – the main binary exports Prometheus metrics on port 9090 (/metrics endpoint) when running.

```
