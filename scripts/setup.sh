#!/bin/bash

# Evolution Simulation - Setup Script
# Installs dependencies and sets up the development environment

set -e

echo "🚀 Evolution Simulation Setup"
echo "=============================="
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✅ Rust installed successfully!"
else
    echo "✅ Rust is already installed"
fi

# Check if we have the correct toolchain
if ! rustup show | grep -q "nightly-2024-08-02"; then
    echo "📦 Installing Rust nightly toolchain..."
    rustup toolchain install nightly-2024-08-02
    echo "✅ Nightly toolchain installed!"
else
    echo "✅ Nightly toolchain is already installed"
fi

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
    echo "✅ wasm-pack installed successfully!"
else
    echo "✅ wasm-pack is already installed"
fi

# Add WASM target
echo "🎯 Adding WASM target..."
rustup target add wasm32-unknown-unknown

# Test build
echo "🔨 Testing build..."
cargo check

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Quick start commands:"
echo "  ./run.sh desktop     - Run desktop application"
echo "  ./run.sh web         - Run web application"
echo "  ./run.sh headless    - Run headless simulation"
echo "  ./run.sh help        - Show all commands"
echo ""
echo "For more information, see README.md" 