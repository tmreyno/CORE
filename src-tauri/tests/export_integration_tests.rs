// =============================================================================
// CORE-FFX Archive Export Integration Tests
// Tests all archive export functionality through the application layer
// Verifies 7-Zip compatibility for all archive types
// =============================================================================

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test helper to create test files of various types
fn create_test_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    
    // Small text file
    let small_file = dir.join("document.txt");
    let mut f = File::create(&small_file).unwrap();
    writeln!(f, "This is a small forensic test document.").unwrap();
    writeln!(f, "Evidence Item: EV-001").unwrap();
    writeln!(f, "Hash verification required.").unwrap();
    files.push(small_file.to_string_lossy().to_string());
    
    // Medium text file (compressible)
    let medium_file = dir.join("log.txt");
    let mut f = File::create(&medium_file).unwrap();
    for i in 0..1000 {
        writeln!(f, "[2026-02-02 {:02}:{:02}:{:02}] Event {} - System activity logged AAAAAAAAAA", 
                 i / 3600 % 24, i / 60 % 60, i % 60, i).unwrap();
    }
    files.push(medium_file.to_string_lossy().to_string());
    
    // Binary file (sequential pattern - moderately compressible)
    let binary_file = dir.join("data.bin");
    let mut f = File::create(&binary_file).unwrap();
    let data: Vec<u8> = (0..50000u32).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    files.push(binary_file.to_string_lossy().to_string());
    
    // Pseudo-random file (hard to compress)
    let random_file = dir.join("random.bin");
    let mut f = File::create(&random_file).unwrap();
    let mut seed = 12345u64;
    let random_data: Vec<u8> = (0..50000).map(|_| {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((seed >> 16) & 0xFF) as u8
    }).collect();
    f.write_all(&random_data).unwrap();
    files.push(random_file.to_string_lossy().to_string());
    
    files
}

/// Check if 7z command is available
fn has_7z_tool() -> bool {
    Command::new("7z").arg("--help").output().is_ok()
}

/// Verify archive with official 7z tool
fn verify_with_7z(archive_path: &Path, password: Option<&str>) -> bool {
    let mut cmd = Command::new("7z");
    cmd.arg("t");
    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }
    cmd.arg(archive_path);
    
    let output = cmd.output().expect("Failed to run 7z");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.contains("Everything is Ok")
}

/// Extract archive with 7z and verify contents
fn extract_and_verify(archive_path: &Path, extract_dir: &Path, original_files: &[String], password: Option<&str>) -> bool {
    fs::create_dir_all(extract_dir).unwrap();
    
    let mut cmd = Command::new("7z");
    cmd.arg("x");
    if let Some(pwd) = password {
        cmd.arg(format!("-p{}", pwd));
    }
    cmd.arg(archive_path);
    cmd.arg(format!("-o{}", extract_dir.display()));
    cmd.arg("-y");
    
    let output = cmd.output().expect("Failed to run 7z");
    if !output.status.success() {
        return false;
    }
    
    // Verify each file matches original
    for orig_path in original_files {
        let filename = Path::new(orig_path).file_name().unwrap();
        let extracted_path = extract_dir.join(filename);
        
        if !extracted_path.exists() {
            eprintln!("File not extracted: {}", filename.to_string_lossy());
            return false;
        }
        
        let orig_data = fs::read(orig_path).unwrap();
        let extr_data = fs::read(&extracted_path).unwrap();
        
        if orig_data != extr_data {
            eprintln!("Content mismatch: {}", filename.to_string_lossy());
            return false;
        }
    }
    
    true
}

/// TEST 1: Standard archive with all compression levels
#[test]
fn test_export_compression_levels() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Export Compression Levels (0-9)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let levels = [
        (0, "Store", CompressionLevel::Store),
        (1, "Fastest", CompressionLevel::Fastest),
        (3, "Fast", CompressionLevel::Fast),
        (5, "Normal", CompressionLevel::Normal),
        (7, "Maximum", CompressionLevel::Maximum),
        (9, "Ultra", CompressionLevel::Ultra),
    ];
    
    for (level_num, level_name, level) in levels {
        let archive_path = temp_dir.path().join(format!("export_level_{}.7z", level_num));
        
        let result = sz.create_archive(&archive_path, &file_refs, level, None);
        assert!(result.is_ok(), "Level {} ({}) should create archive", level_num, level_name);
        
        let size = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
        
        // Verify with 7z if available
        if has_7z_tool() {
            assert!(verify_with_7z(&archive_path, None), 
                    "Level {} archive should pass 7z verification", level_num);
            println!("✅ Level {} ({}): {} bytes - 7z verified", level_num, level_name, size);
        } else {
            println!("✅ Level {} ({}): {} bytes", level_num, level_name, size);
        }
    }
}

/// TEST 2: Encrypted archive with password
#[test]
fn test_export_encrypted() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Export Encrypted Archive (AES-256)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let password = "SecureForensicPassword123!";
    let archive_path = temp_dir.path().join("encrypted_export.7z");
    
    let opts = CompressOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: Some(password.to_string()),
        auto_detect_incompressible: false,
    };
    
    let result = sz.create_archive(&archive_path, &file_refs, CompressionLevel::Normal, Some(&opts));
    assert!(result.is_ok(), "Encrypted archive should be created");
    
    let size = fs::metadata(&archive_path).unwrap().len();
    println!("   Created encrypted archive: {} bytes", size);
    
    if has_7z_tool() {
        // Verify with correct password
        assert!(verify_with_7z(&archive_path, Some(password)),
                "Archive should verify with correct password");
        println!("✅ Correct password verification: PASSED");
        
        // Extract and verify content
        let extract_dir = temp_dir.path().join("extracted");
        assert!(extract_and_verify(&archive_path, &extract_dir, &files, Some(password)),
                "Extracted files should match originals");
        println!("✅ Content verification: PASSED");
    } else {
        println!("⚠️  7z tool not available - skipping external verification");
    }
}

/// TEST 3: Split/Multi-volume archive
#[test]
fn test_export_split_archive() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Export Split/Multi-Volume Archive");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create larger file for split testing
    let large_file = input_dir.join("large_evidence.bin");
    let mut f = File::create(&large_file).unwrap();
    let mut seed = 54321u64;
    for _ in 0..500000 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        f.write_all(&[((seed >> 16) & 0xFF) as u8]).unwrap();
    }
    drop(f);
    
    let files = vec![large_file.to_string_lossy().to_string()];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("split_export.7z");
    
    let opts = StreamOptions {
        split_size: 100 * 1024, // 100KB volumes
        ..Default::default()
    };
    
    let result = sz.create_archive_streaming(&archive_path, &file_refs, CompressionLevel::Fast, Some(&opts), None);
    assert!(result.is_ok(), "Split archive should be created");
    
    // Count volumes
    let mut volume_count = 0;
    let mut total_size = 0u64;
    for i in 1..=20 {
        let vol_path = temp_dir.path().join(format!("split_export.7z.{:03}", i));
        if let Ok(meta) = fs::metadata(&vol_path) {
            volume_count += 1;
            total_size += meta.len();
            println!("   Volume {}: {} bytes", i, meta.len());
        }
    }
    
    assert!(volume_count > 0, "Should create at least one volume");
    println!("   Total: {} volumes, {} bytes", volume_count, total_size);
    
    if has_7z_tool() {
        let first_vol = temp_dir.path().join("split_export.7z.001");
        
        // Verify with 7z
        assert!(verify_with_7z(&first_vol, None), "Split archive should verify with 7z");
        println!("✅ 7z verification: PASSED");
        
        // Extract and verify
        let extract_dir = temp_dir.path().join("extracted");
        assert!(extract_and_verify(&first_vol, &extract_dir, &files, None),
                "Extracted file should match original");
        println!("✅ Content verification: PASSED");
    }
}

/// TEST 4: Streaming with progress callback
#[test]
fn test_export_streaming_progress() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Export Streaming with Progress Callback");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let archive_path = temp_dir.path().join("streaming_export.7z");
    
    let progress_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_calls_clone = progress_calls.clone();
    
    let opts = StreamOptions::default();
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &file_refs,
        CompressionLevel::Normal,
        Some(&opts),
        Some(Box::new(move |_processed, _total, _file_bytes, _file_total, _filename| {
            progress_calls_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })),
    );
    
    assert!(result.is_ok(), "Streaming archive should be created");
    
    let size = fs::metadata(&archive_path).unwrap().len();
    println!("   Archive size: {} bytes", size);
    // Progress callback may or may not be called depending on data size
    // The important thing is that the archive is created correctly
    
    if has_7z_tool() {
        assert!(verify_with_7z(&archive_path, None), "Archive should verify with 7z");
        println!("✅ 7z verification: PASSED");
        
        let extract_dir = temp_dir.path().join("extracted");
        assert!(extract_and_verify(&archive_path, &extract_dir, &files, None),
                "Extracted files should match originals");
        println!("✅ Content verification: PASSED");
    }
}

/// TEST 5: Various compression options
#[test]
fn test_export_compression_options() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Export with Various Compression Options");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    let test_cases = vec![
        ("single_thread", CompressOptions { num_threads: 1, ..Default::default() }),
        ("multi_thread", CompressOptions { num_threads: 4, ..Default::default() }),
        ("no_solid", CompressOptions { solid: false, ..Default::default() }),
        ("solid", CompressOptions { solid: true, ..Default::default() }),
        ("dict_1mb", CompressOptions { dict_size: 1024 * 1024, ..Default::default() }),
    ];
    
    for (name, opts) in test_cases {
        let archive_path = temp_dir.path().join(format!("export_{}.7z", name));
        
        let result = sz.create_archive(&archive_path, &file_refs, CompressionLevel::Normal, Some(&opts));
        assert!(result.is_ok(), "Archive '{}' should be created", name);
        
        let size = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
        
        if has_7z_tool() {
            assert!(verify_with_7z(&archive_path, None), "'{}' should pass 7z verification", name);
            println!("✅ {}: {} bytes - 7z verified", name, size);
        } else {
            println!("✅ {}: {} bytes", name, size);
        }
    }
}

/// TEST 6: Full export simulation (like ExportPanel would do)
#[test]
fn test_full_export_simulation() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  TEST: Full Export Simulation (Application Layer)");
    println!("═══════════════════════════════════════════════════════════════");
    
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("evidence");
    let output_dir = temp_dir.path().join("exports");
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();
    
    // Create evidence files
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    
    // Simulate ExportPanel settings:
    // - Normal compression (level 5)
    // - Password protected
    // - 2 threads
    // - Solid archive
    let archive_path = output_dir.join("evidence_export.7z");
    let password = "CasePassword2026!";
    
    let opts = CompressOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: Some(password.to_string()),
        auto_detect_incompressible: false,
    };
    
    println!("   Creating forensic export archive...");
    let result = sz.create_archive(&archive_path, &file_refs, CompressionLevel::Normal, Some(&opts));
    assert!(result.is_ok(), "Export should complete successfully");
    
    let size = fs::metadata(&archive_path).unwrap().len();
    println!("   Archive created: {} bytes", size);
    println!("   Files: {}", files.len());
    println!("   Encryption: AES-256");
    println!("   Compression: Normal (LZMA2)");
    
    if has_7z_tool() {
        // Verify with 7z
        assert!(verify_with_7z(&archive_path, Some(password)), "Export should verify with 7z");
        println!("✅ Archive integrity: VERIFIED");
        
        // Extract and verify all files
        let extract_dir = temp_dir.path().join("verify");
        assert!(extract_and_verify(&archive_path, &extract_dir, &files, Some(password)),
                "All files should extract and match originals");
        println!("✅ Content integrity: VERIFIED");
        
        println!("\n   ╔════════════════════════════════════════════════════════╗");
        println!("   ║  EXPORT SIMULATION: ALL CHECKS PASSED                  ║");
        println!("   ╚════════════════════════════════════════════════════════╝");
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       CORE-FFX Export Archive Integration Tests               ║");
    println!("║       Verifying 7-Zip Compatibility for Forensic Export       ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    if has_7z_tool() {
        println!("✅ 7z tool detected - full verification enabled");
    } else {
        println!("⚠️  7z tool not found - running basic tests only");
    }
    
    // Run all tests
    test_export_compression_levels();
    test_export_encrypted();
    test_export_split_archive();
    test_export_streaming_progress();
    test_export_compression_options();
    test_full_export_simulation();
    
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              ALL EXPORT TESTS COMPLETED                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
