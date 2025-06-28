# Contributing to Petra

Thank you for your interest in contributing to Petra! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct:
- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Respect differing viewpoints and experiences

## How to Contribute

### Reporting Issues

1. Check if the issue already exists
2. Create a new issue with a clear title and description
3. Include:
   - Petra version (`petra --version`)
   - Operating system and version
   - Relevant configuration files (sanitized)
   - Steps to reproduce
   - Expected vs actual behavior

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test --all-features`)
6. Run clippy (`cargo clippy --all-features -- -D warnings`)
7. Format code (`cargo fmt`)
8. Commit with clear messages
9. Push to your fork
10. Open a Pull Request

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-fork/petra
cd petra

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development dependencies
cargo install cargo-watch cargo-audit cargo-tarpaulin

# Run tests
cargo test --all-features

# Run with example config
cargo run --release -- configs/examples/simple-mqtt.yaml
