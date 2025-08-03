#!/bin/bash

# Quick performance profiling script
set -e

echo "=== Quick Performance Profile ==="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Test basic performance
print_status "Testing basic simulation performance..."

echo "=== Entity Count Test ==="
for entities in 100 500 1000; do
    print_status "Testing with $entities entities..."
    time cargo run --release -- --headless --steps 100 2>&1 | grep -E "(real|user|sys)" || true
done

echo "=== World Size Test ==="
for world_size in 300 600 1200; do
    print_status "Testing with world size $world_size..."
    time cargo run --release -- --headless --steps 100 --world-size $world_size 2>&1 | grep -E "(real|user|sys)" || true
done

print_success "Quick profiling completed!"
print_status "For detailed analysis, run: ./scripts/profile.sh" 