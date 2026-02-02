# sevenzip-ffi Testing & Benchmarking Guide

This document describes the comprehensive test and benchmark suite for the sevenzip-ffi library.

## Quick Start

### Run All Tests
```bash
cd rust
cargo test --all
```

### Run Benchmarks
```bash
cd rust
cargo bench
```

### Quick Performance Test
```bash
cd rust
./quick_bench.sh
```

## Test Organization

### Unit Tests (`cargo test --lib`)

Located in: `src/**/*.rs` (in `#[cfg(test)]` modules)

**Coverage:**
- ✅ Library initialization and versioning
- ✅ Error handling and error code conversion
- ✅ Archive entry metadata
- ✅ Compression level conversion
- ✅ AES encryption constants
- ✅ Encryption context creation
- ✅ Key derivation (PBKDF2)
- ✅ Encryption/decryption roundtrip
- ✅ Password validation
- ✅ Large data encryption (10MB)

**Run:** `cargo test --lib`

### Integration Tests (`cargo test --test integration_tests`)

Located in: `tests/integration_tests.rs`

**Coverage:**
- ✅ Library initialization
- ✅ Archive creation and extraction
- ✅ Encrypted archive operations
- ✅ Password validation (wrong password handling)
- ✅ Archive listing
- ✅ Archive integrity testing
- ✅ All compression levels (None/Fast/Normal/Maximum/Ultra)
- ✅ Multiple files in archive
- ✅ Progress callbacks
- ✅ Streaming compression/extraction
- ✅ Split/multivolume archives
- ✅ Error handling
- ✅ Compression options

**Run:** `cargo test --test integration_tests`

## Benchmark Suite

### Overview

The benchmark suite uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for accurate, statistical performance measurements.

**Location:** `benches/compression_benchmarks.rs`

### Benchmark Categories

#### 1. Compression Levels (`bench_compression_levels`)
Tests compression performance across all levels with 1MB compressible data:
- None
- Fast
- Normal
- Maximum
- Ultra

**Metrics:** Throughput (MB/s), execution time

#### 2. Data Type Performance (`bench_compression_data_types`)
Tests how different data patterns compress:
- Compressible (text-like, repetitive)
- Incompressible (random-like)
- Sequential (predictable patterns)

**Metrics:** Compression ratio, throughput

#### 3. File Size Scaling (`bench_compression_sizes`)
Tests performance across file sizes:
- 1 KB
- 10 KB
- 100 KB
- 1 MB
- 5 MB

**Metrics:** Throughput scaling, overhead analysis

#### 4. Extraction Performance (`bench_extraction_sizes`)
Tests decompression speed across sizes:
- 1 KB → 5 MB

**Metrics:** Decompression throughput

#### 5. Encryption Overhead (`bench_encryption_overhead`)
Compares performance with and without AES-256 encryption (1MB data):
- No encryption baseline
- AES-256-CBC encryption

**Metrics:** Encryption overhead percentage

#### 6. Encrypted Extraction (`bench_encryption_extraction`)
Tests decryption + extraction performance with AES-256.

**Metrics:** Decryption throughput

#### 7. Multi-threading (`bench_threading_performance`)
Tests parallel compression with 5MB data:
- 1 thread
- 2 threads
- 4 threads
- 8 threads

**Metrics:** Scaling efficiency, throughput per thread

#### 8. Multiple Files (`bench_multiple_files`)
Tests archive creation with varying file counts:
- 1 file
- 5 files
- 10 files
- 50 files

**Metrics:** Per-file overhead, total throughput

## Running Benchmarks

### Full Benchmark Suite
```bash
cd rust
cargo bench
```
**Time:** ~20-30 minutes  
**Output:** HTML report at `target/criterion/report/index.html`

### Quick Benchmarks (Reduced Sampling)
```bash
cd rust
./quick_bench.sh
```
**Time:** ~2-5 minutes

### Specific Benchmark
```bash
cargo bench --bench compression_benchmarks -- compression_levels
```

### Compare with Baseline
```bash
# Save baseline
cargo bench -- --save-baseline main

# After changes, compare
cargo bench -- --baseline main
```

## Test Scripts

### `performance_test.sh`
Runs all tests + quick benchmarks:
```bash
./performance_test.sh
```

### `generate_test_report.sh`
Creates comprehensive markdown report with test results:
```bash
./generate_test_report.sh
```
Output: `TEST_REPORT.md`

### `quick_bench.sh`
Runs subset of benchmarks quickly:
```bash
./quick_bench.sh
```

## Continuous Integration

### Recommended CI Configuration

```yaml
# .github/workflows/test.yml
name: Test and Benchmark

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: |
          cd rust
          cargo test --all --verbose
      
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: |
          cd rust
          cargo bench --no-fail-fast
```

## Performance Expectations

### Typical Results (Apple M-series)

**Compression:**
- Fast: 50-100 MB/s
- Normal: 20-40 MB/s
- Maximum: 5-15 MB/s
- Ultra: 2-8 MB/s

**Extraction:**
- Unencrypted: 100-200 MB/s
- Encrypted: 80-150 MB/s

**Encryption Overhead:**
- ~10-20% performance impact
- Depends on compression level

**Multi-threading:**
- 2 threads: 1.6-1.8x speedup
- 4 threads: 2.5-3.2x speedup
- 8 threads: 3.0-4.5x speedup

## Test Data Generation

Benchmarks use synthetic data:
- **Compressible:** Text-like repetitive patterns (high compression ratio)
- **Incompressible:** Pseudo-random data (low compression ratio)
- **Sequential:** Predictable numeric sequences

## Debugging Tests

### Run Single Test
```bash
cargo test test_library_initialization
```

### Run with Output
```bash
cargo test -- --nocapture
```

### Run with Logging
```bash
RUST_LOG=debug cargo test
```

### Release Mode Tests (Faster)
```bash
cargo test --release
```

## Coverage Analysis

### Using Tarpaulin
```bash
cargo install cargo-tarpaulin
cd rust
cargo tarpaulin --out Html --output-dir coverage
```

### Using llvm-cov
```bash
cargo install cargo-llvm-cov
cd rust
cargo llvm-cov --html
```

## Profiling

### CPU Profiling
```bash
# macOS
cargo bench --bench compression_benchmarks --no-run
instruments -t "Time Profiler" target/release/deps/compression_benchmarks-*

# Linux
cargo bench --bench compression_benchmarks -- --profile-time=5
```

### Memory Profiling
```bash
valgrind --tool=massif cargo bench --bench compression_benchmarks -- --test
```

## Adding New Tests

### Unit Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_feature() {
        // Test code
        assert_eq!(result, expected);
    }
}
```

### Integration Test
Add to `tests/integration_tests.rs`:
```rust
#[test]
fn test_new_feature() {
    let temp = TempDir::new().unwrap();
    // Test implementation
}
```

### Benchmark
Add to `benches/compression_benchmarks.rs`:
```rust
fn bench_new_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
        });
    });
}

criterion_group!(benches, bench_new_feature, /* existing... */);
```

## Troubleshooting

### "Library not found" Errors
```bash
# Ensure C library is built
cd ..  # root directory
cmake -B build
cmake --build build
```

### Benchmark Compilation Issues
```bash
cargo clean
cargo bench --no-run  # Just compile
```

### Slow Benchmark Runs
Use `--quick` flag or adjust sample sizes:
```bash
cargo bench -- --quick
```

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust Benchmarking](https://doc.rust-lang.org/nightly/unstable-book/library-features/test.html)

## Summary

✅ **21** unit tests  
✅ **17** integration tests  
✅ **8** benchmark categories  
✅ **40+** individual benchmark scenarios  

**Total Coverage:** Core compression, encryption, extraction, multi-threading, and error handling.
