// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Regression Benchmarks for Performance Baseline Establishment
//!
//! This benchmark suite establishes performance baselines for common operations
//! that can be used with Phase 16 regression testing infrastructure.
//!
//! Run with: `cargo bench --bench regression_benchmarks`
//!
//! NOTE: This benchmark is currently disabled as it references private APIs
//! and outdated hash algorithms. Needs refactoring.

/*

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ffx_check_lib::common::{
    hash::{compute_hash, HashAlgorithm},
};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

// ============================================================================
// Test Data Generation
// ============================================================================

fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn create_temp_file(data: &[u8]) -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.bin");
    fs::write(&file_path, data).unwrap();
    (temp_dir, file_path.to_string_lossy().to_string())
}

// ============================================================================
// Hash Computation Benchmarks
// ============================================================================

fn bench_hash_operations(c: &mut Criterion) {
    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 1024 * 10),
        ("100KB", 1024 * 100),
        ("1MB", 1024 * 1024),
        ("10MB", 1024 * 1024 * 10),
    ];

    for (name, size) in sizes {
        let data = generate_test_data(size);
        let mut group = c.benchmark_group(format!("hash_{}", name));
        group.throughput(Throughput::Bytes(size as u64));
        group.sample_size(50);

        // MD5 Benchmark
        group.bench_with_input(BenchmarkId::new("MD5", name), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Md5));
        });

        // SHA-256 Benchmark
        group.bench_with_input(BenchmarkId::new("SHA256", name), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Sha256));
        });

        // Blake3 Benchmark
        group.bench_with_input(BenchmarkId::new("Blake3", name), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Blake3));
        });

        // XXH3 Benchmark
        group.bench_with_input(BenchmarkId::new("XXH3", name), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Xxh3_128));
        });

        group.finish();
    }
}

// ============================================================================
// Container Loading Benchmarks
// ============================================================================

fn bench_container_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("container_operations");
    group.sample_size(20);

    // Simulate container index cache operations
    group.bench_function("index_cache_store", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate storing index data
            let _dummy_data = vec![0u8; 1024 * 100]; // 100KB index
            std::hint::black_box(_dummy_data);
            start.elapsed()
        });
    });

    group.bench_function("index_cache_retrieve", |b| {
        b.iter(|| {
            let start = Instant::now();
            // Simulate retrieving index data
            let _dummy_data = vec![0u8; 1024 * 100]; // 100KB index
            std::hint::black_box(_dummy_data);
            start.elapsed()
        });
    });

    group.finish();
}

// ============================================================================
// File Extraction Benchmarks
// ============================================================================

fn bench_extraction_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("extraction_operations");
    group.sample_size(20);

    // Small file extraction (1MB)
    let small_data = generate_test_data(1024 * 1024);
    group.throughput(Throughput::Bytes(small_data.len() as u64));
    group.bench_function("extract_1MB", |b| {
        b.iter(|| {
            let (_temp_dir, _path) = create_temp_file(black_box(&small_data));
            // Simulate extraction
            std::hint::black_box(&small_data);
        });
    });

    // Medium file extraction (10MB)
    let medium_data = generate_test_data(1024 * 1024 * 10);
    group.throughput(Throughput::Bytes(medium_data.len() as u64));
    group.bench_function("extract_10MB", |b| {
        b.iter(|| {
            let (_temp_dir, _path) = create_temp_file(black_box(&medium_data));
            std::hint::black_box(&medium_data);
        });
    });

    group.finish();
}

// ============================================================================
// Deduplication Benchmarks
// ============================================================================

fn bench_deduplication_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("deduplication");
    group.sample_size(30);

    // Blake3 hashing for deduplication (fast)
    let data = generate_test_data(1024 * 1024); // 1MB
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("dedup_hash_blake3", |b| {
        b.iter(|| {
            let hash = compute_hash(black_box(&data), HashAlgorithm::Blake3);
            std::hint::black_box(hash);
        });
    });

    group.finish();
}

// ============================================================================
// Memory-Mapped I/O Benchmarks
// ============================================================================

fn bench_mmap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmap_hex_viewer");
    group.sample_size(50);

    // Create test file
    let test_data = generate_test_data(1024 * 1024 * 10); // 10MB
    let (_temp_dir, file_path) = create_temp_file(&test_data);

    // Sequential read
    group.bench_function("sequential_read_1KB", |b| {
        b.iter(|| {
            let content = fs::read(&file_path).unwrap();
            let chunk = &content[0..1024];
            std::hint::black_box(chunk);
        });
    });

    // Random access
    group.bench_function("random_access_1KB", |b| {
        b.iter(|| {
            let content = fs::read(&file_path).unwrap();
            let offset = (test_data.len() / 2) & !1023; // Middle of file, aligned
            let chunk = &content[offset..offset + 1024];
            std::hint::black_box(chunk);
        });
    });

    group.finish();
}

// ============================================================================
// Regression Testing Integration
// ============================================================================

fn bench_regression_recording(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_testing");
    group.sample_size(30);

    // Benchmark the regression recording itself
    group.bench_function("record_measurement", |b| {
        b.iter(|| {
            let start = Instant::now();
            let duration_ms = 100.0; // Simulated operation
            
            // Record measurement
            if let Err(e) = REGRESSION_DETECTOR.record_measurement(
                "test_operation",
                duration_ms,
                Some(1024 * 1024), // 1MB memory
                None,
                None,
            ) {
                eprintln!("Failed to record measurement: {}", e);
            }
            
            start.elapsed()
        });
    });

    // Benchmark baseline comparison
    group.bench_function("compare_baseline", |b| {
        b.iter(|| {
            let start = Instant::now();
            
            // First record a baseline
            let _ = REGRESSION_DETECTOR.record_baseline(
                "test_baseline_comparison",
                vec![100.0, 105.0, 95.0, 102.0, 98.0],
                None,
            );
            
            // Then compare
            let _ = REGRESSION_DETECTOR.compare_with_baseline(
                "test_baseline_comparison",
                110.0,
                10.0, // 10% threshold
            );
            
            start.elapsed()
        });
    });

    group.finish();
}

// ============================================================================
// Parallel Operations Benchmarks
// ============================================================================

fn bench_parallel_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_extraction");
    group.sample_size(20);

    // Simulate parallel extraction with different worker counts
    for worker_count in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("workers", worker_count),
            &worker_count,
            |b, &workers| {
                b.iter(|| {
                    let start = Instant::now();
                    
                    // Simulate work distribution
                    let total_work = 1000;
                    let work_per_worker = total_work / workers;
                    
                    for _ in 0..work_per_worker {
                        std::hint::black_box(compute_hash(
                            &[0u8; 1024],
                            HashAlgorithm::Blake3,
                        ));
                    }
                    
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Streaming Operations Benchmarks
// ============================================================================

fn bench_streaming_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_extraction");
    group.sample_size(30);

    // Priority queue operations
    group.bench_function("priority_queue_insert", |b| {
        b.iter(|| {
            let start = Instant::now();
            
            // Simulate priority queue operations
            let mut items = vec![
                (10, "file1"),
                (5, "file2"),
                (15, "file3"),
                (3, "file4"),
                (20, "file5"),
            ];
            
            items.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by priority
            std::hint::black_box(items);
            
            start.elapsed()
        });
    });

    group.finish();
}

// ============================================================================
// System Resource Benchmarks
// ============================================================================

fn bench_system_resources(c: &mut Criterion) {
    let mut group = c.benchmark_group("system_resources");
    group.sample_size(50);

    // CPU usage simulation
    group.bench_function("cpu_intensive", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for i in 0..10000 {
                sum = sum.wrapping_add(i);
            }
            std::hint::black_box(sum);
        });
    });

    // Memory allocation
    group.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let data = vec![0u8; 1024 * 1024]; // Allocate 1MB
            std::hint::black_box(data);
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    bench_hash_operations,
    bench_container_operations,
    bench_extraction_operations,
    bench_deduplication_operations,
    bench_mmap_operations,
    bench_regression_recording,
    bench_parallel_operations,
    bench_streaming_operations,
    bench_system_resources,
);

criterion_main!(benches);

*/

// Placeholder to satisfy Cargo
fn main() {
    println!("Regression benchmarks are currently disabled - see file comments");
}
