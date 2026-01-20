// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! RAR archive format parsing
//!
//! Provides file listing for RAR archives (RAR4 and RAR5 formats).
//!
//! ## RAR Format Signatures
//! - RAR4: 52 61 72 21 1A 07 00 ("Rar!\x1a\x07\x00")
//! - RAR5: 52 61 72 21 1A 07 01 00 ("Rar!\x1a\x07\x01\x00")
//!
//! ## Requirements
//! This module uses the `unrar` crate which requires the unrar library to be
//! installed on the system. On macOS: `brew install unrar`, on Linux: `apt install unrar`
//!
//! ## Forensic Considerations
//! - RAR archives may have encrypted filenames (headers)
//! - Password-protected archives can still list files if headers aren't encrypted
//! - Multi-volume archives (.part1.rar, .part2.rar) require all parts

use std::path::Path;
use tracing::debug;

use super::extraction::ArchiveEntry;
use crate::containers::ContainerError;

// =============================================================================
// RAR Archive Support
// =============================================================================

/// List all entries in a RAR archive
/// 
/// Uses libarchive as primary backend with unrar crate as fallback.
/// Handles both RAR4 and RAR5 formats automatically.
pub fn list_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    debug!(path = %path, "Listing RAR archive entries");
    
    // Verify file exists
    if !Path::new(path).exists() {
        return Err(ContainerError::FileNotFound(format!("RAR archive not found: {}", path)));
    }
    
    // Try libarchive first (better licensing, no system dependency)
    match list_entries_libarchive(path) {
        Ok(entries) => {
            debug!(path = %path, entries = entries.len(), "RAR listing complete (libarchive)");
            return Ok(entries);
        }
        Err(e) => {
            debug!(path = %path, error = %e, "libarchive failed, falling back to unrar");
        }
    }
    
    // Fallback to unrar crate
    list_entries_unrar(path)
}

/// List entries using libarchive (preferred - BSD licensed, no system deps)
fn list_entries_libarchive(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    super::libarchive_backend::list_entries_as_archive_entry(path, "RAR")
}

/// List entries using unrar crate (fallback)
fn list_entries_unrar(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use unrar::Archive;
    
    // Open archive for listing (not extraction)
    let archive = Archive::new(path)
        .open_for_listing()
        .map_err(|e| format!("Failed to open RAR archive: {:?}", e))?;
    
    let mut entries = Vec::new();
    
    for (index, entry_result) in archive.enumerate() {
        let header = match entry_result {
            Ok(e) => e,
            Err(e) => {
                debug!(index, error = ?e, "Failed to read RAR entry");
                continue;
            }
        };
        
        let entry_path = header.filename.to_string_lossy().to_string();
        
        // Get file name from path
        let _name = Path::new(&entry_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| entry_path.clone());
        
        // Format modification time if available
        let last_modified = format_rar_time(&header);
        
        entries.push(ArchiveEntry {
            index,
            path: entry_path,
            is_directory: header.is_directory(),
            size: header.unpacked_size as u64,
            compressed_size: header.unpacked_size as u64, // RAR doesn't expose packed size easily
            crc32: 0, // CRC not directly exposed
            compression_method: "RAR".to_string(),
            last_modified,
        });
        
        // Safety limit
        if entries.len() >= 100_000 {
            debug!(path = %path, "RAR entry limit reached (100,000)");
            break;
        }
    }
    
    debug!(path = %path, entries = entries.len(), "RAR listing complete (unrar)");
    Ok(entries)
}

/// Check if a RAR archive has encrypted headers (filenames hidden)
pub fn has_encrypted_headers(path: &str) -> Result<bool, ContainerError> {
    use unrar::Archive;
    
    // Try to open for listing - if headers are encrypted, this will fail
    match Archive::new(path).open_for_listing() {
        Ok(_) => Ok(false),
        Err(e) => {
            // Check if error indicates encrypted headers
            let error_str = format!("{:?}", e);
            if error_str.contains("encrypted") || error_str.contains("password") {
                Ok(true)
            } else {
                Err(format!("Failed to check RAR encryption: {:?}", e).into())
            }
        }
    }
}

/// Check if file is a RAR archive by extension
pub fn is_rar_archive(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".rar") || 
    lower.contains(".part") && lower.ends_with(".rar") ||
    lower.ends_with(".r00") ||
    lower.ends_with(".r01")
}

/// Format RAR modification time to string
fn format_rar_time(header: &unrar::FileHeader) -> String {
    // unrar provides file_time as a DOS timestamp
    // Convert to readable format
    let ft = header.file_time;
    if ft == 0 {
        return String::new();
    }
    
    // DOS time format:
    // Bits 0-4: seconds/2 (0-29)
    // Bits 5-10: minutes (0-59)
    // Bits 11-15: hours (0-23)
    // DOS date format:
    // Bits 0-4: day (1-31)
    // Bits 5-8: month (1-12)
    // Bits 9-15: year-1980
    
    let time_part = ft & 0xFFFF;
    let date_part = (ft >> 16) & 0xFFFF;
    
    let second = ((time_part & 0x1F) * 2) as u8;
    let minute = ((time_part >> 5) & 0x3F) as u8;
    let hour = ((time_part >> 11) & 0x1F) as u8;
    
    let day = (date_part & 0x1F) as u8;
    let month = ((date_part >> 5) & 0x0F) as u8;
    let year = ((date_part >> 9) & 0x7F) + 1980;
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second)
}

/// Detect RAR format version from magic bytes
pub fn detect_rar_version(path: &str) -> Result<Option<u8>, ContainerError> {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)
        .map_err(|e| format!("Failed to read magic bytes: {}", e))?;
    
    // RAR4: 52 61 72 21 1A 07 00
    if magic[0..7] == [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00] {
        return Ok(Some(4));
    }
    
    // RAR5: 52 61 72 21 1A 07 01 00
    if magic == [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00] {
        return Ok(Some(5));
    }
    
    Ok(None)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rar_archive() {
        assert!(is_rar_archive("test.rar"));
        assert!(is_rar_archive("test.RAR"));
        assert!(is_rar_archive("test.part1.rar"));
        assert!(is_rar_archive("test.r00"));
        assert!(is_rar_archive("test.r01"));
        assert!(!is_rar_archive("test.zip"));
        assert!(!is_rar_archive("test.7z"));
    }

    #[test]
    fn test_list_rar_nonexistent() {
        let result = list_entries("/nonexistent/archive.rar");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_rar_version_nonexistent() {
        let result = detect_rar_version("/nonexistent/archive.rar");
        assert!(result.is_err());
    }
}
