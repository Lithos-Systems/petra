# Petra Configuration Files

This directory contains example and production-ready configuration files for Petra.

## Directory Structure

```
configs/
├── examples/          # Example configurations for learning and testing
│   ├── basic/        # Simple examples for getting started
│   ├── advanced/     # Complex configurations with external systems
│   └── industrial/   # Industry-specific use cases
├── production/       # Production-ready configurations
└── schemas/         # JSON schemas for configuration validation
```

## Examples

### Basic Examples (`examples/basic/`)
- Simple control loops and logic
- Basic MQTT integration
- Local storage configurations
- Alarm setup examples

### Advanced Examples (`examples/advanced/`)
- ClickHouse integration
- S7 PLC communication
- OPC-UA server configuration
- Complex alarm escalation
- Multi-system integration

### Industrial Examples (`examples/industrial/`)
- `industrial-scada.yaml` - Full SCADA system configuration
- `edge-gateway.yaml` - Edge computing gateway setup
- `building-automation.yaml` - HVAC and energy management
- `water-treatment.yaml` - Process control example

## Production Configurations

The `production/` directory contains templates for production deployments. 
These should be customized with your specific:
- Connection strings
- Security credentials
- Network addresses
- Performance tuning parameters

## Configuration Schema

The `schemas/` directory contains JSON schemas that define the valid structure
for Petra configuration files. Use these for:
- Validation in your CI/CD pipeline
- IDE autocomplete and validation
- Documentation of available options

## Getting Started

1. Start with a basic example:
   ```bash
   cargo run -- configs/examples/basic/simple-control.yaml
   ```

2. Validate your configuration:
   ```bash
   cargo run --bin petra -- your-config.yaml --validate-only
   ```

3. Use the visual designer for creating configurations:
   ```bash
   cd petra-designer && npm run dev
   ```

## Best Practices

- Always validate configurations before deployment
- Use environment variables for sensitive data
- Start with minimal features and add complexity gradually
- Test configurations in a safe environment first
- Keep production configs in version control (with secrets removed)
- Use the schema for validation in your editor
