// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File viewer module for hex/text viewing and document content rendering
//!
//! This module provides:
//! - Chunked file reading for large file hex/text viewing
//! - Document format parsing (PDF, DOCX, HTML, Markdown)
//! - Specialized viewers for forensic artifacts (email, plist, binaries, etc.)
//! - Memory-mapped hex viewing with LRU caching

pub mod document; // Document/content viewers (PDF, DOCX, email, plist, binaries, etc.)
pub mod parsers;
pub mod types;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::common::format_size;
use crate::containers::ContainerError;

// Re-export types for convenience
pub use types::{FileChunk, FileTypeInfo, HeaderRegion, MetadataField, ParsedMetadata};

// Re-export parsers
pub use parsers::{
    parse_7z_header, parse_ad1_header, parse_ewf_header, parse_gzip_header, parse_qcow2_header,
    parse_rar_header, parse_raw_header, parse_vhdx_header, parse_vmdk_header, parse_zip_header,
};

/// Default chunk size (16KB = 1024 lines of 16 bytes)
/// Larger default reduces IPC round-trips for sequential hex viewing
const DEFAULT_CHUNK_SIZE: usize = 16384;

/// Maximum chunk size (64KB)
const MAX_CHUNK_SIZE: usize = 65536;

/// Read a chunk of a file at the given offset
pub fn read_file_chunk(
    path: &str,
    offset: u64,
    size: Option<usize>,
) -> Result<FileChunk, ContainerError> {
    let chunk_size = size.unwrap_or(DEFAULT_CHUNK_SIZE).min(MAX_CHUNK_SIZE);

    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let total_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();

    // Clamp offset to file bounds
    let actual_offset = offset.min(total_size);

    file.seek(SeekFrom::Start(actual_offset))
        .map_err(|e| format!("Failed to seek: {}", e))?;

    // Calculate how much we can actually read
    let remaining = total_size.saturating_sub(actual_offset) as usize;
    let to_read = chunk_size.min(remaining);

    let mut buffer = vec![0u8; to_read];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;

    buffer.truncate(bytes_read);

    let chunk_end = actual_offset + (bytes_read as u64);
    let has_more = chunk_end < total_size;
    let has_prev = actual_offset > 0;

    Ok(FileChunk {
        bytes: buffer,
        offset: actual_offset,
        total_size,
        has_more,
        has_prev,
    })
}

/// Detect file type from magic bytes
pub fn detect_file_type(path: &str) -> Result<FileTypeInfo, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut magic = [0u8; 32];
    let bytes_read = file
        .read(&mut magic)
        .map_err(|e| format!("Failed to read: {}", e))?;

    let magic_hex = magic[..bytes_read.min(16)]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Detect known forensic formats
    let (description, is_forensic, mime_type) = detect_format(&magic[..bytes_read], &extension);

    // Check if likely text
    let is_text = is_likely_text(&magic[..bytes_read])
        || matches!(
            extension.as_str(),
            "txt"
                | "log"
                | "json"
                | "xml"
                | "csv"
                | "md"
                | "html"
                | "htm"
                | "css"
                | "js"
                | "ts"
                | "py"
                | "rs"
                | "c"
                | "h"
                | "cpp"
                | "java"
        );

    Ok(FileTypeInfo {
        mime_type,
        description,
        extension,
        is_text,
        is_forensic_format: is_forensic,
        magic_hex,
    })
}

/// Parse file header and extract metadata with regions for highlighting
pub fn parse_file_header(path: &str) -> Result<ParsedMetadata, ContainerError> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let file_size = file.metadata().map_err(|e| e.to_string())?.len();

    // Read first 512 bytes for header analysis
    let mut header = vec![0u8; 512.min(file_size as usize)];
    file.read_exact(&mut header)
        .map_err(|e| format!("Failed to read header: {}", e))?;

    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Use detailed EWF parser for E01/L01/Ex01/Lx01 files
    if header.len() >= 8 && (&header[0..3] == b"EVF" || &header[0..3] == b"LVF") {
        // Try detailed EWF parser
        match crate::ewf::parser::parse_ewf_file(path) {
            Ok(ewf_info) => {
                return Ok(crate::ewf::parser::ewf_detailed_info_to_metadata(&ewf_info))
            }
            Err(_) => {
                // Fall back to basic parsing
                return parse_header_by_format(&header, &extension, file_size);
            }
        }
    }

    // Parse based on detected format
    parse_header_by_format(&header, &extension, file_size)
}

/// Read file as text (for text viewer)
pub fn read_file_text(path: &str, offset: u64, max_chars: usize) -> Result<String, ContainerError> {
    let chunk = read_file_chunk(path, offset, Some(max_chars * 4))?; // UTF-8 can be up to 4 bytes per char

    // Try to decode as UTF-8, falling back to lossy conversion
    let text = String::from_utf8_lossy(&chunk.bytes);

    // Truncate to max chars if needed
    if text.chars().count() > max_chars {
        Ok(text.chars().take(max_chars).collect())
    } else {
        Ok(text.to_string())
    }
}

// -----------------------------------------------------------------------------
// Internal helper functions
// -----------------------------------------------------------------------------

/// Detect format from magic bytes and extension
fn detect_format(magic: &[u8], extension: &str) -> (String, bool, Option<String>) {
    // EWF/E01 format
    if magic.len() >= 8 && &magic[0..8] == b"EVF\x09\x0d\x0a\xff\x00" {
        return (
            "EWF/E01 Forensic Image".to_string(),
            true,
            Some("application/x-ewf".to_string()),
        );
    }

    // E01 variant
    if magic.len() >= 3 && &magic[0..3] == b"EVF" {
        return (
            "EWF/E01 Forensic Image".to_string(),
            true,
            Some("application/x-ewf".to_string()),
        );
    }

    // AD1 format
    if magic.len() >= 8 && &magic[0..8] == b"ADSEGMEN" {
        return (
            "AD1 Forensic Container".to_string(),
            true,
            Some("application/x-ad1".to_string()),
        );
    }

    // L01 format
    if magic.len() >= 8 && &magic[0..8] == b"LVF\x09\x0d\x0a\xff\x00" {
        return (
            "L01 Logical Evidence".to_string(),
            true,
            Some("application/x-l01".to_string()),
        );
    }

    // ZIP (and derivatives)
    if magic.len() >= 4 && &magic[0..4] == b"PK\x03\x04" {
        match extension {
            "ufdr" | "ufdx" => {
                return (
                    "UFED Report Archive".to_string(),
                    true,
                    Some("application/x-ufdr".to_string()),
                )
            }
            "docx" => {
                return (
                    "Word Document".to_string(),
                    false,
                    Some(
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                            .to_string(),
                    ),
                )
            }
            "xlsx" => {
                return (
                    "Excel Spreadsheet".to_string(),
                    false,
                    Some(
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                            .to_string(),
                    ),
                )
            }
            "apk" => {
                return (
                    "Android Package".to_string(),
                    false,
                    Some("application/vnd.android.package-archive".to_string()),
                )
            }
            _ => {
                return (
                    "ZIP Archive".to_string(),
                    false,
                    Some("application/zip".to_string()),
                )
            }
        }
    }

    // 7z
    if magic.len() >= 6 && &magic[0..6] == b"7z\xbc\xaf\x27\x1c" {
        return (
            "7-Zip Archive".to_string(),
            false,
            Some("application/x-7z-compressed".to_string()),
        );
    }

    // RAR5
    if magic.len() >= 8 && &magic[0..7] == b"Rar!\x1a\x07" && magic[7] == 0x01 {
        return (
            "RAR5 Archive".to_string(),
            false,
            Some("application/vnd.rar".to_string()),
        );
    }

    // RAR4
    if magic.len() >= 7 && &magic[0..7] == b"Rar!\x1a\x07\x00" {
        return (
            "RAR4 Archive".to_string(),
            false,
            Some("application/vnd.rar".to_string()),
        );
    }

    // GZIP
    if magic.len() >= 2 && magic[0] == 0x1f && magic[1] == 0x8b {
        return (
            "GZIP Compressed".to_string(),
            false,
            Some("application/gzip".to_string()),
        );
    }

    // PDF
    if magic.len() >= 5 && &magic[0..5] == b"%PDF-" {
        return (
            "PDF Document".to_string(),
            false,
            Some("application/pdf".to_string()),
        );
    }

    // SQLite
    if magic.len() >= 16 && &magic[0..16] == b"SQLite format 3\x00" {
        return (
            "SQLite Database".to_string(),
            false,
            Some("application/x-sqlite3".to_string()),
        );
    }

    // JPEG
    if magic.len() >= 3 && magic[0] == 0xFF && magic[1] == 0xD8 && magic[2] == 0xFF {
        return (
            "JPEG Image".to_string(),
            false,
            Some("image/jpeg".to_string()),
        );
    }

    // PNG
    if magic.len() >= 8 && &magic[0..8] == b"\x89PNG\x0d\x0a\x1a\x0a" {
        return (
            "PNG Image".to_string(),
            false,
            Some("image/png".to_string()),
        );
    }

    // VMDK sparse header
    if magic.len() >= 4 && &magic[0..4] == b"KDMV" {
        return (
            "VMDK Virtual Disk".to_string(),
            true,
            Some("application/x-vmdk".to_string()),
        );
    }

    // VHDx
    if magic.len() >= 8 && &magic[0..8] == b"vhdxfile" {
        return (
            "VHDx Virtual Hard Disk".to_string(),
            true,
            Some("application/x-vhdx".to_string()),
        );
    }

    // QCOW2
    if magic.len() >= 4 && &magic[0..4] == b"QFI\xfb" {
        return (
            "QCOW2 Disk Image".to_string(),
            true,
            Some("application/x-qcow2".to_string()),
        );
    }

    // VDI (VirtualBox)
    if magic.len() >= 64 && &magic[0..4] == b"<<<" {
        return (
            "VDI Virtual Disk".to_string(),
            true,
            Some("application/x-vdi".to_string()),
        );
    }

    // DMG (Apple Disk Image)
    if extension == "dmg" {
        return (
            "Apple Disk Image".to_string(),
            false,
            Some("application/x-apple-diskimage".to_string()),
        );
    }

    // XML
    if magic.len() >= 5 && &magic[0..5] == b"<?xml" {
        return (
            "XML Document".to_string(),
            false,
            Some("application/xml".to_string()),
        );
    }

    // MBR/GPT detection for raw images
    if magic.len() >= 512 && magic[510] == 0x55 && magic[511] == 0xAA {
        if magic.len() >= 450 && magic[450] == 0xEE {
            return (
                "Raw Disk Image (GPT)".to_string(),
                true,
                Some("application/octet-stream".to_string()),
            );
        }
        return (
            "Raw Disk Image (MBR)".to_string(),
            true,
            Some("application/octet-stream".to_string()),
        );
    }

    // By extension fallback
    match extension {
        "e01" | "e02" | "e03" => (
            "EWF Segment".to_string(),
            true,
            Some("application/x-ewf".to_string()),
        ),
        "ad1" => (
            "AD1 Container".to_string(),
            true,
            Some("application/x-ad1".to_string()),
        ),
        "l01" => (
            "L01 Logical Evidence".to_string(),
            true,
            Some("application/x-l01".to_string()),
        ),
        "dd" | "raw" | "img" | "bin" => (
            "Raw Disk Image".to_string(),
            true,
            Some("application/octet-stream".to_string()),
        ),
        "vmdk" => (
            "VMDK Virtual Disk".to_string(),
            true,
            Some("application/x-vmdk".to_string()),
        ),
        "vhd" => (
            "VHD Virtual Hard Disk".to_string(),
            true,
            Some("application/x-vhd".to_string()),
        ),
        "vhdx" => (
            "VHDx Virtual Hard Disk".to_string(),
            true,
            Some("application/x-vhdx".to_string()),
        ),
        "qcow2" | "qcow" => (
            "QCOW2 Disk Image".to_string(),
            true,
            Some("application/x-qcow2".to_string()),
        ),
        "vdi" => (
            "VDI Virtual Disk".to_string(),
            true,
            Some("application/x-vdi".to_string()),
        ),
        _ if extension.chars().all(|c| c.is_ascii_digit()) => (
            "Raw Disk Segment".to_string(),
            true,
            Some("application/octet-stream".to_string()),
        ),
        _ => ("Unknown".to_string(), false, None),
    }
}

/// Check if bytes are likely text content
fn is_likely_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    // Check if mostly printable ASCII or common whitespace
    let printable_count = bytes
        .iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b) || b == 0x09 || b == 0x0A || b == 0x0D)
        .count();

    // Consider text if >85% printable
    printable_count * 100 / bytes.len() > 85
}

/// Route to appropriate parser based on format
fn parse_header_by_format(
    header: &[u8],
    extension: &str,
    file_size: u64,
) -> Result<ParsedMetadata, ContainerError> {
    // E01/EWF format
    if header.len() >= 8 && (&header[0..3] == b"EVF" || &header[0..3] == b"LVF") {
        return parse_ewf_header(header, file_size);
    }

    // AD1 format
    if header.len() >= 8 && &header[0..8] == b"ADSEGMEN" {
        return parse_ad1_header(header, file_size);
    }

    // ZIP-based formats
    if header.len() >= 4 && &header[0..4] == b"PK\x03\x04" {
        return parse_zip_header(header, extension, file_size);
    }

    // 7-Zip format
    if header.len() >= 6 && header[0..6] == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return parse_7z_header(header, file_size);
    }

    // RAR format (v4 and v5)
    if header.len() >= 7 && &header[0..4] == b"Rar!" {
        return parse_rar_header(header, file_size);
    }

    // GZIP format
    if header.len() >= 10 && header[0] == 0x1F && header[1] == 0x8B {
        return parse_gzip_header(header, file_size);
    }

    // Raw disk image formats (by extension or MBR/GPT detection)
    if matches!(extension, "dd" | "raw" | "img" | "bin")
        || extension.chars().all(|c| c.is_ascii_digit())
    {
        return parse_raw_header(header, extension, file_size);
    }

    // VMDK format
    if header.len() >= 4 && &header[0..4] == b"KDMV" {
        return parse_vmdk_header(header, file_size);
    }

    // VHDx format
    if header.len() >= 8 && &header[0..8] == b"vhdxfile" {
        return parse_vhdx_header(header, file_size);
    }

    // QCOW2 format
    if header.len() >= 4 && &header[0..4] == b"QFI\xfb" {
        return parse_qcow2_header(header, file_size);
    }

    // Default: basic info
    Ok(ParsedMetadata {
        format: "Unknown".to_string(),
        version: None,
        fields: vec![MetadataField::new(
            "File Size",
            format_size(file_size),
            "General",
        )],
        regions: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
        assert_eq!(format_size(1048576), "1.00 MB (1048576 bytes)");
        assert_eq!(format_size(1073741824), "1.00 GB (1073741824 bytes)");
    }

    #[test]
    fn test_is_likely_text() {
        assert!(is_likely_text(b"Hello World!"));
        assert!(is_likely_text(b"Line 1\nLine 2\r\n"));
        assert!(!is_likely_text(&[0x00, 0x01, 0x02, 0x03]));
    }

    #[test]
    fn test_detect_format_forensic() {
        // Test E01 detection
        let e01_magic = [0x45, 0x56, 0x46, 0x09, 0x0D, 0x0A, 0xFF, 0x00];
        let (desc, is_forensic, _) = detect_format(&e01_magic, "e01");
        assert!(desc.contains("E01") || desc.contains("EWF"));
        assert!(is_forensic);

        // Test AD1 detection
        let ad1_magic = b"ADSEGMEN";
        let (desc, is_forensic, _) = detect_format(ad1_magic, "ad1");
        assert!(desc.contains("AD1"));
        assert!(is_forensic);
    }
}
