//! Complete example demonstrating all 7z library features
//!
//! Run with: cargo run --example complete_demo

use seven_zip::{
    SevenZip, CompressionLevel, CompressOptions,
    encryption::EncryptionContext, Error,
};
use std::fs;

fn main() -> Result<(), Error> {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   7z FFI SDK - Complete Rust Bindings Demo              ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // Initialize library
    println!("1. Initializing 7z library...");
    let sz = SevenZip::new()?;
    println!("   ✓ Library initialized\n");

    // Create test data
    println!("2. Creating test data...");
    fs::create_dir_all("test_data")?;
    fs::write("test_data/file1.txt", "Test file 1 content")?;
    fs::write("test_data/file2.txt", "Test file 2 with more data")?;
    fs::create_dir_all("test_data/subdir")?;
    fs::write("test_data/subdir/file3.txt", "Nested file")?;
    println!("   ✓ Test data created\n");

    // Test 1: Create standard 7z archive
    println!("3. Creating standard 7z archive...");
    sz.create_archive(
        "test_archive.7z",
        &["test_data/file1.txt", "test_data/file2.txt"],
        CompressionLevel::Normal,
        None
    )?;
    println!("   ✓ Archive created: test_archive.7z\n");

    // Test 2: List archive contents
    println!("4. Listing archive contents...");
    let entries = sz.list("test_archive.7z", None)?;
    for entry in &entries {
        println!("   - {}: {} bytes → {} bytes ({}% compression)",
            entry.name,
            entry.size,
            entry.packed_size,
            entry.compression_ratio() as i32
        );
    }
    println!();

    // Test 3: Extract archive
    println!("5. Extracting archive...");
    fs::create_dir_all("extracted")?;
    sz.extract("test_archive.7z", "extracted")?;
    println!("   ✓ Extracted to: extracted/\n");

    // Test 4: Create encrypted archive
    println!("6. Creating encrypted archive...");
    let mut opts = CompressOptions::default();
    opts.password = Some("TestPassword123!".to_string());
    opts.num_threads = 4;
    
    sz.create_archive(
        "test_encrypted.7z",
        &["test_data"],
        CompressionLevel::Normal,
        Some(&opts)
    )?;
    println!("   ✓ Encrypted archive created: test_encrypted.7z\n");

    // Test 5: Extract encrypted archive
    println!("7. Extracting encrypted archive...");
    fs::create_dir_all("extracted_encrypted")?;
    sz.extract_with_password(
        "test_encrypted.7z",
        "extracted_encrypted",
        Some("TestPassword123!"),
        Some(Box::new(|completed, total| {
            if total > 0 {
                let pct = (completed as f64 / total as f64) * 100.0;
                print!("\r   Progress: {:.1}%", pct);
            }
        }))
    )?;
    println!("\n   ✓ Extracted encrypted archive\n");

    // Test 6: Test archive integrity
    println!("8. Testing archive integrity...");
    sz.test_archive("test_archive.7z", None)?;
    println!("   ✓ Archive integrity verified\n");

    // Test 7: Direct encryption example
    println!("9. Testing direct encryption...");
    let mut enc_ctx = EncryptionContext::new("DirectEncryptionPassword")?;
    let plaintext = b"This is sensitive forensic evidence data";
    let ciphertext = enc_ctx.encrypt(plaintext)?;
    let decrypted = enc_ctx.decrypt(&ciphertext)?;
    
    assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    println!("   Original:  {} bytes", plaintext.len());
    println!("   Encrypted: {} bytes", ciphertext.len());
    println!("   Decrypted: {} bytes", decrypted.len());
    println!("   ✓ Encryption roundtrip successful\n");

    // Test 8: Single file compression
    // TODO: Implement sevenzip_compress_file and sevenzip_decompress_file in C library
    // println!("10. Compressing single file...");
    // sz.compress_file("test_data/file1.txt", "file1.lzma2", CompressionLevel::Normal)?;
    // println!("    ✓ Compressed to: file1.lzma2\n");

    // Test 9: Single file decompression
    // println!("11. Decompressing single file...");
    // sz.decompress_file("file1.lzma2", "file1_decompressed.txt")?;
    // println!("    ✓ Decompressed to: file1_decompressed.txt\n");

    // Test 10: Extract specific files
    println!("10. Extracting specific files...");
    fs::create_dir_all("extracted_specific")?;
    sz.extract_files(
        "test_archive.7z",
        "extracted_specific",
        &["file1.txt"],
        None
    )?;
    println!("    ✓ Extracted specific files\n");

    // Summary
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                    ALL TESTS PASSED!                      ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    println!("Demonstration complete! Features tested:");
    println!("  ✓ Standard 7z archive creation");
    println!("  ✓ Archive listing");
    println!("  ✓ Archive extraction");
    println!("  ✓ Encrypted archive creation");
    println!("  ✓ Encrypted archive extraction");
    println!("  ✓ Archive integrity testing");
    println!("  ✓ Direct AES-256 encryption");
    // println!("  ✓ Single file compression/decompression"); // TODO: Not yet implemented
    println!("  ✓ Selective file extraction\n");
    // println!("  ✓ Progress callbacks\n"); // TODO: Not yet implemented

    // Cleanup
    println!("Cleaning up test files...");
    let _ = fs::remove_dir_all("test_data");
    let _ = fs::remove_dir_all("extracted");
    let _ = fs::remove_dir_all("extracted_encrypted");
    let _ = fs::remove_dir_all("extracted_specific");
    let _ = fs::remove_file("test_archive.7z");
    let _ = fs::remove_file("test_encrypted.7z");
    let _ = fs::remove_file("file1.lzma2");
    let _ = fs::remove_file("file1_decompressed.txt");
    println!("✓ Cleanup complete\n");

    Ok(())
}
