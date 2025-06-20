# Petra v2.0

Production-ready PLC with MQTT integration for industrial automation.

## Features

- **Rock-solid reliability** - Comprehensive error handling
- **MQTT Integration** - Bidirectional communication via MQTT
- **Node-RED Ready** - Example flow and docker environment provided
- **Simple architecture** - About one thousand lines of readable Rust

## Quick Start

1. **Build Petra**
   ```bash
   cargo build --release
   ```
2. **Start Test Environment**
   ```bash
   cd node-red
   docker-compose up -d
   ```
3. **Run Petra**
   ```bash
   cargo run --release configs/example-mqtt.yaml
   ```
4. **Access Node-RED**
   - Open <http://localhost:1880>
   - Import `flows.json`
   - Access dashboard at <http://localhost:1880/ui>

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
  broker_host: "localhost"
  broker_port: 1883
  client_id: "petra-01"
  topic_prefix: "petra/plc"
  qos: 1
```

## Production Deployment

- Use `--release` build
- Set `RUST_LOG=petra=info`
- Use a systemd service or supervisor
- Monitor MQTT status topic

## Adding Blocks

1. Add a struct in `src/block.rs` implementing `Block`
2. Add a case in `create_block()`
3. Done!

## License

MIT
