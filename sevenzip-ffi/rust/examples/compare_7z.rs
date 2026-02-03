//! Compare our 7z library against official 7z tool
//!
//! This example creates test files, compresses them with both our library
//! and the official 7z tool, then compares results.

use seven_zip::{SevenZip, CompressionLevel, CompressOptions};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        7z Library Comparison Test                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    // Check for 7z tool
    let which = Command::new("which").arg("7z").output();
    if which.is_err() || !which.unwrap().status.success() {
        println!("❌ 7z command not found. Install p7zip or 7-zip to run comparison.");
        return;
    }
    
    // Create temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_dir = temp_dir.path().join("input");
    let our_dir = temp_dir.path().join("ours");
    let official_dir = temp_dir.path().join("official");
    let extract_dir = temp_dir.path().join("extracted");
    
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&our_dir).unwrap();
    fs::create_dir_all(&official_dir).unwrap();
    fs::create_dir_all(&extract_dir).unwrap();
    
    // Create test files
    println!("\n📁 Creating test files...");
    
    // Small text file
    let small_file = input_dir.join("small.txt");
    let mut f = File::create(&small_file).unwrap();
    writeln!(f, "This is a small test file for compression testing.").unwrap();
    
    // Medium text file (compressible)
    let medium_file = input_dir.join("medium.txt");
    let mut f = File::create(&medium_file).unwrap();
    for i in 0..1000 {
        writeln!(f, "Line {} - This is repeated text for compression testing. AAAAAAAAAA", i).unwrap();
    }
    
    // Binary file (less compressible - sequential pattern)
    let binary_file = input_dir.join("binary.bin");
    let mut f = File::create(&binary_file).unwrap();
    let data: Vec<u8> = (0..50000u32).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    
    // Random-ish file (hard to compress)
    let random_file = input_dir.join("random.bin");
    let mut f = File::create(&random_file).unwrap();
    // LCG pseudo-random
    let mut seed = 12345u64;
    let random_data: Vec<u8> = (0..50000).map(|_| {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((seed >> 16) & 0xFF) as u8
    }).collect();
    f.write_all(&random_data).unwrap();
    
    // Collect all files
    let files: Vec<String> = fs::read_dir(&input_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    
    let input_size: u64 = files.iter()
        .map(|f| fs::metadata(f).map(|m| m.len()).unwrap_or(0))
        .sum();
    
    println!("\n   Input files ({} bytes total):", input_size);
    for f in &files {
        let size = fs::metadata(f).map(|m| m.len()).unwrap_or(0);
        println!("   - {} ({} bytes)", Path::new(f).file_name().unwrap().to_string_lossy(), size);
    }
    
    // Create archive with official 7z
    println!("\n🔧 Creating archive with official 7z tool...");
    let official_archive = official_dir.join("test.7z");
    let mut cmd = Command::new("7z");
    cmd.arg("a").arg("-mx5").arg(&official_archive);
    for f in &files {
        cmd.arg(f);
    }
    let output = cmd.output().expect("Failed to run 7z");
    if !output.status.success() {
        println!("❌ Official 7z failed: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }
    let official_size = fs::metadata(&official_archive).map(|m| m.len()).unwrap_or(0);
    println!("   ✅ Created {} bytes", official_size);
    
    // Create archive with our library
    println!("\n🔧 Creating archive with our library...");
    let our_archive = our_dir.join("test.7z");
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let opts = CompressOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: None,
        auto_detect_incompressible: false,
    };
    
    if let Err(e) = sz.create_archive(&our_archive, &file_refs, CompressionLevel::Normal, Some(&opts)) {
        println!("❌ Failed to create archive: {}", e);
        return;
    }
    let our_size = fs::metadata(&our_archive).map(|m| m.len()).unwrap_or(0);
    println!("   ✅ Created {} bytes", our_size);
    
    // Comparison
    println!("\n📊 Archive Size Comparison:");
    println!("┌──────────────────┬────────────────┬────────────────┐");
    println!("│ Archive          │ Size (bytes)   │ Ratio          │");
    println!("├──────────────────┼────────────────┼────────────────┤");
    println!("│ Official 7z      │ {:>14} │ {:>13.1}% │", official_size, (official_size as f64 / input_size as f64) * 100.0);
    println!("│ Our library      │ {:>14} │ {:>13.1}% │", our_size, (our_size as f64 / input_size as f64) * 100.0);
    println!("└──────────────────┴────────────────┴────────────────┘");
    
    let size_diff = (our_size as i64 - official_size as i64).abs();
    let diff_percent = (size_diff as f64 / official_size as f64) * 100.0;
    if our_size <= official_size {
        println!("   🎉 Our library: {} bytes smaller ({:.1}% better)", size_diff, diff_percent);
    } else {
        println!("   📊 Our library: {} bytes larger ({:.1}% larger)", size_diff, diff_percent);
    }
    
    // Verify our archive with official 7z
    println!("\n🔍 Verifying our archive with official 7z tool...");
    let output = Command::new("7z")
        .args(["t", our_archive.to_str().unwrap()])
        .output()
        .expect("Failed to run 7z");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("Everything is Ok") {
        println!("   ✅ Archive verified successfully!");
    } else {
        println!("   ❌ Archive verification failed!");
        println!("{}", stdout);
    }
    
    // List contents
    println!("\n📋 Contents of our archive (via official 7z):");
    let list_output = Command::new("7z")
        .args(["l", our_archive.to_str().unwrap()])
        .output()
        .expect("Failed to run 7z");
    
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let mut in_file_list = false;
    for line in list_stdout.lines() {
        if line.contains("Date") && line.contains("Time") {
            in_file_list = true;
        }
        if in_file_list {
            println!("   {}", line);
        }
        if line.trim().is_empty() && in_file_list && line.trim() == "" {
            // Skip empty lines in list but keep header
        }
        if line.contains("files") && in_file_list {
            break;
        }
    }
    
    // Extract and verify
    println!("\n📤 Extracting our archive...");
    let extract_output = Command::new("7z")
        .args(["x", our_archive.to_str().unwrap(), &format!("-o{}", extract_dir.display()), "-y"])
        .output()
        .expect("Failed to run 7z");
    
    if extract_output.status.success() {
        println!("   ✅ Extraction successful!");
        
        println!("\n🔐 Verifying extracted files match originals:");
        let mut all_match = true;
        for f in &files {
            let filename = Path::new(f).file_name().unwrap();
            let extracted = extract_dir.join(filename);
            
            if extracted.exists() {
                let orig_data = fs::read(f).unwrap();
                let extr_data = fs::read(&extracted).unwrap();
                
                if orig_data == extr_data {
                    println!("   ✅ {} - MATCH ({} bytes)", filename.to_string_lossy(), orig_data.len());
                } else {
                    println!("   ❌ {} - CONTENT MISMATCH", filename.to_string_lossy());
                    all_match = false;
                }
            } else {
                println!("   ❌ {} - NOT FOUND", filename.to_string_lossy());
                all_match = false;
            }
        }
        
        if all_match {
            println!("\n   🎉 All files extracted correctly!");
        }
    } else {
        println!("   ❌ Extraction failed!");
        println!("{}", String::from_utf8_lossy(&extract_output.stderr));
    }
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║                    COMPARISON COMPLETE                         ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
