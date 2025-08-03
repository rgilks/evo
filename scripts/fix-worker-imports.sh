#!/bin/bash

# Fix worker import paths after wasm-pack build
echo "🔧 Fixing worker import paths..."

# Find the worker helpers file
WORKER_FILE=$(find pkg/snippets -name "workerHelpers.js" -type f 2>/dev/null | head -n 1)

if [ -z "$WORKER_FILE" ]; then
    echo "❌ Worker helpers file not found"
    exit 1
fi

echo "📁 Found worker file: $WORKER_FILE"

# Fix the import path
sed -i '' 's/await import('\''\.\.\/\.\.\/\.\.'\'');/await import('\''\.\.\/\.\.\/\.\.\/index\.js'\'');/g' "$WORKER_FILE"

if [ $? -eq 0 ]; then
    echo "✅ Worker import path fixed successfully"
else
    echo "❌ Failed to fix worker import path"
    exit 1
fi 