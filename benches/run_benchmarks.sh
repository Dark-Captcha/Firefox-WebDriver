#!/bin/bash
# Run multi-window benchmarks and generate plots
#
# Usage: ./benches/run_benchmarks.sh
#
# Output:
#   - benches/results/benchmark_results.csv
#   - docs/images/benchmark_spawn.png
#   - docs/images/benchmark_ops.png

set -e

RESULTS_DIR="benches/results"
IMAGES_DIR="docs/images"

mkdir -p "$RESULTS_DIR"
mkdir -p "$IMAGES_DIR"

echo "=== Firefox WebDriver Benchmarks ==="
echo ""

# Run the simple benchmark script (faster than criterion for quick results)
echo "[1] Running benchmarks..."
cargo run --release --example bench_runner 2>&1 | tee "$RESULTS_DIR/benchmark_output.txt"

echo ""
echo "[2] Generating plots..."
python3 benches/plot_results.py

echo ""
echo "=== Done ==="
echo "Results: $RESULTS_DIR/benchmark_results.csv"
echo "Plots:   $IMAGES_DIR/benchmark_*.png"
