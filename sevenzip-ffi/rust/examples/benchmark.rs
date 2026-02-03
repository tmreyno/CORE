//! Archive Creation Performance Benchmark
//!
//! This benchmark compares our library performance against the official 7z tool

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::time::Instant;
use tempfile::TempDir;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        Archive Creation Performance Benchmark                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create test files of various sizes
    println!("Creating test files...");
    
    // 1MB compressible file
    let file_1mb = input_dir.join("1mb_text.txt");
    let mut f = File::create(&file_1mb).unwrap();
    for i in 0..20000 {
        writeln!(f, "Line {} - This is test content for compression AAAAAAAAAAAAAAAAAAAAAAAAA", i).unwrap();
    }
    drop(f);
    
    // 10MB compressible file
    let file_10mb = input_dir.join("10mb_text.txt");
    let mut f = File::create(&file_10mb).unwrap();
    for i in 0..200000 {
        writeln!(f, "Line {} - This is test content for compression AAAAAAAAAAAAAAAAAAAAAAAAA", i).unwrap();
    }
    drop(f);
    
    // 1MB random (incompressible)
    let file_1mb_rand = input_dir.join("1mb_random.bin");
    let mut f = File::create(&file_1mb_rand).unwrap();
    let mut seed = 12345u64;
    for _ in 0..1_000_000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        f.write_all(&[((seed >> 16) & 0xFF) as u8]).unwrap();
    }
    drop(f);
    
    println!("Test files created:");
    for file in [&file_1mb, &file_10mb, &file_1mb_rand] {
        let size = fs::metadata(file).unwrap().len();
        println!("  {} - {} bytes ({:.1} MB)", 
                 file.file_name().unwrap().to_string_lossy(),
                 size,
                 size as f64 / 1_000_000.0);
    }
    
    let sz = SevenZip::new().unwrap();
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 1: create_archive (standard)");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // Test standard create_archive
    for (name, file) in [("1MB text", &file_1mb), ("10MB text", &file_10mb), ("1MB random", &file_1mb_rand)] {
        let archive_path = temp_dir.path().join(format!("bench_std_{}.7z", name.replace(" ", "_")));
        let file_size = fs::metadata(file).unwrap().len();
        
        let start = Instant::now();
        sz.create_archive(
            &archive_path,
            &[file.to_str().unwrap()],
            CompressionLevel::Normal,
            None,
        ).unwrap();
        let elapsed = start.elapsed();
        
        let archive_size = fs::metadata(&archive_path).unwrap().len();
        let speed_mbps = (file_size as f64 / 1_000_000.0) / elapsed.as_secs_f64();
        
        println!("  {} (standard):", name);
        println!("    Time: {:?}", elapsed);
        println!("    Input: {} bytes, Output: {} bytes ({:.1}%)", 
                 file_size, archive_size, (archive_size as f64 / file_size as f64) * 100.0);
        println!("    Speed: {:.1} MB/s\n", speed_mbps);
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 2: create_archive_streaming");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // Test streaming create_archive
    for (name, file) in [("1MB text", &file_1mb), ("10MB text", &file_10mb), ("1MB random", &file_1mb_rand)] {
        let archive_path = temp_dir.path().join(format!("bench_stream_{}.7z", name.replace(" ", "_")));
        let file_size = fs::metadata(file).unwrap().len();
        
        let start = Instant::now();
        sz.create_archive_streaming(
            &archive_path,
            &[file.to_str().unwrap()],
            CompressionLevel::Normal,
            None,
            None,
        ).unwrap();
        let elapsed = start.elapsed();
        
        let archive_size = fs::metadata(&archive_path).unwrap().len();
        let speed_mbps = (file_size as f64 / 1_000_000.0) / elapsed.as_secs_f64();
        
        println!("  {} (streaming):", name);
        println!("    Time: {:?}", elapsed);
        println!("    Input: {} bytes, Output: {} bytes ({:.1}%)", 
                 file_size, archive_size, (archive_size as f64 / file_size as f64) * 100.0);
        println!("    Speed: {:.1} MB/s\n", speed_mbps);
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 3: Compression Levels");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let test_file = &file_10mb;
    let file_size = fs::metadata(test_file).unwrap().len();
    
    for (level_num, level_name, level) in [
        (0, "Store", CompressionLevel::Store),
        (1, "Fastest", CompressionLevel::Fastest),
        (5, "Normal", CompressionLevel::Normal),
        (9, "Ultra", CompressionLevel::Ultra),
    ] {
        let archive_path = temp_dir.path().join(format!("bench_level_{}.7z", level_num));
        
        let start = Instant::now();
        sz.create_archive(
            &archive_path,
            &[test_file.to_str().unwrap()],
            level,
            None,
        ).unwrap();
        let elapsed = start.elapsed();
        
        let archive_size = fs::metadata(&archive_path).unwrap().len();
        let speed_mbps = (file_size as f64 / 1_000_000.0) / elapsed.as_secs_f64();
        
        println!("  Level {} ({}):", level_num, level_name);
        println!("    Time: {:?}, Speed: {:.1} MB/s, Ratio: {:.1}%\n", 
                 elapsed, speed_mbps, (archive_size as f64 / file_size as f64) * 100.0);
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 4: Thread Count Impact");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    for threads in [1, 2, 4, 8] {
        let archive_path = temp_dir.path().join(format!("bench_threads_{}.7z", threads));
        let opts = CompressOptions {
            num_threads: threads,
            ..Default::default()
        };
        
        let start = Instant::now();
        sz.create_archive(
            &archive_path,
            &[test_file.to_str().unwrap()],
            CompressionLevel::Normal,
            Some(&opts),
        ).unwrap();
        let elapsed = start.elapsed();
        
        let speed_mbps = (file_size as f64 / 1_000_000.0) / elapsed.as_secs_f64();
        
        println!("  {} thread(s): {:?} ({:.1} MB/s)", threads, elapsed, speed_mbps);
    }
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  BENCHMARK 5: Compare with Official 7z");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // Official 7z
    let archive_path_7z = temp_dir.path().join("bench_official.7z");
    let start = Instant::now();
    let output = std::process::Command::new("7z")
        .args(["a", "-mx5", archive_path_7z.to_str().unwrap(), test_file.to_str().unwrap()])
        .output();
    let elapsed_7z = start.elapsed();
    
    // Our library
    let archive_path_ours = temp_dir.path().join("bench_ours.7z");
    let start = Instant::now();
    sz.create_archive(
        &archive_path_ours,
        &[test_file.to_str().unwrap()],
        CompressionLevel::Normal,
        None,
    ).unwrap();
    let elapsed_ours = start.elapsed();
    
    if output.is_ok() && output.as_ref().unwrap().status.success() {
        let size_7z = fs::metadata(&archive_path_7z).unwrap().len();
        let size_ours = fs::metadata(&archive_path_ours).unwrap().len();
        
        println!("  Official 7z: {:?} ({:.1} MB/s), {} bytes", 
                 elapsed_7z, 
                 (file_size as f64 / 1_000_000.0) / elapsed_7z.as_secs_f64(),
                 size_7z);
        println!("  Our library: {:?} ({:.1} MB/s), {} bytes", 
                 elapsed_ours,
                 (file_size as f64 / 1_000_000.0) / elapsed_ours.as_secs_f64(),
                 size_ours);
        
        let speed_ratio = elapsed_7z.as_secs_f64() / elapsed_ours.as_secs_f64();
        if speed_ratio > 1.0 {
            println!("\n  ✅ Our library is {:.1}x FASTER than official 7z", speed_ratio);
        } else {
            println!("\n  ⚠️  Our library is {:.1}x SLOWER than official 7z", 1.0 / speed_ratio);
        }
    }
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                  BENCHMARK COMPLETE                            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}

// Additional benchmark for app-realistic scenarios
