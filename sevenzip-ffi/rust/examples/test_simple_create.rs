use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a small test file
    let test_file = "test_simple.txt";
    let mut f = fs::File::create(test_file)?;
    f.write_all(b"Hello, World! This is a test file for 7z compression.\n")?;
    drop(f);
    
    // Create archive using streaming with multi-volume code
    let sz = SevenZip::new()?;
    
    // Use the multi-volume function but with huge split size (effectively single volume)
    let mut options = StreamOptions::default();
    options.split_size = 1024 * 1024 * 1024;  // 1GB - won't split for small file
    
    println!("Creating archive with multi-volume code...");
    sz.create_archive_streaming(
        "test_mv_simple.7z",
        &[test_file],
        CompressionLevel::Fast,
        Some(&options),
        None
    )?;
    
    println!("✓ Archive created!");
    
    // Test with 7-Zip
    println!("\nTesting with 7z tool...");
    let output = std::process::Command::new("7z")
        .args(&["t", "test_mv_simple.7z"])
        .output()?;
    
    println!("Exit code: {}", output.status);
    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    
    // Cleanup
    fs::remove_file(test_file)?;
    
    Ok(())
}
