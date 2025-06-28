#!/bin/bash
# Setup development environment for Petra

set -e

echo "Setting up Petra development environment"

# Check for required tools
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "$1 is required but not installed."
        exit 1
    fi
}

check_command rustc
check_command cargo
check_command docker
check_command docker-compose

# Install development dependencies
echo "Installing development dependencies..."
cargo install cargo-watch cargo-audit cargo-tarpaulin cargo-criterion

# Install pre-commit hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for Petra

# Format code
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy
cargo clippy --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "Clippy warnings found."
    exit 1
fi

# Run tests
cargo test --no-default-features
if [ $? -ne 0 ]; then
    echo "Tests failed."
    exit 1
fi

echo "All pre-commit checks passed!"
EOF

chmod +x .git/hooks/pre-commit

# Create local development config
mkdir -p .dev
cat > .dev/config.yaml << 'EOF'
# Development configuration
signals:
  - name: dev_signal
    type: float
    initial: 0.0

mqtt:
  broker_host: localhost
  broker_port: 1883
  client_id: petra-dev

scan_time_ms: 100
EOF

# Start development services
echo "🐳 Starting development services..."
docker-compose -f docker/compose/docker-compose.dev.yml up -d

echo "Development environment ready!"
echo ""
echo "Quick commands:"
echo "  cargo run -- .dev/config.yaml    # Run with dev config"
echo "  cargo watch -x test              # Watch and test"
echo "  cargo watch -x 'run -- .dev/config.yaml'  # Watch and run"
