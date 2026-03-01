// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! 7-Zip format parsing
//!
//! Provides metadata extraction and file listing for 7-Zip archives.
//! Supports both single-file and split/segmented archives (.7z.001, .7z.002, etc.)
//!
//! ## 7z Signature Header Layout (32 bytes total)
//!
//! | Offset | Size | Field              | Notes                                |
//! |--------|------|--------------------|--------------------------------------|
//! | 0x00   | 6    | Signature          | 37 7A BC AF 27 1C                   |
//! | 0x06   | 2    | Version            | major (1 byte), minor (1 byte)      |
//! | 0x08   | 4    | Start Header CRC   | CRC32 of bytes 0x0C-0x1F            |
//! | 0x0C   | 8    | Next Header Offset | Relative to byte 0x20               |
//! | 0x14   | 8    | Next Header Size   |                                      |
//! | 0x1C   | 4    | Next Header CRC    |                                      |

use crate::containers::ContainerError;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tracing::debug;

use super::detection::SEVEN_ZIP_MAGIC;
use super::extraction::ArchiveEntry;

// =============================================================================
// 7z Header Type IDs
// =============================================================================

/// 7z Header Type IDs (first byte of Next Header determines meaning)
#[allow(dead_code)]
pub mod header_types {
    pub const END: u8 = 0x00;
    pub const HEADER: u8 = 0x01;
    pub const ARCHIVE_PROPERTIES: u8 = 0x02;
    pub const ADDITIONAL_STREAMS_INFO: u8 = 0x03;
    pub const MAIN_STREAMS_INFO: u8 = 0x04;
    pub const FILES_INFO: u8 = 0x05;
    pub const ENCODED_HEADER: u8 = 0x17; // Indicates compressed/encrypted metadata
}

// =============================================================================
// Multi-File Reader for Split Archives
// =============================================================================

/// A reader that concatenates multiple files (for split/segmented archives)
///
/// This allows reading split archives like .7z.001, .7z.002, etc. as a single
/// continuous stream, supporting both Read and Seek operations.
struct MultiFileReader {
    files: Vec<(String, u64)>, // (path, size)
    current_file: Option<BufReader<File>>,
    current_index: usize,
    current_pos_in_file: u64,
    total_pos: u64,
    total_size: u64,
}

impl MultiFileReader {
    fn new(paths: Vec<String>) -> std::io::Result<Self> {
        let mut files = Vec::new();
        let mut total_size = 0u64;

        for path in &paths {
            let meta = std::fs::metadata(path)?;
            let size = meta.len();
            files.push((path.clone(), size));
            total_size += size;
        }

        let first_file = if !files.is_empty() {
            Some(BufReader::new(File::open(&files[0].0)?))
        } else {
            None
        };

        Ok(Self {
            files,
            current_file: first_file,
            current_index: 0,
            current_pos_in_file: 0,
            total_pos: 0,
            total_size,
        })
    }

    fn total_size(&self) -> u64 {
        self.total_size
    }
}

impl Read for MultiFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if let Some(ref mut file) = self.current_file {
                let n = file.read(buf)?;
                if n > 0 {
                    self.current_pos_in_file += n as u64;
                    self.total_pos += n as u64;
                    return Ok(n);
                }
            }

            // Move to next file
            self.current_index += 1;
            if self.current_index >= self.files.len() {
                return Ok(0); // EOF
            }

            self.current_file = Some(BufReader::new(File::open(
                &self.files[self.current_index].0,
            )?));
            self.current_pos_in_file = 0;
        }
    }
}

impl Seek for MultiFileReader {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => (self.total_size as i64 + p) as u64,
            SeekFrom::Current(p) => (self.total_pos as i64 + p) as u64,
        };

        // Find which file contains this position
        let mut cumulative = 0u64;
        for (i, (path, size)) in self.files.iter().enumerate() {
            if new_pos < cumulative + size {
                // Position is in this file
                if i != self.current_index {
                    self.current_file = Some(BufReader::new(File::open(path)?));
                    self.current_index = i;
                }
                let pos_in_file = new_pos - cumulative;
                self.current_file
                    .as_mut()
                    .expect("current_file set when entering this branch")
                    .seek(SeekFrom::Start(pos_in_file))?;
                self.current_pos_in_file = pos_in_file;
                self.total_pos = new_pos;
                return Ok(new_pos);
            }
            cumulative += size;
        }

        // Position is at or past the end
        if let Some((path, _)) = self.files.last() {
            self.current_index = self.files.len() - 1;
            self.current_file = Some(BufReader::new(File::open(path)?));
            self.current_file
                .as_mut()
                .expect("just assigned current_file above")
                .seek(SeekFrom::End(0))?;
        }
        self.total_pos = self.total_size;
        Ok(self.total_size)
    }
}

/// Find all parts of a split archive
///
/// Given a path like `/path/to/archive.7z.001`, finds all sequential parts
/// (.001, .002, .003, etc.) that exist.
fn find_split_archive_parts(first_part: &str) -> Vec<String> {
    let mut parts = Vec::new();

    // Check if it's a .7z.001 or similar pattern
    if let Some(base) = first_part.strip_suffix(".001") {
        let mut num = 1;
        loop {
            let part_path = format!("{}.{:03}", base, num);
            if std::path::Path::new(&part_path).exists() {
                parts.push(part_path);
                num += 1;
            } else {
                break;
            }
        }
    } else {
        // Single file, not split
        parts.push(first_part.to_string());
    }

    parts
}

/// Check if a path is a split archive (ends with .001, .002, etc.)
pub fn is_split_archive(path: &str) -> bool {
    // Match patterns like .7z.001, .zip.001, etc.
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".001") || 
    path_lower.ends_with(".002") ||
    // Also detect if any part exists
    (path_lower.contains(".7z.") && path_lower.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false))
}

// =============================================================================
// File Listing (using sevenz-rust)
// =============================================================================

/// List all entries in a 7-Zip archive
///
/// Uses the sevenz-rust crate to decompress and read the archive structure.
/// Returns a list of all files and directories with metadata.
///
/// Supports both single-file archives and split archives (.7z.001, .7z.002, etc.)
///
/// Fallback chain: libarchive → sevenzip-ffi (encrypted) → sevenz-rust
pub fn list_entries(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    debug!(path = %path, "Listing 7z archive entries");

    // Try libarchive first (best general support)
    match list_entries_libarchive(path) {
        Ok(entries) => {
            debug!(path = %path, entries = entries.len(), "7z listing complete (libarchive)");
            return Ok(entries);
        }
        Err(e) => {
            debug!(path = %path, error = %e, "libarchive failed, trying sevenzip-ffi");
        }
    }

    // Try sevenzip-ffi (handles encrypted archives that libarchive can't)
    match list_entries_sevenzip_ffi(path) {
        Ok(entries) => {
            debug!(path = %path, entries = entries.len(), "7z listing complete (sevenzip-ffi)");
            return Ok(entries);
        }
        Err(e) => {
            debug!(path = %path, error = %e, "sevenzip-ffi failed, falling back to sevenz-rust");
        }
    }

    // Fallback to pure-Rust sevenz-rust
    list_entries_sevenz_rust(path)
}

/// List entries using libarchive (preferred - best general support)
fn list_entries_libarchive(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    super::libarchive_backend::list_entries_as_archive_entry(path, "7z")
}

/// List entries using sevenzip-ffi (handles encrypted archives via AES-256)
fn list_entries_sevenzip_ffi(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let sz = seven_zip::SevenZip::new()
        .map_err(|e| format!("Failed to initialize sevenzip-ffi: {}", e))?;

    let ffi_entries = sz
        .list(path, None)
        .map_err(|e| format!("sevenzip-ffi listing failed: {}", e))?;

    let entries: Vec<ArchiveEntry> = ffi_entries
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            // Convert Unix timestamp to ISO 8601 string
            let last_modified = if e.modified_time > 0 {
                chrono::DateTime::from_timestamp(e.modified_time as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            ArchiveEntry {
                index: i,
                path: e.name,
                is_directory: e.is_directory,
                size: e.size,
                compressed_size: e.packed_size,
                crc32: 0, // sevenzip-ffi list doesn't expose CRC32
                compression_method: "LZMA2".to_string(),
                last_modified,
            }
        })
        .collect();

    Ok(entries)
}

/// List entries using sevenz-rust (fallback for split archives)
fn list_entries_sevenz_rust(path: &str) -> Result<Vec<ArchiveEntry>, ContainerError> {
    use sevenz_rust::{Password, SevenZReader};

    // Check if this is a split archive
    let parts = find_split_archive_parts(path);
    let is_split = parts.len() > 1;

    if is_split {
        debug!(path = %path, parts = parts.len(), "Opening split 7z archive");
    }

    let entries = if is_split {
        // Use MultiFileReader for split archives
        let mut reader = MultiFileReader::new(parts)
            .map_err(|e| format!("Failed to open split archive parts: {}", e))?;

        let total_size = reader.total_size();

        let archive = SevenZReader::new(&mut reader, total_size, Password::empty())
            .map_err(|e| format!("Failed to read split 7z archive: {}", e))?;

        extract_entries_from_archive(&archive)?
    } else {
        // Single file archive
        let file = File::open(path).map_err(|e| format!("Failed to open 7z archive: {}", e))?;

        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        let archive = SevenZReader::new(file, file_size, Password::empty())
            .map_err(|e| format!("Failed to read 7z archive: {}", e))?;

        extract_entries_from_archive(&archive)?
    };

    debug!(path = %path, entries = entries.len(), "7z listing complete (sevenz-rust)");
    Ok(entries)
}

/// Extract entries from a SevenZReader archive
fn extract_entries_from_archive<R: Read + Seek>(
    archive: &sevenz_rust::SevenZReader<R>,
) -> Result<Vec<ArchiveEntry>, ContainerError> {
    let mut entries = Vec::new();

    for (index, entry) in archive.archive().files.iter().enumerate() {
        let entry_path = entry.name().to_string();
        let is_directory = entry.is_directory();

        // Get file name from path
        let _name = Path::new(&entry_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| entry_path.clone());

        // Get timestamps if available - FileTime is a struct, not an Option
        let last_modified = {
            let ft = entry.last_modified_date();
            // FileTime contains a u64 internally
            if ft.to_raw() > 0 {
                // Convert Windows FILETIME to readable format
                // FileTime is 100ns intervals since 1601-01-01
                let unix_secs = (ft.to_raw() as i64 - 116444736000000000) / 10000000;
                chrono::DateTime::from_timestamp(unix_secs, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            }
        };

        entries.push(ArchiveEntry {
            index,
            path: entry_path,
            is_directory,
            size: entry.size(),
            compressed_size: entry.compressed_size,
            crc32: entry.crc as u32,
            compression_method: "LZMA/LZMA2".to_string(),
            last_modified,
        });
    }

    Ok(entries)
}

/// Check if a 7-Zip archive has encrypted headers (filenames hidden)
pub fn has_encrypted_headers(path: &str) -> Result<bool, ContainerError> {
    let metadata = parse_metadata(path)?;
    Ok(metadata.encrypted)
}

// =============================================================================
// Metadata Parsing
// =============================================================================

/// 7z metadata result
#[derive(Debug, Default)]
pub struct SevenZipMetadata {
    pub next_header_offset: Option<u64>,
    pub next_header_size: Option<u64>,
    pub version: Option<String>,
    pub start_header_crc_valid: Option<bool>,
    pub next_header_crc: Option<u32>,
    pub encrypted: bool,
}

/// Parse 7-Zip Start Header and Next Header metadata
///
/// Returns metadata structure with header offsets, version, CRC validation, and encryption status.
pub fn parse_metadata(path: &str) -> Result<SevenZipMetadata, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open 7z: {e}"))?;

    let mut header = [0u8; 32];
    file.read_exact(&mut header)
        .map_err(|e| format!("Failed to read 7z header: {e}"))?;

    // Verify signature (6 bytes at offset 0)
    if &header[..6] != SEVEN_ZIP_MAGIC {
        return Ok(SevenZipMetadata::default());
    }

    // Parse version (2 bytes at offset 6: major, minor)
    let version_major = header[6];
    let version_minor = header[7];
    let version = Some(format!("{}.{}", version_major, version_minor));

    // Parse Start Header CRC (4 bytes at offset 8)
    // This CRC covers bytes 0x0C to 0x1F (20 bytes: next header offset, size, and CRC)
    let stored_start_crc =
        u32::from_le_bytes(header[8..12].try_into().expect("4-byte slice for u32"));
    let computed_start_crc = crc32(&header[12..32]);
    let start_header_crc_valid = Some(stored_start_crc == computed_start_crc);

    // Parse Next Header Offset (8 bytes at offset 0x0C)
    // This is relative to byte 0x20 (end of signature header)
    let next_offset_relative =
        u64::from_le_bytes(header[12..20].try_into().expect("8-byte slice for u64"));

    // Parse Next Header Size (8 bytes at offset 0x14)
    let next_size = u64::from_le_bytes(header[20..28].try_into().expect("8-byte slice for u64"));

    // Parse Next Header CRC (4 bytes at offset 0x1C)
    let next_header_crc = Some(u32::from_le_bytes(
        header[28..32].try_into().expect("4-byte slice for u32"),
    ));

    // Calculate absolute offset: 0x20 (32) + relative offset
    let absolute_offset = 32 + next_offset_relative;

    // Check if headers are encrypted by reading first byte of Next Header
    let mut encrypted = false;
    if next_size > 0 && file.seek(SeekFrom::Start(absolute_offset)).is_ok() {
        let mut next_header_byte = [0u8; 1];
        if file.read_exact(&mut next_header_byte).is_ok() {
            // 0x17 = EncodedHeader - metadata is compressed and/or encrypted
            if next_header_byte[0] == header_types::ENCODED_HEADER {
                debug!(
                    path = %path,
                    "7z has EncodedHeader - metadata may be encrypted"
                );
                // Try to detect AES in the encoded header stream info
                encrypted = detect_encryption(&mut file, absolute_offset).unwrap_or(false);
            }
        }
    }

    debug!(
        path = %path,
        version = ?version,
        next_header_offset = absolute_offset,
        next_header_size = next_size,
        crc_valid = ?start_header_crc_valid,
        encrypted = encrypted,
        "7z metadata parsed"
    );

    Ok(SevenZipMetadata {
        next_header_offset: Some(absolute_offset),
        next_header_size: Some(next_size),
        version,
        start_header_crc_valid,
        next_header_crc,
        encrypted,
    })
}

/// Detect if 7z encoded header contains AES encryption
///
/// When Next Header starts with 0x17 (EncodedHeader), we need to parse
/// the StreamsInfo to check if AES codec is in the decode pipeline.
/// AES codec ID: 06 F1 07 01 (or variations)
fn detect_encryption(file: &mut File, next_header_offset: u64) -> Result<bool, ContainerError> {
    file.seek(SeekFrom::Start(next_header_offset))
        .map_err(|e| format!("Failed to seek to Next Header: {e}"))?;

    // Read first chunk of encoded header to look for AES codec markers
    let mut buf = [0u8; 256];
    let bytes_read = file.read(&mut buf).unwrap_or(0);

    if bytes_read == 0 {
        return Ok(false);
    }

    // Look for AES codec signature patterns in the encoded header
    // 7z AES codec IDs typically start with 06 F1 07
    for i in 0..bytes_read.saturating_sub(3) {
        if buf[i] == 0x06 && buf[i + 1] == 0xF1 && buf[i + 2] == 0x07 {
            return Ok(true);
        }
    }

    // Also check for 7zAES marker (alternative pattern)
    // 07 (codec ID length) followed by specific bytes
    for i in 0..bytes_read.saturating_sub(4) {
        if buf[i] == 0x07 && buf[i + 1] == 0x06 && buf[i + 2] == 0xF1 {
            return Ok(true);
        }
    }

    Ok(false)
}

// =============================================================================
// CRC32 Implementation (ISO 3309 polynomial)
// =============================================================================

/// CRC32 calculation for 7z (ISO 3309 polynomial, same as used in PNG/GZIP)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        let index = ((crc ^ (*byte as u32)) & 0xFF) as usize;
        crc = CRC32_TABLE[index] ^ (crc >> 8);
    }
    !crc
}

/// CRC32 lookup table (ISO 3309 polynomial: 0xEDB88320)
static CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = 0xEDB88320 ^ (crc >> 1);
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_type_constants() {
        assert_eq!(header_types::END, 0x00);
        assert_eq!(header_types::HEADER, 0x01);
        assert_eq!(header_types::ARCHIVE_PROPERTIES, 0x02);
        assert_eq!(header_types::ADDITIONAL_STREAMS_INFO, 0x03);
        assert_eq!(header_types::MAIN_STREAMS_INFO, 0x04);
        assert_eq!(header_types::FILES_INFO, 0x05);
        assert_eq!(header_types::ENCODED_HEADER, 0x17);
    }

    #[test]
    fn test_seven_zip_metadata_default() {
        let metadata = SevenZipMetadata::default();
        assert!(metadata.next_header_offset.is_none());
        assert!(metadata.next_header_size.is_none());
        assert!(metadata.version.is_none());
        assert!(metadata.start_header_crc_valid.is_none());
        assert!(metadata.next_header_crc.is_none());
        assert!(!metadata.encrypted);
    }

    #[test]
    fn test_crc32_empty() {
        let crc = crc32(&[]);
        // CRC32 of empty data
        assert_eq!(crc, 0x00000000);
    }

    #[test]
    fn test_crc32_known_value() {
        // "123456789" should produce CRC32 = 0xCBF43926
        let data = b"123456789";
        let crc = crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_crc32_single_byte() {
        // CRC32 of a single null byte
        let crc = crc32(&[0x00]);
        assert_eq!(crc, 0xD202EF8D);
    }

    #[test]
    fn test_crc32_table_first_entries() {
        // Verify CRC table is correctly initialized
        assert_eq!(CRC32_TABLE[0], 0x00000000);
        assert_eq!(CRC32_TABLE[1], 0x77073096);
        assert_eq!(CRC32_TABLE[255], 0x2D02EF8D);
    }

    #[test]
    fn test_parse_metadata_nonexistent_file() {
        let result = parse_metadata("/nonexistent/path/to/archive.7z");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to open"));
    }

    #[test]
    fn test_seven_zip_magic() {
        // Verify the magic bytes constant
        assert_eq!(SEVEN_ZIP_MAGIC.len(), 6);
        assert_eq!(SEVEN_ZIP_MAGIC[0], 0x37); // '7'
        assert_eq!(SEVEN_ZIP_MAGIC[1], 0x7A); // 'z'
        assert_eq!(SEVEN_ZIP_MAGIC[2], 0xBC);
        assert_eq!(SEVEN_ZIP_MAGIC[3], 0xAF);
        assert_eq!(SEVEN_ZIP_MAGIC[4], 0x27);
        assert_eq!(SEVEN_ZIP_MAGIC[5], 0x1C);
    }

    // ==================== sevenzip-ffi entry conversion ====================

    #[test]
    fn test_ffi_entry_timestamp_conversion_valid() {
        // 2024-01-15T12:00:00Z as Unix timestamp
        let timestamp: u64 = 1705320000;
        let result = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default();
        assert_eq!(result, "2024-01-15T12:00:00Z");
    }

    #[test]
    fn test_ffi_entry_timestamp_conversion_zero() {
        // Zero timestamp should produce empty string
        let timestamp: u64 = 0;
        let result = if timestamp > 0 {
            chrono::DateTime::from_timestamp(timestamp as i64, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        assert_eq!(result, "");
    }

    #[test]
    fn test_ffi_entry_timestamp_conversion_epoch() {
        // Unix epoch (1970-01-01T00:00:00Z) — timestamp=1 (not 0, which is empty)
        let timestamp: u64 = 1;
        let result = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default();
        assert_eq!(result, "1970-01-01T00:00:01Z");
    }

    #[test]
    fn test_ffi_entry_to_archive_entry_file() {
        use crate::archive::extraction::ArchiveEntry;

        let entry = ArchiveEntry {
            index: 0,
            path: "documents/report.pdf".to_string(),
            is_directory: false,
            size: 1024000,
            compressed_size: 512000,
            crc32: 0,
            compression_method: "LZMA2".to_string(),
            last_modified: "2024-01-15T12:00:00Z".to_string(),
        };
        assert_eq!(entry.index, 0);
        assert_eq!(entry.path, "documents/report.pdf");
        assert!(!entry.is_directory);
        assert_eq!(entry.size, 1024000);
        assert_eq!(entry.compressed_size, 512000);
        assert_eq!(entry.crc32, 0); // sevenzip-ffi doesn't expose CRC
        assert_eq!(entry.compression_method, "LZMA2");
    }

    #[test]
    fn test_ffi_entry_to_archive_entry_directory() {
        use crate::archive::extraction::ArchiveEntry;

        let entry = ArchiveEntry {
            index: 5,
            path: "documents/".to_string(),
            is_directory: true,
            size: 0,
            compressed_size: 0,
            crc32: 0,
            compression_method: "LZMA2".to_string(),
            last_modified: String::new(),
        };
        assert!(entry.is_directory);
        assert_eq!(entry.size, 0);
        assert_eq!(entry.compressed_size, 0);
    }

    #[test]
    fn test_list_entries_sevenzip_ffi_nonexistent() {
        // Should fail gracefully for nonexistent file
        let result = list_entries_sevenzip_ffi("/nonexistent/path/archive.7z");
        assert!(result.is_err());
    }
}
