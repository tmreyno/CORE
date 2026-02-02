use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs::{self, File};
use std::io::Write;

fn main() {
    println!("=== Creating Multi-Volume Test Archive ===\n");
    
    let sz = SevenZip::new().expect("Failed to initialize");
    
    fs::create_dir_all("mv_test").ok();
    
    // Create 50MB test file
    println!("Creating 50MB test file...");
    let test_file = "mv_test/test_50mb.bin";
    let mut file = File::create(test_file).unwrap();
    
    // Pseudo-random data
    let mut rng_state: u64 = 0x123456789ABCDEF0;
    for _ in 0..(50 * 1024 * 1024 / 8) {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        file.write_all(&rng_state.to_le_bytes()).unwrap();
    }
    drop(file);
    
    println!("Creating multi-volume archive (10MB volumes)...");
    let mut options = StreamOptions::default();
    options.split_size = 10 * 1024 * 1024;
    options.num_threads = 4;
    
    match sz.create_archive_streaming(
        "mv_test/archive.7z",
        &[test_file],
        CompressionLevel::Fast,
        Some(&options),
        None,
    ) {
        Ok(_) => {
            println!("✓ Archive created!\n");
            println!("Files created:");
            for entry in fs::read_dir("mv_test").unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name();
                let size = entry.metadata().unwrap().len();
                println!("  {} - {:.2} MB", name.to_string_lossy(), size as f64 / (1024.0 * 1024.0));
            }
            println!("\nTest with: 7z x mv_test/archive.7z.001 -omv_test/extracted");
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
