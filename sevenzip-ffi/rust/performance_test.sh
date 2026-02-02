#!/bin/bash
#
# Quick performance test script for sevenzip-ffi
# This runs key benchmarks and outputs results
#

set -e

cd "$(dirname "$0")"

echo "=================================================="
echo "sevenzip-ffi Performance Test Suite"
echo "=================================================="
echo ""

# Run unit tests first
echo "→ Running unit tests..."
cargo test --lib --release --quiet
echo "✓ All unit tests passed"
echo ""

# Run integration tests
echo "→ Running integration tests..."
cargo test --test integration_tests --release --quiet
echo "✓ All integration tests passed"
echo ""

# Run benchmarks
echo "→ Running benchmarks (this may take several minutes)..."
echo ""

# Run quick benchmark to verify setup
cargo bench --bench compression_benchmarks -- --quick

echo ""
echo "=================================================="
echo "✓ Performance tests completed successfully!"
echo ""
echo "Full benchmark results available at:"
echo "  target/criterion/report/index.html"
echo ""
echo "To run full benchmarks with detailed results:"
echo "  cargo bench"
echo "=================================================="
