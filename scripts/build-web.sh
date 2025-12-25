#!/bin/bash

set -e  # Exit on any error

echo "🚀 Building Evolution Simulation for Web..."

# Generate cache busting version number
if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
    # Use git commit hash (first 8 characters) if in a git repository
    CACHE_VERSION=$(git rev-parse --short HEAD)
    echo "🔢 Generated cache version from git: $CACHE_VERSION"
else
    # Fall back to timestamp if not in git repository
    CACHE_VERSION=$(date +%s)
    echo "🔢 Generated cache version from timestamp: $CACHE_VERSION"
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg/

# Build WASM package
echo "🔨 Building WASM package..."
export CARGO_UNSTABLE_BUILD_STD=std,panic_abort
wasm-pack build --target web --out-dir pkg

if [ $? -eq 0 ]; then
    echo "✅ WASM package built successfully!"
else
    echo "❌ Failed to build WASM package"
    exit 1
fi

# Fix worker import paths for wasm-bindgen-rayon
echo "🔧 Fixing worker import paths..."
WORKER_FILE=$(find pkg/snippets -name "workerHelpers.js" -type f 2>/dev/null | head -n 1)

if [ -n "$WORKER_FILE" ]; then
    echo "📁 Found worker file: $WORKER_FILE"
    
    # Fix the import path from '../../..' to '../../../evo.js'
    sed -i '' 's/await import('\''\.\.\/\.\.\/\.\.'\'');/await import('\''\.\.\/\.\.\/\.\.\/evo\.js'\'');/g' "$WORKER_FILE"
    
    if [ $? -eq 0 ]; then
        echo "✅ Worker import path fixed successfully"
    else
        echo "❌ Failed to fix worker import path"
        exit 1
    fi
else
    echo "⚠️  No worker helpers file found (this is normal if not using rayon)"
fi

# Copy files to web directory first
echo "📁 Copying WASM files to web directory..."
cp -r pkg web/

# Fix worker import paths in web directory as well
WEB_WORKER_FILE=$(find web/pkg/snippets -name "workerHelpers.js" -type f 2>/dev/null | head -n 1)

if [ -n "$WEB_WORKER_FILE" ]; then
    echo "📁 Found web worker file: $WEB_WORKER_FILE"
    
    # Fix the import path from '../../..' to '../../../evo.js'
    sed -i '' 's/await import('\''\.\.\/\.\.\/\.\.'\'');/await import('\''\.\.\/\.\.\/\.\.\/evo\.js'\'');/g' "$WEB_WORKER_FILE"
    
    if [ $? -eq 0 ]; then
        echo "✅ Web worker import path fixed successfully"
    else
        echo "❌ Failed to fix web worker import path"
        exit 1
    fi
else
    echo "⚠️  No web worker helpers file found"
fi

# Update cache busting version in app.js
echo "🔄 Updating cache busting version in app.js..."
sed -i '' "s/from \"\.\.\/pkg\/evo\.js?v=[0-9]*\"/from \"..\/pkg\/evo.js?v=$CACHE_VERSION\"/g" web/js/app.js

# Update cache busting version in index.html
echo "🔄 Updating cache busting version in index.html..."
sed -i '' "s/src=\"js\/app\.js?v=[0-9]*\"/src=\"js\/app.js?v=$CACHE_VERSION\"/g" web/index.html

# Verify the build
echo "🔍 Verifying build..."
if [ -f "pkg/evo.js" ] && [ -f "pkg/evo_bg.wasm" ]; then
    echo "✅ Build verification passed"
else
    echo "❌ Build verification failed - missing required files"
    exit 1
fi

echo "🎉 Build complete! Run 'node server.js' to start the server."
echo "📁 Built files:"
ls -la pkg/
echo "🔢 Cache version: $CACHE_VERSION" 