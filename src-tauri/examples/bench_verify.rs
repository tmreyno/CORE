// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Benchmark hash verification speed for EWF (Expert Witness Format) images
use std::time::Instant;
extern crate num_cpus;
extern crate rayon;

// Use the library's ewf module
use ffx_check_lib::ewf;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/Users/terryreynolds/Downloads/2020JimmyWilson.E01".to_string());
    
    let algorithm = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "md5".to_string());
    
    println!("Benchmarking {} verification for: {}", algorithm.to_uppercase(), path);
    println!("===================================================\n");
    
    // Print CPU/thread info
    println!("System Info:");
    println!("  CPU cores available: {}", num_cpus::get());
    println!("  Rayon threads: {}", rayon::current_num_threads());
    println!();
    
    // Get file info
    match ewf::info(&path) {
        Ok(info) => {
            let total_bytes = info.sector_count * info.bytes_per_sector as u64;
            let total_mb = total_bytes as f64 / (1024.0 * 1024.0);
            println!("File Info:");
            println!("  Segments: {}", info.segment_count);
            println!("  Chunks: {}", info.chunk_count);
            println!("  Total Size: {:.2} MiB ({} bytes)", total_mb, total_bytes);
            println!();
        }
        Err(e) => {
            eprintln!("Error reading file info: {}", e);
            return;
        }
    }
    
    // Run verification
    println!("Computing {} hash...", algorithm.to_uppercase());
    let start = Instant::now();
    
    match ewf::verify(&path, &algorithm) {
        Ok(hash) => {
            let duration = start.elapsed();
            let seconds = duration.as_secs_f64();
            
            println!("\nResults:");
            println!("  {}: {}", algorithm.to_uppercase(), hash);
            println!("  Time: {:.2}s", seconds);
            
            // Calculate throughput
            if let Ok(info) = ewf::info(&path) {
                let total_bytes = info.sector_count * info.bytes_per_sector as u64;
                let mb_per_sec = (total_bytes as f64 / (1024.0 * 1024.0)) / seconds;
                println!("  Throughput: {:.2} MiB/s", mb_per_sec);
            }
        }
        Err(e) => {
            eprintln!("\nError: {}", e);
        }
    }
}
