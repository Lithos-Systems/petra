#!/bin/bash
# scripts/test-harness.sh

# Start test services
docker-compose -f docker/compose/docker-compose.test.yml up -d

# Wait for services
timeout 30 bash -c 'until nc -z localhost 1883; do sleep 1; done'

# Run integration tests
export MQTT_HOST=localhost
export CLICKHOUSE_HOST=localhost

cargo test --features integration-tests

# Cleanup
docker-compose -f docker/compose/docker-compose.test.yml down
