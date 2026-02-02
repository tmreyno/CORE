#!/bin/bash
#
# Quick benchmark runner - runs a subset of benchmarks for quick performance testing
#

set -e

cd "$(dirname "$0")"

echo "=================================================="
echo "sevenzip-ffi Quick Benchmark Runner"
echo "=================================================="
echo ""

echo "Building benchmarks in release mode..."
cargo build --release --benches --quiet

echo "✓ Build complete"
echo ""

echo "Running quick benchmarks..."
echo ""
echo "Note: For full benchmark suite, run: cargo bench"
echo ""

# Run a subset of benchmarks with reduced iterations
cargo bench --bench compression_benchmarks \
    -- \
    --measurement-time 5 \
    --sample-size 10 \
    "compression_levels/fast" \
    "compression_levels/normal" \
    "compression_sizes/1KB" \
    "compression_sizes/1MB" \
    "encryption_overhead"

echo ""
echo "=================================================="
echo "✓ Quick benchmarks complete!"
echo ""
echo "View full HTML report at:"
echo "  target/criterion/report/index.html"
echo "=================================================="
