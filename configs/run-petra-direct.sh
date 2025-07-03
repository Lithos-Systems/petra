#!/bin/bash
# run-petra-direct.sh - Run Petra directly without Docker

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🚀 Running Petra Directly (No Docker)${NC}"
echo "======================================="

# Create directories
mkdir -p configs data logs

# Create a minimal config for direct running
if [ ! -f configs/direct-run.yaml ]; then
    echo -e "${YELLOW}Creating direct run configuration...${NC}"
    cat > configs/direct-run.yaml << 'EOF'
# Petra Direct Run Configuration (No Docker)
scan_time_ms: 100

# Signals
signals:
  - name: test_signal_1
    data_type: float
    initial_value: 42.0
    
  - name: test_signal_2
    data_type: int
    initial_value: 100
    
  - name: status_signal
    data_type: bool
    initial_value: true

# Basic blocks for testing
blocks:
  - name: test_math_block
    type: MATH
    inputs:
      a: test_signal_1
      b: test_signal_2
    outputs:
      result: calculated_result
    parameters:
      operation: "add"

# Metrics (local only)
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
  format: "pretty"
  
# No MQTT or external storage for direct run
# This allows Petra to run standalone for testing
EOF
    echo -e "${GREEN}✅ Created configs/direct-run.yaml${NC}"
fi

# Test the Petra binary
echo -e "${YELLOW}Testing Petra binary...${NC}"
if [ ! -f target/release/petra ]; then
    echo -e "${RED}❌ Petra binary not found. Run build-production.sh first.${NC}"
    exit 1
fi

# Test version
echo -e "${YELLOW}Petra version:${NC}"
./target/release/petra --version || {
    echo -e "${RED}❌ Failed to get Petra version${NC}"
    exit 1
}

# Check if ports are available
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Port 8080 is in use. Stopping...${NC}"
    sudo lsof -ti:8080 | xargs kill -9 || true
fi

if lsof -Pi :9090 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Port 9090 is in use. Stopping...${NC}"
    sudo lsof -ti:9090 | xargs kill -9 || true
fi

# Run Petra in the background
echo -e "${GREEN}Starting Petra directly...${NC}"
echo -e "${YELLOW}Config: configs/direct-run.yaml${NC}"
echo -e "${YELLOW}Logs will be shown below. Press Ctrl+C to stop.${NC}"
echo

# Run Petra with the direct config
RUST_LOG=petra=info ./target/release/petra configs/direct-run.yaml &
PETRA_PID=$!

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Stopping Petra...${NC}"
    kill $PETRA_PID 2>/dev/null || true
    wait $PETRA_PID 2>/dev/null || true
    echo -e "${GREEN}✅ Petra stopped${NC}"
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Wait a moment for startup
sleep 3

# Test if Petra is responding
echo -e "${YELLOW}Testing Petra endpoints...${NC}"

# Test health endpoint
if curl -s http://localhost:8080/health >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Health endpoint: http://localhost:8080/health${NC}"
    curl -s http://localhost:8080/health | jq . 2>/dev/null || curl -s http://localhost:8080/health
else
    echo -e "${YELLOW}⚠️  Health endpoint not ready yet${NC}"
fi

echo

# Test metrics endpoint
if curl -s http://localhost:9090/metrics >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Metrics endpoint: http://localhost:9090/metrics${NC}"
    echo "Sample metrics:"
    curl -s http://localhost:9090/metrics | head -10
else
    echo -e "${YELLOW}⚠️  Metrics endpoint not ready yet${NC}"
fi

echo
echo -e "${GREEN}✅ Petra is running directly!${NC}"
echo
echo -e "${YELLOW}Available endpoints:${NC}"
echo "  - Health:  http://localhost:8080/health"
echo "  - Metrics: http://localhost:9090/metrics"
echo
echo -e "${YELLOW}To test in another terminal:${NC}"
echo "  curl http://localhost:8080/health"
echo "  curl http://localhost:9090/metrics"
echo
echo -e "${YELLOW}Press Ctrl+C to stop Petra${NC}"

# Keep the script running and show logs
wait $PETRA_PID
