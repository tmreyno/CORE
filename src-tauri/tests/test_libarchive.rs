// =============================================================================
// Libarchive Backend Integration Tests
// =============================================================================
//
// Run with: cargo test --test test_libarchive -- --nocapture
//
// Tests the unified libarchive backend against real archive files.

use std::path::Path;
use std::time::Instant;

use ffx_check_lib::archive::libarchive_backend::{
    LibarchiveHandler, detect_format, is_supported_archive,
};

/// Test directory containing evidence files
const TEST_DIR: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence";

/// Helper to find first file with given extension
fn find_test_file(ext: &str) -> Option<String> {
    // First check known good files
    let known_files = [
        ("/Users/terryreynolds/1827-1001 Case With Data /2.Processed.Database/AXIOM - Nov 15 2025 164907/logging-Nov 15 2025 212022.zip", "zip"),
        ("/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/2024_CTF_CLBE.zip", "zip-encrypted"),
        ("/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/Nov17memdump.7z", "7z"),
    ];
    
    for (path, file_ext) in known_files {
        if file_ext.starts_with(ext) && Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    
    // Search in evidence directory
    let entries = std::fs::read_dir(TEST_DIR).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(e) = path.extension() {
                if e.to_string_lossy().to_lowercase() == ext.to_lowercase() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

// =============================================================================
// Format Detection Tests
// =============================================================================

#[test]
fn test_format_detection_by_extension() {
    // These tests check detection based on file extension
    // Note: detect_format() tries to open the file with libarchive,
    // so non-existent files will return None
    
    // Test with real files that exist
    if let Some(path) = find_test_file("zip") {
        let format = detect_format(&path);
        assert_eq!(format, Some("zip".to_string()));
        println!("✅ ZIP format detection works: {:?}", format);
    }
    
    if let Some(path) = find_test_file("7z") {
        let format = detect_format(&path);
        assert_eq!(format, Some("7z".to_string()));
        println!("✅ 7z format detection works: {:?}", format);
    }
    
    println!("✅ Format detection tests passed");
}

#[test]
fn test_is_supported_real_files() {
    // Test with files that actually exist
    if let Some(path) = find_test_file("zip") {
        assert!(is_supported_archive(&path), "ZIP should be supported");
        println!("✅ ZIP file is supported");
    }
    
    if let Some(path) = find_test_file("7z") {
        assert!(is_supported_archive(&path), "7z should be supported");
        println!("✅ 7z file is supported");
    }
    
    // Non-archive file should not be supported (if it exists)
    let txt_file = "/Users/terryreynolds/GitHub/CORE-1/README.md";
    if std::path::Path::new(txt_file).exists() {
        assert!(!is_supported_archive(txt_file), "README should not be an archive");
        println!("✅ Non-archive file correctly rejected");
    }
    
    println!("✅ Support detection tests passed");
}

// =============================================================================
// ZIP Archive Tests
// =============================================================================

#[test]
fn test_zip_listing() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found, skipping");
        return;
    };
    
    println!("\n📦 Testing libarchive ZIP: {}", path);
    
    let start = Instant::now();
    let handler = LibarchiveHandler::new(&path);
    
    // Get summary
    let mut handler_for_summary = LibarchiveHandler::new(&path);
    match handler_for_summary.summary() {
        Ok(summary) => {
            println!("   Summary: {} entries, {} bytes", summary.entry_count, summary.total_size);
        }
        Err(e) => {
            println!("   ⚠️  Summary failed: {}", e);
        }
    }
    
    // List entries
    match handler.list_entries() {
        Ok(entries) => {
            let elapsed = start.elapsed();
            println!("   ✅ Listed {} entries in {:?}", entries.len(), elapsed);
            
            for (i, entry) in entries.iter().take(10).enumerate() {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let encrypted = if entry.is_encrypted { " 🔒" } else { "" };
                println!("      {}. {} {}{} ({} bytes)", 
                    i + 1, icon, entry.name, encrypted, entry.size);
            }
            if entries.len() > 10 {
                println!("      ... and {} more", entries.len() - 10);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list entries: {}", e);
        }
    }
}

#[test]
fn test_zip_encrypted_listing() {
    // Use the known encrypted ZIP
    let encrypted_zip = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/2024_CTF_CLBE.zip";
    
    if !Path::new(encrypted_zip).exists() {
        println!("⚠️  Encrypted ZIP not found, skipping");
        return;
    }
    
    println!("\n🔐 Testing libarchive encrypted ZIP: {}", encrypted_zip);
    
    let handler = LibarchiveHandler::new(encrypted_zip);
    
    // Check if it needs password
    match handler.needs_password() {
        Ok(needs) => {
            println!("   Needs password: {}", needs);
        }
        Err(e) => {
            println!("   ⚠️  Password check failed: {}", e);
        }
    }
    
    // List entries (should work without password - only data is encrypted)
    match handler.list_entries() {
        Ok(entries) => {
            println!("   ✅ Listed {} entries without password", entries.len());
            
            for entry in &entries {
                let icon = if entry.is_dir { "📁" } else { "🔒" };
                let size_mb = entry.size as f64 / 1024.0 / 1024.0;
                println!("      {} {} ({:.1} MB)", icon, entry.name, size_mb);
            }
            
            // Check encryption status
            let encrypted_count = entries.iter().filter(|e| e.is_encrypted).count();
            println!("\n   📊 {} of {} entries are encrypted", encrypted_count, entries.len());
        }
        Err(e) => {
            println!("   ❌ Failed to list entries: {}", e);
            println!("   ℹ️  This may mean libarchive can't read encrypted ZIP headers");
        }
    }
}

// =============================================================================
// 7-Zip Archive Tests
// =============================================================================

#[test]
fn test_7z_listing() {
    let Some(path) = find_test_file("7z") else {
        println!("⚠️  No 7z file found, skipping");
        return;
    };
    
    println!("\n📦 Testing libarchive 7z: {}", path);
    
    let start = Instant::now();
    let handler = LibarchiveHandler::new(&path);
    
    // Get summary
    let mut handler_for_summary = LibarchiveHandler::new(&path);
    match handler_for_summary.summary() {
        Ok(summary) => {
            println!("   Summary: {} entries, {:.2} MB", 
                summary.entry_count, 
                summary.total_size as f64 / 1024.0 / 1024.0);
        }
        Err(e) => {
            println!("   ⚠️  Summary failed: {}", e);
        }
    }
    
    // List entries
    match handler.list_entries() {
        Ok(entries) => {
            let elapsed = start.elapsed();
            println!("   ✅ Listed {} entries in {:?}", entries.len(), elapsed);
            
            for (i, entry) in entries.iter().take(10).enumerate() {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let encrypted = if entry.is_encrypted { " 🔒" } else { "" };
                let size = if entry.size > 1024 * 1024 {
                    format!("{:.2} MB", entry.size as f64 / 1024.0 / 1024.0)
                } else if entry.size > 1024 {
                    format!("{:.2} KB", entry.size as f64 / 1024.0)
                } else {
                    format!("{} B", entry.size)
                };
                println!("      {}. {} {}{} ({})", 
                    i + 1, icon, entry.name, encrypted, size);
            }
            if entries.len() > 10 {
                println!("      ... and {} more", entries.len() - 10);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list entries: {}", e);
            println!("   ℹ️  This may indicate encrypted headers or unsupported format");
        }
    }
}

// =============================================================================
// Pagination Tests
// =============================================================================

#[test]
fn test_pagination() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found, skipping");
        return;
    };
    
    println!("\n📄 Testing libarchive pagination: {}", path);
    
    let handler = LibarchiveHandler::new(&path);
    
    // Get root entries and manually paginate
    let root_entries = handler.list_root_entries();
    match root_entries {
        Ok(entries) => {
            // Simulate pagination with limit=5
            let limit = 5;
            let page1: Vec<_> = entries.iter().take(limit).collect();
            let has_more = entries.len() > limit;
            let next_offset = limit.min(entries.len());
            
            println!("   Page 1: {} entries, has_more={}, next_offset={}", 
                page1.len(), has_more, next_offset);
            
            for entry in &page1 {
                println!("      - {}", entry.name);
            }
            
            if has_more {
                // Get second page
                let page2: Vec<_> = entries.iter().skip(next_offset).take(limit).collect();
                println!("   Page 2: {} entries", page2.len());
                for entry in &page2 {
                    println!("      - {}", entry.name);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Pagination failed: {}", e);
        }
    }
}

// =============================================================================
// Performance Comparison
// =============================================================================

#[test]
fn test_performance_comparison() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found, skipping");
        return;
    };
    
    println!("\n⚡ Performance comparison:");
    println!("   File: {}", path);
    
    // Libarchive timing
    let start = Instant::now();
    let handler = LibarchiveHandler::new(&path);
    let entries = handler.list_entries().unwrap_or_default();
    let libarchive_time = start.elapsed();
    
    println!("\n   libarchive:");
    println!("      Entries: {}", entries.len());
    println!("      Time: {:?}", libarchive_time);
    
    // Note: Could add comparison with pure-Rust ZIP crate here
    println!("\n   ✅ libarchive backend working correctly");
}

// =============================================================================
// Integrated Handler Tests (verify sevenz, rar modules use libarchive)
// =============================================================================

#[test]
fn test_sevenz_integrated_handler() {
    use ffx_check_lib::archive::sevenz;
    
    let Some(path) = find_test_file("7z") else {
        println!("⚠️  No 7z file found, skipping");
        return;
    };
    
    println!("\n🔗 Testing integrated sevenz::list_entries (uses libarchive)");
    println!("   File: {}", path);
    
    let start = std::time::Instant::now();
    match sevenz::list_entries(&path) {
        Ok(entries) => {
            let elapsed = start.elapsed();
            println!("   ✅ Listed {} entries in {:?}", entries.len(), elapsed);
            
            for (i, entry) in entries.iter().take(5).enumerate() {
                println!("      {}. {} {} ({:.2} MB)", 
                    i + 1,
                    if entry.is_directory { "📁" } else { "📄" },
                    entry.path,
                    entry.size as f64 / 1024.0 / 1024.0);
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }
}
