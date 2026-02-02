#!/bin/bash
#
# Simple benchmark runner with real-time output
# Tests key performance scenarios
#

set -e

cd "$(dirname "$0")"

echo "=================================================="
echo "sevenzip-ffi Performance Benchmark"
echo "=================================================="
echo ""
echo "Testing compression and encryption performance..."
echo ""

# Create temporary test files
mkdir -p /tmp/7z_bench
dd if=/dev/urandom of=/tmp/7z_bench/test_1mb.dat bs=1048576 count=1 2>/dev/null
dd if=/dev/urandom of=/tmp/7z_bench/test_10mb.dat bs=1048576 count=10 2>/dev/null

echo "✓ Test files created"
echo ""

# Test 1: Basic compression
echo "→ Test 1: Compression (1MB, Fast)"
time cargo run --release --example demo -- \
    create /tmp/7z_bench/test_fast.7z /tmp/7z_bench/test_1mb.dat --level fast 2>/dev/null || true
echo ""

# Test 2: Maximum compression
echo "→ Test 2: Compression (1MB, Maximum)"
time cargo run --release --example demo -- \
    create /tmp/7z_bench/test_max.7z /tmp/7z_bench/test_1mb.dat --level maximum 2>/dev/null || true
echo ""

# Test 3: Encrypted compression
echo "→ Test 3: Encrypted (1MB, Fast)"
time cargo run --release --example encryption -- \
    /tmp/7z_bench/test_1mb.dat /tmp/7z_bench/test_enc.7z password123 2>/dev/null || true
echo ""

# Test 4: Large file
echo "→ Test 4: Large file (10MB, Normal)"
time cargo run --release --example demo -- \
    create /tmp/7z_bench/test_large.7z /tmp/7z_bench/test_10mb.dat --level normal 2>/dev/null || true
echo ""

# Show results
echo "=================================================="
echo "Results:"
echo ""
echo "Archive sizes:"
ls -lh /tmp/7z_bench/*.7z | awk '{print $9": "$5}'
echo ""

# Compression ratios
ORIG_SIZE=$(stat -f%z /tmp/7z_bench/test_1mb.dat 2>/dev/null || stat -c%s /tmp/7z_bench/test_1mb.dat)
FAST_SIZE=$(stat -f%z /tmp/7z_bench/test_fast.7z 2>/dev/null || stat -c%s /tmp/7z_bench/test_fast.7z)
MAX_SIZE=$(stat -f%z /tmp/7z_bench/test_max.7z 2>/dev/null || stat -c%s /tmp/7z_bench/test_max.7z)

FAST_RATIO=$(echo "scale=2; $FAST_SIZE * 100 / $ORIG_SIZE" | bc)
MAX_RATIO=$(echo "scale=2; $MAX_SIZE * 100 / $ORIG_SIZE" | bc)

echo "Compression ratios (1MB file):"
echo "  Fast:    ${FAST_RATIO}% of original"
echo "  Maximum: ${MAX_RATIO}% of original"
echo ""

# Cleanup
rm -rf /tmp/7z_bench

echo "=================================================="
echo "✓ Benchmark complete!"
echo ""
echo "For detailed benchmarks, run:"
echo "  cargo bench"
echo "=================================================="
