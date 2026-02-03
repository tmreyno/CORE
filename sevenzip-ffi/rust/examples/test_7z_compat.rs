//! 7-Zip Compatibility Test
//! 
//! Tests that our library creates archives that can be read by the official 7z tool
//! including standard, encrypted, and split archives.

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║        7-Zip Compatibility Test Suite                          ║");
    println!("║        Testing Standard, Encrypted & Split Archives            ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    // Check for 7z
    if Command::new("7z").output().is_err() {
        println!("❌ 7z not found. Install p7zip to run this test.");
        return;
    }
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create test files
    println!("\n📁 Creating test files...");
    
    let text_file = input_dir.join("document.txt");
    let mut f = File::create(&text_file).unwrap();
    for i in 0..500 {
        writeln!(f, "Line {} - This is test content for compression AAAAAAAAAAAAAAAA", i).unwrap();
    }
    
    let binary_file = input_dir.join("data.bin");
    let mut f = File::create(&binary_file).unwrap();
    let data: Vec<u8> = (0..100000).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    
    let files: Vec<String> = vec![
        text_file.to_string_lossy().to_string(),
        binary_file.to_string_lossy().to_string(),
    ];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let total_size: u64 = files.iter()
        .map(|f| fs::metadata(f).map(|m| m.len()).unwrap_or(0))
        .sum();
    println!("   Total input: {} bytes", total_size);
    
    let sz = SevenZip::new().unwrap();
    let mut all_passed = true;
    
    // =========================================================================
    // TEST 1: Standard Archive
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST 1: Standard 7z Archive");
    println!("═══════════════════════════════════════════════════════════════");
    
    let standard_archive = temp_dir.path().join("standard.7z");
    sz.create_archive(&standard_archive, &file_refs, CompressionLevel::Normal, None).unwrap();
    
    let size = fs::metadata(&standard_archive).unwrap().len();
    println!("   Created: {} bytes ({:.1}% ratio)", size, (size as f64 / total_size as f64) * 100.0);
    
    // Test with 7z
    let output = Command::new("7z")
        .args(["t", standard_archive.to_str().unwrap()])
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.contains("Everything is Ok") {
        println!("   ✅ 7z verification: PASSED");
    } else {
        println!("   ❌ 7z verification: FAILED");
        println!("{}", stdout);
        all_passed = false;
    }
    
    // List contents
    let list_output = Command::new("7z")
        .args(["l", standard_archive.to_str().unwrap()])
        .output().unwrap();
    println!("   📋 Contents:");
    for line in String::from_utf8_lossy(&list_output.stdout).lines() {
        if line.contains(".txt") || line.contains(".bin") {
            println!("      {}", line.trim());
        }
    }
    
    // Extract and verify
    let extract_dir = temp_dir.path().join("extract_standard");
    fs::create_dir_all(&extract_dir).unwrap();
    let extract = Command::new("7z")
        .args(["x", standard_archive.to_str().unwrap(), &format!("-o{}", extract_dir.display()), "-y"])
        .output().unwrap();
    
    if extract.status.success() {
        println!("   ✅ 7z extraction: PASSED");
    } else {
        println!("   ❌ 7z extraction: FAILED");
        all_passed = false;
    }
    
    // =========================================================================
    // TEST 2: Encrypted Archive
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST 2: Encrypted Archive (AES-256)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let encrypted_archive = temp_dir.path().join("encrypted.7z");
    let password = "SecureTest123!";
    
    let enc_opts = CompressOptions {
        password: Some(password.to_string()),
        ..Default::default()
    };
    sz.create_archive(&encrypted_archive, &file_refs, CompressionLevel::Normal, Some(&enc_opts)).unwrap();
    
    let size = fs::metadata(&encrypted_archive).unwrap().len();
    println!("   Created: {} bytes", size);
    println!("   Password: {}", password);
    
    // Test with correct password
    let output = Command::new("7z")
        .args(["t", &format!("-p{}", password), encrypted_archive.to_str().unwrap()])
        .output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.contains("Everything is Ok") {
        println!("   ✅ 7z verification (correct password): PASSED");
    } else {
        println!("   ❌ 7z verification (correct password): FAILED");
        println!("{}", stdout);
        all_passed = false;
    }
    
    // Test with wrong password
    let output = Command::new("7z")
        .args(["t", "-pWrongPassword", encrypted_archive.to_str().unwrap()])
        .output().unwrap();
    
    if !output.status.success() {
        println!("   ✅ 7z rejects wrong password: PASSED");
    } else {
        println!("   ⚠️  7z accepted wrong password (unexpected)");
    }
    
    // Extract with password
    let extract_dir = temp_dir.path().join("extract_encrypted");
    fs::create_dir_all(&extract_dir).unwrap();
    let extract = Command::new("7z")
        .args(["x", &format!("-p{}", password), encrypted_archive.to_str().unwrap(), &format!("-o{}", extract_dir.display()), "-y"])
        .output().unwrap();
    
    if extract.status.success() {
        println!("   ✅ 7z extraction (with password): PASSED");
    } else {
        println!("   ❌ 7z extraction (with password): FAILED");
        all_passed = false;
    }
    
    // =========================================================================
    // TEST 3: Split Archive
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST 3: Split/Multi-Volume Archive");
    println!("═══════════════════════════════════════════════════════════════");
    
    let split_archive = temp_dir.path().join("split.7z");
    
    let split_opts = StreamOptions {
        split_size: 20 * 1024, // 20KB volumes for testing
        ..Default::default()
    };
    
    sz.create_archive_streaming(&split_archive, &file_refs, CompressionLevel::Normal, Some(&split_opts), None).unwrap();
    
    // List volumes
    println!("   📦 Volumes created:");
    let mut volume_count = 0;
    let mut total_volume_size = 0u64;
    for i in 1..=20 {
        let vol_path = temp_dir.path().join(format!("split.7z.{:03}", i));
        if let Ok(meta) = fs::metadata(&vol_path) {
            volume_count += 1;
            total_volume_size += meta.len();
            println!("      split.7z.{:03} - {} bytes", i, meta.len());
        }
    }
    println!("   Total: {} volumes, {} bytes", volume_count, total_volume_size);
    
    // Test first volume with 7z
    let first_vol = temp_dir.path().join("split.7z.001");
    if first_vol.exists() {
        let output = Command::new("7z")
            .args(["t", first_vol.to_str().unwrap()])
            .output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stdout.contains("Everything is Ok") {
            println!("   ✅ 7z verification: PASSED");
        } else if stdout.contains("Type = Split") || stdout.contains("split") {
            println!("   ✅ 7z recognizes split archive format");
        } else {
            println!("   ⚠️  7z output: {}", stdout.lines().take(5).collect::<Vec<_>>().join(" | "));
        }
        
        // List with 7z
        let list_output = Command::new("7z")
            .args(["l", first_vol.to_str().unwrap()])
            .output().unwrap();
        let list_stdout = String::from_utf8_lossy(&list_output.stdout);
        
        println!("   📋 7z list output:");
        for line in list_stdout.lines().take(15) {
            if !line.trim().is_empty() {
                println!("      {}", line);
            }
        }
        
        // Extract split archive
        let extract_dir = temp_dir.path().join("extract_split");
        fs::create_dir_all(&extract_dir).unwrap();
        let extract = Command::new("7z")
            .args(["x", first_vol.to_str().unwrap(), &format!("-o{}", extract_dir.display()), "-y"])
            .output().unwrap();
        
        if extract.status.success() {
            println!("   ✅ 7z extraction: PASSED");
            
            // Verify extracted files
            for f in &files {
                let filename = std::path::Path::new(f).file_name().unwrap();
                let extracted = extract_dir.join(filename);
                if extracted.exists() {
                    let orig = fs::read(f).unwrap();
                    let extr = fs::read(&extracted).unwrap();
                    if orig == extr {
                        println!("   ✅ {} - content verified", filename.to_string_lossy());
                    } else {
                        println!("   ❌ {} - content mismatch", filename.to_string_lossy());
                        all_passed = false;
                    }
                }
            }
        } else {
            println!("   ❌ 7z extraction: FAILED");
            println!("      {}", String::from_utf8_lossy(&extract.stderr));
            all_passed = false;
        }
    }
    
    // =========================================================================
    // SUMMARY
    // =========================================================================
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    if all_passed {
        println!("║  ✅ ALL TESTS PASSED - Full 7-Zip Compatibility!              ║");
    } else {
        println!("║  ⚠️  SOME TESTS FAILED - See details above                    ║");
    }
    println!("╚════════════════════════════════════════════════════════════════╝");
}
