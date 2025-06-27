# PETRA

**Programmable Engine for Telemetry, Runtime, and Automation**
A high-performance, production-ready automation engine built in **Rust** with advanced industrial connectivity, alarm management, and enterprise data storage capabilities.

---

## Project Structure

```text
Cargo.toml                  – crate manifest with comprehensive feature flags
configs/                    – example YAML configurations for various use cases
src/                        – library and binary sources
  ├── bin/                  – executable binaries for testing and utilities
  ├── storage/              – enterprise storage backends (local, remote, WAL)
  └── *.rs                  – core modules
petra-designer/             – React-based visual configuration designer
tests/                      – integration and security tests
benches/                    – performance benchmarks
data/                       – runtime data directories (created automatically)
.github/workflows/          – CI/CD pipelines with smoke tests
docker/                     – containerization and deployment configs
```

---

## Core Modules

The main library provides these production-ready modules:

```rust
pub mod error;           // Comprehensive error handling
pub mod value;           // Type-safe value system (Bool, Int, Float)
pub mod signal;          // Thread-safe signal bus with DashMap
pub mod block;           // Extensible block system with 15+ built-in types
pub mod config;          // YAML configuration with validation
pub mod engine;          // Real-time scan engine with jitter monitoring
pub mod mqtt;            // Full MQTT integration with subscriptions
pub mod twilio;          // SMS/Voice alerting with escalation
pub mod alarms;          // Advanced alarm management with contacts
pub mod security;        // Authentication, authorization, audit logging
pub mod validation;      // Input validation and sanitization

#[cfg(feature = "history")]
pub mod history;         // Parquet-based historical data logging

#[cfg(feature = "s7-support")]
pub mod s7;              // Siemens S7 PLC communication

#[cfg(feature = "advanced-storage")]
pub mod storage;         // Enterprise storage with ClickHouse, S3, WAL

#[cfg(feature = "opcua-support")]
pub mod opcua;           // OPC-UA server for standards compliance

#[cfg(feature = "modbus-support")]
pub mod modbus;          // Modbus TCP/RTU drivers
```

### Enhanced Components

| Module           | Description                                                                                                                |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `signal.rs`      | **SignalBus** — thread-safe with DashMap, hot caching, access pattern optimization                                       |
| `engine.rs`      | **Enhanced scan engine** — jitter monitoring, scan variance metrics, overrun detection                                   |
| `alarms.rs`      | **Alarm management** — escalation chains, work hours, acknowledgment, severity levels                                    |
| `mqtt.rs`        | **Enhanced MQTT** — subscriptions, JSON path extraction, wildcard matching, authentication                               |
| `twilio.rs`      | **Advanced alerting** — TwiML support, escalation levels, cooldowns, result tracking                                     |
| `storage/`       | **Multi-tier storage** — local-first/remote-first strategies, automatic failover, compaction                            |
| `security.rs`    | **Security framework** — role-based access, signed configs, audit trails, TLS                                           |

---

## Binary Targets

| Binary                | Description                                                                                  |
| --------------------- | -------------------------------------------------------------------------------------------- |
| `petra`               | **Main runtime**: loads YAML config, runs all subsystems with graceful shutdown            |
| `petra_dashboard`     | **Real-time GUI**: egui-based dashboard with live signals, plots, and controls             |
| `s7_test`             | **S7 PLC testing tool**: comprehensive CLI for connection, read, write, monitor operations  |
| `simple_s7_test`      | **Basic S7 connectivity**: minimal connection verification tool                              |
| `twilio_test`         | **Twilio integration tester**: test SMS/voice with signal triggers and config files        |
| `storage_test`        | **Storage validation**: generates test data, validates Parquet output with metrics         |
| `parquet_viewer`      | **Data analysis tool**: view, export, analyze historical Parquet files with statistics     |
| `mqtt_publisher`      | **MQTT test publisher**: simulate sensor data with configurable topics and patterns        |
| `generate_schema`     | **JSON Schema generator**: auto-generate validation schemas from Rust types                |

---

## Visual Configuration Designer

**Petra Designer** - React-based visual configuration tool:

```bash
cd petra-designer
npm install
npm run dev
```

**Features:**
- Drag-and-drop node editor with 8+ node types
- Real-time YAML generation and validation
- Type-safe connection validation
- Import/export configurations
- Built-in examples and templates
- Keyboard shortcuts for productivity

**Node Types:**
- **Signal**: Input/output signals with type validation
- **Block**: Logic blocks (AND, OR, timers, comparisons, PID)
- **Alarm**: Monitoring with severity levels and escalation
- **Contact**: Alert recipients with work hours and preferences
- **Twilio**: SMS/voice alerts with TwiML support
- **Email**: SMTP-based email notifications
- **MQTT**: Bi-directional MQTT communication
- **S7**: Siemens PLC integration with optimized mappings

---

## Configuration System

### Enhanced YAML Configuration

```yaml
# Signal definitions with validation
signals:
  - name: "temperature"
    type: "float"
    initial: 20.0
  - name: "system_healthy"
    type: "bool"
    initial: true

# Logic blocks (15+ types available)
blocks:
  - name: "temp_alarm"
    type: "GT"
    inputs:
      in1: "temperature"
      in2: "75.0"
    outputs:
      out: "high_temp_alarm"
  
  - name: "data_gen"
    type: "DATA_GENERATOR"
    inputs:
      enable: "system_healthy"
    outputs:
      sine_out: "sensor_simulation"
      count_out: "sample_count"
    params:
      frequency: 1.0
      amplitude: 10.0

# Advanced alarm management
alarms:
  alarms:
    - id: "temp_critical"
      name: "Critical Temperature"
      signal: "temperature"
      condition: "above"
      setpoint: 80.0
      severity: "critical"
      delay_seconds: 10
      repeat_interval_seconds: 300
      message_template: "CRITICAL: {name} is {value}°C (limit: {setpoint}°C)"
      require_acknowledgment: true
      auto_reset: false
      
  contacts:
    - id: "operator"
      name: "Shift Operator"
      email: "operator@company.com"
      phone: "+1234567890"
      preferred_method: "sms"
      priority: 1
      escalation_delay_seconds: 300
      work_hours_only: false
      
  escalation_chains:
    temp_critical: ["operator", "supervisor", "manager"]

# Enhanced MQTT with subscriptions
mqtt:
  broker_host: "mqtt.lithos.systems"
  broker_port: 1883
  client_id: "petra-01"
  topic_prefix: "petra/plc"
  username: "${MQTT_USERNAME}"
  password: "${MQTT_PASSWORD}"
  publish_on_change: true
  subscriptions:
    - topic: "sensors/pressure/value"
      signal: "external_pressure"
      data_type: "float"
    - topic: "sensors/status/json"
      signal: "device_temp"
      value_path: "temperature.value"
      data_type: "float"

# Multi-tier enterprise storage
storage:
  strategy: "local_first"  # local_first, remote_first, parallel
  local:
    data_dir: "./data/local"
    max_file_size_mb: 100
    compression: "zstd"
    retention_days: 7
    compact_after_hours: 24
  remote:
    type: "clickhouse"
    url: "http://clickhouse:8123"
    database: "petra_timeseries"
    username: "petra"
    password: "${CLICKHOUSE_PASSWORD}"
    batch_size: 10000
    async_insert: true
  wal:
    wal_dir: "./data/wal"
    sync_on_write: true
    retention_hours: 48

# S7 PLC with optimized mappings
s7:
  ip: "192.168.1.100"
  rack: 0
  slot: 2
  poll_interval_ms: 100
  timeout_ms: 5000
  mappings:
    - signal: "motor_running"
      area: "DB"
      db_number: 100
      address: 0
      data_type: "bool"
      bit: 0
      direction: "read"

# Twilio with advanced features
twilio:
  from_number: "+1987654321"
  status_callback_url: "https://webhook.com/twilio-status"
  actions:
    - name: "emergency_alert"
      trigger_signal: "emergency_stop"
      action_type: "call"
      to_number: "+1234567890"
      content: |
        <Response>
          <Say voice="alice">Emergency stop activated. Immediate attention required.</Say>
          <Gather timeout="10" numDigits="1">
            <Say>Press 1 to acknowledge.</Say>
          </Gather>
        </Response>
      cooldown_seconds: 300

# Security configuration
security:
  enable_audit_logging: true
  max_failed_auth_attempts: 5
  session_timeout_minutes: 30
  require_tls: true
  allowed_cipher_suites: ["TLS_AES_256_GCM_SHA384"]

# Engine optimization
scan_time_ms: 100
```

---

## Build & Run

### Quick Start with Docker

```bash
# Clone repository
git clone https://github.com/your-org/petra
cd petra

# Quick start with Docker Compose
chmod +x quick-start.sh
./quick-start.sh

# Access services
# - MQTT: localhost:1883
# - ClickHouse: http://localhost:8123
# - Metrics: http://localhost:9090/metrics
```

### Development Build

```bash
# Standard build with default features
cargo build --release

# Full enterprise build
cargo build --release --features advanced-storage,security,opcua-support

# Minimal build (no PLC drivers)
cargo build --release --no-default-features

# Run with configuration
cargo run --release -- configs/example-mqtt.yaml

# Run with all features
cargo run --release --features advanced-storage -- configs/production-clickhouse.yaml
```

### Testing & Utilities

```bash
# === S7 PLC Testing ===
# Test connectivity
cargo run --bin s7_test -- --ip 192.168.1.100 connect

# Read specific values
cargo run --bin s7_test -- --ip 192.168.1.100 read \
  --area DB --db 100 --address 0 --data-type bool --bit 0

# Write values
cargo run --bin s7_test -- --ip 192.168.1.100 write \
  --area DB --db 100 --address 4 --data-type real --value 25.5

# Monitor continuously
cargo run --bin s7_test -- --ip 192.168.1.100 monitor \
  --config configs/s7-example.yaml

# Get PLC info
cargo run --bin s7_test -- --ip 192.168.1.100 info

# === Twilio Testing ===
# Send test SMS
cargo run --bin twilio_test sms \
  --to "+1234567890" --message "Test from Petra"

# Make test call
cargo run --bin twilio_test call \
  --to "+1234567890" --message "Test voice call"

# Test with signal triggers
cargo run --bin twilio_test signal \
  --config configs/twilio-example.yaml \
  --signal "high_temp_alarm" --value "true"

# === MQTT Testing ===
# Publish test data
cargo run --bin mqtt_publisher sensors/pressure/value 100

# === Storage Testing ===
# Run storage validation
cargo run --bin storage_test

# === Data Analysis ===
# List Parquet files
cargo run --bin parquet_viewer list --dir ./data/storage_test

# View file info
cargo run --bin parquet_viewer info data/petra_123.parquet

# Show data samples
cargo run --bin parquet_viewer show data/petra_123.parquet \
  --rows 20 --signal temperature

# Export to CSV
cargo run --bin parquet_viewer export data/petra_123.parquet \
  --output analysis.csv

# View statistics
cargo run --bin parquet_viewer stats ./data/history/

# === Schema Generation ===
# Generate JSON schema for validation
cargo run --bin generate_schema --features json-schema

# === Visual Designer ===
# Launch visual configuration designer
cd petra-designer
npm run dev
# Access at http://localhost:3000
```

### Performance & Monitoring

```bash
# Run benchmarks
cargo bench

# Enable detailed metrics
RUST_LOG=petra=debug cargo run --release -- config.yaml

# Memory profiling (with pprof feature)
cargo run --release --features pprof -- config.yaml

# Monitor with custom scan times
cargo run --release -- config.yaml --scan-time 50
```

---

## Feature Flags

| Feature              | Description                                                                        |
| -------------------- | ---------------------------------------------------------------------------------- |
| `s7-support`         | Siemens S7 PLC communication via rust-snap7                                        |
| `history`            | Parquet-based historical data logging with retention                               |
| `advanced-storage`   | Enterprise storage: ClickHouse, S3, RocksDB WAL, failover                          |
| `opcua-support`      | OPC-UA server for standards compliance                                             |
| `modbus-support`     | Modbus TCP/RTU drivers with tokio-modbus                                           |
| `security`           | Authentication, signed configs, TLS, audit logging                                 |
| `json-schema`        | JSON schema generation and validation                                              |

### Build Examples

```bash
# Production build with all enterprise features
cargo build --release --features advanced-storage,security,opcua-support

# Edge device build (minimal footprint)
cargo build --release --no-default-features --features mqtt

# Development build with validation
cargo build --features json-schema,advanced-storage

# Containerized build
docker build -f docker/base/Dockerfile -t petra:latest .
```

---

## Production Features

### **Industrial Connectivity**
* **Siemens S7 PLCs**: Optimized communication with S7-300/400/1200/1500 series
* **MQTT Integration**: Bi-directional with subscriptions, wildcards, authentication
* **Modbus Support**: RS485 and TCP with multiple device support
* **OPC-UA Server**: Standards-compliant server with security policies

### **Advanced Alarm Management**
* **Escalation Chains**: Multi-level contact notification with delays
* **Work Hours**: Contact filtering based on schedules and timezones
* **Severity Levels**: Info, Warning, Critical, Emergency with priority handling
* **Acknowledgment**: Operator acknowledgment with audit trails
* **Message Templates**: Dynamic content with signal value substitution

### **Enterprise Data Management**
* **Multi-tier Storage**: Local-first, remote-first, or parallel write strategies
* **ClickHouse Integration**: High-performance analytics with materialized views
* **S3 Archival**: Automated lifecycle management and compression
* **Write-Ahead Log**: RocksDB-based WAL for guaranteed data durability
* **Compression**: ZSTD, LZ4, Snappy with configurable levels

### **Security & Compliance**
* **Signed Configurations**: Ed25519 signatures for tamper protection
* **Role-Based Access**: Operator, Engineer, Administrator roles
* **Audit Logging**: Comprehensive security event tracking
* **TLS Encryption**: Configurable cipher suites and certificate management

### **Real-time Engine**
* **Jitter Monitoring**: Scan variance tracking with configurable thresholds
* **15+ Logic Blocks**: AND, OR, timers, PID, comparisons, data generators
* **Signal Optimization**: Hot caching for frequently accessed signals
* **Prometheus Metrics**: Production-ready monitoring on port 9090

### **Development Tools**
* **Visual Designer**: React-based drag-and-drop configuration builder
* **Schema Validation**: Auto-generated JSON schemas for config validation
* **Performance Benchmarks**: Criterion-based performance testing suite
* **Integration Tests**: Comprehensive CI/CD with smoke tests

---

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   S7 PLCs       │────│                  │────│   MQTT Brokers  │
│   Modbus RTUs   │    │                  │    │ (Authenticated) │
└─────────────────┘    │   PETRA ENGINE   │    └─────────────────┘
                       │                  │
┌─────────────────┐    │  - Signal Bus    │    ┌─────────────────┐
│ Twilio/Email    │────│  - Alarm Manager │────│   ClickHouse    │
│ Alerts          │    │  - Logic Engine  │    │   Time-series   │
└─────────────────┘    │  - Security      │    └─────────────────┘
                       │                  │
┌─────────────────┐    │                  │    ┌─────────────────┐
│ Visual Designer │────│                  │────│ Parquet Files   │
│ (Web UI)        │    └──────────────────┘    │ + S3 Archive    │
└─────────────────┘                            └─────────────────┘
                                │
                       ┌──────────────────┐
                       │   OPC-UA Server  │
                       │   (Standards)    │
                       └──────────────────┘
```

---

## Performance Characteristics

* **Scan Performance**: 10,000+ signals at 50ms scan time with <1ms jitter
* **MQTT Throughput**: 10,000+ messages/second with batching and QoS 2
* **Storage Rate**: 1GB+ Parquet files/hour with ZSTD compression
* **S7 Communication**: <10ms read/write latency with bulk operations
* **Memory Usage**: <512MB for 10,000-signal configuration
* **Alarm Processing**: <1ms latency for condition evaluation and escalation

---

## Example Configurations

### **Complete Industrial SCADA**
```yaml
# configs/industrial-scada.yaml - Full production setup
signals: [temperature, pressure, motor_speeds, production_counts]
blocks: [safety_interlocks, production_logic, efficiency_calculations]
alarms: [critical_faults, maintenance_alerts, quality_deviations]
s7: {multiple_plcs_with_redundancy}
mqtt: {scada_integration_with_historians}
storage: {clickhouse_with_s3_backup}
security: {rbac_with_audit_trails}
```

### **Edge IoT Gateway**
```yaml
# configs/edge-gateway.yaml - Lightweight edge deployment
mqtt: {encrypted_uplink_to_cloud}
storage: {local_buffer_with_sync}
alarms: {cellular_sms_alerts}
security: {signed_configs_tamper_protection}
```

### **Building Automation**
```yaml
# configs/building-automation.yaml - HVAC and energy management
blocks: [pid_controllers, schedule_logic, energy_optimization]
modbus: {hvac_equipment_integration}
alarms: {maintenance_scheduling, energy_alerts}
opcua: {bms_standards_compliance}
```

---

## CI/CD & DevOps

### **GitHub Actions Pipeline**
- **Matrix Testing**: Ubuntu, Windows, macOS across Rust stable/beta
- **Security Audits**: cargo-audit, dependency scanning
- **Smoke Tests**: End-to-end validation with real MQTT/ClickHouse
- **Performance Regression**: Criterion benchmarks with historical comparison
- **Multi-arch Builds**: x86_64, ARM64, ARMv7 Docker images

### **Container Deployment**
```bash
# Full stack with dependencies
docker-compose up -d

# Production deployment with secrets
docker run -e MQTT_PASSWORD=secret \
  -e CLICKHOUSE_PASSWORD=secret \
  -v ./configs:/app/configs \
  -v ./data:/app/data \
  petra:latest /app/configs/production.yaml

# Kubernetes deployment
kubectl apply -f k8s/petra-deployment.yaml
```

---

## Contributing

Petra is licensed under **AGPL-3.0-or-later**. Priority contribution areas:

### **High Impact**
- **Additional PLC Drivers**: EtherNet/IP, BACnet, Profinet
- **Enhanced Alarming**: SMS gateways beyond Twilio, voice synthesis
- **Advanced Blocks**: PID auto-tuning, statistical functions, ML inference
- **Security Hardening**: Hardware security modules, certificate management

### **Storage & Analytics**
- **Time-series Backends**: InfluxDB, TimescaleDB, Prometheus remote write
- **Data Pipelines**: Apache Kafka integration, real-time analytics
- **Compression**: Additional algorithms, adaptive compression selection

### **Developer Experience**
- **Visual Designer**: Advanced block library, simulation mode, debugging
- **Documentation**: Interactive tutorials, video guides, best practices
- **Testing**: Property-based testing, chaos engineering, load testing

---

## License

```
Lithos Systems
PETRA - Programmable Engine for Telemetry, Runtime, and Automation
Copyright (C) 2024

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.
```

See `LICENSE` file for full AGPL-3.0 terms.

---

## Support & Community

- **Issues**: GitHub Issues for bugs and feature requests
- **Discussions**: Discord Discussions for Community Support
- **Enterprise Support**: Contact nathan@lithos.systems for commercial licensing and support
```
