# Petra Quick Start Guide

## 30-Second Install

### Option 1: Docker (Fastest)
```bash
# Download and run the quick start script
curl -sSL https://raw.githubusercontent.com/lithos-systems/petra/main/quick-start.sh | bash

# Or clone and run locally
git clone https://github.com/lithos-systems/petra && cd petra
./quick-start.sh
```

### Option 2: Pre-built Binary
```bash
# Download latest release
curl -sSL https://github.com/lithos-systems/petra/releases/latest/download/petra-linux-x64 -o petra
chmod +x petra

# Run with example config
./petra configs/examples/basic/simple-mqtt.yaml
```

### Option 3: Build from Source
```bash
# Clone repository
git clone https://github.com/lithos-systems/petra && cd petra

# Build with default features
cargo build --release

# Run with example config
./target/release/petra configs/examples/basic/simple-mqtt.yaml
```

## Development Setup

### First Time Setup
```bash
# Clone the repository
git clone https://github.com/lithos-systems/petra
cd petra

# Run the cleanup script to organize project structure
chmod +x scripts/cleanup-project.sh
./scripts/cleanup-project.sh

# Copy the new unified dev tool
cp scripts/petra-dev.sh scripts/
chmod +x scripts/petra-dev.sh

# Quick test to verify everything works
./scripts/petra-dev.sh test --level quick
```

### Using the Unified Dev Tool

The `petra-dev.sh` script consolidates all development tasks:

```bash
# Run quick tests (30 seconds)
./scripts/petra-dev.sh test --level quick

# Run standard tests (2 minutes)
./scripts/petra-dev.sh test --level standard

# Run full pre-release checks
./scripts/petra-dev.sh check

# Run benchmarks
./scripts/petra-dev.sh bench

# Run security audit
./scripts/petra-dev.sh security

# Update version
./scripts/petra-dev.sh version 0.2.0

# Clean and reorganize project
./scripts/petra-dev.sh clean
```

## Project Structure

After running the cleanup script, your project will be organized as:

```
petra/
├── configs/
│   ├── examples/
│   │   ├── basic/        # Simple examples for getting started
│   │   ├── advanced/     # Complex configurations  
│   │   └── industrial/   # Industry-specific use cases
│   ├── production/       # Production-ready configurations
│   └── schemas/          # JSON schemas for validation
├── scripts/
│   └── petra-dev.sh      # Unified development tool
├── tools/
│   ├── bin/              # Utility binaries
│   ├── scripts/          # Helper scripts
│   └── examples/         # Example tools
└── docs/
    └── architecture.md   # System architecture documentation
```

## Common Tasks

### Running Tests
```bash
# Quick validation (format, clippy, unit tests)
./scripts/petra-dev.sh test --level quick

# Full test suite with all features
./scripts/petra-dev.sh test --level full

# Security audit
./scripts/petra-dev.sh security
```

### Building for Production
```bash
# Standard production build
cargo build --release --features production

# Full enterprise build  
cargo build --release --features enterprise

# Edge device build (minimal)
cargo build --release --features edge
```

### Working with Configurations
```bash
# Validate a configuration
cargo run -- configs/my-config.yaml --validate-only

# Run with a specific config
cargo run -- configs/examples/basic/simple-mqtt.yaml

# Use the visual designer
cd petra-designer && npm run dev
```

### Docker Development
```bash
# Start development stack
docker-compose -f docker/compose/docker-compose.dev.yml up -d

# View logs
docker-compose logs -f petra

# Stop services
docker-compose down
```

## Example Configuration

Create `my-first-config.yaml`:

```yaml
# Basic signals
signals:
  - name: temperature
    type: float
    initial: 20.0
    
  - name: high_temp
    type: bool
    initial: false

# Logic blocks  
blocks:
  - name: temp_check
    type: Compare
    inputs:
      a: temperature
      b: 30.0
    outputs:
      result: high_temp
    params:
      operation: ">"

# MQTT connection
mqtt:
  broker_host: localhost
  broker_port: 1883
  client_id: petra-demo
  
# Scan rate
scan_time_ms: 100
```

Run it:
```bash
cargo run -- my-first-config.yaml
```

## Troubleshooting

### Common Issues

**Build fails with "cannot find -lssl"**
```bash
# Install OpenSSL development files
sudo apt-get install libssl-dev  # Debian/Ubuntu
sudo yum install openssl-devel   # RHEL/CentOS
```

**"No such file or directory" for scripts**
```bash
# Make scripts executable
chmod +x scripts/*.sh
```

**Docker permission denied**
```bash
# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker
```

### Getting Help

- Check the [documentation](docs/)
- Review [example configurations](configs/examples/)
- Open an [issue](https://github.com/lithos-systems/petra/issues)
- Join our [Discord](https://discord.gg/DeR9UYGU))

## Next Steps

1. Explore the [example configurations](configs/examples/)
2. Read the [architecture documentation](docs/architecture.md)
3. Try connecting to a real PLC or MQTT broker
4. Build your own custom blocks
5. Set up monitoring and alerts
