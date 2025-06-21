# PETRA
#### Programmable Engine for Telemetry, Runtime, and Automation

## Features

- **Entirely Rust-Based** - All the modern benefits of Rust to modernize legacy systems
- **Rock-Solid Reliability** - Comprehensive error handling
- **Built for Pure Function** - No nonsense, no fluff
- **S7 PLC Integration** - Direct communication with Siemens S7 PLCs via snap7
- **MQTT Support** - Built-in MQTT client for IoT integration
    
## Quick Start

1. **Build Petra**
   ```bash
   cargo build --release
   ```
2. **Start Test Environment** (optional - for MQTT testing)
   ```bash
   cd node-red
   docker-compose up -d
   ```
3. **Run Petra**
   ```bash
   cargo run --release configs/example-mqtt.yaml
   ```

## S7 PLC Integration

Petra now supports direct communication with Siemens S7 PLCs using the snap7 library.

### S7 Test Tool

Test your PLC connection:
```bash
# Test connection
cargo run --bin s7_test -- --ip 192.168.1.100 connect

# Get PLC info
cargo run --bin s7_test -- --ip 192.168.1.100 info

# Read a value
cargo run --bin s7_test -- --ip 192.168.1.100 read -a DB -d 100 -A 0 -t bool -b 0

# Write a value
cargo run --bin s7_test -- --ip 192.168.1.100 write -a DB -d 101 -A 2 -t int -v 1500

# Monitor values from config
cargo run --bin s7_test -- --ip 192.168.1.100 monitor -c configs/s7-example.yaml
```

### S7 Configuration

Add S7 configuration to your YAML file:

```yaml
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
    
    - signal: "motor_speed"
      area: "DB"
      db_number: 101
      address: 2
      data_type: "int"
      direction: "write"
```

### Supported S7 Data Types
- `bool` - Boolean (single bit)
- `byte` - Unsigned 8-bit
- `word` - Unsigned 16-bit
- `int` - Signed 16-bit
- `dword` - Unsigned 32-bit
- `dint` - Signed 32-bit
- `real` - 32-bit floating point

### Supported Memory Areas
- `DB` - Data Blocks
- `I` - Inputs
- `Q` - Outputs
- `M` - Markers/Flags
- `C` - Counters
- `T` - Timers

## MQTT Topics

Commands (subscribe)
- `petra/plc/cmd` - Send commands to PLC

Status (publish)
- `petra/plc/status` - Online/offline status
- `petra/plc/signals/{name}` - Individual signal values
- `petra/plc/signals` - All signals
- `petra/plc/stats` - Engine statistics

### MQTT Message Examples

Set Signal
```json
{
  "type": "SetSignal",
  "name": "start_button",
  "value": true
}
```

Get Signal
```json
{
  "type": "GetSignal",
  "name": "motor_run"
}
```

Get All Signals
```json
{
  "type": "GetAllSignals"
}
```

## Configuration

```yaml
signals:
  - name: "signal_name"
    type: "bool"    # bool, int, float
    initial: false

blocks:
  - name: "block_name"
    type: "AND"     # See available blocks
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

s7:  # Optional - only if using S7 PLC
  ip: "192.168.1.100"
  rack: 0
  slot: 2
  mappings: []
```

## Available Blocks

- **Logic**: AND, OR, NOT
- **Timers**: TON (Timer On Delay)
- **Edge Detection**: R_TRIG (Rising Edge)
- **Memory**: SR_LATCH (Set-Reset Latch)
- **Comparison**: GT (Greater Than), LT (Less Than)

## Production Deployment

- Use `--release` build
- Set `RUST_LOG=petra=info`
- Use a systemd service or supervisor
- Monitor MQTT status topic
- For S7 PLCs: ensure network connectivity and proper rack/slot configuration

## Adding Blocks

1. Add a struct in `src/block.rs` implementing `Block`
2. Add a case in `create_block()`
3. Done!

## Dependencies

- Rust 1.70+
- snap7 library (for S7 support)
- MQTT broker (optional)

## License

AGPLv3
