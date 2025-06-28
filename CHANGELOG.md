# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Petra Automation Engine
- Core signal bus with thread-safe DashMap implementation
- S7 PLC communication support via snap7
- MQTT integration with TLS and authentication
- Twilio alerting with SMS/voice escalation
- Advanced alarm management with acknowledgment and escalation chains
- Parquet-based historical data storage
- ClickHouse integration for time-series data
- Visual configuration designer (petra-designer)
- Docker and Docker Compose support
- Comprehensive security module with RBAC
- Real-time engine with configurable scan times
- 15+ built-in logic blocks (AND, OR, PID, etc.)
- Modbus TCP/RTU support (optional feature)
- OPC-UA server support (optional feature)

### Security
- Input validation and sanitization
- Rate limiting for API endpoints
- TLS support for all network communications
- Signed configuration support

## [0.1.0] - 2024-01-XX (Planned)
- Initial public release
