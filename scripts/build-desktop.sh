#!/bin/bash

# Evolution Simulation Desktop Build Script
# This script builds and runs the desktop application with proper target handling

set -e  # Exit on any error

echo "🚀 Building Evolution Simulation for Desktop..."

# Detect platform and set appropriate target
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - detect architecture
    if [[ $(uname -m) == "arm64" ]]; then
        TARGET="aarch64-apple-darwin"
        echo "📱 Detected macOS Apple Silicon (ARM64)"
    else
        TARGET="x86_64-apple-darwin"
        echo "💻 Detected macOS Intel (x86_64)"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    TARGET="x86_64-unknown-linux-gnu"
    echo "🐧 Detected Linux (x86_64)"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    TARGET="x86_64-pc-windows-msvc"
    echo "🪟 Detected Windows (x86_64)"
else
    echo "⚠️  Unknown platform, using default target"
    TARGET=""
fi

# Check if target is specified
if [ -n "$TARGET" ]; then
    echo "🎯 Using target: $TARGET"
    TARGET_FLAG="--target $TARGET"
else
    TARGET_FLAG=""
fi

# Build the application
echo "🔨 Building application..."
cargo build --release $TARGET_FLAG

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed"
    exit 1
fi

# Check if we should run the application
if [ "$1" != "--no-run" ]; then
    echo "🎮 Starting simulation..."
    echo "   Press Ctrl+C to exit"
    echo ""
    
    # Run the application
    if [ -n "$TARGET" ]; then
        cargo run --release $TARGET_FLAG
    else
        cargo run --release
    fi
else
    echo "📦 Build complete! Run with: cargo run --release $TARGET_FLAG"
fi 