// =============================================================================
// Document Module Integration Tests
// =============================================================================
//
// Run with: cargo test --test test_document_formats -- --nocapture

use std::path::Path;

// Test files from the case data folder
const TEST_TEXT: &str = "/Users/terryreynolds/1827-1001 Case With Data /2.Processed.Database/24-042.0854-1000.CAA.Performance.Calibration/Case Information.txt";
const TEST_XML: &str = "/Users/terryreynolds/1827-1001 Case With Data /2.Processed.Database/24-042.0854-1000.CAA.Performance.Calibration/Case Information.xml";
const TEST_PDF: &str = "/Users/terryreynolds/1827-1001 Case With Data /3.Exports.Results/1.Export.For.Review/25-053 08282-1003 (NESHAP - ABC Kid City)/RP reviewed case-specific PAC/case-specific PACP/Dependencies/documentation/ref-artifacts/Content/Resources/PDFs/Artifact Reference.pdf";
const TEST_HTML: &str = "/Users/terryreynolds/1827-1001 Case With Data /3.Exports.Results/1.Export.For.Review/25-053 08282-1003 (NESHAP - ABC Kid City)/RP reviewed case-specific PAC/case-specific PACP/Dependencies/bin/libvlc/win-x86/lua/http/index.html";
const TEST_PNG: &str = "/Users/terryreynolds/1827-1001 Case With Data /3.Exports.Results/1.Export.For.Review/25-053 08282-1003 (NESHAP - ABC Kid City)/RP reviewed case-specific PAC/case-specific PACP/Dependencies/documentation/portable-case/help-portable-case-en-us/Skins/Default/Stylesheets/Images/collapse.png";

// Project files
const TEST_MD: &str = "/Users/terryreynolds/GitHub/CORE-1/README.md";
const TEST_JSON: &str = "/Users/terryreynolds/GitHub/CORE-1/package.json";
const TEST_TOML: &str = "/Users/terryreynolds/GitHub/CORE-1/src-tauri/Cargo.toml";

#[test]
fn test_universal_format_detection() {
    use ffx_check_lib::viewer::document::universal::UniversalFormat;
    
    println!("\n=== Testing UniversalFormat Detection ===\n");
    
    let tests = vec![
        (TEST_TEXT, "Text"),
        (TEST_XML, "Xml"),
        (TEST_PDF, "Pdf"),
        (TEST_HTML, "Html"),
        (TEST_PNG, "Png"),
        (TEST_MD, "Markdown"),
        (TEST_JSON, "Json"),
        (TEST_TOML, "Text"),
    ];
    
    for (path, expected) in tests {
        let p = Path::new(path);
        if !p.exists() {
            println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
            continue;
        }
        
        let format = UniversalFormat::from_path(p);
        let format_str = format!("{:?}", format);
        
        if format_str.contains(expected) {
            println!("✅ PASS: {} -> {:?}", p.file_name().unwrap().to_string_lossy(), format);
        } else {
            println!("❌ FAIL: {} -> {:?} (expected {})", 
                     p.file_name().unwrap().to_string_lossy(), format, expected);
        }
        
        assert!(format.is_some(), "Format should be detected for {}", path);
    }
}

#[test]
fn test_viewer_hints() {
    use ffx_check_lib::viewer::document::universal::{UniversalFormat, ViewerType};
    
    println!("\n=== Testing Viewer Type Hints ===\n");
    
    let tests = vec![
        (TEST_PNG, ViewerType::Image),
        (TEST_PDF, ViewerType::Pdf),
        (TEST_HTML, ViewerType::Html),
        (TEST_TEXT, ViewerType::Text),
        (TEST_MD, ViewerType::Text),
        (TEST_JSON, ViewerType::Text),
    ];
    
    for (path, expected_viewer) in tests {
        let p = Path::new(path);
        if !p.exists() {
            println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
            continue;
        }
        
        let format = UniversalFormat::from_path(p).unwrap();
        let viewer = format.viewer_type();
        
        if viewer == expected_viewer {
            println!("✅ PASS: {} -> {:?}", p.file_name().unwrap().to_string_lossy(), viewer);
        } else {
            println!("❌ FAIL: {} -> {:?} (expected {:?})", 
                     p.file_name().unwrap().to_string_lossy(), viewer, expected_viewer);
        }
        
        assert_eq!(viewer, expected_viewer);
    }
}

#[test]
fn test_file_info() {
    use ffx_check_lib::viewer::document::universal::FileInfo;
    
    println!("\n=== Testing FileInfo Extraction ===\n");
    
    let test_files = vec![
        TEST_TEXT,
        TEST_XML,
        TEST_PDF,
        TEST_HTML,
        TEST_PNG,
        TEST_MD,
        TEST_JSON,
    ];
    
    for path in test_files {
        let p = Path::new(path);
        if !p.exists() {
            println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
            continue;
        }
        
        match FileInfo::from_path(p) {
            Ok(info) => {
                println!("✅ PASS: {}", p.file_name().unwrap().to_string_lossy());
                println!("   Format: {:?}", info.format);
                println!("   MIME: {}", info.mime_type);
                println!("   Size: {} bytes", info.size);
                println!("   Is Binary: {}", info.is_binary);
                println!("   Viewer Type: {:?}", info.viewer_type);
                println!();
            }
            Err(e) => {
                println!("❌ FAIL: {} - {}", p.file_name().unwrap().to_string_lossy(), e);
            }
        }
    }
}

#[test]
fn test_read_as_text() {
    use ffx_check_lib::viewer::document::universal::read_as_text;
    
    println!("\n=== Testing Text Reading (first 500 chars) ===\n");
    
    let test_files = vec![
        (TEST_TEXT, true),
        (TEST_HTML, true),
        (TEST_MD, true),
        (TEST_JSON, true),
        (TEST_PNG, false), // Binary, should fail or return garbage
    ];
    
    for (path, should_succeed) in test_files {
        let p = Path::new(path);
        if !p.exists() {
            println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
            continue;
        }
        
        match read_as_text(p, 500) {
            Ok((text, truncated)) => {
                if should_succeed {
                    println!("✅ PASS: {} ({} chars, truncated: {})", 
                             p.file_name().unwrap().to_string_lossy(), 
                             text.len(),
                             truncated);
                    println!("   Preview: {}...", &text[..text.len().min(100)].replace('\n', "\\n"));
                    println!();
                } else {
                    println!("⚠️  WARN: {} returned text but expected binary", 
                             p.file_name().unwrap().to_string_lossy());
                }
            }
            Err(e) => {
                if !should_succeed {
                    println!("✅ PASS: {} correctly failed for binary file", 
                             p.file_name().unwrap().to_string_lossy());
                } else {
                    println!("❌ FAIL: {} - {}", p.file_name().unwrap().to_string_lossy(), e);
                }
            }
        }
    }
}

#[test]
fn test_image_dimensions() {
    use ffx_check_lib::viewer::document::universal::get_image_dimensions;
    
    println!("\n=== Testing Image Dimension Reading ===\n");
    
    let p = Path::new(TEST_PNG);
    if !p.exists() {
        println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
        return;
    }
    
    match get_image_dimensions(p) {
        Ok(dims) => {
            println!("✅ PASS: {} -> {}x{} pixels", 
                     p.file_name().unwrap().to_string_lossy(),
                     dims.width, dims.height);
            assert!(dims.width > 0 && dims.height > 0);
        }
        Err(e) => {
            println!("❌ FAIL: {} - {}", p.file_name().unwrap().to_string_lossy(), e);
            panic!("Failed to get image dimensions");
        }
    }
}

#[test]
fn test_document_service_metadata() {
    use ffx_check_lib::viewer::document::DocumentService;
    
    println!("\n=== Testing DocumentService Metadata ===\n");
    
    let service = DocumentService::new();
    
    let test_files = vec![
        TEST_PDF,
        TEST_HTML,
        TEST_MD,
    ];
    
    for path in test_files {
        let p = Path::new(path);
        if !p.exists() {
            println!("⏭️  SKIP: {} (not found)", p.file_name().unwrap().to_string_lossy());
            continue;
        }
        
        match service.get_metadata(p) {
            Ok(meta) => {
                println!("✅ PASS: {}", p.file_name().unwrap().to_string_lossy());
                println!("   Format: {:?}", meta.format);
                println!("   Title: {:?}", meta.title);
                println!("   Author: {:?}", meta.author);
                println!("   Page Count: {:?}", meta.page_count);
                println!("   File Size: {} bytes", meta.file_size);
                println!();
            }
            Err(e) => {
                println!("⚠️  WARN: {} - {} (may not support this format)", 
                         p.file_name().unwrap().to_string_lossy(), e);
            }
        }
    }
}
