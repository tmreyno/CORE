#!/bin/bash
# Quick benchmark of smart features

cd /Users/terryreynolds/GitHub/sevenzip-ffi/rust

echo "🔥 Running key performance benchmarks..."
echo ""
echo "This will take ~2-3 minutes..."
echo ""

# Run only the new smart benchmarks
cargo bench --bench compression_benchmarks \
    -- "incompressible_detection" "smart_threading" "convenience_methods" \
    2>&1 | grep -E "(Benchmarking|time:|thrpt:|change:)" | head -60

echo ""
echo "✅ Benchmark complete!"
echo ""
echo "Key findings:"
echo "• Incompressible detection: Check 'with_auto_detect' vs 'without_auto_detect'"
echo "• Smart threading: Check 'auto_threads' vs 'manual_threads'"  
echo "• Convenience methods: All should have similar performance"
echo ""
