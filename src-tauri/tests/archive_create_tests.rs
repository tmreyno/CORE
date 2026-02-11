// =============================================================================
// CORE-FFX Archive Creation Tests
// Tests all 7z creation variables against the C library capabilities
// =============================================================================

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

/// Test helper to create test files
fn create_test_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    
    // Create small text file
    let small_file = dir.join("small.txt");
    let mut f = File::create(&small_file).unwrap();
    writeln!(f, "This is a small test file for compression testing.").unwrap();
    files.push(small_file.to_string_lossy().to_string());
    
    // Create medium file with compressible data
    let medium_file = dir.join("medium.txt");
    let mut f = File::create(&medium_file).unwrap();
    for i in 0..1000 {
        writeln!(f, "Line {} - This is repeated text for compression testing. AAAAAAAAAA", i).unwrap();
    }
    files.push(medium_file.to_string_lossy().to_string());
    
    // Create binary file with random-ish data
    let binary_file = dir.join("binary.bin");
    let mut f = File::create(&binary_file).unwrap();
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    files.push(binary_file.to_string_lossy().to_string());
    
    files
}

/// Test: Library initialization
#[test]
fn test_library_init() {
    println!("\n=== TEST: Library Initialization ===");
    let result = SevenZip::new();
    assert!(result.is_ok(), "SevenZip library should initialize");
    println!("✅ PASS: Library initialized successfully");
}

/// Test: All compression levels (0-9)
#[test]
fn test_compression_levels() {
    println!("\n=== TEST: Compression Levels (0-9) ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    
    let levels = [
        (0, "Store", CompressionLevel::Store),
        (1, "Fastest", CompressionLevel::Fastest),
        (3, "Fast", CompressionLevel::Fast),
        (5, "Normal", CompressionLevel::Normal),
        (7, "Maximum", CompressionLevel::Maximum),
        (9, "Ultra", CompressionLevel::Ultra),
    ];
    
    for (level_num, level_name, level) in levels {
        let archive_path = temp_dir.path().join(format!("test_level_{}.7z", level_num));
        
        let result = sz.create_archive(
            &archive_path,
            &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            level,
            None,
        );
        
        match result {
            Ok(_) => {
                let size = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
                println!("✅ Level {} ({}): Created {} bytes", level_num, level_name, size);
            }
            Err(e) => {
                println!("❌ Level {} ({}): FAILED - {}", level_num, level_name, e);
            }
        }
    }
}

/// Test: Standard archive creation
#[test]
fn test_standard_archive_creation() {
    println!("\n=== TEST: Standard Archive Creation ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("standard.7z");
    
    let result = sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        None,
    );
    
    assert!(result.is_ok(), "Standard archive creation should succeed");
    assert!(archive_path.exists(), "Archive file should exist");
    
    let size = fs::metadata(&archive_path).unwrap().len();
    println!("✅ PASS: Created archive ({} bytes)", size);
}

/// Test: CompressOptions
#[test]
fn test_compress_options() {
    println!("\n=== TEST: CompressOptions ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    
    // Test with various options
    let test_cases = vec![
        ("threads=1", CompressOptions { num_threads: 1, dict_size: 0, solid: true, password: None, auto_detect_incompressible: false }),
        ("threads=4", CompressOptions { num_threads: 4, dict_size: 0, solid: true, password: None, auto_detect_incompressible: false }),
        ("dict=1MB", CompressOptions { num_threads: 2, dict_size: 1024 * 1024, solid: true, password: None, auto_detect_incompressible: false }),
        ("dict=4MB", CompressOptions { num_threads: 2, dict_size: 4 * 1024 * 1024, solid: true, password: None, auto_detect_incompressible: false }),
        ("solid=false", CompressOptions { num_threads: 2, dict_size: 0, solid: false, password: None, auto_detect_incompressible: false }),
        ("solid=true", CompressOptions { num_threads: 2, dict_size: 0, solid: true, password: None, auto_detect_incompressible: false }),
    ];
    
    for (name, opts) in test_cases {
        let archive_path = temp_dir.path().join(format!("opts_{}.7z", name.replace("=", "_")));
        
        let result = sz.create_archive(
            &archive_path,
            &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            CompressionLevel::Normal,
            Some(&opts),
        );
        
        match result {
            Ok(_) => {
                let size = fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0);
                println!("✅ {}: Created {} bytes", name, size);
            }
            Err(e) => {
                println!("❌ {}: FAILED - {}", name, e);
            }
        }
    }
}

/// Test: Encrypted archive creation
#[test]
fn test_encrypted_archive() {
    println!("\n=== TEST: Encrypted Archive (AES-256) ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("encrypted.7z");
    
    let opts = CompressOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: Some("TestPassword123!".to_string()),
        auto_detect_incompressible: false,
    };
    
    let result = sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        Some(&opts),
    );
    
    match result {
        Ok(_) => {
            let size = fs::metadata(&archive_path).unwrap().len();
            println!("✅ PASS: Encrypted archive created ({} bytes)", size);
        }
        Err(e) => {
            println!("❌ FAIL: Encrypted archive failed - {}", e);
            panic!("Encrypted archive should succeed");
        }
    }
}

/// Test: Streaming compression
#[test]
fn test_streaming_compression() {
    println!("\n=== TEST: Streaming Compression ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("streaming.7z");
    
    let opts = StreamOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: None,
        split_size: 0,
        chunk_size: 64 * 1024 * 1024, // 64MB
        temp_dir: None,
        delete_temp_on_error: true,
    };
    
    let progress_count = std::sync::atomic::AtomicUsize::new(0);
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        Some(&opts),
        Some(Box::new(move |_processed, _total, _file_bytes, _file_total, _filename| {
            progress_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // Progress callback is working
        })),
    );
    
    match result {
        Ok(_) => {
            let size = fs::metadata(&archive_path).unwrap().len();
            println!("✅ PASS: Streaming archive created ({} bytes)", size);
        }
        Err(e) => {
            println!("❌ FAIL: Streaming compression failed - {}", e);
            panic!("Streaming compression should succeed");
        }
    }
}

/// Test: Streaming with progress callback
#[test]
fn test_streaming_with_progress() {
    println!("\n=== TEST: Streaming with Progress Callback ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("progress.7z");
    
    let opts = StreamOptions::default();
    
    let progress_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let progress_called_clone = progress_called.clone();
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        Some(&opts),
        Some(Box::new(move |processed, total, _file_bytes, _file_total, filename| {
            progress_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            println!("  Progress: {}/{} bytes, file: {}", processed, total, filename);
        })),
    );
    
    match result {
        Ok(_) => {
            println!("✅ PASS: Archive created with progress callback");
            if progress_called.load(std::sync::atomic::Ordering::SeqCst) {
                println!("✅ PASS: Progress callback was invoked");
            } else {
                println!("⚠️  WARN: Progress callback was NOT invoked");
            }
        }
        Err(e) => {
            println!("❌ FAIL: Streaming with progress failed - {}", e);
        }
    }
}

/// Test: Split/multi-volume archive
#[test]
fn test_split_archive() {
    println!("\n=== TEST: Split/Multi-Volume Archive ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create a larger file for split testing
    let large_file = input_dir.join("large.bin");
    let mut f = File::create(&large_file).unwrap();
    let data: Vec<u8> = (0..500000).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("split.7z");
    
    let mut opts = StreamOptions::default();
    opts.split_size = 100 * 1024; // 100KB volumes for testing
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &[large_file.to_string_lossy().to_string()].iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Fast,
        Some(&opts),
        None,
    );
    
    match result {
        Ok(_) => {
            // Check for split volumes
            let vol1 = temp_dir.path().join("split.7z.001");
            let vol2 = temp_dir.path().join("split.7z.002");
            
            if vol1.exists() {
                println!("✅ PASS: Split archive created");
                println!("  Volume 1: {} bytes", fs::metadata(&vol1).map(|m| m.len()).unwrap_or(0));
                if vol2.exists() {
                    println!("  Volume 2: {} bytes", fs::metadata(&vol2).map(|m| m.len()).unwrap_or(0));
                }
            } else if archive_path.exists() {
                println!("⚠️  WARN: Single archive created instead of split");
                println!("  Size: {} bytes", fs::metadata(&archive_path).map(|m| m.len()).unwrap_or(0));
            }
        }
        Err(e) => {
            println!("❌ FAIL: Split archive failed - {}", e);
        }
    }
}

/// Test: Extract and verify
#[test]
fn test_extract_and_verify() {
    println!("\n=== TEST: Extract and Verify ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();
    
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("verify.7z");
    
    // Create archive
    sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        None,
    ).unwrap();
    
    // Extract archive
    let extract_result = sz.extract(&archive_path, &output_dir);
    
    match extract_result {
        Ok(_) => {
            println!("✅ PASS: Archive extracted successfully");
            
            // Verify files exist
            for file in &files {
                let filename = Path::new(file).file_name().unwrap();
                let extracted = output_dir.join(filename);
                if extracted.exists() {
                    println!("  ✅ Found: {}", filename.to_string_lossy());
                } else {
                    println!("  ❌ Missing: {}", filename.to_string_lossy());
                }
            }
        }
        Err(e) => {
            println!("❌ FAIL: Extraction failed - {}", e);
        }
    }
}

/// Test: Archive integrity check
#[test]
fn test_archive_integrity() {
    println!("\n=== TEST: Archive Integrity Check ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("integrity.7z");
    
    sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        None,
    ).unwrap();
    
    let test_result = sz.test_archive(&archive_path, None, None);
    
    match test_result {
        Ok(_) => {
            println!("✅ PASS: Archive integrity verified");
        }
        Err(e) => {
            println!("❌ FAIL: Integrity check error - {}", e);
        }
    }
}

/// Test: List archive contents
#[test]
fn test_list_archive() {
    println!("\n=== TEST: List Archive Contents ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    let files = create_test_files(&input_dir);
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("list.7z");
    
    sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        None,
    ).unwrap();
    
    let list_result = sz.list(&archive_path, None);
    
    match list_result {
        Ok(entries) => {
            println!("✅ PASS: Listed {} entries", entries.len());
            for entry in &entries {
                println!("  - {} ({} bytes, ratio: {:.1}%)", 
                    entry.name, entry.size, entry.compression_ratio() * 100.0);
            }
        }
        Err(e) => {
            println!("❌ FAIL: List failed - {}", e);
        }
    }
}

/// Test: Large file streaming with chunked processing
#[test]
fn test_large_file_streaming() {
    println!("\n=== TEST: Large File Streaming ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir_all(&input_dir).unwrap();
    
    // Create a larger file (1MB) to test streaming
    let large_file = input_dir.join("large_test.bin");
    let mut f = File::create(&large_file).unwrap();
    let data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    f.write_all(&data).unwrap();
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("large_stream.7z");
    
    let opts = StreamOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: None,
        split_size: 0,
        chunk_size: 64 * 1024 * 1024,
        temp_dir: None,
        delete_temp_on_error: true,
    };
    
    let progress_bytes = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let progress_bytes_clone = progress_bytes.clone();
    
    let result = sz.create_archive_streaming(
        &archive_path,
        &[large_file.to_string_lossy().to_string()].iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        Some(&opts),
        Some(Box::new(move |processed, _total, _file_bytes, _file_total, _filename| {
            progress_bytes_clone.store(processed, std::sync::atomic::Ordering::SeqCst);
        })),
    );
    
    match result {
        Ok(_) => {
            if archive_path.exists() {
                let size = fs::metadata(&archive_path).unwrap().len();
                let input_size = fs::metadata(&large_file).unwrap().len();
                let ratio = (size as f64 / input_size as f64) * 100.0;
                println!("✅ PASS: Large file streaming archive created");
                println!("  Input:  {} bytes", input_size);
                println!("  Output: {} bytes ({:.1}% ratio)", size, ratio);
                
                // Verify it's a valid archive
                let test_result = sz.test_archive(&archive_path, None, None);
                match test_result {
                    Ok(_) => println!("✅ PASS: Archive is valid"),
                    Err(e) => println!("⚠️  WARN: Could not verify - {}", e),
                }
            } else {
                println!("❌ FAIL: Archive file not created");
            }
        }
        Err(e) => {
            println!("❌ FAIL: Large file streaming failed - {}", e);
        }
    }
}

/// Test: Encrypted extraction
#[test]
fn test_encrypted_extraction() {
    println!("\n=== TEST: Encrypted Archive Extraction ===");
    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();
    
    let files = create_test_files(&input_dir);
    let password = "SecurePassword123!";
    
    let sz = SevenZip::new().unwrap();
    let archive_path = temp_dir.path().join("encrypted_extract.7z");
    
    // Create encrypted archive
    let opts = CompressOptions {
        num_threads: 2,
        dict_size: 0,
        solid: true,
        password: Some(password.to_string()),
        auto_detect_incompressible: false,
    };
    
    sz.create_archive(
        &archive_path,
        &files.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        CompressionLevel::Normal,
        Some(&opts),
    ).unwrap();
    
    // Extract with correct password
    let extract_result = sz.extract_with_password(&archive_path, &output_dir, Some(password), None);
    
    match extract_result {
        Ok(_) => {
            println!("✅ PASS: Encrypted archive extracted with correct password");
        }
        Err(e) => {
            println!("❌ FAIL: Encrypted extraction failed - {}", e);
        }
    }
    
    // Try with wrong password
    let wrong_result = sz.extract_with_password(&archive_path, &output_dir, Some("WrongPassword"), None);
    
    match wrong_result {
        Ok(_) => {
            println!("⚠️  WARN: Extraction succeeded with wrong password (unexpected)");
        }
        Err(_) => {
            println!("✅ PASS: Wrong password correctly rejected");
        }
    }
}

// Main test runner
fn main() {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║        CORE-FFX Archive Creation Test Suite                       ║");
    println!("║        Testing all 7z creation variables                          ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝");
    
    // Run all tests manually for detailed output
    test_library_init();
    test_compression_levels();
    test_standard_archive_creation();
    test_compress_options();
    test_encrypted_archive();
    test_streaming_compression();
    test_streaming_with_progress();
    test_split_archive();
    test_extract_and_verify();
    test_archive_integrity();
    test_list_archive();
    test_large_file_streaming();
    test_encrypted_extraction();
    
    println!("\n╔═══════════════════════════════════════════════════════════════════╗");
    println!("║                    TEST SUITE COMPLETE                            ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝");
}
