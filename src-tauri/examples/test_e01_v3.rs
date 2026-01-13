// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Test EWF (Expert Witness Format) implementation and compare with libewf results
// Run with: cargo run --example test_e01_v3

use ffx_check_lib::ewf;
use std::time::Instant;

fn main() {
    let test_file = "/Users/terryreynolds/Downloads/4Dell Latitude CPi.E01";
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║           EWF Implementation Test & Comparison                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    println!("Test file: {}\n", test_file);
    
    // Test 1: Get Info
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 1: Get Container Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    match ewf::info(test_file) {
        Ok(info) => {
            println!("✓ Successfully opened E01 file\n");
            println!("Container Details:");
            println!("  Segment count:      {}", info.segment_count);
            println!("  Chunk count:        {}", info.chunk_count);
            println!("  Sector count:       {}", info.sector_count);
            println!("  Bytes per sector:   {}", info.bytes_per_sector);
            println!("  Sectors per chunk:  {}", info.sectors_per_chunk);
            println!("  Total size:         {} bytes ({:.2} GB)", 
                     info.sector_count * info.bytes_per_sector as u64,
                     (info.sector_count * info.bytes_per_sector as u64) as f64 / 1_073_741_824.0);
        }
        Err(e) => {
            eprintln!("✗ Failed to get info: {}", e);
            return;
        }
    }
    
    // Test 2: Verify with MD5
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 2: MD5 Hash Verification");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\nComputing MD5 hash of extracted data...");
    let start = Instant::now();
    
    match ewf::verify(test_file, "md5") {
        Ok(hash) => {
            let duration = start.elapsed();
            println!("✓ MD5 computation completed in {:.2}s\n", duration.as_secs_f64());
            println!("EWF MD5 hash: {}", hash);
            println!("\nExpected from ewfverify:");
            println!("  aee4fcd9301c03b3b054623ca261959a");
            println!("\nMatch: {}", if hash == "aee4fcd9301c03b3b054623ca261959a" {
                "✓ YES - Hashes match!"
            } else {
                "✗ NO - Hashes differ!"
            });
        }
        Err(e) => {
            eprintln!("✗ Failed to verify: {}", e);
        }
    }
    
    // Test 3: Performance comparison
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TEST 3: Performance Test - Read All Chunks");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    match ewf::EwfHandle::open(test_file) {
        Ok(mut handle) => {
            let chunk_count = handle.get_chunk_count();
            println!("\nReading {} chunks...", chunk_count);
            
            let start = Instant::now();
            let mut total_bytes = 0u64;
            let mut success_count = 0;
            let mut error_count = 0;
            
            for i in 0..chunk_count {
                match handle.read_chunk(i) {
                    Ok(data) => {
                        total_bytes += data.len() as u64;
                        success_count += 1;
                    }
                    Err(_) => {
                        error_count += 1;
                    }
                }
                
                // Progress indicator every 1000 chunks
                if (i + 1) % 1000 == 0 {
                    let progress = ((i + 1) as f64 / chunk_count as f64) * 100.0;
                    print!("\rProgress: {:.1}% ({}/{})", progress, i + 1, chunk_count);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
            }
            
            let duration = start.elapsed();
            println!("\n");
            println!("Results:");
            println!("  Success:     {} chunks", success_count);
            println!("  Errors:      {} chunks", error_count);
            println!("  Total bytes: {} ({:.2} GB)", total_bytes, total_bytes as f64 / 1_073_741_824.0);
            println!("  Duration:    {:.2}s", duration.as_secs_f64());
            println!("  Throughput:  {:.2} MB/s", 
                     (total_bytes as f64 / 1_048_576.0) / duration.as_secs_f64());
        }
        Err(e) => {
            eprintln!("✗ Failed to open handle: {}", e);
        }
    }
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("COMPARISON WITH LIBEWF");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nTo compare with libewf, run:");
    println!("  cd /Users/terryreynolds/GitHub/CORE/libewf");
    println!("  time ./ewftools/ewfverify \"{}\"", test_file);
    println!("\nExpected libewf result:");
    println!("  MD5: aee4fcd9301c03b3b054623ca261959a");
    println!("  Time: ~16 seconds");
    println!("  Speed: ~290 MB/s");
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                         Test Complete                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
}
