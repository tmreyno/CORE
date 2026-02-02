#!/bin/bash
#
# Generate a comprehensive test and benchmark report
#

set -e

cd "$(dirname "$0")"

REPORT_FILE="TEST_REPORT.md"

echo "Generating comprehensive test report..."

cat > "$REPORT_FILE" << 'EOF'
# sevenzip-ffi Test and Performance Report

Generated: $(date)

## Test Results

### Unit Tests

```
EOF

echo "Running unit tests..."
cargo test --lib --release 2>&1 | tee -a "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'
```

### Integration Tests

```
EOF

echo "Running integration tests..."
cargo test --test integration_tests --release 2>&1 | tee -a "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'
```

## Build Information

EOF

echo "### Cargo Version" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
cargo --version >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "### Rustc Version" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
rustc --version >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "### Target" >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"
rustc -vV | grep host >> "$REPORT_FILE"
echo '```' >> "$REPORT_FILE"

cat >> "$REPORT_FILE" << 'EOF'

## Performance Notes

For detailed benchmark results, run:

```bash
cargo bench
```

Results will be available at: `target/criterion/report/index.html`

### Quick Benchmark Categories

- **Compression Levels**: Tests performance across None/Fast/Normal/Maximum/Ultra
- **Data Types**: Compressible vs incompressible data
- **File Sizes**: 1KB to 5MB
- **Encryption Overhead**: Encrypted vs unencrypted
- **Multi-threading**: 1-8 threads
- **Multiple Files**: 1-50 files

## Test Coverage

- ✓ Library initialization
- ✓ Archive creation and extraction
- ✓ Encryption/decryption with AES-256
- ✓ Password validation
- ✓ Compression level handling
- ✓ Multi-file archives
- ✓ Progress callbacks
- ✓ Error handling
- ✓ Split/multivolume archives
- ✓ Streaming operations

EOF

echo ""
echo "✓ Report generated: $REPORT_FILE"
