#!/bin/bash

# Performance profiling script for the evolution simulation
set -e

echo "=== Evolution Simulation Performance Profiling ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Create output directory
mkdir -p profiling_output

# Function to run benchmarks
run_benchmarks() {
    print_status "Running benchmarks..."
    
    if command -v cargo &> /dev/null; then
        cargo bench --bench simulation_benchmark -- --verbose
        print_success "Benchmarks completed"
    else
        print_error "cargo not found"
        return 1
    fi
}

# Function to run profiling with flamegraph
run_flamegraph() {
    print_status "Generating flamegraph..."
    
    if command -v flamegraph &> /dev/null; then
        flamegraph --output profiling_output/flamegraph.svg -- cargo run --release -- --headless --steps 1000
        print_success "Flamegraph generated: profiling_output/flamegraph.svg"
    else
        print_warning "flamegraph not found, skipping flamegraph generation"
        print_status "Install with: cargo install flamegraph"
    fi
}

# Function to run performance profiling
run_performance_profile() {
    print_status "Running performance profiling..."
    
    # Run with different configurations to identify bottlenecks
    echo "=== Testing different entity counts ==="
    for entities in 100 500 1000 2000; do
        print_status "Testing with $entities entities..."
        time cargo run --release -- --headless --steps 100 --config config.json 2>&1 | tee "profiling_output/profile_${entities}_entities.log"
    done
    
    echo "=== Testing different world sizes ==="
    for world_size in 300 600 1200; do
        print_status "Testing with world size $world_size..."
        time cargo run --release -- --headless --steps 100 --world-size $world_size 2>&1 | tee "profiling_output/profile_world_${world_size}.log"
    done
}

# Function to run memory profiling
run_memory_profile() {
    print_status "Running memory profiling..."
    
    if command -v valgrind &> /dev/null; then
        valgrind --tool=massif --massif-out-file=profiling_output/massif.out cargo run --release -- --headless --steps 100
        print_success "Memory profile generated: profiling_output/massif.out"
        
        if command -v ms_print &> /dev/null; then
            ms_print profiling_output/massif.out > profiling_output/memory_profile.txt
            print_success "Memory profile summary: profiling_output/memory_profile.txt"
        fi
    else
        print_warning "valgrind not found, skipping memory profiling"
    fi
}

# Function to analyze performance data
analyze_performance() {
    print_status "Analyzing performance data..."
    
    echo "=== Performance Summary ===" > profiling_output/analysis.txt
    echo "Generated at: $(date)" >> profiling_output/analysis.txt
    echo "" >> profiling_output/analysis.txt
    
    # Analyze log files for timing information
    for log_file in profiling_output/profile_*.log; do
        if [ -f "$log_file" ]; then
            echo "=== Analysis of $(basename $log_file) ===" >> profiling_output/analysis.txt
            grep -E "(Performance|Bottleneck|Slow operation)" "$log_file" >> profiling_output/analysis.txt || true
            echo "" >> profiling_output/analysis.txt
        fi
    done
    
    print_success "Performance analysis saved to: profiling_output/analysis.txt"
}

# Function to generate performance report
generate_report() {
    print_status "Generating performance report..."
    
    cat > profiling_output/performance_report.md << EOF
# Evolution Simulation Performance Report

Generated on: $(date)

## Summary

This report contains performance analysis results for the evolution simulation.

## Files Generated

- \`flamegraph.svg\`: CPU usage flamegraph (if available)
- \`memory_profile.txt\`: Memory usage analysis (if available)
- \`analysis.txt\`: Detailed performance analysis
- \`profile_*.log\`: Individual profiling runs

## Recommendations

Based on the profiling results, consider the following optimizations:

1. **Spatial Grid Optimization**: If grid operations are slow, consider:
   - Adjusting cell size
   - Using a more efficient spatial data structure
   - Implementing grid partitioning

2. **Entity Processing**: If entity processing is slow, consider:
   - Reducing the number of nearby entity checks
   - Optimizing interaction calculations
   - Using more efficient algorithms

3. **Memory Usage**: If memory usage is high, consider:
   - Reducing entity count
   - Optimizing data structures
   - Implementing object pooling

4. **Parallelization**: If single-threaded operations are slow, consider:
   - Increasing parallel processing
   - Optimizing thread pool usage
   - Reducing synchronization overhead

## Next Steps

1. Review the flamegraph for CPU bottlenecks
2. Check memory profile for memory leaks
3. Analyze specific slow operations
4. Implement targeted optimizations
5. Re-run profiling to measure improvements

EOF

    print_success "Performance report generated: profiling_output/performance_report.md"
}

# Main execution
main() {
    print_status "Starting performance profiling..."
    
    # Run different profiling tools
    run_benchmarks || print_warning "Benchmarks failed"
    run_flamegraph || print_warning "Flamegraph generation failed"
    run_performance_profile || print_warning "Performance profiling failed"
    run_memory_profile || print_warning "Memory profiling failed"
    
    # Analyze results
    analyze_performance
    generate_report
    
    print_success "Profiling completed! Check the profiling_output/ directory for results."
    print_status "Files generated:"
    ls -la profiling_output/
}

# Run main function
main "$@" 