use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a 1MB test file
    let test_file = "test_1mb.bin";
    let mut f = fs::File::create(test_file)?;
    let data = vec![0xAAu8; 1024 * 1024];  // 1MB of 0xAA
    f.write_all(&data)?;
    drop(f);
    
    // Create multi-volume archive with 512KB volumes
    let sz = SevenZip::new()?;
    let mut options = StreamOptions::default();
    options.split_size = 512 * 1024;  // 512KB volumes
    
    println!("Creating 1MB file split into 512KB volumes...");
    sz.create_archive_streaming(
        "test_small_mv.7z",
        &[test_file],
        CompressionLevel::Fast,
        Some(&options),
        None
    )?;
    
    println!("✓ Archive created!");
    
    // List created files
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("test_small_mv.7z") {
            let size = entry.metadata()?.len();
            println!("  {} - {} bytes", name, size);
        }
    }
    
    // Test with native 7z
    println!("\nTesting with 7zz...");
    let output = std::process::Command::new("7zz")
        .args(&["t", "test_small_mv.7z.001"])
        .output()?;
    
    println!("Exit code: {}", output.status);
    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Cleanup
    fs::remove_file(test_file)?;
    
    Ok(())
}
