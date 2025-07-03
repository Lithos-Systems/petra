#!/bin/bash
# quick-start-fixed.sh - Fixed Petra Quick Start

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Petra Quick Start (Fixed)${NC}"
echo "============================"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check Docker
echo -e "${YELLOW}Checking Docker installation...${NC}"
if ! command_exists docker; then
    echo -e "${RED}❌ Docker not found. Please install Docker first.${NC}"
    echo "Installation guide: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check Docker daemon
if ! docker info >/dev/null 2>&1; then
    echo -e "${RED}❌ Docker daemon not running or accessible.${NC}"
    echo "Try:"
    echo "  sudo systemctl start docker"
    echo "  sudo usermod -aG docker \$USER"
    echo "  newgrp docker"
    exit 1
fi

echo -e "${GREEN}✅ Docker is ready${NC}"

# Check docker-compose
if ! command_exists docker-compose; then
    echo -e "${YELLOW}⚠️  docker-compose not found, trying docker compose...${NC}"
    if ! docker compose version >/dev/null 2>&1; then
        echo -e "${RED}❌ Neither docker-compose nor 'docker compose' found.${NC}"
        exit 1
    fi
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo -e "${GREEN}✅ Using: $COMPOSE_CMD${NC}"

# Create necessary directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p configs data logs

# Generate basic config if not exists
if [ ! -f configs/production.yaml ]; then
    echo -e "${YELLOW}Creating production configuration...${NC}"
    cat > configs/production.yaml << 'EOF'
# Petra Production Configuration
scan_time_ms: 100

# Signals
signals:
  - name: temperature_sensor_1
    data_type: float
    unit: "°C"
    initial_value: 20.0
    
  - name: pressure_sensor_1
    data_type: float
    unit: "bar"
    initial_value: 1.0
    
  - name: production_count
    data_type: int
    initial_value: 0

# MQTT Configuration  
mqtt:
  broker_address: "mqtt://mqtt:1883"
  client_id: "petra-production"
  keep_alive: 60
  topics:
    - topic: "petra/signals/+"
      qos: 1

# Storage Configuration
storage:
  type: clickhouse
  host: "clickhouse"
  port: 9000
  database: "petra_history"
  username: "petra"
  password: "secure_password"
  
# Metrics
metrics:
  enabled: true
  port: 9090
  endpoint: "/metrics"

# Health API
health:
  enabled: true
  port: 8080
  endpoint: "/health"

# Logging
logging:
  level: "info"
  format: "json"
EOF
    echo -e "${GREEN}✅ Created configs/production.yaml${NC}"
fi

# Create mosquitto config if not exists
if [ ! -f configs/mosquitto.conf ]; then
    echo -e "${YELLOW}Creating Mosquitto configuration...${NC}"
    # Using the mosquitto config from the artifact above
    cat > configs/mosquitto.conf << 'EOF'
# Mosquitto configuration for Petra
persistence true
persistence_location /mosquitto/data/
log_dest file /mosquitto/log/mosquitto.log
log_dest stdout
log_type error
log_type warning
log_type notice
log_type information
listener 1883
protocol mqtt
listener 9001
protocol websockets
allow_anonymous true
max_connections 1000
max_queued_messages 100
message_size_limit 1048576
keepalive_interval 60
EOF
    echo -e "${GREEN}✅ Created configs/mosquitto.conf${NC}"
fi

# Check if Petra binary exists
if [ ! -f target/release/petra ]; then
    echo -e "${YELLOW}Petra binary not found. Building...${NC}"
    if [ -f build-production.sh ]; then
        chmod +x build-production.sh
        # Run build script non-interactively (choose production)
        echo "1" | ./build-production.sh
    else
        echo -e "${YELLOW}Building with cargo directly...${NC}"
        cargo build --release --features production
    fi
fi

# Stop any existing containers
echo -e "${YELLOW}Stopping existing containers...${NC}"
$COMPOSE_CMD down --remove-orphans || true

# Start services
echo -e "${GREEN}Starting Petra services...${NC}"
$COMPOSE_CMD up -d

# Wait for services to be ready
echo -e "${YELLOW}Waiting for services to start...${NC}"
sleep 10

# Check service health
echo -e "${YELLOW}Checking service health...${NC}"

# Check ClickHouse
if docker exec petra-clickhouse wget -q --spider http://localhost:8123/ping; then
    echo -e "${GREEN}✅ ClickHouse is healthy${NC}"
else
    echo -e "${YELLOW}⚠️  ClickHouse not ready yet${NC}"
fi

# Check Mosquitto
if docker exec petra-mqtt mosquitto_pub -h localhost -t test -m "health_check" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Mosquitto is healthy${NC}"
else
    echo -e "${YELLOW}⚠️  Mosquitto not ready yet${NC}"
fi

# Check Petra health endpoint (may take longer to start)
echo -e "${YELLOW}Waiting for Petra to start...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Petra is healthy${NC}"
        break
    elif [ $i -eq 30 ]; then
        echo -e "${YELLOW}⚠️  Petra not responding yet (check logs)${NC}"
    else
        echo -n "."
        sleep 2
    fi
done

echo
echo -e "${GREEN}✅ Petra is running!${NC}"
echo
echo -e "${YELLOW}Services:${NC}"
echo "  - Petra Health:    http://localhost:8080/health"
echo "  - Petra Metrics:   http://localhost:9090/metrics"
echo "  - ClickHouse UI:   http://localhost:8123"
echo "  - MQTT Broker:     mqtt://localhost:1883"
echo "  - MQTT WebSocket:  ws://localhost:9001"
echo
echo -e "${YELLOW}Management Commands:${NC}"
echo "  - View logs:       $COMPOSE_CMD logs -f"
echo "  - View Petra logs: $COMPOSE_CMD logs -f petra"
echo "  - Stop services:   $COMPOSE_CMD down"
echo "  - Restart:         $COMPOSE_CMD restart"
echo
echo -e "${YELLOW}Configuration:${NC}"
echo "  - Main config:     configs/production.yaml"
echo "  - MQTT config:     configs/mosquitto.conf"
echo "  - Data directory:  data/"
