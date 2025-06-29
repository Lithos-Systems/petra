# Petra Installation Quick Reference

## Minimum System Requirements
- Ubuntu 20.04+ / Debian 11+ / RHEL 8+
- 2GB RAM (4GB recommended)
- 10GB disk space
- x86_64 or ARM64 architecture

## Essential Dependencies
```bash
# One-liner for Ubuntu/Debian
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev libpq-dev g++ llvm clang libclang-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source "$HOME/.cargo/env"
