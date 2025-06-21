# PETRA
#### Programmable Engine for Telemetry, Runtime, and Automation

## Features

- **Entirely Rust-Based** - All the modern benefits of Rust to modernize legacy systems
- **Rock-Solid Reliability** - Comprehensive error handling
- **Built for Pure Function** - No nonsense, no fluff
- **S7 PLC Integration** - Direct communication with Siemens S7 PLCs via rust-snap7
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

Petra now supports direct communication with Siemens S7 PLCs using the rust-snap7 library, which provides:
- Built-in utility functions for data conversion
- Better error handling and debugging
- More robust connection management
- Support for all S7 PLC families (S7-300, S7-400, S7-1200, S7-1500)

### S7 Test Tool

Test your PLC connection:
```bash
# Test connection
cargo run --bin s7_test -- --ip 192.168.1.100 connect

# Get PLC info (order code, CPU info, protection status)
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
  rack: 0              # Usually 0 for S7-300/400/1200/1500
  slot: 2              # Usually 2 for S7-300/400, 1 for S7-1200/1500
  poll_interval_ms: 100
  timeout_ms: 5000
  mappings:
    # Read motor status from PLC
    - signal: "motor_running"
      area: "DB"
      db_number: 100
      address: 0
      data_type: "bool"
      bit: 0
      direction: "read"
    
    # Write motor speed setpoint
    - signal: "motor_speed"
      area: "DB"
      db_number: 101
      address: 2
      data_type: "int"
      direction: "write"
    
    # Read analog input
    - signal: "temperature"
      area: "DB"
      db_number: 100
      address: 4
      data_type: "real"
      direction: "read"
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
- `I` - Inputs (Process Image)
- `Q` - Outputs (Process Image)
- `M` - Markers/Flags
- `C` - Counters
- `T` - Timers

## MQTT Integration

Petra can act as a bridge between S7 PLCs and MQTT brokers, enabling:
- Real-time data streaming from PLC to cloud
- Remote control of PLC from MQTT clients
- Integration with IoT platforms and SCADA systems

### MQTT Topics

Commands (subscribe):
- `petra/plc/cmd` - Send commands to PLC

Status (publish):
- `petra/plc/status` - Online/offline status
- `petra/plc/signals/{name}` - Individual signal values
- `petra/plc/signals` - All signals snapshot
- `petra/plc/stats` - Engine statistics

### MQTT Message Examples

Set Signal:
```json
{
  "type": "SetSignal",
  "name": "start_button",
  "value": true
}
```

Get Signal:
```json
{
  "type": "GetSignal",
  "name": "motor_run"
}
```

Get All Signals:
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
  publish_on_change: true  # Only publish when values change

s7:  # Optional - only if using S7 PLC
  ip: "192.168.1.100"
  rack: 0
  slot: 2
  poll_interval_ms: 100
  timeout_ms: 5000
  mappings: []
```

## Available Blocks

- **Logic**: AND, OR, NOT
- **Timers**: TON (Timer On Delay)
- **Edge Detection**: R_TRIG (Rising Edge)
- **Memory**: SR_LATCH (Set-Reset Latch)
- **Comparison**: GT (Greater Than), LT (Less Than)

## Production Deployment

1. **Build for Production**
   ```bash
   cargo build --release
   ```

2. **Environment Variables**
   ```bash
   export RUST_LOG=petra=info
   export MQTT_USERNAME=your_username  # Optional
   export MQTT_PASSWORD=your_password  # Optional
   ```

3. **Systemd Service** (Linux)
   ```ini
   [Unit]
   Description=Petra PLC Engine
   After=network.target

   [Service]
   Type=simple
   ExecStart=/usr/local/bin/petra /etc/petra/config.yaml
   Restart=always
   RestartSec=10
   Environment="RUST_LOG=petra=info"

   [Install]
   WantedBy=multi-user.target
   ```

4. **Monitoring**
   - Monitor MQTT status topic for health checks
   - Check logs for scan overruns or communication errors
   - Use the S7 test tool for diagnostics

## Adding Custom Blocks

1. Add a struct in `src/block.rs` implementing the `Block` trait:
   ```rust
   pub struct MyBlock {
       name: String,
       // ... fields
   }
   
   impl Block for MyBlock {
       fn execute(&mut self, bus: &SignalBus) -> Result<()> {
           // Implementation
       }
       fn name(&self) -> &str { &self.name }
       fn block_type(&self) -> &str { "MY_BLOCK" }
   }
   ```

2. Add a case in `create_block()` function
3. Document the block in this README

## Troubleshooting

### S7 Connection Issues
- Verify PLC IP address is reachable: `ping 192.168.1.100`
- Check rack and slot numbers (use TIA Portal or Step 7)
- Ensure PLC allows PUT/GET communication
- Check firewall rules for port 102 (ISO-TSAP)

### MQTT Issues
- Verify broker connectivity: `mosquitto_sub -h broker -t '#' -v`
- Check authentication credentials
- Monitor MQTT logs for connection errors

## Dependencies

- Rust 1.70+
- rust-snap7 (includes snap7 library)
- MQTT broker (optional, e.g., Mosquitto)

## License

AGPLv3 - See LICENSE file for details

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## Support

For issues and questions:
- GitHub Issues: [Report bugs or request features]
- Documentation: Check the `/docs` folder
- Examples: See `/configs` for example configurations
