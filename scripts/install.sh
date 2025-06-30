#!/bin/bash
# install.sh - One-stop installation script for Petra

set -e

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

# Installation modes
case "${1:-quick}" in
    "quick")
        echo "🚀 Quick Install - Binary download"
        curl -L https://github.com/your-org/petra/releases/latest/download/petra-$OS-amd64.tar.gz | tar xz
        sudo mv petra /usr/local/bin/
        echo "✅ Petra installed! Run: petra --version"
        ;;
    
    "docker")
        echo "🐳 Docker Install"
        docker pull ghcr.io/your-org/petra:latest
        echo "✅ Docker image ready! Run: docker run ghcr.io/your-org/petra:latest"
        ;;
    
    "source")
        echo "📦 Source Install"
        ./scripts/install-deps.sh
        cargo build --release
        sudo cp target/release/petra /usr/local/bin/
        echo "✅ Built from source! Run: petra --version"
        ;;
    
    "dev")
        echo "🔧 Development Install"
        ./scripts/install-deps.sh
        cargo install cargo-watch cargo-nextest cargo-deny
        echo "✅ Dev environment ready! Run: cargo test"
        ;;
esac
