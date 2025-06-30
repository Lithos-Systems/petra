# Petra Quick Start

## 30-Second Install

```bash
# Option 1: Download binary (fastest)
# curl -sSL https://lithos.systems/install.sh | bash

# Option 2: Docker (no dependencies)
docker run -d \
  -v $(pwd)/config.yaml:/app/config.yaml \
  ghcr.io/your-org/petra:latest

# Option 3: Build from source
git clone https://github.com/your-org/petra && cd petra
./install.sh source
