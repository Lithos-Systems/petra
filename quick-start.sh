#!/bin/bash
# quick-start.sh
set -e

echo "🚀 Petra Quick Start"
echo "==================="

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose not found. Please install docker-compose first."
    exit 1
fi

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p mosquitto/{config,data,log}
mkdir -p clickhouse/{data,config}  
mkdir -p prometheus
mkdir -p data

# Create mosquitto config
cat > mosquitto/config/mosquitto.conf << EOF
persistence true
persistence_location /mosquitto/data/
log_dest file /mosquitto/log/mosquitto.log
listener 1883
allow_anonymous true
EOF

# Create prometheus config
cat > prometheus/prometheus.yml << EOF
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'petra'
    static_configs:
      - targets: ['petra:9090']
EOF

# Set random password for ClickHouse
export CLICKHOUSE_PASSWORD=$(openssl rand -base64 12)
echo "🔐 Generated ClickHouse password: $CLICKHOUSE_PASSWORD"

# Start services
echo "🐳 Starting services..."
docker-compose up -d

# Wait for services
echo "⏳ Waiting for services to start..."
sleep 10

# Check status
echo "✅ Checking service status..."
docker-compose ps

echo ""
echo "🎉 Petra is running!"
echo ""
echo "📊 Access points:"
echo "  - MQTT: localhost:1883"
echo "  - ClickHouse: http://localhost:8123 (user: petra, pass: $CLICKHOUSE_PASSWORD)"
echo "  - Metrics: http://localhost:9090/metrics"
echo ""
echo "📝 View logs: docker-compose logs -f petra"
echo "🛑 Stop: docker-compose down"
echo ""
