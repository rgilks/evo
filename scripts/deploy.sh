#!/bin/bash

# Evolution Simulation - Cloudflare Deployment Script
# Deploys the web application to Cloudflare Workers

set -e

echo "🚀 Evolution Simulation - Cloudflare Deployment"
echo "================================================"
echo ""

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ Wrangler CLI not found. Installing..."
    npm install -g wrangler
    echo "✅ Wrangler CLI installed!"
fi

# Check if user is logged in to Cloudflare
if ! wrangler whoami &> /dev/null; then
    echo "🔐 Please login to Cloudflare..."
    wrangler login
fi

# Build the web application
echo "🔨 Building web application..."
npm run build:web

# Check if build was successful
if [ ! -f "pkg/evo_bg.wasm" ]; then
    echo "❌ Build failed - WASM file not found"
    exit 1
fi

echo "✅ Build completed successfully!"

# Deploy to Cloudflare
echo "🚀 Deploying to Cloudflare Workers..."
wrangler deploy

echo ""
echo "🎉 Deployment completed!"
echo ""
echo "Your application should be available at:"
echo "https://evo-simulation.your-subdomain.workers.dev"
echo ""
echo "To view logs: wrangler tail"
echo "To update: just deploy" 