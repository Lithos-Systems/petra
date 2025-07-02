#!/bin/bash
# quick-start.sh - Quick production setup

set -e

echo "🚀 Petra Quick Start"
echo "==================="

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "Docker not found. Please install Docker first."
    exit 1
fi

# Create necessary directories
mkdir -p configs data logs

# Generate basic config if not exists
if [ ! -f configs/production.yaml ]; then
    cat > configs/production.yaml << 'EOF'
# Petra Production Configuration
scan_time_ms: 100

# Signals
signals:
  - name: temperature_sensor_1
    data_type: float
    unit: "°C"
    
  - name: pressure_sensor_1
    data_type: float
    unit: "bar"
    
  - name: production_count
    data_type: int
    initial_value: 0

# MQTT Configuration
mqtt:
  broker_address: "mqtt://mqtt:1883"
  client_id: "petra-production"
  topics:
    - topic: "petra/signals/+"
      qos: 1

# Storage Configuration
storage:
  type: clickhouse
  connection_string: "tcp://petra:secure_password@clickhouse:9000/petra_history"
  
# Metrics
metrics:
  enabled: true
  port: 9090

# Health API
health:
  enabled: true
  port: 8080
EOF
fi

# Create mosquitto config
cat > configs/mosquitto.conf << 'EOF'
persistence true
persistence_location /mosquitto/data/
log_dest file /mosquitto/log/mosquitto.log
listener 1883
allow_anonymous true
EOF

# Build and start
echo "Building Petra..."
./build-production.sh

echo "Starting services..."
docker-compose up -d

echo "✅ Petra is running!"
echo ""
echo "Services:"
echo "  - Petra Metrics: http://localhost:9090/metrics"
echo "  - Petra Health: http://localhost:8080/health"
echo "  - ClickHouse: http://localhost:8123"
echo "  - MQTT: localhost:1883"
echo ""
echo "Logs: docker-compose logs -f petra"
