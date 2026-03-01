// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! ZIP format parsing
//!
//! Provides metadata extraction for ZIP and ZIP64 archives.
//!
//! ## ZIP Format Layout
//! ```text
//! [Local File Header 1]
//! [File Data 1]
//! [Data Descriptor 1] (optional)
//! ...
//! [Local File Header n]
//! [File Data n]
//! [Data Descriptor n] (optional)
//! [Central Directory]
//! [End of Central Directory (EOCD)]
//! ```
//!
//! ## EOCD Layout
//! | Offset | Size | Field              |
//! |--------|------|--------------------|
//! | 0x00   | 4    | Signature (PK\x05\x06) |
//! | 0x0A   | 2    | Total Entries      |
//! | 0x0C   | 4    | Central Dir Size   |
//! | 0x10   | 4    | Central Dir Offset |

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::detection::ZIP_EOCD_SIG;
use crate::containers::ContainerError;

// =============================================================================
// ZIP Metadata
// =============================================================================

/// ZIP metadata result
#[derive(Debug, Default)]
pub struct ZipMetadata {
    pub entry_count: Option<u32>,
    pub central_dir_offset: Option<u64>,
    pub central_dir_size: Option<u32>,
    pub encrypted_headers: bool,
    pub aes_encrypted: bool,
}

/// Parse ZIP End of Central Directory (EOCD) to get metadata
///
/// EOCD Layout (search backwards for PK\x05\x06):
/// | Offset | Size | Field              |
/// | 0x10   | 4    | Central Dir Offset |
/// | 0x0C   | 4    | Central Dir Size   |
/// | 0x0A   | 2    | Total Entries      |
pub fn parse_metadata(path: &str) -> Result<ZipMetadata, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open ZIP: {e}"))?;

    let size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();

    // Search backwards for EOCD (max 65557 bytes from end)
    let search_size = size.min(65557) as usize;
    let mut buf = vec![0u8; search_size];

    file.seek(SeekFrom::End(-(search_size as i64)))
        .map_err(|e| format!("Failed to seek: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("Failed to read: {e}"))?;

    // Find EOCD signature (PK\x05\x06)
    let eocd_offset = (0..buf.len().saturating_sub(4))
        .rev()
        .find(|&i| &buf[i..i + 4] == ZIP_EOCD_SIG);

    let Some(eocd_pos) = eocd_offset else {
        return Ok(ZipMetadata::default());
    };

    // Parse EOCD fields
    // Offset 0x0A: Total entries (2 bytes)
    let entry_count = if eocd_pos + 12 <= buf.len() {
        Some(u16::from_le_bytes([buf[eocd_pos + 10], buf[eocd_pos + 11]]) as u32)
    } else {
        None
    };

    // Offset 0x0C: Central Directory Size (4 bytes)
    let cd_size = if eocd_pos + 16 <= buf.len() {
        Some(u32::from_le_bytes([
            buf[eocd_pos + 12],
            buf[eocd_pos + 13],
            buf[eocd_pos + 14],
            buf[eocd_pos + 15],
        ]))
    } else {
        None
    };

    // Offset 0x10: Central Directory Offset (4 bytes)
    let cd_offset = if eocd_pos + 20 <= buf.len() {
        let offset = u32::from_le_bytes([
            buf[eocd_pos + 16],
            buf[eocd_pos + 17],
            buf[eocd_pos + 18],
            buf[eocd_pos + 19],
        ]);
        // Check for ZIP64 marker (0xFFFFFFFF)
        if offset == 0xFFFFFFFF {
            None // Would need to parse ZIP64 EOCD
        } else {
            Some(offset as u64)
        }
    } else {
        None
    };

    // Check for AES encryption by scanning Central Directory for extra field 0x9901
    let aes_encrypted = if let (Some(offset), Some(size)) = (cd_offset, cd_size) {
        check_aes(&mut file, offset, size).unwrap_or(false)
    } else {
        false
    };

    Ok(ZipMetadata {
        entry_count,
        central_dir_offset: cd_offset,
        central_dir_size: cd_size,
        encrypted_headers: false,
        aes_encrypted,
    })
}

/// Check if ZIP uses AES encryption (Extra Field Header ID 0x9901)
fn check_aes(file: &mut File, cd_offset: u64, cd_size: u32) -> Result<bool, ContainerError> {
    file.seek(SeekFrom::Start(cd_offset))
        .map_err(|e| format!("Failed to seek to Central Directory: {e}"))?;

    let mut buf = vec![0u8; cd_size.min(4096) as usize];
    let bytes_read = file
        .read(&mut buf)
        .map_err(|e| format!("Failed to read Central Directory: {e}"))?;

    // Look for AES extra field header (0x9901)
    for i in 0..bytes_read.saturating_sub(2) {
        if buf[i] == 0x01 && buf[i + 1] == 0x99 {
            return Ok(true);
        }
    }

    Ok(false)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_zip_metadata_default() {
        let metadata = ZipMetadata::default();
        assert!(metadata.entry_count.is_none());
        assert!(metadata.central_dir_offset.is_none());
        assert!(metadata.central_dir_size.is_none());
        assert!(!metadata.encrypted_headers);
        assert!(!metadata.aes_encrypted);
    }

    #[test]
    fn test_parse_metadata_nonexistent_file() {
        let result = parse_metadata("/nonexistent/path/to/archive.zip");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to open"));
    }

    #[test]
    fn test_parse_metadata_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = parse_metadata(temp.path().to_str().unwrap());
        // Empty file won't have EOCD - should return default metadata
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert!(metadata.entry_count.is_none());
    }

    #[test]
    fn test_parse_metadata_invalid_zip() {
        let mut temp = NamedTempFile::new().unwrap();
        // Write some non-ZIP data
        temp.write_all(b"This is not a ZIP file").unwrap();

        let result = parse_metadata(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let metadata = result.unwrap();
        // Should return default metadata since no EOCD found
        assert!(metadata.entry_count.is_none());
    }

    #[test]
    fn test_zip_eocd_signature() {
        // Verify the EOCD signature constant
        assert_eq!(ZIP_EOCD_SIG.len(), 4);
        assert_eq!(ZIP_EOCD_SIG[0], 0x50); // 'P'
        assert_eq!(ZIP_EOCD_SIG[1], 0x4B); // 'K'
        assert_eq!(ZIP_EOCD_SIG[2], 0x05);
        assert_eq!(ZIP_EOCD_SIG[3], 0x06);
    }

    #[test]
    fn test_zip_metadata_with_values() {
        let metadata = ZipMetadata {
            entry_count: Some(10),
            central_dir_offset: Some(1024),
            central_dir_size: Some(512),
            encrypted_headers: false,
            aes_encrypted: true,
        };

        assert_eq!(metadata.entry_count, Some(10));
        assert_eq!(metadata.central_dir_offset, Some(1024));
        assert!(metadata.aes_encrypted);
    }

    #[test]
    fn test_minimal_valid_zip() {
        // Create a minimal valid ZIP file (empty archive)
        // ZIP EOCD for empty archive:
        // PK\x05\x06 + 18 bytes of zeros (disk numbers, entry counts, CD size/offset, comment len)
        let mut temp = NamedTempFile::new().unwrap();

        // Minimal empty ZIP: just EOCD
        let eocd = [
            0x50, 0x4B, 0x05, 0x06, // Signature
            0x00, 0x00, // Disk number
            0x00, 0x00, // Disk with CD start
            0x00, 0x00, // Entries on this disk
            0x00, 0x00, // Total entries
            0x00, 0x00, 0x00, 0x00, // CD size
            0x00, 0x00, 0x00, 0x00, // CD offset
            0x00, 0x00, // Comment length
        ];
        temp.write_all(&eocd).unwrap();

        let result = parse_metadata(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.entry_count, Some(0));
        assert_eq!(metadata.central_dir_offset, Some(0));
        assert_eq!(metadata.central_dir_size, Some(0));
    }
}
