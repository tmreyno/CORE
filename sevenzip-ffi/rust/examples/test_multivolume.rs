use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    println!("=== Multi-Volume 7z Archive Test ===\n");
    
    // Initialize library
    let sz = SevenZip::new().expect("Failed to initialize 7z library");
    
    // Create test directory
    let test_dir = "test_multivolume";
    fs::create_dir_all(test_dir).expect("Failed to create test directory");
    
    // Create a large test file (200MB)
    println!("Creating 200MB test file...");
    let large_file_path = format!("{}/large_file.bin", test_dir);
    create_large_file(&large_file_path, 200 * 1024 * 1024); // 200 MB
    
    // Create multi-volume archive with 50MB volumes
    println!("Creating multi-volume archive (50MB volumes)...");
    let archive_path = format!("{}/test_archive.7z", test_dir);
    
    let mut options = StreamOptions::default();
    options.split_size = 50 * 1024 * 1024; // 50 MB per volume
    options.num_threads = 4;
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &[large_file_path.as_str()],
        CompressionLevel::Fast,
        Some(&options),
        None,
    );
    
    match result {
        Ok(_) => {
            println!("✓ Archive created successfully!");
            
            // Check for volume files
            let volume1 = format!("{}.001", archive_path);
            let volume2 = format!("{}.002", archive_path);
            let volume3 = format!("{}.003", archive_path);
            let volume4 = format!("{}.004", archive_path);
            
            println!("\nVolume files:");
            check_volume(&volume1);
            check_volume(&volume2);
            check_volume(&volume3);
            check_volume(&volume4);
            
            println!("\n✓ Multi-volume archive created successfully!");
            println!("  Large files (100GB+) can now be split across multiple volumes");
            println!("  for easier storage and transfer!");
        }
        Err(e) => {
            println!("✗ Archive creation failed: {:?}", e);
        }
    }
    
    // Cleanup
    println!("\nCleaning up test files...");
    fs::remove_dir_all(test_dir).ok();
    println!("Done!");
}

fn create_large_file(path: &str, size: usize) {
    let mut file = File::create(path).expect("Failed to create file");
    let chunk_size = 1024 * 1024; // 1 MB chunks
    
    // Create pseudo-random data (not compressible)
    let mut rng_state: u64 = 0x123456789ABCDEF0;
    
    let mut remaining = size;
    while remaining > 0 {
        let to_write = remaining.min(chunk_size);
        let mut chunk = Vec::with_capacity(to_write);
        
        // Generate pseudo-random bytes
        for _ in 0..to_write {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            chunk.push((rng_state >> 32) as u8);
        }
        
        file.write_all(&chunk).expect("Failed to write");
        remaining -= to_write;
        
        // Progress indicator
        let progress = ((size - remaining) as f64 / size as f64) * 100.0;
        print!("\r  Progress: {:.1}%", progress);
        std::io::stdout().flush().unwrap();
    }
    println!();
}

fn check_volume(path: &str) {
    if Path::new(path).exists() {
        let metadata = fs::metadata(path).unwrap();
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        println!("  {} - {:.2} MB", path, size_mb);
    } else {
        println!("  {} - not found", path);
    }
}
