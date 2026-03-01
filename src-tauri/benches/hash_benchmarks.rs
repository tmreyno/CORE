// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Hash Performance Benchmarks
//!
//! Compares performance of different hash algorithms (MD5, SHA-256, BLAKE3, XXH3)
//! across various data sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ffx_check_lib::common::hash::{compute_hash, HashAlgorithm};

fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

fn bench_hash_algorithms(c: &mut Criterion) {
    let sizes = vec![
        1024,             // 1 KB
        1024 * 10,        // 10 KB
        1024 * 100,       // 100 KB
        1024 * 1024,      // 1 MB
        1024 * 1024 * 10, // 10 MB
    ];

    for size in sizes {
        let data = generate_test_data(size);
        let mut group = c.benchmark_group(format!("hash_{}", format_size(size)));
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("md5", size), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Md5));
        });

        group.bench_with_input(BenchmarkId::new("sha256", size), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Sha256));
        });

        group.bench_with_input(BenchmarkId::new("blake3", size), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Blake3));
        });

        group.bench_with_input(BenchmarkId::new("xxh3", size), &data, |b, data| {
            b.iter(|| compute_hash(black_box(data), HashAlgorithm::Xxh3));
        });

        group.finish();
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{}KB", bytes / 1024)
    } else {
        format!("{}MB", bytes / (1024 * 1024))
    }
}

criterion_group!(benches, bench_hash_algorithms);
criterion_main!(benches);
