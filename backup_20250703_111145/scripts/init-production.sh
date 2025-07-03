#!/bin/bash
# Initialize production environment

set -e

echo "Initializing Petra production environment..."

# Check for required environment variables
required_vars=("POSTGRES_PASSWORD" "CLICKHOUSE_PASSWORD" "STRIPE_WEBHOOK_SECRET")
for var in "${required_vars[@]}"; do
    if [[ -z "${!var}" ]]; then
        echo "Error: $var is not set"
        exit 1
    fi
done

# Create necessary directories
mkdir -p ca/root ca/clients
mkdir -p mosquitto/{config,data,log,certs}
mkdir -p clickhouse/{data,config}
mkdir -p nginx/{conf.d,ssl,html}

# Generate MQTT server certificate if it doesn't exist
if [[ ! -f mosquitto/certs/server.crt ]]; then
    echo "Generating MQTT server certificate..."
    ./scripts/generate-mqtt-server-cert.sh
fi

# Set up database
echo "Waiting for PostgreSQL to be ready..."
docker-compose -f docker-compose.production.yml up -d postgres
sleep 10

# Run database migrations
echo "Running database migrations..."
docker-compose -f docker-compose.production.yml run --rm ca-service petra-ca-service migrate

# Start all services
echo "Starting all services..."
docker-compose -f docker-compose.production.yml up -d

echo "Production environment initialized successfully!"
echo "CA Service: https://localhost:8443"
echo "MQTT Broker: mqtts://localhost:8883"
echo "ClickHouse: http://localhost:8123"
