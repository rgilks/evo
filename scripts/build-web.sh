#!/bin/bash

# Evolution Simulation Web Build Script
# This script builds the WASM package and serves the web application

set -e  # Exit on any error

echo "🚀 Building Evolution Simulation for Web..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Check if we're using the correct Rust toolchain
if ! rustup show | grep -q "nightly-2024-08-02"; then
    echo "⚠️  Warning: Not using nightly-2024-08-02 toolchain"
    echo "   Consider running: rustup default nightly-2024-08-02"
fi

# Build the WASM package
echo "📦 Building WASM package..."
wasm-pack build --target web --out-dir pkg

if [ $? -eq 0 ]; then
    echo "✅ WASM package built successfully!"
else
    echo "❌ Failed to build WASM package"
    exit 1
fi

# Fix worker import paths
./scripts/fix-worker-imports.sh

# Check if pkg directory exists and has files
if [ ! -d "pkg" ] || [ -z "$(ls -A pkg)" ]; then
    echo "❌ pkg directory is empty or missing"
    exit 1
fi

echo "🌐 Starting development server..."
echo "   Open your browser to: http://localhost:8000"
echo "   Press Ctrl+C to stop the server"
echo ""

# Start the Python server
python3 web/server.py 