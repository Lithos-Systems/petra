#!/bin/bash
set -e

# Build frontend
cd petra-designer
npm install
npm run build
cd ..

# Build backend with web support
cargo build --features "web,mqtt,history,scada"
