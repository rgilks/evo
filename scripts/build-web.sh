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
    
    # Fix the import path and add cache busting
    sed -i.bak "s|await import('\.\./\.\./\.\.');|await import('../../../evo.js?v=$CACHE_VERSION');|g" "$WORKER_FILE"
    # Version the worker-spawn URL too: `new URL('./workerHelpers.js', import.meta.url)`
    # drops the query, so the rayon worker is otherwise fetched UNVERSIONED. A stale
    # cached copy then imports an old evo.js and the wasm fails to link after a deploy
    # (LinkError: __wbindgen_closure_wrapper… requires a callable).
    sed -i.bak "s|new URL('\./workerHelpers\.js', import\.meta\.url)|new URL('./workerHelpers.js?v=$CACHE_VERSION', import.meta.url)|g" "$WORKER_FILE"
    rm "${WORKER_FILE}.bak"

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
    
    # Fix the import path and add cache busting
    sed -i.bak "s|await import('\.\./\.\./\.\.');|await import('../../../evo.js?v=$CACHE_VERSION');|g" "$WEB_WORKER_FILE"
    # Version the worker-spawn URL too (see the pkg/ block above for why).
    sed -i.bak "s|new URL('\./workerHelpers\.js', import\.meta\.url)|new URL('./workerHelpers.js?v=$CACHE_VERSION', import.meta.url)|g" "$WEB_WORKER_FILE"
    rm "${WEB_WORKER_FILE}.bak"

    if [ $? -eq 0 ]; then
        echo "✅ Web worker import path fixed successfully"
    else
        echo "❌ Failed to fix web worker import path"
        exit 1
    fi
else
    echo "⚠️  No web worker helpers file found"
fi

# Update cache-busting version in app.js
echo "🔄 Updating cache busting version in app.js..."
sed -i.bak "s/from \"\.\.\/pkg\/evo\.js?v=[0-9a-f]*\"/from \"..\/pkg\/evo.js?v=$CACHE_VERSION\"/g" web/js/app.js
rm web/js/app.js.bak

# Update cache-busting version for WASM fetch in evo.js
echo "🔄 Updating cache busting version for WASM fetch in evo.js..."
sed -i.bak "s/'evo_bg\.wasm'/'evo_bg.wasm?v=$CACHE_VERSION'/g" pkg/evo.js
sed -i.bak "s/'evo_bg\.wasm'/'evo_bg.wasm?v=$CACHE_VERSION'/g" web/pkg/evo.js

# Update cache-busting version for workerHelpers.js import in evo.js
echo "🔄 Updating cache busting version for workerHelpers.js import in evo.js..."
sed -i.bak "s|from './snippets/|from './snippets/|g; s|workerHelpers.js'|workerHelpers.js?v=$CACHE_VERSION'|g" pkg/evo.js
sed -i.bak "s|from './snippets/|from './snippets/|g; s|workerHelpers.js'|workerHelpers.js?v=$CACHE_VERSION'|g" web/pkg/evo.js
rm pkg/evo.js.bak web/pkg/evo.js.bak

# Update cache-busting version in index.html
echo "🔄 Updating cache busting version in index.html..."
sed -i.bak "s/src=\"js\/app\.js?v=[0-9a-f]*\"/src=\"js\/app.js?v=$CACHE_VERSION\"/g" web/index.html
# Cache-bust the stylesheet too — without a ?v= a stale style.css is served after a deploy.
sed -i.bak "s|href=\"css/style.css?v=[0-9a-z]*\"|href=\"css/style.css?v=$CACHE_VERSION\"|g" web/index.html
rm web/index.html.bak

# Verify the build
echo "🔍 Verifying build..."
if [ -f "pkg/evo.js" ] && [ -f "pkg/evo_bg.wasm" ]; then
    echo "✅ Build verification passed"
else
    echo "❌ Build verification failed - missing required files"
    exit 1
fi

echo "🎉 Build complete! Run 'npm run dev' to start the server."
echo "📁 Built files:"
ls -la pkg/
echo "🔢 Cache version: $CACHE_VERSION" 