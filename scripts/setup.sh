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

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js:"
    echo "   macOS: brew install node"
    echo "   Ubuntu/Debian: sudo apt install nodejs npm"
    echo "   Windows: Download from https://nodejs.org/"
    exit 1
else
    echo "✅ Node.js is already installed"
fi

# Test build
echo "🔨 Testing build..."
cargo check --target wasm32-unknown-unknown

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Quick start commands:"
echo "  just desktop         - Run desktop application"
echo "  just web             - Run web application"
echo "  just headless        - Run headless simulation"
echo "  just                 - Show all commands"
echo ""
echo "For more information, see README.md" 