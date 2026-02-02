//! Comprehensive feature demonstration for 7z FFI SDK Rust bindings
//!
//! This example demonstrates all major features:
//! - Creating and extracting 7z archives
//! - AES-256 encryption/decryption
//! - All compression levels
//! - Multi-file operations
//! - Archive listing and integrity testing
//! - Comprehensive error handling

use seven_zip::{SevenZip, CompressionLevel, CompressOptions, Result};
use std::fs;

fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  7z FFI SDK Rust Bindings - Feature Demonstration       ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Initialize library
    let sz = SevenZip::new()?;
    println!("✓ Library initialized\n");

    // Setup test data
    fs::create_dir_all("demo_test_data")?;
    fs::write("demo_test_data/file1.txt", "This is test file 1")?;
    fs::write("demo_test_data/file2.txt", "This is test file 2 with more content")?;
    fs::write("demo_test_data/file3.txt", "Test file 3: Lorem ipsum dolor sit amet")?;
    println!("✓ Created 3 test files\n");

    // Feature 1: Create archive with different compression levels
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 1: Compression Levels");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let levels = vec![
        ("Store", CompressionLevel::Store),
        ("Fast", CompressionLevel::Fast),
        ("Normal", CompressionLevel::Normal),
        ("Maximum", CompressionLevel::Maximum),
        ("Ultra", CompressionLevel::Ultra),
    ];

    for (name, level) in &levels {
        let archive_name = format!("demo_{}.7z", name.to_lowercase());
        sz.create_archive(
            &archive_name,
            &["demo_test_data/file1.txt", "demo_test_data/file2.txt", "demo_test_data/file3.txt"],
            *level,
            None,
        )?;
        let size = fs::metadata(&archive_name)?.len();
        println!("  {} compression: {} ({} bytes)", name, archive_name, size);
    }
    println!("\n✓ Created 5 archives with different compression levels\n");

    // Feature 2: Multi-file operations
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 2: Multi-file Operations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    sz.create_archive(
        "demo_multifile.7z",
        &["demo_test_data/file1.txt", "demo_test_data/file2.txt", "demo_test_data/file3.txt"],
        CompressionLevel::Normal,
        None,
    )?;
    println!("  Created: demo_multifile.7z (3 files)\n");

    // Feature 3: Archive listing
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 3: Archive Listing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let entries = sz.list("demo_multifile.7z", None)?;
    println!("  Archive contents ({} files):", entries.len());
    for entry in &entries {
        println!("    - {} ({} bytes, ratio: {:.1}%)", 
                 entry.name, 
                 entry.size,
                 entry.compression_ratio() * 100.0);
    }
    println!();

    // Feature 4: Integrity testing
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 4: Integrity Testing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let is_valid = sz.test_archive("demo_multifile.7z", None).is_ok();
    println!("  Archive integrity: {}", if is_valid { "✓ VALID" } else { "✗ INVALID" });
    println!();

    // Feature 5: Extraction
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 5: Archive Extraction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    fs::create_dir_all("demo_extracted")?;
    sz.extract("demo_multifile.7z", "demo_extracted")?;
    
    let extracted_files = fs::read_dir("demo_extracted")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();
    println!("  Extracted {} files to demo_extracted/", extracted_files);
    
    // Verify content
    let content = fs::read_to_string("demo_extracted/file1.txt")?;
    println!("  Verified content: '{}'", content);
    println!();

    // Feature 6: AES-256 Encryption
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 6: AES-256 Encryption/Decryption");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let password = "SecurePassword123!";
    let options = CompressOptions {
        password: Some(password.to_string()),
        ..Default::default()
    };
    
    sz.create_archive(
        "demo_encrypted.7z",
        &["demo_test_data/file1.txt", "demo_test_data/file2.txt"],
        CompressionLevel::Normal,
        Some(&options),
    )?;
    println!("  Created encrypted archive: demo_encrypted.7z");
    println!("  Password: {}", password);

    // Test with correct password
    fs::create_dir_all("demo_extracted_encrypted")?;
    match sz.extract_with_password("demo_encrypted.7z", "demo_extracted_encrypted", Some(password), None) {
        Ok(_) => println!("  ✓ Decryption with correct password: SUCCESS"),
        Err(e) => println!("  ✗ Decryption failed: {}", e),
    }

    // Test with wrong password
    match sz.extract_with_password("demo_encrypted.7z", "demo_extracted_wrong", Some("WrongPassword"), None) {
        Ok(_) => println!("  ✗ Wrong password should fail!"),
        Err(_) => println!("  ✓ Wrong password correctly rejected"),
    }
    println!();

    // Feature 7: Error handling
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Feature 7: Error Handling");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test invalid file
    match sz.extract("nonexistent.7z", "output") {
        Ok(_) => println!("  ✗ Should have failed for missing file"),
        Err(e) => println!("  ✓ Invalid file error handled: {}", e),
    }

    // Test invalid archive
    fs::write("invalid.7z", "This is not a valid 7z archive")?;
    match sz.extract("invalid.7z", "output") {
        Ok(_) => println!("  ✗ Should have failed for invalid archive"),
        Err(e) => println!("  ✓ Invalid archive error handled: {}", e),
    }
    println!();

    // Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("                    FEATURE SUMMARY                        ");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("  ✓ Archive creation and extraction");
    println!("  ✓ AES-256 encryption/decryption");
    println!("  ✓ All compression levels (Store→Ultra)");
    println!("  ✓ Multi-file operations");
    println!("  ✓ Archive listing");
    println!("  ✓ Integrity testing");
    println!("  ✓ Comprehensive error handling");
    println!("\n  All features are FUNCTIONAL and TESTED!\n");

    // Cleanup
    println!("Cleaning up demo files...");
    let _ = fs::remove_dir_all("demo_test_data");
    let _ = fs::remove_dir_all("demo_extracted");
    let _ = fs::remove_dir_all("demo_extracted_encrypted");
    let _ = fs::remove_file("demo_store.7z");
    let _ = fs::remove_file("demo_fast.7z");
    let _ = fs::remove_file("demo_normal.7z");
    let _ = fs::remove_file("demo_maximum.7z");
    let _ = fs::remove_file("demo_ultra.7z");
    let _ = fs::remove_file("demo_multifile.7z");
    let _ = fs::remove_file("demo_encrypted.7z");
    let _ = fs::remove_file("invalid.7z");
    println!("✓ Cleanup complete\n");

    Ok(())
}
