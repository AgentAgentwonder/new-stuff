#!/bin/bash

# Data Pipeline Benchmarking and Profiling Script
# This script runs benchmarks and generates profiling data

set -e

echo "======================================"
echo " Data Pipeline Benchmark & Profiling"
echo "======================================"
echo ""

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

cd "$(dirname "$0")/../src-tauri"

# Function to run benchmarks
run_benchmarks() {
    echo -e "${BLUE}Running Criterion Benchmarks...${NC}"
    echo ""
    cargo bench --bench price_engine_bench
    echo ""
    echo -e "${GREEN}✓ Benchmarks complete!${NC}"
    echo "  Results saved to target/criterion/"
    echo ""
}

# Function to run with perf (Linux only)
run_perf() {
    if ! command -v perf &> /dev/null; then
        echo -e "${YELLOW}⚠ perf not found. Install with: sudo apt-get install linux-tools-generic${NC}"
        return
    fi
    
    echo -e "${BLUE}Running perf profiling...${NC}"
    echo ""
    
    # Build in release mode
    cargo build --release --bench price_engine_bench
    
    # Run with perf
    perf record -g --call-graph dwarf -- ./target/release/deps/price_engine_bench-* --bench
    
    # Generate report
    perf report
    
    echo ""
    echo -e "${GREEN}✓ Perf profiling complete!${NC}"
    echo "  Data saved to perf.data"
    echo ""
}

# Function to generate flamegraph (requires cargo-flamegraph)
run_flamegraph() {
    if ! command -v flamegraph &> /dev/null; then
        echo -e "${YELLOW}⚠ flamegraph not found. Install with: cargo install flamegraph${NC}"
        return
    fi
    
    echo -e "${BLUE}Generating flamegraph...${NC}"
    echo ""
    
    # Generate flamegraph
    cargo flamegraph --bench price_engine_bench -- --bench
    
    echo ""
    echo -e "${GREEN}✓ Flamegraph generated!${NC}"
    echo "  Open flamegraph.svg in your browser"
    echo ""
}

# Function to run tests with coverage
run_tests() {
    echo -e "${BLUE}Running tests...${NC}"
    echo ""
    cargo test --lib cache_manager -- --nocapture
    cargo test --lib event_store -- --nocapture
    cargo test --lib price_engine -- --nocapture
    echo ""
    echo -e "${GREEN}✓ Tests complete!${NC}"
    echo ""
}

# Main menu
show_menu() {
    echo "Select profiling option:"
    echo "  1) Run benchmarks only"
    echo "  2) Run benchmarks + tests"
    echo "  3) Run benchmarks + perf (Linux only)"
    echo "  4) Run benchmarks + flamegraph"
    echo "  5) Run all (benchmarks + tests + flamegraph)"
    echo "  0) Exit"
    echo ""
    read -p "Enter choice [0-5]: " choice
    
    case $choice in
        1)
            run_benchmarks
            ;;
        2)
            run_benchmarks
            run_tests
            ;;
        3)
            run_benchmarks
            run_perf
            ;;
        4)
            run_benchmarks
            run_flamegraph
            ;;
        5)
            run_benchmarks
            run_tests
            run_flamegraph
            ;;
        0)
            echo "Exiting..."
            exit 0
            ;;
        *)
            echo -e "${YELLOW}Invalid choice. Please try again.${NC}"
            show_menu
            ;;
    esac
}

# Check if arguments provided
if [ $# -eq 0 ]; then
    show_menu
else
    case "$1" in
        bench)
            run_benchmarks
            ;;
        test)
            run_tests
            ;;
        perf)
            run_benchmarks
            run_perf
            ;;
        flamegraph)
            run_benchmarks
            run_flamegraph
            ;;
        all)
            run_benchmarks
            run_tests
            run_flamegraph
            ;;
        *)
            echo "Usage: $0 [bench|test|perf|flamegraph|all]"
            exit 1
            ;;
    esac
fi
