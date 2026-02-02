//! Comprehensive benchmarks for sevenzip-ffi
//!
//! These benchmarks measure:
//! - Compression performance at various levels
//! - Encryption/decryption performance
//! - Archive creation and extraction
//! - Memory usage patterns
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use seven_zip::{SevenZip, CompressionLevel, CompressOptions};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Generate test data of specified size
fn generate_test_data(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    for i in 0..size {
        data.push((i % 256) as u8);
    }
    data
}

/// Generate compressible test data (text-like)
fn generate_compressible_data(size: usize) -> Vec<u8> {
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut data = Vec::with_capacity(size);
    while data.len() < size {
        data.extend_from_slice(pattern);
    }
    data.truncate(size);
    data
}

/// Generate incompressible test data (random-like)
fn generate_incompressible_data(size: usize) -> Vec<u8> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    
    let mut data = Vec::with_capacity(size);
    let hasher_builder = RandomState::new();
    
    for i in 0..size {
        let mut hasher = hasher_builder.build_hasher();
        hasher.write_usize(i);
        data.push((hasher.finish() % 256) as u8);
    }
    data
}

fn create_temp_file(dir: &std::path::Path, name: &str, data: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, data).unwrap();
    path
}

// ===== Compression Benchmarks =====

fn bench_compression_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_levels");
    let data_size = 1024 * 1024; // 1 MB
    let data = generate_compressible_data(data_size);
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    let levels = vec![
        (CompressionLevel::Store, "store"),
        (CompressionLevel::Fast, "fast"),
        (CompressionLevel::Normal, "normal"),
        (CompressionLevel::Maximum, "maximum"),
        (CompressionLevel::Ultra, "ultra"),
    ];
    
    for (level, name) in levels {
        group.bench_with_input(BenchmarkId::from_parameter(name), &level, |b, &level| {
            b.iter(|| {
                let temp = TempDir::new().unwrap();
                let test_file = create_temp_file(temp.path(), "test.dat", &data);
                let archive_path = temp.path().join("test.7z");
                
                let sz = SevenZip::new().unwrap();
                sz.create_archive(
                    archive_path.to_str().unwrap(),
                    &[test_file.to_str().unwrap()],
                    level,
                    None,
                ).unwrap();
                
                black_box(archive_path);
            });
        });
    }
    
    group.finish();
}

fn bench_compression_data_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_data_types");
    let data_size = 1024 * 1024; // 1 MB
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    let data_types = vec![
        ("compressible", generate_compressible_data(data_size)),
        ("incompressible", generate_incompressible_data(data_size)),
        ("sequential", generate_test_data(data_size)),
    ];
    
    for (name, data) in data_types {
        group.bench_with_input(BenchmarkId::from_parameter(name), &data, |b, data| {
            b.iter(|| {
                let temp = TempDir::new().unwrap();
                let test_file = create_temp_file(temp.path(), "test.dat", data);
                let archive_path = temp.path().join("test.7z");
                
                let sz = SevenZip::new().unwrap();
                sz.create_archive(
                    archive_path.to_str().unwrap(),
                    &[test_file.to_str().unwrap()],
                    CompressionLevel::Normal,
                    None,
                ).unwrap();
                
                black_box(archive_path);
            });
        });
    }
    
    group.finish();
}

fn bench_compression_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_sizes");
    
    let sizes = vec![
        (1024, "1KB"),
        (10 * 1024, "10KB"),
        (100 * 1024, "100KB"),
        (1024 * 1024, "1MB"),
        (5 * 1024 * 1024, "5MB"),
    ];
    
    for (size, name) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let data = generate_compressible_data(size);
            b.iter(|| {
                let temp = TempDir::new().unwrap();
                let test_file = create_temp_file(temp.path(), "test.dat", &data);
                let archive_path = temp.path().join("test.7z");
                
                let sz = SevenZip::new().unwrap();
                sz.create_archive(
                    archive_path.to_str().unwrap(),
                    &[test_file.to_str().unwrap()],
                    CompressionLevel::Fast,
                    None,
                ).unwrap();
                
                black_box(archive_path);
            });
        });
    }
    
    group.finish();
}

// ===== Extraction Benchmarks =====

fn bench_extraction_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("extraction_sizes");
    
    let sizes = vec![
        (1024, "1KB"),
        (10 * 1024, "10KB"),
        (100 * 1024, "100KB"),
        (1024 * 1024, "1MB"),
        (5 * 1024 * 1024, "5MB"),
    ];
    
    for (size, name) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            // Setup: Create archive once
            let setup_temp = TempDir::new().unwrap();
            let data = generate_compressible_data(size);
            let test_file = create_temp_file(setup_temp.path(), "test.dat", &data);
            let archive_path = setup_temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Fast,
                None,
            ).unwrap();
            
            b.iter(|| {
                let temp = TempDir::new().unwrap();
                let extract_dir = temp.path().join("extracted");
                fs::create_dir(&extract_dir).unwrap();
                
                sz.extract(
                    archive_path.to_str().unwrap(),
                    extract_dir.to_str().unwrap(),
                ).unwrap();
                
                black_box(extract_dir);
            });
        });
    }
    
    group.finish();
}

// ===== Encryption Benchmarks =====

fn bench_encryption_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_overhead");
    let data_size = 1024 * 1024; // 1 MB
    let data = generate_compressible_data(data_size);
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    // Without encryption
    group.bench_function("no_encryption", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "test.dat", &data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Fast,
                None,
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    // With encryption
    group.bench_function("with_encryption", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "test.dat", &data);
            let archive_path = temp.path().join("test.7z");
            
            let mut opts = CompressOptions::default();
            opts.password = Some("BenchmarkPassword123!".to_string());
            opts.num_threads = 2;
            
            let sz = SevenZip::new().unwrap();
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Fast,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    group.finish();
}

fn bench_encryption_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_extraction");
    let data_size = 1024 * 1024; // 1 MB
    let password = "BenchmarkPassword123!";
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    // Setup: Create encrypted archive once
    let setup_temp = TempDir::new().unwrap();
    let data = generate_compressible_data(data_size);
    let test_file = create_temp_file(setup_temp.path(), "test.dat", &data);
    let archive_path = setup_temp.path().join("test.7z");
    
    let mut opts = CompressOptions::default();
    opts.password = Some(password.to_string());
    opts.num_threads = 2;
    
    let sz = SevenZip::new().unwrap();
    sz.create_archive(
        archive_path.to_str().unwrap(),
        &[test_file.to_str().unwrap()],
        CompressionLevel::Fast,
        Some(&opts),
    ).unwrap();
    
    group.bench_function("decrypt_extract", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let extract_dir = temp.path().join("extracted");
            fs::create_dir(&extract_dir).unwrap();
            
            sz.extract_with_password(
                archive_path.to_str().unwrap(),
                extract_dir.to_str().unwrap(),
                Some(password),
                None,
            ).unwrap();
            
            black_box(extract_dir);
        });
    });
    
    group.finish();
}

// ===== Multi-threading Benchmarks =====

fn bench_threading_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("threading_performance");
    let data_size = 5 * 1024 * 1024; // 5 MB
    let data = generate_compressible_data(data_size);
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}threads", num_threads)),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let temp = TempDir::new().unwrap();
                    let test_file = create_temp_file(temp.path(), "test.dat", &data);
                    let archive_path = temp.path().join("test.7z");
                    
                    let mut opts = CompressOptions::default();
                    opts.num_threads = num_threads;
                    
                    let sz = SevenZip::new().unwrap();
                    sz.create_archive(
                        archive_path.to_str().unwrap(),
                        &[test_file.to_str().unwrap()],
                        CompressionLevel::Normal,
                        Some(&opts),
                    ).unwrap();
                    
                    black_box(archive_path);
                });
            },
        );
    }
    
    group.finish();
}

// ===== Multiple Files Benchmarks =====

fn bench_multiple_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_files");
    
    for num_files in [1, 5, 10, 50] {
        let total_size = num_files * 10 * 1024; // 10 KB per file
        group.throughput(Throughput::Bytes(total_size as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}files", num_files)),
            &num_files,
            |b, &num_files| {
                b.iter(|| {
                    let temp = TempDir::new().unwrap();
                    let archive_path = temp.path().join("test.7z");
                    
                    let mut file_paths = Vec::new();
                    for i in 0..num_files {
                        let data = generate_compressible_data(10 * 1024);
                        let file_path = create_temp_file(
                            temp.path(),
                            &format!("file_{}.dat", i),
                            &data,
                        );
                        file_paths.push(file_path);
                    }
                    
                    let file_strs: Vec<&str> = file_paths
                        .iter()
                        .map(|p| p.to_str().unwrap())
                        .collect();
                    
                    let sz = SevenZip::new().unwrap();
                    sz.create_archive(
                        archive_path.to_str().unwrap(),
                        &file_strs,
                        CompressionLevel::Fast,
                        None,
                    ).unwrap();
                    
                    black_box(archive_path);
                });
            },
        );
    }
    
    group.finish();
}

// ===== NEW SMART FEATURE BENCHMARKS =====

fn bench_incompressible_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("incompressible_detection");
    let data_size = 1024 * 1024; // 1 MB
    let random_data = generate_incompressible_data(data_size);
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    // Benchmark WITHOUT auto-detection (slow path)
    group.bench_function("without_auto_detect", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "random.dat", &random_data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            let mut opts = CompressOptions::default();
            opts.auto_detect_incompressible = false;
            
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    // Benchmark WITH auto-detection (fast path - should use Store mode)
    group.bench_function("with_auto_detect", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "random.dat", &random_data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            let mut opts = CompressOptions::default();
            opts.auto_detect_incompressible = true;
            
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    group.finish();
}

fn bench_smart_threading(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_threading");
    
    // Small file benchmark
    let small_data = generate_compressible_data(512 * 1024); // 512KB
    group.throughput(Throughput::Bytes(small_data.len() as u64));
    
    group.bench_function("small_file_manual_threads", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "small.dat", &small_data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            let mut opts = CompressOptions::default();
            opts.num_threads = 4; // Manually set (suboptimal for small files)
            
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    group.bench_function("small_file_auto_threads", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "small.dat", &small_data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            // Use auto-tuned (should select 1 thread for small file)
            let opts = CompressOptions::auto_tuned(&[test_file.to_str().unwrap()]).unwrap();
            
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    group.finish();
}

fn bench_convenience_methods(c: &mut Criterion) {
    let mut group = c.benchmark_group("convenience_methods");
    let data_size = 1024 * 1024; // 1 MB
    let data = generate_compressible_data(data_size);
    
    group.throughput(Throughput::Bytes(data_size as u64));
    
    // Benchmark traditional method
    group.bench_function("traditional_encrypted", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "test.dat", &data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            let mut opts = CompressOptions::default();
            opts.password = Some("test123".to_string());
            
            sz.create_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
                Some(&opts),
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    // Benchmark new convenience method
    group.bench_function("convenience_encrypted", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "test.dat", &data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            sz.create_encrypted_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                "test123",
                CompressionLevel::Normal,
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    // Benchmark smart archive method
    group.bench_function("smart_archive", |b| {
        b.iter(|| {
            let temp = TempDir::new().unwrap();
            let test_file = create_temp_file(temp.path(), "test.dat", &data);
            let archive_path = temp.path().join("test.7z");
            
            let sz = SevenZip::new().unwrap();
            sz.create_smart_archive(
                archive_path.to_str().unwrap(),
                &[test_file.to_str().unwrap()],
                CompressionLevel::Normal,
            ).unwrap();
            
            black_box(archive_path);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_compression_levels,
    bench_compression_data_types,
    bench_compression_sizes,
    bench_extraction_sizes,
    bench_encryption_overhead,
    bench_encryption_extraction,
    bench_threading_performance,
    bench_multiple_files,
    bench_incompressible_detection,
    bench_smart_threading,
    bench_convenience_methods,
);

criterion_main!(benches);
