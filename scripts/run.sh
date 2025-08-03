#!/bin/bash

# Evolution Simulation - Simple Run Script
# Provides easy access to common commands

set -e

echo "🎮 Evolution Simulation"
echo "========================"
echo ""

# Check if command is provided
if [ $# -eq 0 ]; then
    echo "Usage: ./run.sh [command]"
    echo ""
    echo "Commands:"
    echo "  desktop     - Run desktop application with UI"
    echo "  headless    - Run headless simulation"
    echo "  web         - Build and serve web application"
    echo "  test        - Run tests"
    echo "  build       - Build only (no run)"
    echo "  clean       - Clean build artifacts"
    echo "  help        - Show this help"
    echo ""
    echo "Examples:"
    echo "  ./run.sh desktop"
    echo "  ./run.sh headless --steps 1000"
    echo "  ./run.sh web"
    exit 1
fi

COMMAND=$1
shift  # Remove first argument, pass rest to cargo

case $COMMAND in
    "desktop")
        echo "🚀 Starting desktop application..."
        ./scripts/build-desktop.sh
        ;;
    "headless")
        echo "🖥️  Starting headless simulation..."
        cargo run --release -- --headless "$@"
        ;;
    "web")
        echo "🌐 Building and serving web application..."
        ./scripts/build-web.sh
        ;;
    "test")
        echo "🧪 Running tests..."
        # Use native target to avoid build-std issues
        if [[ "$OSTYPE" == "darwin"* ]]; then
            if [[ $(uname -m) == "arm64" ]]; then
                cargo test --target aarch64-apple-darwin
            else
                cargo test --target x86_64-apple-darwin
            fi
        else
            cargo test
        fi
        ;;
    "build")
        echo "🔨 Building application..."
        ./scripts/build-desktop.sh --no-run
        ;;
    "clean")
        echo "🧹 Cleaning build artifacts..."
        cargo clean
        ;;
    "help")
        echo "Usage: ./run.sh [command]"
        echo ""
        echo "Commands:"
        echo "  desktop     - Run desktop application with UI"
        echo "  headless    - Run headless simulation"
        echo "  web         - Build and serve web application"
        echo "  test        - Run tests"
        echo "  build       - Build only (no run)"
        echo "  clean       - Clean build artifacts"
        echo "  help        - Show this help"
        ;;
    *)
        echo "❌ Unknown command: $COMMAND"
        echo "Run './run.sh help' for available commands"
        exit 1
        ;;
esac 