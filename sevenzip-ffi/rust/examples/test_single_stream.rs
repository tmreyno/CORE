use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs;
use std::process::Command;

fn main() {
    println!("=== Testing Single-Volume Streaming Archive ===\n");
    
    let sz = SevenZip::new().expect("Failed to initialize");
    
    // Use the existing test file
    let test_file = "test_5mb.bin";
    
    if !std::path::Path::new(test_file).exists() {
        println!("Creating test file...");
        let data: Vec<u8> = (0..5*1024*1024).map(|i| (i % 256) as u8).collect();
        fs::write(test_file, &data).unwrap();
    }
    
    println!("Creating single-volume archive using streaming API...");
    let mut options = StreamOptions::default();
    options.split_size = 1024 * 1024 * 1024;  // 1GB - won't split for 5MB file
    options.num_threads = 2;
    
    match sz.create_archive_streaming(
        "test_stream_single.7z",
        &[test_file],
        CompressionLevel::Fast,
        Some(&options),
        None,
    ) {
        Ok(_) => {
            println!("✓ Archive created!\n");
            
            // List created files
            for entry in fs::read_dir(".").unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.contains("test_stream_single") {
                    let size = entry.metadata().unwrap().len();
                    println!("  {} - {:.2} MB", name_str, size as f64 / (1024.0 * 1024.0));
                }
            }
            
            // Test with 7zz
            println!("\nTesting with 7zz...");
            
            // Check for .001 or plain .7z file
            let archive_path = if std::path::Path::new("test_stream_single.7z.001").exists() {
                "test_stream_single.7z.001"
            } else {
                "test_stream_single.7z"
            };
            
            let output = Command::new("7zz")
                .args(&["t", archive_path])
                .output()
                .expect("Failed to run 7zz");
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                println!("stdout:\n{}", stdout);
            }
            if !stderr.is_empty() {
                println!("stderr:\n{}", stderr);
            }
            
            if output.status.success() {
                println!("\n✅ SUCCESS! Archive validates correctly with 7-Zip!");
            } else {
                println!("\n❌ FAILED: 7-Zip could not validate the archive");
                println!("Exit code: {:?}", output.status.code());
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
