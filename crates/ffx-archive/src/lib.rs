// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive container support (7z, ZIP, RAR, TAR, ISO, etc.)
//!
//! This crate provides archive detection, metadata extraction, content listing,
//! extraction, and verification for common archive formats used in forensic workflows.

pub mod detection;
pub mod extraction;
pub mod libarchive_backend;
pub mod rar;
pub mod segments;
pub mod sevenz;
pub mod tar;
pub mod types;
pub mod vfs;
pub mod zip;
pub mod zip_index;

use ffx_errors::ContainerError;

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::OnceLock;
use tracing::debug;

use ffx_common::escape_csv;
use ffx_common::hash::{HashAlgorithm, StreamingHasher};

// Re-exports for convenience
pub use detection::{detect_archive_format, is_7z_segment, is_archive};
pub use extraction::{
    extract, extract_gzip, extract_with_progress, extract_zip, extract_zip_entry,
    extract_zip_with_progress, get_zip_children_at_path, get_zip_entry_count, get_zip_root_entries,
    list_zip_entries, ArchiveEntry, ExtractError, ExtractResult,
};
pub use libarchive_backend::{
    detect_format as libarchive_detect_format, is_supported_archive as libarchive_is_supported,
    list_all_entries as libarchive_list_all, list_root as libarchive_list_root,
    quick_summary as libarchive_summary, read_file as libarchive_read_file,
    read_file_encrypted as libarchive_read_encrypted, ArchiveEntryInfo, LibarchiveHandler,
};
pub use sevenz::is_split_archive;
pub use types::{ArchiveFormat, ArchiveInfo};
pub use vfs::ArchiveVfs;
pub use zip_index::{ZipIndex, ZipIndexEntry};

// =============================================================================
// UFED Detection Bridge (decouples archive from ufed module)
// =============================================================================

type UfedDetector = dyn Fn(&str) -> Result<(bool, Vec<String>), String> + Send + Sync;

static UFED_DETECTOR: OnceLock<Box<UfedDetector>> = OnceLock::new();

/// Register a UFED-in-ZIP detector function.
/// Called by the host app shim to inject the real `ufed::detect_in_zip`.
pub fn register_ufed_detector(detector: Box<UfedDetector>) {
    let _ = UFED_DETECTOR.set(detector);
}

/// Detect UFED content in a ZIP archive using the registered detector.
fn detect_ufed_in_zip(path: &str) -> (bool, Vec<String>) {
    UFED_DETECTOR
        .get()
        .and_then(|f| f(path).ok())
        .unwrap_or((false, vec![]))
}

// =============================================================================
// Archive Hash Verification
// =============================================================================

/// Compute hash of archive file
pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Compute hash of archive file with progress callback
pub fn verify_with_progress<F>(
    path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    use ffx_common::{BUFFER_SIZE, MMAP_THRESHOLD};

    debug!(path = %path, algorithm = %algorithm, "Computing archive hash (optimized)");

    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let total_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();

    let algorithm_lower = algorithm.to_lowercase();

    let report_interval = (total_size / 50).max(BUFFER_SIZE as u64);
    let mut bytes_processed: u64 = 0;
    let mut last_report: u64 = 0;

    // BLAKE3: Use memory-mapped I/O + rayon parallel hashing
    if algorithm_lower == "blake3" {
        use memmap2::Mmap;

        let mut hasher = blake3::Hasher::new();

        if total_size >= MMAP_THRESHOLD {
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map file: {e}"))?;

            for chunk in mmap.chunks(BUFFER_SIZE) {
                hasher.update_rayon(chunk);
                bytes_processed += chunk.len() as u64;

                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

            loop {
                let buf = reader.fill_buf().map_err(|e| format!("Read error: {e}"))?;
                let len = buf.len();
                if len == 0 {
                    break;
                }

                hasher.update_rayon(buf);
                reader.consume(len);

                bytes_processed += len as u64;
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        }

        progress_callback(total_size, total_size);
        let hash = hasher.finalize().to_hex().to_string();
        debug!(hash = %hash, "Archive hash computed (BLAKE3 optimized)");
        return Ok(hash);
    }

    // XXH3: Use memory-mapped I/O for maximum speed
    if algorithm_lower == "xxh3" || algorithm_lower == "xxhash3" {
        use memmap2::Mmap;
        use xxhash_rust::xxh3::Xxh3;

        let mut hasher = Xxh3::new();

        if total_size >= MMAP_THRESHOLD {
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map file: {e}"))?;

            for chunk in mmap.chunks(BUFFER_SIZE) {
                hasher.update(chunk);
                bytes_processed += chunk.len() as u64;

                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

            loop {
                let buf = reader.fill_buf().map_err(|e| format!("Read error: {e}"))?;
                let len = buf.len();
                if len == 0 {
                    break;
                }

                hasher.update(buf);
                reader.consume(len);

                bytes_processed += len as u64;
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        }

        progress_callback(total_size, total_size);
        let hash = format!("{:032x}", hasher.digest128());
        debug!(hash = %hash, "Archive hash computed (XXH3 optimized)");
        return Ok(hash);
    }

    // Other algorithms: Use optimized buffered I/O with 16MB buffer
    let algo = algorithm
        .parse::<HashAlgorithm>()
        .map_err(|e| format!("Unsupported algorithm: {}", e))?;
    let mut hasher = StreamingHasher::new(algo);

    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

    loop {
        let buf = reader
            .fill_buf()
            .map_err(|e| format!("Read error: {}", e))?;
        let len = buf.len();
        if len == 0 {
            break;
        }

        hasher.update(buf);
        reader.consume(len);

        bytes_processed += len as u64;
        if bytes_processed - last_report >= report_interval {
            progress_callback(bytes_processed, total_size);
            last_report = bytes_processed;
        }
    }

    progress_callback(total_size, total_size);
    let hash = hasher.finalize();
    debug!(hash = %hash, "Archive hash computed");
    Ok(hash)
}

// =============================================================================
// Entry Verification
// =============================================================================

/// Per-entry verification result
#[derive(Debug, Clone, serde::Serialize)]
pub struct EntryVerifyResult {
    pub path: String,
    pub is_directory: bool,
    pub expected_crc32: u32,
    pub computed_crc32: Option<u32>,
    pub crc_match: bool,
    pub size: u64,
    pub error: Option<String>,
}

/// Verify all entries in a ZIP archive by checking CRC32
pub fn verify_entries(path: &str) -> Result<Vec<EntryVerifyResult>, ContainerError> {
    verify_entries_with_progress(path, |_, _| {})
}

/// Verify all entries in a ZIP archive with progress callback
pub fn verify_entries_with_progress<F>(
    path: &str,
    progress_callback: F,
) -> Result<Vec<EntryVerifyResult>, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, "Verifying archive entries");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive not found: {}",
            path
        )));
    }

    let format = detection::detect_archive_format(path)?.ok_or_else(|| {
        ContainerError::InvalidFormat(format!("Unable to detect archive format: {}", path))
    })?;

    match format {
        types::ArchiveFormat::Zip | types::ArchiveFormat::Zip64 => {
            verify_zip_entries_with_progress(path, progress_callback)
        }
        _ => Err(ContainerError::UnsupportedOperation(format!(
            "Entry verification not supported for format: {}",
            format
        ))),
    }
}

fn verify_zip_entries_with_progress<F>(
    path: &str,
    mut progress_callback: F,
) -> Result<Vec<EntryVerifyResult>, ContainerError>
where
    F: FnMut(u64, u64),
{
    use std::hash::Hasher;

    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ::zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

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
                crc_match: true,
                size: 0,
                error: None,
            });
        } else {
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

// =============================================================================
// Archive Info
// =============================================================================

/// Get archive information including segment discovery
pub fn info(path: &str) -> Result<ArchiveInfo, ContainerError> {
    debug!(path = %path, "Getting archive info");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive file not found: {path}"
        )));
    }

    let format = detection::detect_archive_format(path)?
        .ok_or_else(|| format!("Unable to detect archive format: {path}"))?;

    let format_str = format.to_string();

    let (segment_names, segment_sizes) = segments::discover_segments(path, format)?;
    let segment_count = segment_names.len() as u32;
    let total_size: u64 = segment_sizes.iter().sum();

    let first_segment = segment_names.first().cloned().unwrap_or_default();
    let last_segment = segment_names.last().cloned().unwrap_or_default();
    let is_multipart = segment_count > 1;

    let (entry_count, central_dir_offset, central_dir_size, mut encrypted_headers, aes_encrypted) =
        match format {
            ArchiveFormat::Zip | ArchiveFormat::Zip64 => {
                let meta = zip::parse_metadata(path).unwrap_or_default();
                (
                    meta.entry_count,
                    meta.central_dir_offset,
                    meta.central_dir_size,
                    meta.encrypted_headers,
                    meta.aes_encrypted,
                )
            }
            _ => (None, None, None, false, false),
        };

    let (
        next_header_offset,
        next_header_size,
        version,
        start_header_crc_valid,
        next_header_crc,
        sevenz_encrypted,
    ) = match format {
        ArchiveFormat::SevenZip => {
            let meta = sevenz::parse_metadata(path).unwrap_or_default();
            (
                meta.next_header_offset,
                meta.next_header_size,
                meta.version,
                meta.start_header_crc_valid,
                meta.next_header_crc,
                meta.encrypted,
            )
        }
        _ => (None, None, None, None, None, false),
    };

    if sevenz_encrypted {
        encrypted_headers = true;
    }

    // Detect UFED files via registered detector (decoupled from ufed module)
    let (ufed_detected, ufed_files) = match format {
        ArchiveFormat::Zip | ArchiveFormat::Zip64 => detect_ufed_in_zip(path),
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
pub fn info_fast(path: &str) -> Result<ArchiveInfo, ContainerError> {
    debug!(path = %path, "Getting fast archive info");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Archive file not found: {path}"
        )));
    }

    let format = detection::detect_archive_format(path)?
        .ok_or_else(|| format!("Unable to detect archive format: {path}"))?;

    let format_str = format.to_string();

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
        entry_count: None,
        encrypted_headers: false,
        aes_encrypted: false,
        central_dir_offset: None,
        central_dir_size: None,
        next_header_offset: None,
        next_header_size: None,
        version: None,
        start_header_crc_valid: None,
        next_header_crc: None,
        ufed_detected: false,
        ufed_files: vec![],
    })
}

// =============================================================================
// Archive Statistics
// =============================================================================

/// Archive container statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ArchiveStats {
    pub format: String,
    pub total_size: u64,
    pub total_size_formatted: String,
    pub segment_count: u32,
    pub entry_count: Option<u64>,
    pub encrypted_headers: bool,
    pub aes_encrypted: bool,
    pub is_multipart: bool,
    pub ufed_detected: bool,
    pub compression_ratio: Option<f64>,
}

impl ArchiveStats {
    #[inline]
    pub fn new(format: impl Into<String>, total_size: u64) -> Self {
        let format = format.into();
        Self {
            format,
            total_size,
            total_size_formatted: ffx_common::format_size(total_size),
            segment_count: 1,
            ..Default::default()
        }
    }

    #[inline]
    pub fn with_entries(mut self, count: u64) -> Self {
        self.entry_count = Some(count);
        self
    }

    #[inline]
    pub fn with_encryption(mut self, encrypted_headers: bool, aes: bool) -> Self {
        self.encrypted_headers = encrypted_headers;
        self.aes_encrypted = aes;
        self
    }

    #[inline]
    pub fn multipart(mut self, segment_count: u32) -> Self {
        self.is_multipart = segment_count > 1;
        self.segment_count = segment_count;
        self
    }

    #[inline]
    pub fn with_ufed(mut self, detected: bool) -> Self {
        self.ufed_detected = detected;
        self
    }

    #[inline]
    pub fn with_compression_ratio(mut self, ratio: f64) -> Self {
        self.compression_ratio = Some(ratio);
        self
    }
}

/// Get archive statistics
pub fn get_stats(path: &str) -> Result<ArchiveStats, ContainerError> {
    debug!(path = %path, "Getting archive stats");

    let info = info(path)?;

    let compression_ratio = if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
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

    let total_size_formatted = ffx_common::format_size(info.total_size);

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

    let entries = if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        extraction::list_zip_entries(path).ok().map(|list| {
            list.into_iter()
                .map(|e| EntryDetail {
                    path: e.path.clone(),
                    name: Path::new(&e.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| e.path.clone()),
                    is_directory: e.is_directory,
                    size: e.size,
                    size_formatted: ffx_common::format_size(e.size),
                    compressed_size: Some(e.compressed_size),
                    last_modified: e.last_modified,
                    crc32: Some(format!("{:08X}", e.crc32)),
                })
                .collect()
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

    serde_json::to_string_pretty(&metadata).map_err(|e| {
        ContainerError::SerializationError(format!(
            "Failed to serialize archive metadata to JSON: {e}"
        ))
    })
}

/// Export archive metadata as CSV
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    debug!(path = %path, "Exporting archive metadata as CSV");

    let info = info(path)?;
    let stats = get_stats(path)?;

    let mut csv = String::new();

    csv.push_str("# Archive Metadata\n");
    csv.push_str(&format!("Format,{}\n", stats.format));
    csv.push_str(&format!("Total Size,{}\n", stats.total_size));
    csv.push_str(&format!(
        "Total Size (Formatted),\"{}\"\n",
        stats.total_size_formatted
    ));
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

    if matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        if let Ok(entries) = extraction::list_zip_entries(path) {
            csv.push_str("# Archive Entries\n");
            csv.push_str(
                "Path,Is Directory,Size,Size (Formatted),Compressed Size,Last Modified,CRC32\n",
            );
            for entry in entries {
                csv.push_str(&format!(
                    "\"{}\",{},{},\"{}\",{},{},{:08X}\n",
                    escape_csv(&entry.path),
                    entry.is_directory,
                    entry.size,
                    ffx_common::format_size(entry.size),
                    entry.compressed_size,
                    entry.last_modified,
                    entry.crc32
                ));
            }
        }
    }

    Ok(csv)
}

// =============================================================================
// Search Functions
// =============================================================================

/// Archive search result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveSearchResult {
    pub entry: ArchiveEntry,
    pub match_type: String,
    pub depth: u32,
}

/// Search for files by name pattern in an archive
pub fn search_by_name(
    path: &str,
    pattern: &str,
) -> Result<Vec<ArchiveSearchResult>, ContainerError> {
    debug!(path = %path, pattern = %pattern, "Searching archive by name");

    let info = info(path)?;

    if !matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        return Err(ContainerError::UnsupportedOperation(format!(
            "Search not supported for format: {}",
            info.format
        )));
    }

    let entries = extraction::list_zip_entries(path)?;
    let pattern_lower = pattern.to_lowercase();

    let results: Vec<ArchiveSearchResult> = entries
        .into_iter()
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
pub fn search_by_extension(
    path: &str,
    extension: &str,
) -> Result<Vec<ArchiveSearchResult>, ContainerError> {
    debug!(path = %path, extension = %extension, "Searching archive by extension");

    let info = info(path)?;

    if !matches!(info.format.as_str(), "ZIP" | "ZIP64") {
        return Err(ContainerError::UnsupportedOperation(format!(
            "Search not supported for format: {}",
            info.format
        )));
    }

    let entries = extraction::list_zip_entries(path)?;
    let ext_lower = extension.to_lowercase().trim_start_matches('.').to_string();

    let results: Vec<ArchiveSearchResult> = entries
        .into_iter()
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

// =============================================================================
// Native Fallback Reader (pure-Rust crates, Windows-safe)
// =============================================================================

/// Read an entry from an archive using pure-Rust crates, bypassing libarchive.
///
/// This is used as a fallback when libarchive fails (e.g., Windows builds
/// without vcpkg lack decompression support). Dispatches to the `zip`,
/// `sevenz-rust`, or `tar`+decompression crates based on the detected format.
pub fn read_entry_native(archive_path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let format = detection::detect_archive_format(archive_path)?;
    let normalized = entry_path.replace('\\', "/");
    let search = normalized.trim_start_matches('/').trim_end_matches('/');
    let path_lower = archive_path.to_lowercase();

    match format {
        Some(ArchiveFormat::Zip) | Some(ArchiveFormat::Zip64) => {
            read_zip_entry_native(archive_path, search)
        }
        Some(ArchiveFormat::SevenZip) => read_7z_entry_native(archive_path, search),
        Some(ArchiveFormat::TarGz) => read_tar_entry_native(archive_path, search, "gz"),
        Some(ArchiveFormat::Gzip) => {
            if path_lower.contains(".tar.gz") || path_lower.ends_with(".tgz") {
                read_tar_entry_native(archive_path, search, "gz")
            } else {
                read_compressed_stream(archive_path, "gz")
            }
        }
        Some(ArchiveFormat::Bzip2) => {
            if path_lower.contains(".tar.bz2")
                || path_lower.ends_with(".tbz2")
                || path_lower.ends_with(".tbz")
            {
                read_tar_entry_native(archive_path, search, "bz2")
            } else {
                read_compressed_stream(archive_path, "bz2")
            }
        }
        Some(ArchiveFormat::Xz) => {
            if path_lower.contains(".tar.xz") || path_lower.ends_with(".txz") {
                read_tar_entry_native(archive_path, search, "xz")
            } else {
                read_compressed_stream(archive_path, "xz")
            }
        }
        Some(ArchiveFormat::Zstd) => {
            if path_lower.contains(".tar.zst") || path_lower.ends_with(".tzst") {
                read_tar_entry_native(archive_path, search, "zst")
            } else {
                read_compressed_stream(archive_path, "zst")
            }
        }
        Some(ArchiveFormat::Tar) => read_tar_entry_native(archive_path, search, "none"),
        _ => Err(ContainerError::from(format!(
            "No native reader for format: {:?}",
            format
        ))),
    }
}

/// Read a single entry from a ZIP archive using the `zip` crate.
fn read_zip_entry_native(archive_path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let file =
        File::open(archive_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let mut archive =
        ::zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

    let with_slash = format!("{}/", entry_path);

    let entry_index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index(i)
                .map(|e: ::zip::read::ZipFile| {
                    let name = e.name().replace('\\', "/");
                    let name = name.trim_start_matches('/').trim_end_matches('/');
                    name == entry_path || e.name() == with_slash
                })
                .unwrap_or(false)
        })
        .ok_or_else(|| {
            ContainerError::from(format!("Entry not found in ZIP: {}", entry_path))
        })?;

    let mut entry = archive
        .by_index(entry_index)
        .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

    let mut data = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut data)
        .map_err(|e| format!("Failed to decompress ZIP entry: {}", e))?;

    debug!(path = %entry_path, bytes = data.len(), "Read ZIP entry (native)");
    Ok(data)
}

/// Read a single entry from a 7z archive using sevenz-rust.
fn read_7z_entry_native(
    archive_path: &str,
    entry_path: &str,
) -> Result<Vec<u8>, ContainerError> {
    use sevenz_rust::{Password, SevenZReader};

    let file =
        File::open(archive_path).map_err(|e| format!("Failed to open 7z: {}", e))?;
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);

    let mut archive = SevenZReader::new(file, file_size, Password::empty())
        .map_err(|e| format!("Failed to read 7z: {}", e))?;

    let mut result_data: Option<Vec<u8>> = None;

    archive
        .for_each_entries(|entry, reader| {
            let name = entry.name().replace('\\', "/");
            let name = name.trim_start_matches('/').trim_end_matches('/');
            if name == entry_path {
                let mut data = Vec::new();
                reader.read_to_end(&mut data)?;
                result_data = Some(data);
            }
            Ok(true)
        })
        .map_err(|e| format!("Failed to read 7z entries: {}", e))?;

    match result_data {
        Some(data) => {
            debug!(path = %entry_path, bytes = data.len(), "Read 7z entry (native)");
            Ok(data)
        }
        None => Err(ContainerError::from(format!(
            "Entry not found in 7z: {}",
            entry_path
        ))),
    }
}

/// Read a single entry from a TAR archive (optionally compressed).
fn read_tar_entry_native(
    archive_path: &str,
    entry_path: &str,
    compression: &str,
) -> Result<Vec<u8>, ContainerError> {
    let file =
        File::open(archive_path).map_err(|e| format!("Failed to open TAR: {}", e))?;
    let reader = BufReader::new(file);

    match compression {
        "gz" => {
            let decoder = flate2::read::GzDecoder::new(reader);
            read_from_tar(decoder, entry_path)
        }
        "bz2" => {
            let decoder = bzip2::read::BzDecoder::new(reader);
            read_from_tar(decoder, entry_path)
        }
        "xz" => {
            let decoder = xz2::read::XzDecoder::new(reader);
            read_from_tar(decoder, entry_path)
        }
        "zst" => {
            let decoder = zstd::stream::read::Decoder::new(reader)
                .map_err(|e| format!("Failed to create zstd decoder: {}", e))?;
            read_from_tar(decoder, entry_path)
        }
        "none" => read_from_tar(reader, entry_path),
        _ => Err(ContainerError::from(format!(
            "Unknown TAR compression: {}",
            compression
        ))),
    }
}

/// Read a single entry from a tar stream.
fn read_from_tar<R: Read>(reader: R, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let mut archive = ::tar::Archive::new(reader);

    for entry_result in archive
        .entries()
        .map_err(|e| format!("Failed to list TAR entries: {}", e))?
    {
        let mut entry =
            entry_result.map_err(|e| format!("Failed to read TAR entry: {}", e))?;
        let path = entry
            .path()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let normalized = path
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();

        if normalized == entry_path {
            let mut data = Vec::new();
            entry
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to read TAR entry data: {}", e))?;
            debug!(path = %entry_path, bytes = data.len(), "Read TAR entry (native)");
            return Ok(data);
        }
    }

    Err(ContainerError::from(format!(
        "Entry not found in TAR: {}",
        entry_path
    )))
}

/// Decompress a standalone compressed file (not a TAR archive).
/// Used for .gz, .bz2, .xz, .zst files that contain a single stream.
fn read_compressed_stream(
    archive_path: &str,
    compression: &str,
) -> Result<Vec<u8>, ContainerError> {
    let file = File::open(archive_path)
        .map_err(|e| format!("Failed to open {}: {}", archive_path, e))?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();

    match compression {
        "gz" => {
            let mut decoder = flate2::read::GzDecoder::new(reader);
            decoder
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to decompress gz: {}", e))?;
        }
        "bz2" => {
            let mut decoder = bzip2::read::BzDecoder::new(reader);
            decoder
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to decompress bz2: {}", e))?;
        }
        "xz" => {
            let mut decoder = xz2::read::XzDecoder::new(reader);
            decoder
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to decompress xz: {}", e))?;
        }
        "zst" => {
            let mut decoder = zstd::stream::read::Decoder::new(reader)
                .map_err(|e| format!("Failed to init zstd decoder: {}", e))?;
            decoder
                .read_to_end(&mut data)
                .map_err(|e| format!("Failed to decompress zst: {}", e))?;
        }
        _ => {
            return Err(ContainerError::from(format!(
                "Unsupported compression for stream read: {}",
                compression
            )));
        }
    }

    debug!(path = %archive_path, bytes = data.len(), "Read compressed stream (native)");
    Ok(data)
}
