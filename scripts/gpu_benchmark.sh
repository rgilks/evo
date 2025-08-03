#!/bin/bash

# GPU Acceleration Benchmark Script
# This script compares CPU vs GPU performance for the evolution simulation

echo "🧬 GPU Acceleration Benchmark"
echo "=============================="

# Configuration for large-scale testing
CONFIG_FILE="config_100000.json"
STEPS=100
WORLD_SIZE=600

echo "Testing with configuration: $CONFIG_FILE"
echo "Steps: $STEPS"
echo "World Size: $WORLD_SIZE"
echo ""

# Test CPU performance
echo "🖥️  Testing CPU Performance..."
CPU_START=$(date +%s.%N)
cargo run --release -- --headless --steps $STEPS --world-size $WORLD_SIZE --config $CONFIG_FILE
CPU_END=$(date +%s.%N)
CPU_TIME=$(echo "$CPU_END - $CPU_START" | bc -l)

echo ""
echo "🚀 Testing GPU Performance..."
GPU_START=$(date +%s.%N)
cargo run --release -- --headless --steps $STEPS --world-size $WORLD_SIZE --config $CONFIG_FILE --gpu
GPU_END=$(date +%s.%N)
GPU_TIME=$(echo "$GPU_END - $GPU_START" | bc -l)

echo ""
echo "📊 Performance Results"
echo "====================="
echo "CPU Time: ${CPU_TIME}s"
echo "GPU Time: ${GPU_TIME}s"

# Calculate speedup
if (( $(echo "$GPU_TIME > 0" | bc -l) )); then
    SPEEDUP=$(echo "$CPU_TIME / $GPU_TIME" | bc -l)
    echo "Speedup: ${SPEEDUP}x"
    
    if (( $(echo "$SPEEDUP > 1.0" | bc -l) )); then
        echo "✅ GPU acceleration is faster!"
    else
        echo "⚠️  CPU is faster (GPU overhead may be too high for this scale)"
    fi
else
    echo "❌ GPU test failed or took 0 time"
fi

echo ""
echo "💡 Tips:"
echo "- GPU acceleration works best with 1000+ entities"
echo "- Smaller simulations may be faster on CPU due to GPU overhead"
echo "- Check GPU utilization with: nvidia-smi (NVIDIA) or radeontop (AMD)" 