// =============================================================================
// Unified Container Handler Integration Tests
// =============================================================================
//
// Run with: cargo test --test test_unified_containers -- --nocapture
//
// These tests exercise the unified container API against real evidence files.

use std::path::Path;
use std::time::Instant;

// Note: We test against the library crate
use ffx_check_lib::containers::unified::{
    ContainerType, get_summary, get_root_children, get_children, get_handler_for_path
};

/// Test directory containing evidence files
const TEST_DIR: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence";

/// Helper to check if a file exists
#[allow(dead_code)]
fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if a ZIP file has valid EOCD (not truncated)
fn is_valid_zip(path: &str) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("   Failed to open {}: {}", path, e);
            return false;
        }
    };
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            eprintln!("   Failed to get metadata: {}", e);
            return false;
        }
    };
    
    // Search last 65KB for EOCD signature  
    let search_size = file_size.min(65557) as usize;
    let mut buf = vec![0u8; search_size];
    
    // Seek to position for last 65KB
    let seek_pos = file_size.saturating_sub(search_size as u64);
    if file.seek(SeekFrom::Start(seek_pos)).is_err() {
        eprintln!("   Failed to seek in {}", path);
        return false;
    }
    if file.read_exact(&mut buf).is_err() {
        eprintln!("   Failed to read {} bytes from {}", search_size, path);
        return false;
    }
    
    // Find EOCD signature (PK\x05\x06)
    let eocd_sig = [0x50, 0x4B, 0x05, 0x06];
    let found = (0..buf.len().saturating_sub(4))
        .rev()
        .any(|i| buf[i..i + 4] == eocd_sig);
    
    if !found {
        eprintln!("   No EOCD found in last {}KB of {}", search_size / 1024, 
            std::path::Path::new(path).file_name().unwrap().to_string_lossy());
    }
    found
}

/// Helper to find first file matching extension in test dir
fn find_test_file(ext: &str) -> Option<String> {
    // First check the processed database folder for smaller test files
    let alt_paths = [
        "/Users/terryreynolds/1827-1001 Case With Data /2.Processed.Database/AXIOM - Nov 15 2025 164907/logging-Nov 15 2025 212022.zip",
    ];
    
    if ext.to_lowercase() == "zip" {
        for path in alt_paths {
            if std::path::Path::new(path).exists() && is_valid_zip(path) {
                return Some(path.to_string());
            }
        }
    }
    
    let entries = std::fs::read_dir(TEST_DIR).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(e) = path.extension() {
                if e.to_string_lossy().to_lowercase() == ext.to_lowercase() {
                    let path_str = path.to_string_lossy().to_string();
                    // For ZIPs, verify they have valid EOCD
                    if ext.to_lowercase() == "zip" && !is_valid_zip(&path_str) {
                        println!("   ⚠️  Skipping truncated ZIP: {}", path.file_name().unwrap().to_string_lossy());
                        continue;
                    }
                    return Some(path_str);
                }
            }
        }
    }
    None
}

// =============================================================================
// Container Type Detection Tests
// =============================================================================

#[test]
fn test_type_detection_ad1() {
    assert_eq!(ContainerType::detect("test.ad1"), ContainerType::Ad1);
    assert_eq!(ContainerType::detect("test.AD1"), ContainerType::Ad1);
    assert_eq!(ContainerType::detect("/path/to/Evidence.ad1"), ContainerType::Ad1);
}

#[test]
fn test_type_detection_ewf() {
    assert_eq!(ContainerType::detect("test.e01"), ContainerType::Ewf);
    assert_eq!(ContainerType::detect("test.E01"), ContainerType::Ewf);
    assert_eq!(ContainerType::detect("test.l01"), ContainerType::Ewf);
    assert_eq!(ContainerType::detect("test.ex01"), ContainerType::Ewf);
    assert_eq!(ContainerType::detect("test.lx01"), ContainerType::Ewf);
}

#[test]
fn test_type_detection_zip() {
    assert_eq!(ContainerType::detect("test.zip"), ContainerType::Zip);
    assert_eq!(ContainerType::detect("test.ZIP"), ContainerType::Zip);
}

#[test]
fn test_type_detection_7z() {
    assert_eq!(ContainerType::detect("test.7z"), ContainerType::SevenZip);
    assert_eq!(ContainerType::detect("test.7z.001"), ContainerType::SevenZip);
}

#[test]
fn test_type_detection_tar() {
    assert_eq!(ContainerType::detect("test.tar"), ContainerType::Tar);
    assert_eq!(ContainerType::detect("test.tar.gz"), ContainerType::Tar);
    assert_eq!(ContainerType::detect("test.tgz"), ContainerType::Tar);
    assert_eq!(ContainerType::detect("test.tar.bz2"), ContainerType::Tar);
}

#[test]
fn test_type_detection_rar() {
    assert_eq!(ContainerType::detect("test.rar"), ContainerType::Rar);
    assert_eq!(ContainerType::detect("test.RAR"), ContainerType::Rar);
}

#[test]
fn test_type_detection_raw() {
    assert_eq!(ContainerType::detect("test.dd"), ContainerType::Raw);
    assert_eq!(ContainerType::detect("test.raw"), ContainerType::Raw);
    assert_eq!(ContainerType::detect("test.img"), ContainerType::Raw);
    assert_eq!(ContainerType::detect("test.001"), ContainerType::Raw);
}

#[test]
fn test_type_detection_memory() {
    // Standard memory dump naming patterns
    assert_eq!(ContainerType::detect("02606-0900_1E_GRCDH2_mem.raw.001"), ContainerType::Memory);
    assert_eq!(ContainerType::detect("/path/to/device_mem.raw"), ContainerType::Memory);
    assert_eq!(ContainerType::detect("capture.vmem"), ContainerType::Memory);
    assert_eq!(ContainerType::detect("MEMORY.DMP"), ContainerType::Memory);
    assert_eq!(ContainerType::detect("system_memdump.bin"), ContainerType::Memory);
    
    // These should NOT be memory dumps - should be raw images
    assert_eq!(ContainerType::detect("disk_image.raw"), ContainerType::Raw);
    assert_eq!(ContainerType::detect("backup.dd"), ContainerType::Raw);
}

// =============================================================================
// E01 Container Tests
// =============================================================================

#[test]
fn test_e01_summary() {
    let Some(path) = find_test_file("e01") else {
        println!("⚠️  No E01 file found in test directory, skipping");
        return;
    };
    
    println!("\n📦 Testing E01: {}", path);
    let start = Instant::now();
    
    let result = get_summary(&path);
    println!("   Summary took: {:?}", start.elapsed());
    
    match result {
        Ok(summary) => {
            println!("   ✅ Container Type: {}", summary.container_type);
            println!("   ✅ Total Size: {} bytes", summary.total_size);
            println!("   ✅ Entry Count: {}", summary.entry_count);
            println!("   ✅ Lazy Loading Recommended: {}", summary.lazy_loading_recommended);
            assert_eq!(summary.container_type, "ewf");
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

#[test]
fn test_e01_root_children() {
    let Some(path) = find_test_file("e01") else {
        println!("⚠️  No E01 file found, skipping");
        return;
    };
    
    println!("\n📦 Testing E01 root children: {}", path);
    let start = Instant::now();
    
    // E01 requires VFS mounting - this tests the proper error message
    let result = get_root_children(&path, Some(0), Some(10));
    println!("   Root children took: {:?}", start.elapsed());
    
    match result {
        Ok(children) => {
            println!("   ✅ Loaded {} entries (total: {})", children.entries.len(), children.total_count);
            for (i, entry) in children.entries.iter().take(5).enumerate() {
                println!("      {}. {} ({})", i + 1, entry.name, if entry.is_dir { "dir" } else { "file" });
            }
            if children.has_more {
                println!("      ... and {} more", children.total_count - children.entries.len());
            }
        }
        Err(e) => {
            // E01 may require VFS mounting first - this is expected
            println!("   ⚠️  E01 note: {:?}", e);
        }
    }
}

// =============================================================================
// ZIP Container Tests
// =============================================================================

#[test]
fn test_zip_summary() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found in test directory, skipping");
        return;
    };
    
    println!("\n📦 Testing ZIP: {}", path);
    let start = Instant::now();
    
    let result = get_summary(&path);
    println!("   Summary took: {:?}", start.elapsed());
    
    match result {
        Ok(summary) => {
            println!("   ✅ Container Type: {}", summary.container_type);
            println!("   ✅ Total Size: {} bytes ({:.2} MB)", summary.total_size, summary.total_size as f64 / 1_000_000.0);
            println!("   ✅ Entry Count: {}", summary.entry_count);
            println!("   ✅ Lazy Loading Recommended: {}", summary.lazy_loading_recommended);
            assert_eq!(summary.container_type, "zip");
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

#[test]
fn test_zip_root_children() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found, skipping");
        return;
    };
    
    println!("\n📦 Testing ZIP root children: {}", path);
    let start = Instant::now();
    
    let result = get_root_children(&path, Some(0), Some(20));
    println!("   Root children took: {:?}", start.elapsed());
    
    match result {
        Ok(children) => {
            println!("   ✅ Loaded {} entries (total: {})", children.entries.len(), children.total_count);
            for (i, entry) in children.entries.iter().take(10).enumerate() {
                let size_str = if entry.is_dir { "dir".to_string() } else { format!("{} bytes", entry.size) };
                println!("      {}. {} ({})", i + 1, entry.name, size_str);
            }
            if children.has_more {
                println!("      ... and {} more", children.total_count - children.entries.len());
            }
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

#[test]
fn test_zip_pagination() {
    let Some(path) = find_test_file("zip") else {
        println!("⚠️  No ZIP file found, skipping");
        return;
    };
    
    println!("\n📦 Testing ZIP pagination: {}", path);
    
    // Load first page
    let page1 = get_root_children(&path, Some(0), Some(5));
    let Ok(p1) = page1 else {
        println!("   ❌ Failed to load page 1");
        return;
    };
    
    println!("   Page 1: {} entries, has_more={}", p1.entries.len(), p1.has_more);
    
    if p1.has_more {
        // Load second page
        let page2 = get_root_children(&path, Some(5), Some(5));
        let Ok(p2) = page2 else {
            println!("   ❌ Failed to load page 2");
            return;
        };
        println!("   Page 2: {} entries, has_more={}", p2.entries.len(), p2.has_more);
        
        // Verify different entries
        if !p1.entries.is_empty() && !p2.entries.is_empty() {
            assert_ne!(p1.entries[0].name, p2.entries[0].name, "Pages should have different entries");
            println!("   ✅ Pagination working correctly");
        }
    } else {
        println!("   ℹ️  Only one page of results");
    }
}

// =============================================================================
// Password-Protected ZIP Tests
// =============================================================================

/// Test: Password-protected ZIPs should list their contents without a password
/// (only file DATA is encrypted, not the Central Directory)
#[test]
fn test_encrypted_zip_listing() {
    // This is a known password-protected ZIP in the evidence folder
    let encrypted_zip = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/2024_CTF_CLBE.zip";
    
    if !std::path::Path::new(encrypted_zip).exists() {
        println!("⚠️  Encrypted ZIP not found, skipping");
        return;
    }
    
    println!("\n🔐 Testing encrypted ZIP: {}", encrypted_zip);
    
    // 1. Verify it's detected as ZIP
    let handler = get_handler_for_path(encrypted_zip);
    println!("   Handler: {:?}", handler.container_type());
    assert!(matches!(handler.container_type(), ContainerType::Zip));
    
    // 2. Get summary - should work without password
    let summary = get_summary(encrypted_zip);
    match &summary {
        Ok(s) => {
            println!("   ✅ Summary: type={}, {} entries", s.container_type, s.entry_count);
            assert!(s.entry_count > 0, "Should have entries listed");
        },
        Err(e) => {
            panic!("   ❌ Failed to get summary of encrypted ZIP: {}", e);
        }
    }
    
    // 3. List root children - should show the folder
    let start = std::time::Instant::now();
    let result = get_root_children(encrypted_zip, None, None);
    let elapsed = start.elapsed();
    
    match result {
        Ok(page) => {
            println!("   ✅ Listed {} root entries in {:?}", page.entries.len(), elapsed);
            assert!(!page.entries.is_empty(), "Should list encrypted ZIP contents");
            
            for (i, entry) in page.entries.iter().enumerate() {
                let entry_type = if entry.is_dir { "📁" } else { "📄" };
                println!("      {}. {} {} ({} bytes)", 
                    i + 1, 
                    entry_type,
                    entry.name, 
                    entry.size);
                
                // If this is a folder, list its children (the encrypted files)
                if entry.is_dir {
                    let children_result = get_children(encrypted_zip, &entry.id, None, None);
                    if let Ok(children) = children_result {
                        println!("         └─ {} encrypted files:", children.entries.len());
                        for child in &children.entries {
                            let child_type = if child.is_dir { "📁" } else { "🔒" };
                            let size_mb = child.size as f64 / 1024.0 / 1024.0;
                            println!("            {} {} ({:.1} MB)", 
                                child_type,
                                child.name, 
                                size_mb);
                        }
                    }
                }
            }
            
            println!("\n   ✅ Password-protected ZIP directory listing works!");
            println!("   ℹ️  Files can be listed but data requires password to extract");
        },
        Err(e) => {
            panic!("   ❌ Failed to list encrypted ZIP: {}", e);
        }
    }
}

// =============================================================================
// 7z Container Tests
// =============================================================================

#[test]
fn test_7z_summary() {
    let Some(path) = find_test_file("7z") else {
        println!("⚠️  No 7z file found in test directory, skipping");
        return;
    };
    
    println!("\n📦 Testing 7z: {}", path);
    let start = Instant::now();
    
    let result = get_summary(&path);
    println!("   Summary took: {:?}", start.elapsed());
    
    match result {
        Ok(summary) => {
            println!("   ✅ Container Type: {}", summary.container_type);
            println!("   ✅ Total Size: {} bytes ({:.2} MB)", summary.total_size, summary.total_size as f64 / 1_000_000.0);
            println!("   ✅ Entry Count: {}", summary.entry_count);
            println!("   ✅ Lazy Loading Recommended: {}", summary.lazy_loading_recommended);
            assert_eq!(summary.container_type, "7z");
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

#[test]
fn test_7z_root_children() {
    let Some(path) = find_test_file("7z") else {
        println!("⚠️  No 7z file found, skipping");
        return;
    };
    
    println!("\n📦 Testing 7z root children: {}", path);
    let start = Instant::now();
    
    let result = get_root_children(&path, Some(0), Some(20));
    println!("   Root children took: {:?}", start.elapsed());
    
    match result {
        Ok(children) => {
            println!("   ✅ Loaded {} entries (total: {})", children.entries.len(), children.total_count);
            for (i, entry) in children.entries.iter().take(10).enumerate() {
                let size_str = if entry.is_dir { "dir".to_string() } else { format!("{} bytes", entry.size) };
                println!("      {}. {} ({})", i + 1, entry.name, size_str);
            }
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

// =============================================================================
// TAR Container Tests
// =============================================================================

#[test]
fn test_tar_summary() {
    let Some(path) = find_test_file("tar") else {
        println!("⚠️  No TAR file found in test directory, skipping");
        return;
    };
    
    println!("\n📦 Testing TAR: {}", path);
    let start = Instant::now();
    
    let result = get_summary(&path);
    println!("   Summary took: {:?}", start.elapsed());
    
    match result {
        Ok(summary) => {
            println!("   ✅ Container Type: {}", summary.container_type);
            println!("   ✅ Total Size: {} bytes ({:.2} GB)", summary.total_size, summary.total_size as f64 / 1_000_000_000.0);
            println!("   ✅ Entry Count: {}", summary.entry_count);
            println!("   ✅ Lazy Loading Recommended: {}", summary.lazy_loading_recommended);
            assert_eq!(summary.container_type, "tar");
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

#[test]
fn test_tar_root_children() {
    let Some(path) = find_test_file("tar") else {
        println!("⚠️  No TAR file found, skipping");
        return;
    };
    
    println!("\n📦 Testing TAR root children: {}", path);
    let start = Instant::now();
    
    let result = get_root_children(&path, Some(0), Some(20));
    println!("   Root children took: {:?}", start.elapsed());
    
    match result {
        Ok(children) => {
            println!("   ✅ Loaded {} entries (total: {})", children.entries.len(), children.total_count);
            for (i, entry) in children.entries.iter().take(10).enumerate() {
                let size_str = if entry.is_dir { "dir".to_string() } else { format!("{} bytes", entry.size) };
                println!("      {}. {} ({})", i + 1, entry.name, size_str);
            }
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

// =============================================================================
// RAR Container Tests
// =============================================================================

#[test]
fn test_rar_summary() {
    let Some(path) = find_test_file("rar") else {
        println!("⚠️  No RAR file found in test directory, skipping");
        return;
    };
    
    println!("\n📦 Testing RAR: {}", path);
    let start = Instant::now();
    
    let result = get_summary(&path);
    println!("   Summary took: {:?}", start.elapsed());
    
    match result {
        Ok(summary) => {
            println!("   ✅ Container Type: {}", summary.container_type);
            println!("   ✅ Total Size: {} bytes", summary.total_size);
            println!("   ✅ Entry Count: {}", summary.entry_count);
            println!("   ✅ Lazy Loading Recommended: {}", summary.lazy_loading_recommended);
            assert_eq!(summary.container_type, "rar");
        }
        Err(e) => {
            println!("   ❌ Error: {:?}", e);
        }
    }
}

// =============================================================================
// Performance Benchmark
// =============================================================================

#[test]
fn test_unified_performance() {
    println!("\n⏱️  Performance Benchmark");
    println!("   ========================");
    
    let extensions = ["e01", "zip", "7z", "tar", "rar"];
    
    for ext in extensions {
        if let Some(path) = find_test_file(ext) {
            let filename = Path::new(&path).file_name().unwrap().to_string_lossy();
            
            // Benchmark summary
            let start = Instant::now();
            let _summary_result = get_summary(&path);
            let summary_time = start.elapsed();
            
            // Benchmark root children
            let start = Instant::now();
            let children_result = get_root_children(&path, Some(0), Some(100));
            let children_time = start.elapsed();
            
            print!("   {} ({}) - Summary: {:?}", filename, ext.to_uppercase(), summary_time);
            
            if let Ok(children) = children_result {
                println!(", Root: {:?} ({} entries)", children_time, children.entries.len());
            } else {
                println!(", Root: {:?} (requires VFS)", children_time);
            }
        }
    }
}
