# Contributing to Petra

Thank you for your interest in contributing to **Petra**!  
This guide explains the ground-rules for contributing code, documentation, or ideas.

---

## Code of Conduct

By participating in this project you agree to follow our Code of Conduct:

- Be **respectful** and **inclusive**  
- Welcome newcomers and help them get started  
- Keep criticism **constructive**  
- Respect differing viewpoints and experiences  

---

## How to Contribute

### Reporting Issues

1. **Search** the issue tracker to be sure your issue hasn’t already been reported.  
2. If new, open an issue with a clear title and thorough description.  
3. Include:  
   - Petra version (`petra --version`)  
   - Operating system and version  
   - Relevant configuration files (sanitized)  
   - Steps to reproduce  
   - Expected vs actual behavior  

### Submitting Pull Requests

1. **Fork** the repository.  
2. Create a feature branch  
   ```bash
   git checkout -b feature/amazing-feature

    Make your changes.

    Add tests for new functionality.

    Ensure all tests pass

cargo test --all-features

Run Clippy (no warnings allowed)

cargo clippy --all-features -- -D warnings

Format the code

    cargo fmt

    Commit with clear, descriptive messages.

    Push to your fork.

    Open a Pull Request (PR) against main.

Development Setup

# Clone your fork
git clone https://github.com/your-fork/petra
cd petra

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development helpers
cargo install cargo-watch cargo-audit cargo-tarpaulin

# Run tests
cargo test --all-features

# Run with an example config
cargo run --release -- configs/examples/simple-mqtt.yaml

Code Style

    Follow standard Rust naming conventions.

    Run rustfmt for consistent formatting.

    Document all public APIs with /// comments.

    Keep functions focused and small.

    Prefer explicit error handling over unwrap().

Testing

    Write unit tests for new functions.

    Add integration tests for new features.

    Aim for ≥ 80 % code coverage.

    Test failure paths, not just the happy path.

Documentation

    Update API docs for any public changes.

    Provide runnable examples for new features.

    Update the README when behavior or usage changes.

    Include configuration examples where helpful.

Feature Requests

    Open an issue prefixed with [Feature Request].

    Describe the use-case and motivation.

    Provide examples if possible.

    Be open to discussion and alternatives.

Release Process

    Update CHANGELOG.md.

    Run the full test suite.

    Bump the version in Cargo.toml.

    Tag the commit, e.g. git tag v0.1.0.

    Push the tag – GitHub Actions will publish the release.

Getting Help

    Discord – join our community server.

    GitHub Discussions – ask questions, share ideas.

    Email – support@petra.systems for private support.

License

By contributing you agree that your work will be licensed under the GNU AGPL-3.0.
