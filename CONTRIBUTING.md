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

Code Style

Follow Rust standard naming conventions
Use rustfmt for formatting
Add documentation comments for public APIs
Keep functions focused and small
Prefer explicit error handling over unwrap()

Testing

Write unit tests for new functions
Add integration tests for new features
Aim for 80% code coverage
Test error cases, not just happy paths

Documentation

Update API documentation for public changes
Add examples for new features
Update README if needed
Add configuration examples

Feature Requests

Open an issue with [Feature Request] prefix
Describe the use case
Provide examples if possible
Be open to discussion and alternatives

Release Process

Update CHANGELOG.md
Run full test suite
Update version in Cargo.toml
Tag release with v prefix (e.g., v0.1.0)
GitHub Actions will handle the rest

Getting Help

Discord: Join our server
GitHub Discussions: For questions and ideas
Email: support@petra.systems

License
By contributing, you agree that your contributions will be licensed under the AGPL-3.0 license.
