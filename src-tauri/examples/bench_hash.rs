// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Benchmark hash performance across all algorithms
// Usage: cargo run --release --example bench_hash [file_path]

use std::fs::File;
use std::io::{Read, BufReader};
use std::time::Instant;

extern crate rayon;
extern crate memmap2;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/terryreynolds/Downloads/2020JimmyWilson.E01".to_string());
    
    println!("Hash Performance Benchmark");
    println!("==========================\n");
    
    // Print system info
    println!("System Info:");
    println!("  CPU cores: {}", std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
    println!("  Rayon threads: {}", rayon::current_num_threads());
    println!("  File: {}", path);
    
    // Get file size
    let metadata = std::fs::metadata(&path).expect("Cannot read file metadata");
    let file_size = metadata.len();
    let file_size_mb = file_size as f64 / (1024.0 * 1024.0);
    println!("  Size: {:.2} MiB ({} bytes)\n", file_size_mb, file_size);
    
    // Algorithms to test
    type HashBenchFn = fn(&str) -> Result<(String, f64), String>;
    let algorithms: Vec<(&str, HashBenchFn)> = vec![
        ("CRC32", bench_crc32),
        ("XXH64", bench_xxh64),
        ("XXH3-128", bench_xxh3),
        ("MD5", bench_md5),
        ("SHA-1", bench_sha1),
        ("SHA-256", bench_sha256),
        ("BLAKE3", bench_blake3),
        ("BLAKE3 (mmap)", bench_blake3_mmap),
        ("BLAKE3 (mmap+rayon)", bench_blake3_mmap_rayon),
    ];
    
    println!("{:<22} {:>12} {:>14} {:>12}", "Algorithm", "Time", "Throughput", "Hash (first 16)");
    println!("{}", "-".repeat(62));
    
    for (name, bench_fn) in algorithms {
        match bench_fn(&path) {
            Ok((hash, duration)) => {
                let mb_per_sec = file_size_mb / duration;
                let hash_preview = if hash.len() > 16 { &hash[..16] } else { &hash };
                println!("{:<22} {:>10.3}s {:>12.2} MiB/s  {}...", 
                    name, duration, mb_per_sec, hash_preview);
            }
            Err(e) => {
                println!("{:<22} ERROR: {}", name, e);
            }
        }
    }
    
    println!();
}

fn bench_crc32(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = crc32fast::Hasher::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = format!("{:08x}", hasher.finalize());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_xxh64(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = xxhash_rust::xxh64::Xxh64::new(0);
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = format!("{:016x}", hasher.digest());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_xxh3(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = format!("{:032x}", hasher.digest128());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_md5(path: &str) -> Result<(String, f64), String> {
    use md5::{Md5, Digest};
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hex::encode(hasher.finalize());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_sha1(path: &str) -> Result<(String, f64), String> {
    use sha1::{Sha1, Digest};
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hex::encode(hasher.finalize());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_sha256(path: &str) -> Result<(String, f64), String> {
    use sha2::{Sha256, Digest};
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hex::encode(hasher.finalize());
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_blake3(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::with_capacity(16 * 1024 * 1024, file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 16 * 1024 * 1024];
    
    let start = Instant::now();
    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hasher.finalize().to_hex().to_string();
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_blake3_mmap(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file).map_err(|e| e.to_string())? };
    
    let start = Instant::now();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&mmap);
    let hash = hasher.finalize().to_hex().to_string();
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}

fn bench_blake3_mmap_rayon(path: &str) -> Result<(String, f64), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file).map_err(|e| e.to_string())? };
    
    let start = Instant::now();
    let mut hasher = blake3::Hasher::new();
    hasher.update_rayon(&mmap);
    let hash = hasher.finalize().to_hex().to_string();
    let duration = start.elapsed().as_secs_f64();
    
    Ok((hash, duration))
}
