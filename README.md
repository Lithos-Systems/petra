# PETRA
#### Programmable Engine for Telemetry, Runtime, and Automation

## Features

- **Entirely Rust-Based** - All the modern benefits of Rust to modernize legacy systems
- **Rock-Solid Reliability** - Comprehensive error handling
- **Built for Pure Function** - No nonsense, no fluff
- **S7 PLC Integration** - Direct communication with Siemens S7 PLCs via rust-snap7
- **MQTT Support** - Built-in MQTT client for IoT integration
- **Twilio Integration** - Send SMS and make phone calls directly from PLC logic
    
## Quick Start

1. **Build Petra**
   ```bash
   cargo build --release
   ```
2. **Run Petra**
   ```bash
   cargo run --release configs/example-mqtt.yaml
   ```

## Twilio Integration

Petra includes powerful Twilio integration for sending SMS messages and making phone calls directly from your PLC logic. This is perfect for:
- Critical alarm notifications
- Production status updates
- Emergency response automation
- Shift change notifications
- Equipment maintenance alerts

### Getting Started with Twilio

**Prerequisites:**
1. A Twilio account (sign up free at [twilio.com](https://twilio.com))
2. A verified phone number for sending messages/calls
3. Your Twilio credentials

**Step 1: Set up your Twilio account**

After creating your Twilio account:
1. Go to the [Twilio Console](https://console.twilio.com)
2. Find your **Account SID** and **Auth Token** on the dashboard
3. Buy a phone number (Phone Numbers → Manage → Buy a number)
4. Verify your personal phone number for testing (if using trial account)

**Step 2: Configure environment variables**

Set these environment variables with your Twilio credentials:
```bash
export TWILIO_ACCOUNT_SID="ACxxxxxxxxxxxxxxxxxxxxxxxxxx"
export TWILIO_AUTH_TOKEN="your_auth_token_here"
export TWILIO_FROM_NUMBER="+1234567890"  # Your Twilio phone number
```

**Important:** Phone numbers must be in [E.164 format](https://www.twilio.com/docs/glossary/what-e164) (e.g., +1234567890)

### Twilio Configuration Methods

Petra offers two ways to use Twilio:

#### Method 1: Direct Block Integration
Add Twilio blocks directly to your PLC logic:

```yaml
blocks:
  - name: "emergency_sms"
    type: "TWILIO"
    inputs:
      trigger: "emergency_alarm"
    outputs:
      success: "sms_sent"
    params:
      action_type: "sms"
      to_number: "+1234567890"
      content: "EMERGENCY: Production line stopped!"
      cooldown_ms: 300000  # 5 minutes between messages
```

#### Method 2: Signal-Based Connector
Use the Twilio connector for more complex scenarios:

```yaml
twilio:
  from_number: "+1987654321"
  poll_interval_ms: 500
  actions:
    - name: "temp_warning"
      trigger_signal: "high_temp_alarm"
      action_type: "sms"
      to_number: "+1234567890"
      content: "Temperature warning: {{value}}°C"
      cooldown_seconds: 1800
```

### Testing Your Setup

**Test 1: Simple SMS**
```bash
cargo run --bin twilio_test sms --to "+1234567890" --message "Hello from Petra!"
```

**Test 2: Voice Call**
```bash
cargo run --bin twilio_test call --to "+1234567890" --message "This is a test call from Petra PLC system"
```

**Test 3: Signal-Based Trigger**
```bash
# Using a config file with Twilio actions
cargo run --bin twilio_test signal --config configs/twilio-example.yaml --signal "emergency_stop" --value "true"
```

### Twilio Block Parameters

| Parameter | Required | Description | Example |
|-----------|----------|-------------|---------|
| `action_type` | Yes | "sms" or "call" | `"sms"` |
| `to_number` | Yes | Destination phone (E.164) | `"+1234567890"` |
| `content` | Yes | Message text or TwiML | `"Motor fault detected"` |
| `cooldown_ms` | No | Delay between triggers | `300000` (5 min) |
| `from_number` | No | Override default sender | `"+1987654321"` |

### Advanced TwiML for Calls

For voice calls, you can use simple text or advanced TwiML:

**Simple text:**
```yaml
content: "Emergency alert. Please check the production line immediately."
```

**Advanced TwiML:**
```yaml
content: |
  <Response>
    <Say voice="alice" language="en-US">
      Emergency alert from Petra PLC system.
      Production line has stopped due to safety alarm.
    </Say>
    <Pause length="2"/>
    <Say>Press 1 to acknowledge this alert.</Say>
    <Gather timeout="10" numDigits="1" action="https://your-webhook.com/acknowledge">
      <Say>Waiting for response.</Say>
    </Gather>
  </Response>
```

### Example Configurations

**Emergency Notification System:**
```yaml
signals:
  - name: "emergency_stop"
    type: "bool"
    initial: false
  - name: "notification_sent"
    type: "bool"
    initial: false

blocks:
  - name: "emergency_call"
    type: "TWILIO"
    inputs:
      trigger: "emergency_stop"
    outputs:
      success: "notification_sent"
    params:
      action_type: "call"
      to_number: "+1234567890"
      content: "Emergency stop activated. Immediate attention required."
      cooldown_ms: 600000  # 10 minutes
```

**Temperature Monitoring with Escalation:**
```yaml
twilio:
  actions:
    # Level 1: SMS warning
    - name: "temp_warning"
      trigger_signal: "high_temp_warning"
      action_type: "sms"
      to_number: "+1111111111"  # Operator
      content: "Temperature warning: approaching limit"
      cooldown_seconds: 900  # 15 minutes
      result_signal: "warning_sent"
    
    # Level 2: Emergency call
    - name: "temp_emergency"
      trigger_signal: "high_temp_alarm"
      action_type: "call"
      to_number: "+2222222222"  # Supervisor
      content: "Critical temperature alarm. Immediate shutdown required."
      cooldown_seconds: 300  # 5 minutes
```

### Troubleshooting Twilio

**Common Issues:**

1. **"Phone number not verified" (Trial accounts)**
   - Verify recipient numbers in Twilio Console
   - Upgrade to paid account for unrestricted sending

2. **"Invalid phone number format"**
   - Ensure E.164 format: `+1234567890`
   - Include country code (+1 for US/Canada)

3. **"Authentication failed"**
   - Check `TWILIO_ACCOUNT_SID` and `TWILIO_AUTH_TOKEN`
   - Verify credentials in Twilio Console

4. **"From number not owned"**
   - Use a number purchased through Twilio
   - Check `TWILIO_FROM_NUMBER` environment variable

**Testing Connection:**
```bash
# Test credentials and basic connectivity
curl -X GET "https://api.twilio.com/2010-04-01/Accounts/$TWILIO_ACCOUNT_SID/Messages.json" \
    -u "$TWILIO_ACCOUNT_SID:$TWILIO_AUTH_TOKEN"
```

### Twilio Costs

Twilio pricing varies by service and region. SMS typically costs $0.0075-$0.0140 per message, while voice calls cost around $0.0085-$0.025 per minute. Check [Twilio's pricing page](https://www.twilio.com/pricing) for current rates.

**Cost Management Tips:**
- Use `cooldown_ms` to prevent spam
- Set up result signals to track success/failure
- Monitor usage in Twilio Console
- Consider SMS vs. voice based on urgency

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

twilio:  # Optional - only if using Twilio
  from_number: "+1234567890"
  poll_interval_ms: 1000
  actions: []
```

## Available Blocks

- **Logic**: AND, OR, NOT
- **Timers**: TON (Timer On Delay)
- **Edge Detection**: R_TRIG (Rising Edge)
- **Memory**: SR_LATCH (Set-Reset Latch)
- **Comparison**: GT (Greater Than), LT (Less Than)
- **Communication**: TWILIO (SMS/Voice)

## Complete Example Scenarios

### 1. Basic Motor Control with Notifications
```yaml
signals:
  - name: "start_button"
    type: "bool"
    initial: false
  - name: "motor_fault"
    type: "bool"
    initial: false
  - name: "motor_running"
    type: "bool"
    initial: false
  - name: "operator_notified"
    type: "bool"
    initial: false

blocks:
  # Start motor on button press
  - name: "motor_start"
    type: "AND"
    inputs:
      in1: "start_button"
      in2: "motor_fault"  # NOT gate would be better
    outputs:
      out: "motor_running"
  
  # Notify operator of faults
  - name: "fault_notification"
    type: "TWILIO"
    inputs:
      trigger: "motor_fault"
    outputs:
      success: "operator_notified"
    params:
      action_type: "sms"
      to_number: "+1234567890"
      content: "Motor fault detected on Line 1"
      cooldown_ms: 600000

scan_time_ms: 100
```

### 2. Temperature Monitoring with Escalation
```yaml
signals:
  - name: "temperature"
    type: "float"
    initial: 20.0
  - name: "temp_limit_warning"
    type: "float"
    initial: 75.0
  - name: "temp_limit_critical"
    type: "float"
    initial: 85.0
  - name: "temp_warning"
    type: "bool"
    initial: false
  - name: "temp_critical"
    type: "bool"
    initial: false

blocks:
  - name: "warning_check"
    type: "GT"
    inputs:
      in1: "temperature"
      in2: "temp_limit_warning"
    outputs:
      out: "temp_warning"
  
  - name: "critical_check"
    type: "GT"
    inputs:
      in1: "temperature"
      in2: "temp_limit_critical"
    outputs:
      out: "temp_critical"

twilio:
  actions:
    - name: "warning_sms"
      trigger_signal: "temp_warning"
      action_type: "sms"
      to_number: "+1234567890"
      content: "Temperature warning: {{value}}°C"
      cooldown_seconds: 900
    
    - name: "critical_call"
      trigger_signal: "temp_critical"
      action_type: "call"
      to_number: "+1234567890"
      content: "Critical temperature alarm requires immediate attention"
      cooldown_seconds: 300
```

## Testing Everything

### 1. Test Basic PLC Logic
```bash
# Run integration tests
cargo test

# Run with specific config
cargo run configs/example.yaml
```

### 2. Test S7 Connection
```bash
# Quick connection test
cargo run --bin simple_s7_test 192.168.1.100

# Full test with monitoring
cargo run --bin s7_test -- --ip 192.168.1.100 monitor -c configs/s7-example.yaml
```

### 3. Test MQTT Integration
```bash
# Subscribe to all topics
mosquitto_sub -h mqtt.lithos.systems -t 'petra/plc/#' -v

# Send commands
mosquitto_pub -h mqtt.lithos.systems -t 'petra/plc/cmd' -m '{"type":"SetSignal","name":"start_button","value":true}'
```

### 4. Test Twilio Integration
```bash
# Test SMS
cargo run --bin twilio_test sms --to "+1234567890" --message "Test from Petra"

# Test voice call
cargo run --bin twilio_test call --to "+1234567890" --message "This is a test call"

# Test with real signals
cargo run --bin twilio_test signal --config configs/twilio-example.yaml --signal "emergency_stop" --value "true"
```

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
   export TWILIO_ACCOUNT_SID=ACxxxxx   # Required for Twilio
   export TWILIO_AUTH_TOKEN=xxxxx      # Required for Twilio
   export TWILIO_FROM_NUMBER=+1xxxxx   # Required for Twilio
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
   Environment="TWILIO_ACCOUNT_SID=ACxxxxx"
   Environment="TWILIO_AUTH_TOKEN=xxxxx"
   Environment="TWILIO_FROM_NUMBER=+1xxxxx"

   [Install]
   WantedBy=multi-user.target
   ```

4. **Monitoring**
   - Monitor MQTT status topic for health checks
   - Check logs for scan overruns or communication errors
   - Use the S7 test tool for diagnostics
   - Monitor Twilio usage in Twilio Console

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

### Twilio Issues
- Verify phone numbers are in E.164 format (+1234567890)
- Check account balance and trial account restrictions
- Verify phone numbers in Twilio Console (trial accounts)
- Test credentials with curl or Twilio CLI
- Monitor rate limits and usage

## Dependencies

- Rust 1.70+
- rust-snap7 (includes snap7 library)
- MQTT broker (optional, e.g., Mosquitto)
- Twilio account (optional, for SMS/voice features)

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
```
