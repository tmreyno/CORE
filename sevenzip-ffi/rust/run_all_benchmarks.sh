#!/bin/bash
#
# Run all benchmarks with live progress and results display
#

set -e

cd "$(dirname "$0")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo "=================================================="
echo -e "${CYAN}sevenzip-ffi Comprehensive Benchmark Suite${NC}"
echo "=================================================="
echo ""
echo "Platform: $(uname -s) $(uname -m)"
echo "Date: $(date)"
echo "Rust: $(rustc --version)"
echo ""
echo "This will run 8 benchmark categories:"
echo "  1. Compression Levels"
echo "  2. Data Types"
echo "  3. File Sizes"
echo "  4. Extraction Performance"
echo "  5. Encryption Overhead"
echo "  6. Encrypted Extraction"
echo "  7. Multi-threading"
echo "  8. Multiple Files"
echo ""
echo "Estimated time: 20-30 minutes"
echo "=================================================="
echo ""

# Build benchmarks first
echo -e "${BLUE}→ Building benchmarks...${NC}"
cargo bench --bench compression_benchmarks --no-run
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Run benchmarks with progress
echo -e "${BLUE}→ Running benchmarks...${NC}"
echo ""

# Capture output and display progress
cargo bench --bench compression_benchmarks -- --verbose 2>&1 | while IFS= read -r line; do
    # Highlight benchmark names
    if echo "$line" | grep -q "Benchmarking"; then
        echo -e "${YELLOW}$line${NC}"
    # Highlight completion
    elif echo "$line" | grep -q "time:"; then
        echo -e "${GREEN}$line${NC}"
    # Highlight throughput
    elif echo "$line" | grep -q "thrpt:"; then
        echo -e "${CYAN}$line${NC}"
    # Regular output
    else
        echo "$line"
    fi
done

echo ""
echo "=================================================="
echo -e "${GREEN}✓ All benchmarks complete!${NC}"
echo "=================================================="
echo ""

# Extract and display summary results
echo -e "${CYAN}Performance Summary:${NC}"
echo ""

# Function to extract and display results
display_results() {
    local group=$1
    local title=$2
    
    if [ -d "target/criterion/$group" ]; then
        echo -e "${BLUE}$title:${NC}"
        for subdir in target/criterion/$group/*/new/estimates.json; do
            if [ -f "$subdir" ]; then
                local name=$(dirname $(dirname $subdir))
                name=$(basename $name)
                local result=$(python3 -c "
import json
try:
    d = json.load(open('$subdir'))
    mean_ms = d['mean']['point_estimate'] / 1e6
    if 'throughput' in d:
        tp = d['throughput']
        if tp:
            bytes_val = tp['per_iteration']
            throughput_mbs = bytes_val / (d['mean']['point_estimate'] / 1e9) / 1024 / 1024
            print(f'  {name:20} {mean_ms:8.2f} ms    {throughput_mbs:8.2f} MB/s')
        else:
            print(f'  {name:20} {mean_ms:8.2f} ms')
    else:
        print(f'  {name:20} {mean_ms:8.2f} ms')
except Exception as e:
    print(f'  {name:20} [data processing]')
" 2>/dev/null)
                echo "$result"
            fi
        done
        echo ""
    fi
}

# Display results for each category
display_results "compression_levels" "Compression Levels (1MB file)"
display_results "compression_data_types" "Data Types (1MB each)"
display_results "compression_sizes" "File Size Scaling"
display_results "extraction_sizes" "Extraction Performance"
display_results "encryption_overhead" "Encryption Overhead (1MB)"
display_results "encryption_extraction" "Encrypted Extraction"
display_results "threading_performance" "Multi-threading (5MB)"
display_results "multiple_files" "Multiple Files"

echo "=================================================="
echo ""
echo -e "${CYAN}Detailed HTML Report:${NC}"
echo "  file://$(pwd)/target/criterion/report/index.html"
echo ""
echo "To view:"
echo "  open target/criterion/report/index.html     # macOS"
echo "  xdg-open target/criterion/report/index.html # Linux"
echo ""
echo "=================================================="
