#!/bin/bash

# Evolution Simulation - Setup Script
# Installs dependencies and sets up the development environment

set -e

echo "🚀 Evolution Simulation Setup"
echo "=============================="

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✅ Rust installed!"
else
    echo "✅ Rust is already installed"
fi

# Check if we have the correct toolchain (single source: rust-toolchain.toml)
TOOLCHAIN=$(grep '^channel' rust-toolchain.toml | sed -E 's/.*"(.*)".*/\1/')
if ! rustup show | grep -q "$TOOLCHAIN"; then
    echo "📦 Installing Rust $TOOLCHAIN toolchain..."
    rustup toolchain install "$TOOLCHAIN"
fi

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
fi

# Add WASM target
echo "🎯 Adding WASM target..."
rustup target add wasm32-unknown-unknown

# Install npm dependencies
echo "📦 Installing npm dependencies..."
npm install

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Commands:"
echo "  npm run build    - Build WASM"
echo "  npm run dev      - Run local server"
echo "  npm run deploy   - Deploy to Cloudflare"