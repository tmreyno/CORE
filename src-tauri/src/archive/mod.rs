// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive container support (7z, ZIP, RAR, TAR)
//!
//! This module provides archive detection and metadata extraction for common
//! archive formats used in forensic workflows. Based on format specifications:
//!
//! ## ZIP Format
//! - Layout: `[Local Headers][Data][Central Directory][EOCD]`
//! - Metadata authority: Central Directory (not Local File Headers)
//! - ZIP64: Triggered when any value == 0xFFFFFFFF
//! - Signatures: PK\x03\x04 (Local), PK\x01\x02 (Central), PK\x05\x06 (EOCD)
//!
//! ## 7-Zip Format
//! - Signature: 37 7A BC AF 27 1C
//! - Layout: [Signature][Start Header][Compressed Streams][Main Header]
//! - Encrypted headers hide filenames and directory structure
//!
//! ## RAR Format
//! - RAR4: 52 61 72 21 1A 07 00
//! - RAR5: 52 61 72 21 1A 07 01 00
//!
//! ## TAR Format
//! - No magic signature (uses header checksums)
//! - Supports: .tar, .tar.gz, .tar.bz2, .tar.xz, .tar.zst
//!
//! ## Forensic Notes
//! - Container hash = metadata + payload (chain-of-custody)
//! - CRC32 is for error-detection, NOT tamper resistance
//! - Use cryptographic hashes (SHA-256) for evidentiary integrity
//!
//! ## Module Structure
//! ```text
//! archive/
//! ├── mod.rs        - Main entry point, info() function
//! ├── types.rs      - ArchiveFormat, ArchiveInfo
//! ├── detection.rs  - Magic signatures, format detection
//! ├── sevenz.rs     - 7-Zip parsing and listing
//! ├── zip.rs        - ZIP/ZIP64 EOCD parsing
//! ├── tar.rs        - TAR and compressed TAR listing
//! ├── rar.rs        - RAR archive listing
//! ├── segments.rs   - Multi-part archive discovery
//! └── extraction.rs - Archive extraction support
//! ```
//!
//! Note: UFED detection in ZIPs is handled by `ufed::archive_scan`

pub mod types;
pub mod detection;
pub mod sevenz;
pub mod zip;
pub mod tar;
pub mod rar;
pub mod segments;

use crate::containers::ContainerError;
pub mod vfs;
pub mod extraction;

// Re-exports for convenience
pub use types::{ArchiveFormat, ArchiveInfo};
pub use detection::{is_archive, detect_archive_format, is_7z_segment};
pub use vfs::ArchiveVfs;
pub use extraction::{
    extract, extract_zip, extract_gzip, extract_with_progress, extract_zip_with_progress,
    extract_zip_entry, list_zip_entries, ExtractResult, ExtractError, ArchiveEntry,
};
pub use sevenz::is_split_archive;
// Note: For TAR/RAR listing, use tar::list_entries and rar::list_entries directly
// Note: is_first_segment, is_continuation_segment are in containers::segments
// which provides unified handling for all container types

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::debug;

use crate::common::hash::{StreamingHasher, HashAlgorithm};

/// Compute hash of archive file
/// 
/// Computes a cryptographic hash of the entire archive file for
/// chain-of-custody verification.
pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Compute hash of archive file with progress callback
pub fn verify_with_progress<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, algorithm = %algorithm, "Computing archive hash");
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    
    let total_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let algo = algorithm.parse::<HashAlgorithm>()
        .map_err(|e| format!("Unsupported algorithm: {}", e))?;
    let mut hasher = StreamingHasher::new(algo);
    
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut bytes_read = 0u64;
    
    loop {
        let n = reader.read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        
        if n == 0 {
            break;
        }
        
        hasher.update(&buffer[..n]);
        bytes_read += n as u64;
        progress_callback(bytes_read, total_size);
    }
    
    let hash = hasher.finalize();
    debug!(hash = %hash, "Archive hash computed");
    Ok(hash)
}

/// Per-entry verification result
#[derive(Debug, Clone, serde::Serialize)]
pub struct EntryVerifyResult {
    /// Entry path within the archive
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Expected CRC32 from archive metadata
    pub expected_crc32: u32,
    /// Computed CRC32 from content
    pub computed_crc32: Option<u32>,
    /// Whether the CRC matches
    pub crc_match: bool,
    /// Entry size
    pub size: u64,
    /// Error message if verification failed
    pub error: Option<String>,
}

/// Verify all entries in a ZIP archive by checking CRC32
pub fn verify_entries(path: &str) -> Result<Vec<EntryVerifyResult>, ContainerError> {
    verify_entries_with_progress(path, |_, _| {})
}

/// Verify all entries in a ZIP archive with progress callback
pub fn verify_entries_with_progress<F>(path: &str, progress_callback: F) -> Result<Vec<EntryVerifyResult>, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, "Verifying archive entries");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("Archive not found: {}", path)));
    }
    
    let format = detection::detect_archive_format(path)?
        .ok_or_else(|| ContainerError::InvalidFormat(format!("Unable to detect archive format: {}", path)))?;
    
    match format {
        types::ArchiveFormat::Zip | types::ArchiveFormat::Zip64 => {
            verify_zip_entries_with_progress(path, progress_callback)
        }
        _ => Err(ContainerError::UnsupportedOperation(format!("Entry verification not supported for format: {}", format))),
    }
}

/// Verify ZIP entries by computing CRC32
fn verify_zip_entries_with_progress<F>(path: &str, mut progress_callback: F) -> Result<Vec<EntryVerifyResult>, ContainerError>
where
    F: FnMut(u64, u64),
{
    use std::hash::Hasher;
    
    let file = File::open(path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    
    let mut archive = ::zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read ZIP archive: {}", e))?;
    
    let total_entries = archive.len();
    let mut results = Vec::with_capacity(total_entries);
    
    for i in 0..total_entries {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                results.push(EntryVerifyResult {
                    path: format!("entry {}", i),
                    is_directory: false,
                    expected_crc32: 0,
                    computed_crc32: None,
                    crc_match: false,
                    size: 0,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };
        
        let entry_name = entry.name().to_string();
        let expected_crc = entry.crc32();
        let size = entry.size();
        let is_dir = entry.is_dir();
        
        if is_dir {
            results.push(EntryVerifyResult {
                path: entry_name,
                is_directory: true,
                expected_crc32: expected_crc,
                computed_crc32: None,
                crc_match: true, // Directories always pass
                size: 0,
                error: None,
            });
        } else {
            // Read and compute CRC32
            let mut hasher = crc32fast::Hasher::new();
            let mut buffer = [0u8; 65536];
            let mut error = None;
            
            loop {
                match entry.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => hasher.write(&buffer[..n]),
                    Err(e) => {
                        error = Some(e.to_string());
                        break;
                    }
                }
            }
            
            let computed = hasher.finalize();
            let crc_match = error.is_none() && computed == expected_crc;
            
            results.push(EntryVerifyResult {
                path: entry_name,
                is_directory: false,
                expected_crc32: expected_crc,
                computed_crc32: Some(computed),
                crc_match,
                size,
                error,
            });
        }
        
        progress_callback((i + 1) as u64, total_entries as u64);
    }
    
    Ok(results)
}

/// Get archive information including segment discovery
pub fn info(path: &str) -> Result<ArchiveInfo, ContainerError> {
    debug!(path = %path, "Getting archive info");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("Archive file not found: {path}")));
    }
    
    let format = detection::detect_archive_format(path)?
        .ok_or_else(|| format!("Unable to detect archive format: {path}"))?;
    
    let format_str = format.to_string();
    
    // Discover segments for multi-part archives
    let (segment_names, segment_sizes) = segments::discover_segments(path, format)?;
    let segment_count = segment_names.len() as u32;
    let total_size: u64 = segment_sizes.iter().sum();
    
    let first_segment = segment_names.first().cloned().unwrap_or_default();
    let last_segment = segment_names.last().cloned().unwrap_or_default();
    let is_multipart = segment_count > 1;
    
    // Parse format-specific metadata
    let (entry_count, central_dir_offset, central_dir_size, mut encrypted_headers, aes_encrypted) = 
        match format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
                let meta = zip::parse_metadata(path).unwrap_or_default();
                (meta.entry_count, meta.central_dir_offset, meta.central_dir_size, 
                 meta.encrypted_headers, meta.aes_encrypted)
            }
            _ => (None, None, None, false, false),
        };
    
    // Parse 7z-specific metadata with full Start Header details
    let (next_header_offset, next_header_size, version, start_header_crc_valid, next_header_crc, sevenz_encrypted) = 
        match format {
            ArchiveFormat::SevenZip => {
                let meta = sevenz::parse_metadata(path).unwrap_or_default();
                (meta.next_header_offset, meta.next_header_size, meta.version,
                 meta.start_header_crc_valid, meta.next_header_crc, meta.encrypted)
            }
            _ => (None, None, None, None, None, false),
        };
    
    // Set encrypted_headers for 7z if detected
    if sevenz_encrypted {
        encrypted_headers = true;
    }
    
    // Detect UFED files (UFDR/UFDX/UFD) inside the archive
    let (ufed_detected, ufed_files) = match format {
        ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
            crate::ufed::detect_in_zip(path).unwrap_or((false, vec![]))
        }
        _ => (false, vec![]),
    };
    
    debug!(
        path = %path,
        format = %format_str,
        segment_count = segment_count,
        total_size = total_size,
        entry_count = ?entry_count,
        ufed_detected = ufed_detected,
        "Archive info loaded"
    );
    
    Ok(ArchiveInfo {
        format: format_str,
        segment_count,
        total_size,
        segment_names,
        segment_sizes,
        first_segment,
        last_segment,
        is_multipart,
        entry_count,
        encrypted_headers,
        aes_encrypted,
        central_dir_offset,
        central_dir_size,
        next_header_offset,
        next_header_size,
        version,
        start_header_crc_valid,
        next_header_crc,
        ufed_detected,
        ufed_files,
    })
}

/// Fast archive info - only reads basic metadata, skips expensive operations
/// 
/// This is faster than `info()` because it:
/// - Skips parsing individual archive entries
/// - Skips UFED detection inside ZIP files
/// - Only gets basic format and size information
/// 
/// Use this for quick container detection/listing.
pub fn info_fast(path: &str) -> Result<ArchiveInfo, ContainerError> {
    debug!(path = %path, "Getting fast archive info");
    
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!("Archive file not found: {path}")));
    }
    
    let format = detection::detect_archive_format(path)?
        .ok_or_else(|| format!("Unable to detect archive format: {path}"))?;
    
    let format_str = format.to_string();
    
    // Get file size directly - avoid full segment discovery for single files
    let file_size = std::fs::metadata(path)
        .map_err(|e| format!("Failed to read file metadata: {e}"))?
        .len();
    
    let filename = path_obj
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    debug!(
        path = %path,
        format = %format_str,
        size = file_size,
        "Fast archive info loaded"
    );
    
    Ok(ArchiveInfo {
        format: format_str,
        segment_count: 1,
        total_size: file_size,
        segment_names: vec![filename.clone()],
        segment_sizes: vec![file_size],
        first_segment: filename.clone(),
        last_segment: filename,
        is_multipart: false,
        entry_count: None,  // Skip expensive entry counting
        encrypted_headers: false,
        aes_encrypted: false,
        central_dir_offset: None,
        central_dir_size: None,
        next_header_offset: None,
        next_header_size: None,
        version: None,
        start_header_crc_valid: None,
        next_header_crc: None,
        ufed_detected: false,  // Skip expensive UFED detection
        ufed_files: vec![],
    })
}

/// Archive container statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveStats {
    /// Archive format (ZIP, 7z, RAR, etc.)
    pub format: String,
    /// Total size in bytes
    pub total_size: u64,
    /// Formatted total size (human readable)
    pub total_size_formatted: String,
    /// Number of segments
    pub segment_count: u32,
    /// Number of entries (files/folders) in the archive
    pub entry_count: Option<u64>,
    /// Whether the archive has encrypted headers
    pub encrypted_headers: bool,
    /// Whether AES encryption is used
    pub aes_encrypted: bool,
    /// Whether this is a multi-part archive
    pub is_multipart: bool,
    /// Whether UFED content was detected
    pub ufed_detected: bool,
    /// Compression ratio (if calculable)
    pub compression_ratio: Option<f64>,
}

/// Get archive statistics
pub fn get_stats(path: &str) -> Result<ArchiveStats, ContainerError> {
    debug!(path = %path, "Getting archive stats");
    
    let info = info(path)?;
    
    // Calculate compression ratio for ZIP archives
    let compression_ratio = if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        // Try to get uncompressed size from ZIP entries
        if let Ok(entries) = extraction::list_zip_entries(path) {
            let uncompressed: u64 = entries.iter().map(|e| e.size).sum();
            if uncompressed > 0 {
                Some(info.total_size as f64 / uncompressed as f64)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    
    let total_size_formatted = crate::common::format_size(info.total_size);
    
    Ok(ArchiveStats {
        format: info.format,
        total_size: info.total_size,
        total_size_formatted,
        segment_count: info.segment_count,
        entry_count: info.entry_count.map(|c| c as u64),
        encrypted_headers: info.encrypted_headers,
        aes_encrypted: info.aes_encrypted,
        is_multipart: info.is_multipart,
        ufed_detected: info.ufed_detected,
        compression_ratio,
    })
}

// =============================================================================
// Export Metadata Functions
// =============================================================================

/// Export archive metadata as JSON
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting archive metadata as JSON");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    #[derive(serde::Serialize)]
    struct ArchiveMetadata {
        format: String,
        total_size: u64,
        total_size_formatted: String,
        segment_count: u32,
        entry_count: Option<u64>,
        encrypted_headers: bool,
        aes_encrypted: bool,
        is_multipart: bool,
        ufed_detected: bool,
        compression_ratio: Option<f64>,
        entries: Option<Vec<EntryDetail>>,
    }
    
    #[derive(serde::Serialize)]
    struct EntryDetail {
        path: String,
        name: String,
        is_directory: bool,
        size: u64,
        size_formatted: String,
        compressed_size: Option<u64>,
        last_modified: String,
        crc32: Option<String>,
    }
    
    // Try to list entries for supported formats
    let entries = if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        extraction::list_zip_entries(path).ok().map(|list| {
            list.into_iter().map(|e| EntryDetail {
                path: e.path.clone(),
                name: Path::new(&e.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.path.clone()),
                is_directory: e.is_directory,
                size: e.size,
                size_formatted: crate::common::format_size(e.size),
                compressed_size: Some(e.compressed_size),
                last_modified: e.last_modified,
                crc32: Some(format!("{:08X}", e.crc32)),
            }).collect()
        })
    } else {
        None
    };
    
    let metadata = ArchiveMetadata {
        format: stats.format,
        total_size: stats.total_size,
        total_size_formatted: stats.total_size_formatted,
        segment_count: stats.segment_count,
        entry_count: stats.entry_count,
        encrypted_headers: stats.encrypted_headers,
        aes_encrypted: stats.aes_encrypted,
        is_multipart: stats.is_multipart,
        ufed_detected: stats.ufed_detected,
        compression_ratio: stats.compression_ratio,
        entries,
    };
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize archive metadata to JSON: {e}")))
}

/// Export archive metadata as CSV
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting archive metadata as CSV");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    let mut csv = String::new();
    
    // Header section
    csv.push_str("# Archive Metadata\n");
    csv.push_str(&format!("Format,{}\n", stats.format));
    csv.push_str(&format!("Total Size,{}\n", stats.total_size));
    csv.push_str(&format!("Total Size (Formatted),\"{}\"\n", stats.total_size_formatted));
    csv.push_str(&format!("Segment Count,{}\n", stats.segment_count));
    if let Some(count) = stats.entry_count {
        csv.push_str(&format!("Entry Count,{}\n", count));
    }
    csv.push_str(&format!("Encrypted Headers,{}\n", stats.encrypted_headers));
    csv.push_str(&format!("AES Encrypted,{}\n", stats.aes_encrypted));
    csv.push_str(&format!("Is Multipart,{}\n", stats.is_multipart));
    csv.push_str(&format!("UFED Detected,{}\n", stats.ufed_detected));
    if let Some(ratio) = stats.compression_ratio {
        csv.push_str(&format!("Compression Ratio,{:.4}\n", ratio));
    }
    csv.push('\n');
    
    // Try to list entries for ZIP archives
    if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        if let Ok(entries) = extraction::list_zip_entries(path) {
            csv.push_str("# Archive Entries\n");
            csv.push_str("Path,Is Directory,Size,Size (Formatted),Compressed Size,Last Modified,CRC32\n");
            for entry in entries {
                csv.push_str(&format!(
                    "\"{}\",{},{},\"{}\",{},{},{:08X}\n",
                    escape_csv(&entry.path),
                    entry.is_directory,
                    entry.size,
                    crate::common::format_size(entry.size),
                    entry.compressed_size,
                    entry.last_modified,
                    entry.crc32
                ));
            }
        }
    }
    
    Ok(csv)
}

/// Escape a value for CSV output
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

// =============================================================================
// Search Functions
// =============================================================================

/// Archive search result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveSearchResult {
    /// The matching entry
    pub entry: ArchiveEntry,
    /// How the match was found (name, extension, etc.)
    pub match_type: String,
    /// Depth in the path hierarchy
    pub depth: u32,
}

/// Search for files by name pattern in an archive
pub fn search_by_name(path: &str, pattern: &str) -> Result<Vec<ArchiveSearchResult>, ContainerError> {
    debug!(path = %path, pattern = %pattern, "Searching archive by name");
    
    let info = info(path)?;
    
    // Currently only supports ZIP
    if !matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        return Err(ContainerError::UnsupportedOperation(format!("Search not supported for format: {}", info.format)));
    }
    
    let entries = extraction::list_zip_entries(path)?;
    let pattern_lower = pattern.to_lowercase();
    
    let results: Vec<ArchiveSearchResult> = entries.into_iter()
        .filter(|entry| {
            let name = Path::new(&entry.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.clone());
            name.to_lowercase().contains(&pattern_lower)
        })
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            ArchiveSearchResult {
                entry,
                match_type: "name".to_string(),
                depth,
            }
        })
        .collect();
    
    Ok(results)
}

/// Search for files by extension in an archive
pub fn search_by_extension(path: &str, extension: &str) -> Result<Vec<ArchiveSearchResult>, ContainerError> {
    debug!(path = %path, extension = %extension, "Searching archive by extension");
    
    let info = info(path)?;
    
    // Currently only supports ZIP
    if !matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        return Err(ContainerError::UnsupportedOperation(format!("Search not supported for format: {}", info.format)));
    }
    
    let entries = extraction::list_zip_entries(path)?;
    let ext_lower = extension.to_lowercase().trim_start_matches('.').to_string();
    
    let results: Vec<ArchiveSearchResult> = entries.into_iter()
        .filter(|entry| {
            if entry.is_directory {
                return false;
            }
            if let Some(file_ext) = entry.path.rsplit('.').next() {
                file_ext.to_lowercase() == ext_lower
            } else {
                false
            }
        })
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            ArchiveSearchResult {
                entry,
                match_type: "extension".to_string(),
                depth,
            }
        })
        .collect();
    
    Ok(results)
}
