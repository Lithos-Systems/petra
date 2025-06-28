# Getting Started with Petra

This tutorial gets **Petra** up and running in just a few minutes.

---

## Prerequisites

- Linux, macOS, or Windows (WSL 2)  
- **Rust 1.75+** (for building from source)  
- **Docker** (for the quickest start)  

---

## Quick-Start with Docker

```bash
# Download the quick-start script
curl -sSL https://github.com/your-org/petra/raw/main/quick-start.sh -o quick-start.sh
chmod +x quick-start.sh

# Launch the demo stack
./quick-start.sh
````

**The script will start:**

* *Mosquitto* MQTT broker
* *ClickHouse* time-series database
* Petra with a demo configuration
* A summary banner with connection details

---

## Building from Source

```bash
# Clone the repository
git clone https://github.com/your-org/petra
cd petra

# Build with default features
cargo build --release

# Run with the example configuration
./target/release/petra configs/examples/simple-mqtt.yaml
```

---

## Your First Configuration

Save the following as **`my-first-config.yaml`**:

```yaml
# ---- Signals -----------------------------------------------------
signals:
  - name: temperature
    type: float
    initial: 20.0

  - name: temperature_high
    type: bool
    initial: false

  - name: fan_speed
    type: int
    initial: 0

# ---- Logic Blocks ------------------------------------------------
blocks:
  - name: temp_monitor            # Detect high temperature
    type: Compare
    inputs:
      a: temperature
      b: 30.0
    outputs:
      result: temperature_high
    params:
      operation: ">"

  - name: fan_control             # Drive the fan
    type: Select
    inputs:
      condition: temperature_high
      true_value: 100
      false_value: 0
    outputs:
      output: fan_speed

# ---- MQTT --------------------------------------------------------
mqtt:
  broker_host: localhost
  broker_port: 1883
  client_id: petra-demo
  publish:
    - signal: temperature
      topic: sensors/temperature
      interval_ms: 1000
    - signal: fan_speed
      topic: actuators/fan/speed
      on_change: true

# ---- Basic Alarm -------------------------------------------------
alarms:
  - name: high_temp_alarm
    condition: temperature_high
    message: "Temperature exceeded 30 °C"
    severity: warning

# ---- Scan Engine -------------------------------------------------
scan_time_ms: 100
```

### Run it

```bash
petra my-first-config.yaml
```

---

## Testing Your Setup

1. **Publish a temperature value**

   ```bash
   # In another terminal
   mosquitto_pub -t sensors/temperature/set -m "32.5"
   ```

2. **Watch the fan-speed output**

   ```bash
   mosquitto_sub -t actuators/fan/speed
   ```

   You should see the fan speed change to **100** when the temperature exceeds 30 °C.

3. **Check metrics**

   ```bash
   curl -s http://localhost:9090/metrics | grep petra
   ```

---

## Next Steps

* Connect Petra to a **real PLC**
* Build **custom logic blocks**
* Set up **alerting** (e-mail, SMS, etc.)
* Plan your **production deployment**

---

## Common Issues

### Port already in use

```bash
# Find what’s using the port
lsof -i :1883   # MQTT
lsof -i :9090   # Metrics

# Stop conflicting service
sudo systemctl stop mosquitto
```

### Permission denied (Docker)

```bash
# Add your user to the docker group
sudo usermod -aG docker $USER
newgrp docker
```

### “Signal not found” errors

Ensure the signal names referenced in blocks **exactly** match the names in the `signals` section (case-sensitive).

```
