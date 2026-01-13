// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED detection in archive files
//!
//! Scans ZIP archives for embedded UFED files (UFDR/UFDX/UFD),
//! including nested ZIP files.

use std::fs::File;
use std::io::Read;
use tracing::debug;

use super::types::UFED_EXTENSIONS;
use crate::containers::ContainerError;

/// Check if a filename has a UFED extension
pub fn is_ufed_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    UFED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Detect UFED files (UFDR/UFDX/UFD) inside a ZIP archive
/// 
/// Also checks nested ZIPs (one level deep) that might contain UFED files.
/// 
/// Returns: (detected, list of UFED file paths found)
pub fn detect_in_zip(path: &str) -> Result<(bool, Vec<String>), ContainerError> {
    let file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open ZIP: {e}")))?;
    
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut ufed_files: Vec<String> = Vec::new();
    let mut nested_zips: Vec<String> = Vec::new();
    
    // First pass: scan all entries in the archive
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            let lower_name = name.to_lowercase();
            
            // Check for UFED files
            if is_ufed_file(&lower_name) {
                debug!(path = %path, entry = %name, "Found UFED file in ZIP");
                ufed_files.push(name.clone());
            }
            
            // Track nested ZIP files for deeper inspection
            if lower_name.ends_with(".zip") {
                nested_zips.push(name);
            }
        }
    }
    
    // Second pass: check inside nested ZIPs (one level deep)
    for nested_zip_name in &nested_zips {
        if let Ok(nested_files) = scan_nested_zip(&mut archive, nested_zip_name) {
            for nested_file in nested_files {
                let full_path = format!("{}/{}", nested_zip_name, nested_file);
                debug!(path = %path, entry = %full_path, "Found UFED file in nested ZIP");
                ufed_files.push(full_path);
            }
        }
    }
    
    let detected = !ufed_files.is_empty();
    
    if detected {
        debug!(
            path = %path,
            count = ufed_files.len(),
            files = ?ufed_files,
            "UFED files detected in archive"
        );
    }
    
    Ok((detected, ufed_files))
}

/// Scan a nested ZIP inside the parent archive for UFED files
fn scan_nested_zip(
    parent_archive: &mut zip::ZipArchive<File>,
    nested_zip_name: &str,
) -> Result<Vec<String>, ContainerError> {
    use std::io::Cursor;
    
    let mut ufed_files: Vec<String> = Vec::new();
    
    // Extract the nested ZIP to memory
    let nested_data = {
        let mut entry = parent_archive.by_name(nested_zip_name)
            .map_err(|e| format!("Failed to read nested ZIP {}: {e}", nested_zip_name))?;
        
        // Limit nested ZIP size to prevent memory issues (100MB max)
        let size = entry.size();
        if size > 100 * 1024 * 1024 {
            debug!(nested_zip = %nested_zip_name, size = size, "Nested ZIP too large, skipping");
            return Ok(vec![]);
        }
        
        let mut data = Vec::with_capacity(size as usize);
        entry.read_to_end(&mut data)
            .map_err(|e| format!("Failed to extract nested ZIP: {e}"))?;
        data
    };
    
    // Parse the nested ZIP
    let cursor = Cursor::new(nested_data);
    let mut nested_archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => {
            debug!(nested_zip = %nested_zip_name, error = %e, "Failed to parse nested ZIP");
            return Ok(vec![]);
        }
    };
    
    // Scan nested archive entries
    for i in 0..nested_archive.len() {
        if let Ok(entry) = nested_archive.by_index(i) {
            let name = entry.name().to_string();
            if is_ufed_file(&name) {
                ufed_files.push(name);
            }
        }
    }
    
    Ok(ufed_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use zip::write::SimpleFileOptions;

    // ==================== is_ufed_file tests ====================

    #[test]
    fn test_is_ufed_file_ufdr() {
        assert!(is_ufed_file("extraction.ufdr"));
        assert!(is_ufed_file("EXTRACTION.UFDR"));
        assert!(is_ufed_file("path/to/file.ufdr"));
    }

    #[test]
    fn test_is_ufed_file_ufdx() {
        assert!(is_ufed_file("metadata.ufdx"));
        assert!(is_ufed_file("METADATA.UFDX"));
    }

    #[test]
    fn test_is_ufed_file_ufd() {
        assert!(is_ufed_file("extraction.ufd"));
        assert!(is_ufed_file("EXTRACTION.UFD"));
    }

    #[test]
    fn test_is_ufed_file_non_ufed() {
        assert!(!is_ufed_file("document.pdf"));
        assert!(!is_ufed_file("archive.zip"));
        assert!(!is_ufed_file("data.txt"));
        assert!(!is_ufed_file("image.png"));
    }

    #[test]
    fn test_is_ufed_file_partial_match() {
        // Should NOT match if extension is only partially similar
        assert!(!is_ufed_file("file.ufd.bak"));
        assert!(!is_ufed_file("file.ufdr.old"));
    }

    // ==================== detect_in_zip tests ====================

    #[test]
    fn test_detect_in_zip_nonexistent_file() {
        let result = detect_in_zip("/nonexistent/file.zip");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_in_zip_empty_zip() {
        // Create an empty ZIP file
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let zip = zip::ZipWriter::new(file);
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(!detected);
        assert!(files.is_empty());
    }

    #[test]
    fn test_detect_in_zip_with_ufdr() {
        // Create a ZIP file containing a .ufdr file
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = SimpleFileOptions::default();
        zip.start_file("extraction.ufdr", options).unwrap();
        zip.write_all(b"UFDR content").unwrap();
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(detected);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "extraction.ufdr");
    }

    #[test]
    fn test_detect_in_zip_with_multiple_ufed_files() {
        // Create a ZIP file with multiple UFED files
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = SimpleFileOptions::default();
        
        zip.start_file("data.ufdr", options).unwrap();
        zip.write_all(b"UFDR data").unwrap();
        
        zip.start_file("metadata.ufdx", options).unwrap();
        zip.write_all(b"UFDX metadata").unwrap();
        
        zip.start_file("readme.txt", options).unwrap();
        zip.write_all(b"Just a readme").unwrap();
        
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(detected);
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"data.ufdr".to_string()));
        assert!(files.contains(&"metadata.ufdx".to_string()));
    }

    #[test]
    fn test_detect_in_zip_no_ufed_files() {
        // Create a ZIP file without any UFED files
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = SimpleFileOptions::default();
        
        zip.start_file("document.pdf", options).unwrap();
        zip.write_all(b"PDF content").unwrap();
        
        zip.start_file("image.png", options).unwrap();
        zip.write_all(b"PNG data").unwrap();
        
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(!detected);
        assert!(files.is_empty());
    }

    #[test]
    fn test_detect_in_zip_ufed_in_subdirectory() {
        // Create a ZIP with UFED file in subdirectory
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = SimpleFileOptions::default();
        
        zip.start_file("folder/subfolder/data.ufdr", options).unwrap();
        zip.write_all(b"UFDR in subdirectory").unwrap();
        
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(detected);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "folder/subfolder/data.ufdr");
    }

    #[test]
    fn test_detect_in_zip_case_insensitive() {
        // Test case insensitivity of UFED extension detection
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        
        let options = SimpleFileOptions::default();
        
        zip.start_file("DATA.UFDR", options).unwrap();
        zip.write_all(b"Uppercase extension").unwrap();
        
        zip.start_file("metadata.UFDX", options).unwrap();
        zip.write_all(b"Mixed case extension").unwrap();
        
        zip.finish().unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let (detected, files) = result.unwrap();
        assert!(detected);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_detect_in_zip_invalid_zip() {
        // Create a file that's not a valid ZIP
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(b"This is not a ZIP file").unwrap();
        
        let result = detect_in_zip(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
    }
}
