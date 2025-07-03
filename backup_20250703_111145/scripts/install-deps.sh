#!/bin/bash
# Petra Development Environment Setup Script
# Tested on Ubuntu 20.04+, Debian 11+

set -e

echo "🚀 Setting up Petra development environment..."
echo "This script will install all required dependencies for building Petra"
echo ""

# Update package lists
echo "📦 Updating package lists..."
sudo apt update

# Install basic build tools
echo "🔧 Installing basic build tools..."
sudo apt install -y \
    build-essential \
    pkg-config \
    curl \
    git \
    cmake

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✓ Rust already installed ($(rustc --version))"
fi

# Update Rust to latest stable
echo "🔄 Updating Rust to latest stable..."
rustup update stable
rustup default stable

# Install C++ compiler for rust-snap7
echo "🔧 Installing C++ compiler and tools..."
sudo apt install -y \
    g++ \
    gcc \
    make

# Install OpenSSL and PostgreSQL development libraries
echo "🔐 Installing OpenSSL and PostgreSQL libraries..."
sudo apt install -y \
    libssl-dev \
    libpq-dev

# Install LLVM/Clang for bindgen (required by zstd-sys and other -sys crates)
echo "🔗 Installing LLVM/Clang for bindgen..."
sudo apt install -y \
    llvm \
    clang \
    libclang-dev

# Install Docker and Docker Compose
if ! command -v docker &> /dev/null; then
    echo "🐳 Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
else
    echo "✓ Docker already installed"
fi

if ! command -v docker-compose &> /dev/null; then
    echo "🐳 Installing Docker Compose..."
    sudo apt install -y docker-compose
else
    echo "✓ Docker Compose already installed"
fi

# Install additional libraries that might be needed
echo "📚 Installing additional libraries..."
sudo apt install -y \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libgtk-3-dev

# Install development tools
echo "🛠️  Installing Rust development tools..."
cargo install cargo-watch cargo-audit || true

# Clean up
echo "🧹 Cleaning up..."
sudo apt autoremove -y
sudo apt autoclean

echo ""
echo "✅ Installation complete!"
echo ""
echo "⚠️  IMPORTANT: If Docker was just installed, you need to log out and back in"
echo "   for the docker group changes to take effect, or run: newgrp docker"
echo ""
echo "📝 Next steps:"
echo "   1. Clone the Petra repository: git clone <repo-url>"
echo "   2. cd petra"
echo "   3. Build with: cargo build --release"
echo "   4. Or build specific features: cargo build --release --features advanced-storage"
echo ""
echo "🔍 To verify installation:"
echo "   rustc --version"
echo "   cargo --version"
echo "   docker --version"
echo "   clang --version"
