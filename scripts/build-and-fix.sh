#!/bin/bash

# Build WASM and fix worker imports
echo "🚀 Building Evolution Simulation for Web..."

# Build WASM package
wasm-pack build --target web --out-dir pkg

if [ $? -eq 0 ]; then
    echo "✅ WASM package built successfully!"
else
    echo "❌ Failed to build WASM package"
    exit 1
fi

# Fix worker import paths
./scripts/fix-worker-imports.sh

echo "🎉 Build complete! Run 'python3 web/server.py' to start the server." 