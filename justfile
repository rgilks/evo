# Evolution Simulation - Just Commands
# Install with: cargo install just

default:
    @just --list

# Build commands
build-web: # Build WASM package for web with worker import fixes
    ./scripts/build-web.sh

build-desktop: # Build desktop application with GPU graphics
    ./scripts/build-desktop.sh

build-all: build-web build-desktop # Build both web and desktop targets

# Run commands
run-web: build-web # Build and serve web application
    @echo "🌐 Starting web server at http://localhost:8000"
    node web/server.js

run-desktop: build-desktop # Run desktop application with UI
    @echo "🚀 Starting desktop application..."
    ./target/release/evo

run-headless: # Run headless simulation (faster for testing)
    @echo "🖥️  Starting headless simulation..."
    cargo run --release -- --headless

# Cloudflare deployment commands
deploy: build-web # Build and deploy to Cloudflare
    @echo "🚀 Deploying to Cloudflare..."
    ./scripts/deploy.sh

deploy-staging: build-web # Deploy to staging environment
    @echo "🚀 Deploying to Cloudflare staging..."
    wrangler deploy --env staging

deploy-production: build-web # Deploy to production environment
    @echo "🚀 Deploying to Cloudflare production..."
    wrangler deploy --env production

dev-worker: build-web # Run Cloudflare worker locally
    @echo "🔧 Starting Cloudflare worker locally..."
    npm run dev:worker

# Development commands
test: # Run all tests
    @echo "🧪 Running tests..."
    cargo test --target x86_64-apple-darwin

check: # Check code without building
    @echo "🔍 Checking code..."
    cargo check --target x86_64-apple-darwin

fmt: # Format code with rustfmt
    @echo "🎨 Formatting code..."
    cargo fmt

clippy: # Run clippy linter
    @echo "🔧 Running clippy..."
    cargo clippy --target x86_64-apple-darwin

# Cleanup commands
clean: # Clean all build artifacts
    @echo "🧹 Cleaning build artifacts..."
    cargo clean
    rm -rf pkg/

clean-web: # Clean only web build artifacts
    @echo "🧹 Cleaning web build artifacts..."
    rm -rf pkg/

# Setup commands
setup: # Install dependencies and setup environment
    @echo "⚙️  Setting up development environment..."
    ./scripts/setup.sh

# Shortcuts
web: run-web
desktop: run-desktop
headless: run-headless 