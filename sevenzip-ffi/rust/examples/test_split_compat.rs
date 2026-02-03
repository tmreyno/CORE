//! Test split archive creation and verification with 7z

use seven_zip::{SevenZip, CompressionLevel, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

fn main() {
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create a larger file that will actually need to be split
    println!("Creating test file (500KB of pseudo-random data)...");
    let large_file = input_dir.join("large.bin");
    let mut f = File::create(&large_file).unwrap();
    // Create data that doesn't compress well (pseudo-random)
    let mut seed = 12345u64;
    for _ in 0..500000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        f.write_all(&[((seed >> 16) & 0xFF) as u8]).unwrap();
    }
    drop(f);
    
    let input_size = fs::metadata(&large_file).unwrap().len();
    println!("Input file: {} bytes", input_size);
    
    let sz = SevenZip::new().unwrap();
    let split_archive = temp_dir.path().join("split.7z");
    
    // Test with 100KB volumes
    let split_opts = StreamOptions {
        split_size: 100 * 1024, // 100KB
        ..Default::default()
    };
    
    println!("\nCreating split archive with 100KB volumes...");
    sz.create_archive_streaming(
        &split_archive,
        &[large_file.to_str().unwrap()],
        CompressionLevel::Fast, // Use fast to avoid too much compression
        Some(&split_opts),
        None,
    ).unwrap();
    
    // List volumes
    println!("\nVolumes created:");
    let mut vol_count = 0;
    let mut total_size = 0u64;
    for i in 1..=20 {
        let vol = temp_dir.path().join(format!("split.7z.{:03}", i));
        if let Ok(meta) = fs::metadata(&vol) {
            vol_count += 1;
            total_size += meta.len();
            println!("  split.7z.{:03}: {} bytes", i, meta.len());
        }
    }
    // Also check for non-split output
    if let Ok(meta) = fs::metadata(&split_archive) {
        println!("  split.7z (main): {} bytes", meta.len());
    }
    println!("Total: {} volumes, {} bytes compressed", vol_count, total_size);
    
    // Test with 7z
    let first_vol = temp_dir.path().join("split.7z.001");
    if first_vol.exists() {
        println!("\nTesting with 7z tool...");
        let output = Command::new("7z")
            .args(["l", first_vol.to_str().unwrap()])
            .output().unwrap();
        println!("{}", String::from_utf8_lossy(&output.stdout));
        
        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir_all(&extract_dir).unwrap();
        
        println!("\nExtracting with 7z...");
        let extract = Command::new("7z")
            .args(["x", first_vol.to_str().unwrap(), &format!("-o{}", extract_dir.display()), "-y"])
            .output().unwrap();
        
        if extract.status.success() {
            println!("✅ Extraction succeeded!");
            let extracted = extract_dir.join("large.bin");
            if extracted.exists() {
                let orig = fs::read(&large_file).unwrap();
                let extr = fs::read(&extracted).unwrap();
                if orig == extr {
                    println!("✅ Content verified - files match!");
                } else {
                    println!("❌ Content mismatch!");
                    println!("  Original: {} bytes", orig.len());
                    println!("  Extracted: {} bytes", extr.len());
                }
            }
        } else {
            println!("❌ Extraction failed!");
            println!("stderr: {}", String::from_utf8_lossy(&extract.stderr));
            println!("stdout: {}", String::from_utf8_lossy(&extract.stdout));
        }
    } else {
        println!("No split volumes created - checking main archive...");
        if split_archive.exists() {
            let output = Command::new("7z")
                .args(["t", split_archive.to_str().unwrap()])
                .output().unwrap();
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
}
