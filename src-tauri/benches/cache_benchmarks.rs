// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive Cache Performance Benchmarks
//!
//! Measures performance of the index cache system with various hit rates
//! and access patterns.
//!
//! NOTE: This benchmark is currently disabled as the IndexCache API has been updated.
//! It needs to be rewritten to work with the new database-backed cache system.

/*

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ffx_check_lib::common::index_cache::{IndexCache, IndexEntry};
use std::path::PathBuf;

fn create_test_entry(path: &str, index: usize) -> IndexEntry {
    IndexEntry {
        path: path.to_string(),
        size: (index * 1024) as u64,
        is_dir: false,
        modified_time: Some(0),
        hash: Some(format!("hash{}", index)),
    }
}

fn bench_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache");

    // Benchmark cache insertion
    group.bench_function("insert_1000_entries", |b| {
        b.iter(|| {
            let cache = IndexCache::new(1000);
            for i in 0..1000 {
                let path = format!("/path/to/file{}.txt", i);
                let entries = vec![create_test_entry(&path, i)];
                cache.store(&path, entries, format!("hash{}", i));
            }
        });
    });

    // Benchmark cache hit (warm cache)
    group.bench_function("lookup_hit", |b| {
        let cache = IndexCache::new(1000);
        for i in 0..100 {
            let path = format!("/path/to/file{}.txt", i);
            let entries = vec![create_test_entry(&path, i)];
            cache.store(&path, entries, format!("hash{}", i));
        }

        b.iter(|| {
            for i in 0..100 {
                let path = format!("/path/to/file{}.txt", i);
                let _result = cache.load(black_box(&path), black_box(&format!("hash{}", i)));
            }
        });
    });

    // Benchmark cache miss
    group.bench_function("lookup_miss", |b| {
        let cache = IndexCache::new(1000);
        b.iter(|| {
            let _result = cache.load(black_box("/nonexistent.txt"), black_box("hash123"));
        });
    });

    // Benchmark mixed access pattern (80% hit rate)
    group.bench_function("mixed_access_80pct_hit", |b| {
        let cache = IndexCache::new(1000);
        for i in 0..100 {
            let path = format!("/cached/file{}.txt", i);
            let entries = vec![create_test_entry(&path, i)];
            cache.store(&path, entries, format!("hash{}", i));
        }

        b.iter(|| {
            for i in 0..100 {
                if i % 5 == 0 {
                    // 20% miss
                    let path = format!("/uncached/file{}.txt", i);
                    let _result = cache.load(black_box(&path), black_box(&format!("hash{}", i)));
                } else {
                    // 80% hit
                    let path = format!("/cached/file{}.txt", i);
                    let _result = cache.load(black_box(&path), black_box(&format!("hash{}", i)));
                }
            }
        });
    });

    // Benchmark cache invalidation
    group.bench_function("invalidate_100_entries", |b| {
        b.iter(|| {
            let cache = IndexCache::new(1000);
            for i in 0..100 {
                let path = format!("/path/to/file{}.txt", i);
                let entries = vec![create_test_entry(&path, i)];
                cache.store(&path, entries, format!("hash{}", i));
            }
            for i in 0..100 {
                let path = format!("/path/to/file{}.txt", i);
                cache.invalidate(black_box(&path));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_cache_operations);
criterion_main!(benches);
*/

// Placeholder to satisfy Cargo
fn main() {
    println!("Cache benchmarks are currently disabled - see file comments");
}
