// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Core container operations (info, verify, extract)
//!
//! This module provides the main entry points for working with forensic containers.

mod export;
mod search;

// Re-export search and export functions
pub use export::{export_metadata_csv, export_metadata_json};
pub use search::search;

use std::path::Path;
use tracing::debug;

use crate::ad1;
use crate::archive;
use crate::common::audit::{log_data_export, log_evidence_access};
use crate::ewf;
use crate::raw;
use crate::ufed;

use super::companion::find_companion_log;
use super::types::{ContainerInfo, ContainerKind, StoredHash, VerifyEntry};

/// Enrich EwfInfo with L01 ltree source/record metadata.
/// Parses the ltree to extract source names, evidence numbers, file count, and total bytes.
/// Failures are logged and ignored — ltree data is supplementary.
fn enrich_l01_info(path: &str, info: &mut ewf::EwfInfo) {
    match ewf::parse_l01_file_tree(path) {
        Ok(tree) => {
            // Record summary: file count + total bytes
            if let Some(rec) = &tree.record_summary {
                info.l01_file_count = Some(rec.file_count);
                if rec.total_bytes > 0 {
                    info.l01_total_bytes = Some(rec.total_bytes);
                }
            }

            // Source info: first source provides device name + evidence number
            if let Some(src) = tree.sources.first() {
                if !src.name.is_empty() {
                    info.l01_source_name = Some(src.name.clone());
                }
                if !src.evidence_number.is_empty() {
                    info.l01_source_evidence_number = Some(src.evidence_number.clone());
                    // Also fill top-level evidence_number if the EWF header didn't have one
                    if info.evidence_number.is_none() {
                        info.evidence_number = Some(src.evidence_number.clone());
                    }
                }
            }

            debug!(
                "L01 ltree enrichment: source={:?}, ev={:?}, files={:?}, bytes={:?}",
                info.l01_source_name,
                info.l01_source_evidence_number,
                info.l01_file_count,
                info.l01_total_bytes
            );
        }
        Err(e) => {
            debug!("L01 ltree parse failed (non-fatal): {}", e);
        }
    }
}

/// Get only stored hashes from a container - minimal parsing
/// This is the fastest option for just extracting hash values.
/// Returns empty vec if no stored hashes are found.
pub fn get_stored_hashes_only(path: &str) -> Result<Vec<StoredHash>, String> {
    debug!("get_stored_hashes_only: {}", path);

    // First, try companion log - this is always fast (just text file parsing)
    if let Some(companion) = find_companion_log(path) {
        if !companion.stored_hashes.is_empty() {
            debug!(
                "Found {} hashes in companion log",
                companion.stored_hashes.len()
            );
            return Ok(companion.stored_hashes);
        }
    }

    // Detect container type for embedded hashes
    let kind = detect_container(path).map_err(|e| {
        debug!("get_stored_hashes_only: detect_container failed: {}", e);
        e
    })?;

    match kind {
        ContainerKind::E01 | ContainerKind::L01 => {
            // E01/L01 have hashes embedded - need to parse sections
            // Use the existing ewf::info which extracts stored_hashes
            match ewf::info(path) {
                Ok(info) => {
                    let hashes: Vec<StoredHash> = info
                        .stored_hashes
                        .iter()
                        .map(|h| StoredHash {
                            algorithm: h.algorithm.clone(),
                            hash: h.hash.clone(),
                            verified: h.verified,
                            timestamp: None,
                            source: Some("container".to_string()),
                            offset: None,
                            size: None,
                        })
                        .collect();
                    debug!("Found {} hashes in E01/L01", hashes.len());
                    Ok(hashes)
                }
                Err(e) => {
                    debug!("Failed to get E01/L01 hashes: {}", e);
                    Ok(vec![])
                }
            }
        }
        ContainerKind::Ufed => {
            // UFED has hashes in XML metadata
            match ufed::info(path) {
                Ok(info) => {
                    if let Some(stored) = info.stored_hashes {
                        let hashes: Vec<StoredHash> = stored
                            .iter()
                            .map(|h| {
                                StoredHash {
                                    algorithm: h.algorithm.clone(),
                                    hash: h.hash.clone(),
                                    verified: None, // UFED doesn't track verified state
                                    timestamp: h.timestamp.clone(),
                                    source: Some("container".to_string()),
                                    offset: None,
                                    size: None,
                                }
                            })
                            .collect();
                        debug!("Found {} hashes in UFED", hashes.len());
                        Ok(hashes)
                    } else {
                        Ok(vec![])
                    }
                }
                Err(e) => {
                    debug!("Failed to get UFED hashes: {}", e);
                    Ok(vec![])
                }
            }
        }
        // AD1, Raw, Archive - rely on companion log (already checked above)
        _ => {
            debug!("No embedded hashes for container type {:?}", kind);
            Ok(vec![])
        }
    }
}

/// Fast info - only reads headers, doesn't parse full item trees
/// Use this for quick container listing/display.
///
/// For archives: uses `archive::info_fast()` which skips segment discovery,
/// ZIP central directory parsing, and UFED detection inside ZIPs.
/// For UFED: uses `ufed::info_fast()` which skips full metadata extraction.
pub fn info_fast(path: &str) -> Result<ContainerInfo, String> {
    debug!("info_fast: loading {}", path);
    // Audit log: evidence container access
    log_evidence_access("info_fast", Path::new(path), None, None);
    let kind = detect_container(path).map_err(|e| {
        debug!("info_fast: detect_container failed for {}: {}", path, e);
        e
    })?;
    let companion_log = find_companion_log(path);

    match kind {
        ContainerKind::Ad1 => {
            let info = ad1::info_fast(path).map_err(|e| e.to_string())?;
            Ok(ContainerInfo {
                container: "AD1".to_string(),
                ad1: Some(info),
                e01: None,
                l01: None,
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::E01 => {
            let info = ewf::info(path)?;
            Ok(ContainerInfo {
                container: "E01".to_string(),
                ad1: None,
                e01: Some(info),
                l01: None,
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::L01 => {
            // L01 uses the same EWF format as E01 (logical evidence vs physical)
            let mut info = ewf::info(path)?;
            enrich_l01_info(path, &mut info);
            Ok(ContainerInfo {
                container: "L01".to_string(),
                ad1: None,
                e01: None,
                l01: Some(info),
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Raw => {
            let info = raw::info(path)?;
            Ok(ContainerInfo {
                container: "RAW".to_string(),
                ad1: None,
                e01: None,
                l01: None,
                raw: Some(info),
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Archive => {
            // Use info_fast — skips segment discovery, central dir parsing, UFED detection
            let info = archive::info_fast(path)?;
            Ok(ContainerInfo {
                container: format!("Archive ({})", info.format),
                ad1: None,
                e01: None,
                l01: None,
                raw: None,
                archive: Some(info),
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Ufed => {
            // Use info_fast — skips full UFED metadata extraction
            let info = ufed::info_fast(path)?;
            Ok(ContainerInfo {
                container: format!("UFED ({})", info.format),
                ad1: None,
                e01: None,
                l01: None,
                raw: None,
                archive: None,
                ufed: Some(info),
                note: None,
                companion_log,
            })
        }
    }
}

/// Full info - reads headers and optionally parses item trees
pub fn info(path: &str, include_tree: bool) -> Result<ContainerInfo, String> {
    // Audit log: evidence container access (full info)
    log_evidence_access("info", Path::new(path), None, None);

    let kind = detect_container(path)?;
    let companion_log = find_companion_log(path);

    match kind {
        ContainerKind::Ad1 => {
            let info = ad1::info(path, include_tree).map_err(|e| e.to_string())?;
            Ok(ContainerInfo {
                container: "AD1".to_string(),
                ad1: Some(info),
                e01: None,
                l01: None,
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::E01 => {
            let info = ewf::info(path)?;
            Ok(ContainerInfo {
                container: "E01".to_string(),
                ad1: None,
                e01: Some(info),
                l01: None,
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::L01 => {
            // L01 uses the same EWF format as E01 (logical evidence vs physical)
            let mut info = ewf::info(path)?;
            enrich_l01_info(path, &mut info);
            Ok(ContainerInfo {
                container: "L01".to_string(),
                ad1: None,
                e01: None,
                l01: Some(info),
                raw: None,
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Raw => {
            let info = raw::info(path)?;
            Ok(ContainerInfo {
                container: "RAW".to_string(),
                ad1: None,
                e01: None,
                l01: None,
                raw: Some(info),
                archive: None,
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Archive => {
            let info = archive::info(path)?;
            Ok(ContainerInfo {
                container: format!("Archive ({})", info.format),
                ad1: None,
                e01: None,
                l01: None,
                raw: None,
                archive: Some(info),
                ufed: None,
                note: None,
                companion_log,
            })
        }
        ContainerKind::Ufed => {
            let info = ufed::info(path)?;
            Ok(ContainerInfo {
                container: format!("UFED ({})", info.format),
                ad1: None,
                e01: None,
                l01: None,
                raw: None,
                archive: None,
                ufed: Some(info),
                note: None,
                companion_log,
            })
        }
    }
}

/// Verify container integrity using the specified hash algorithm
pub fn verify(path: &str, algorithm: &str) -> Result<Vec<VerifyEntry>, String> {
    // Audit log: verification operation
    log_evidence_access("verify", Path::new(path), Some(algorithm), None);

    match detect_container(path)? {
        ContainerKind::Ad1 => {
            let ad1_results = ad1::verify(path, algorithm).map_err(|e| e.to_string())?;
            Ok(ad1_results
                .into_iter()
                .map(|entry| VerifyEntry {
                    path: Some(entry.path),
                    chunk_index: None,
                    status: entry.status.to_string(),
                    message: None,
                })
                .collect())
        }
        ContainerKind::E01 => {
            let ewf_results = ewf::verify_chunks(path, algorithm)?;
            Ok(ewf_results
                .into_iter()
                .map(|entry| VerifyEntry {
                    path: None,
                    chunk_index: Some(entry.chunk_index),
                    status: entry.status,
                    message: entry.message,
                })
                .collect())
        }
        ContainerKind::L01 => {
            // L01 uses the same EWF format - use ewf::verify_chunks
            let ewf_results = ewf::verify_chunks(path, algorithm)?;
            Ok(ewf_results
                .into_iter()
                .map(|entry| VerifyEntry {
                    path: None,
                    chunk_index: Some(entry.chunk_index),
                    status: entry.status,
                    message: entry.message,
                })
                .collect())
        }
        ContainerKind::Raw => {
            let computed_hash = raw::verify(path, algorithm)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), computed_hash)),
            }])
        }
        ContainerKind::Archive => {
            // Use archive hash verification
            let hash = archive::verify(path, algorithm)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
        ContainerKind::Ufed => {
            // Use UFED hash verification
            let hash = ufed::verify(path, algorithm)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
    }
}

/// Verify container integrity with progress callback
pub fn verify_with_progress<F>(
    path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<Vec<VerifyEntry>, String>
where
    F: FnMut(u64, u64),
{
    log_evidence_access(
        "verify_with_progress",
        Path::new(path),
        Some(algorithm),
        None,
    );

    match detect_container(path)? {
        ContainerKind::Ad1 => {
            // Wrap the u64 callback for AD1's usize signature
            let ad1_results = ad1::verify_with_progress(path, algorithm, &mut progress_callback)
                .map_err(|e| e.to_string())?;
            Ok(ad1_results
                .into_iter()
                .map(|entry| VerifyEntry {
                    path: Some(entry.path),
                    chunk_index: None,
                    status: entry.status.to_string(),
                    message: None,
                })
                .collect())
        }
        ContainerKind::E01 | ContainerKind::L01 => {
            // Wrap the u64 callback for EWF's usize signature
            let hash = ewf::verify_with_progress(path, algorithm, &mut progress_callback)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
        ContainerKind::Raw => {
            let hash = raw::verify_with_progress(path, algorithm, progress_callback)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
        ContainerKind::Archive => {
            let hash = archive::verify_with_progress(path, algorithm, progress_callback)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
        ContainerKind::Ufed => {
            let hash = ufed::verify_with_progress(path, algorithm, progress_callback)?;
            Ok(vec![VerifyEntry {
                path: None,
                chunk_index: None,
                status: "computed".to_string(),
                message: Some(format!("{}: {}", algorithm.to_uppercase(), hash)),
            }])
        }
    }
}

/// Extract container contents to the specified output directory
pub fn extract(path: &str, output_dir: &str) -> Result<(), String> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract container contents with progress callback
pub fn extract_with_progress<F>(
    path: &str,
    output_dir: &str,
    mut progress_callback: F,
) -> Result<(), String>
where
    F: FnMut(u64, u64),
{
    // Audit log: extraction operation (sensitive - exports evidence)
    log_data_export(
        Path::new(path),
        Path::new(output_dir),
        0, // Size determined during extraction
    );

    match detect_container(path)? {
        ContainerKind::Ad1 => {
            // Wrap the u64 callback for AD1's usize signature
            ad1::extract_with_progress(path, output_dir, &mut progress_callback)
                .map_err(|e| e.to_string())
        }
        ContainerKind::E01 | ContainerKind::L01 => {
            ewf::extract_with_progress(path, output_dir, progress_callback)
                .map_err(|e| e.to_string())
        }
        ContainerKind::Raw => raw::extract_with_progress(path, output_dir, progress_callback)
            .map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            archive::extract_with_progress(path, output_dir, progress_callback)?;
            Ok(())
        }
        ContainerKind::Ufed => {
            ufed::extract_with_progress(path, output_dir, progress_callback)?;
            Ok(())
        }
    }
}

// =============================================================================
// Statistics Functions
// =============================================================================

/// Unified container statistics
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ContainerStats {
    /// Container type identifier
    pub container_type: String,
    /// Total size in bytes
    pub total_size: u64,
    /// Formatted total size
    pub total_size_formatted: String,
    /// Number of segments (for multi-file containers)
    pub segment_count: u32,
    /// Number of entries (for tree-based containers)
    pub entry_count: Option<u64>,
    /// Whether the container has stored hashes
    pub has_stored_hashes: bool,
    /// Additional format-specific stats (serialized)
    pub format_specific: Option<String>,
}

impl ContainerStats {
    /// Create new ContainerStats with type and size
    #[inline]
    pub fn new(container_type: impl Into<String>, total_size: u64) -> Self {
        Self {
            container_type: container_type.into(),
            total_size,
            total_size_formatted: crate::common::format_size(total_size),
            segment_count: 1,
            ..Default::default()
        }
    }

    /// Set segment count
    #[inline]
    pub fn with_segments(mut self, count: u32) -> Self {
        self.segment_count = count;
        self
    }

    /// Set entry count
    #[inline]
    pub fn with_entries(mut self, count: u64) -> Self {
        self.entry_count = Some(count);
        self
    }

    /// Mark as having stored hashes
    #[inline]
    pub fn with_stored_hashes(mut self) -> Self {
        self.has_stored_hashes = true;
        self
    }

    /// Set format-specific data
    #[inline]
    pub fn with_format_specific<T: serde::Serialize>(mut self, data: &T) -> Self {
        self.format_specific = serde_json::to_string(data).ok();
        self
    }
}

/// Get unified container statistics
pub fn get_stats(path: &str) -> Result<ContainerStats, String> {
    debug!(path = %path, "Getting container stats");

    match detect_container(path)? {
        ContainerKind::Ad1 => {
            let stats = ad1::get_stats(path).map_err(|e| e.to_string())?;
            let info = ad1::info_fast(path).map_err(|e| e.to_string())?;
            let segment_count = info
                .segment_files
                .as_ref()
                .map(|f| f.len() as u32)
                .unwrap_or(1);
            Ok(ContainerStats {
                container_type: "AD1".to_string(),
                total_size: stats.total_size,
                total_size_formatted: crate::common::format_size(stats.total_size),
                segment_count,
                entry_count: Some(stats.total_files + stats.total_folders),
                has_stored_hashes: true,
                format_specific: serde_json::to_string(&stats).ok(),
            })
        }
        ContainerKind::E01 | ContainerKind::L01 => {
            let stats = ewf::get_stats(path)?;
            Ok(ContainerStats {
                container_type: if path.to_lowercase().contains(".l01") {
                    "L01"
                } else {
                    "E01"
                }
                .to_string(),
                total_size: stats.total_size,
                total_size_formatted: crate::common::format_size(stats.total_size),
                segment_count: stats.total_segments,
                entry_count: None,
                has_stored_hashes: stats.stored_hash_count > 0,
                format_specific: serde_json::to_string(&stats).ok(),
            })
        }
        ContainerKind::Raw => {
            let stats = raw::get_stats(path)?;
            let format_specific = serde_json::to_string(&stats).ok();
            Ok(ContainerStats {
                container_type: "RAW".to_string(),
                total_size: stats.total_size,
                total_size_formatted: stats.total_size_formatted,
                segment_count: stats.segment_count,
                entry_count: None,
                has_stored_hashes: false,
                format_specific,
            })
        }
        ContainerKind::Archive => {
            let stats = archive::get_stats(path)?;
            let format_specific = serde_json::to_string(&stats).ok();
            Ok(ContainerStats {
                container_type: format!("Archive ({})", stats.format),
                total_size: stats.total_size,
                total_size_formatted: stats.total_size_formatted,
                segment_count: stats.segment_count,
                entry_count: stats.entry_count,
                has_stored_hashes: false,
                format_specific,
            })
        }
        ContainerKind::Ufed => {
            let stats = ufed::get_stats(path)?;
            Ok(ContainerStats {
                container_type: format!("UFED ({})", stats.format),
                total_size: stats.file_size,
                total_size_formatted: crate::common::format_size(stats.file_size),
                segment_count: stats.associated_file_count as u32 + 1,
                entry_count: None,
                has_stored_hashes: stats.stored_hash_count > 0,
                format_specific: serde_json::to_string(&stats).ok(),
            })
        }
    }
}

// =============================================================================
// Segment Functions
// =============================================================================

/// Get segment paths for multi-file containers
pub fn get_segment_paths(path: &str) -> Result<Vec<std::path::PathBuf>, String> {
    debug!(path = %path, "Getting segment paths");

    match detect_container(path)? {
        ContainerKind::Ad1 => ad1::get_segment_paths(path).map_err(|e| e.to_string()),
        ContainerKind::E01 | ContainerKind::L01 => {
            ewf::get_segment_paths(path).map_err(|e| e.to_string())
        }
        ContainerKind::Raw => raw::get_segment_paths(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Archives typically don't have segments, return single path
            Ok(vec![std::path::PathBuf::from(path)])
        }
        ContainerKind::Ufed => ufed::get_segment_paths(path).map_err(|e| e.to_string()),
    }
}

/// Detect the container type from the file path and magic bytes
pub(crate) fn detect_container(path: &str) -> Result<ContainerKind, String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("Input file not found: {path}"));
    }

    let lower = path.to_lowercase();

    // Check Cellebrite UFED formats first (UFD, UFDR, UFDX)
    if ufed::is_ufed(path) {
        return Ok(ContainerKind::Ufed);
    }

    // Check E01/EWF first (before L01 to avoid .lx01 confusion)
    // Support .e01, .ex01, .e02, .e03, etc., and .ewf extensions
    if lower.ends_with(".e01")
        || lower.ends_with(".ex01")
        || lower.ends_with(".ewf")
        || lower.contains(".e0")
        || lower.contains(".ex")
    {
        debug!("Checking E01 signature for: {}", path);
        if ewf::is_e01(path).unwrap_or(false) {
            return Ok(ContainerKind::E01);
        } else {
            debug!("E01 signature check failed for: {}", path);
        }
    }

    // Check L01/Lx01 - use ewf::is_l01_file for proper LVF signature detection
    if (lower.ends_with(".l01")
        || lower.ends_with(".lx01")
        || lower.contains(".l0")
        || lower.contains(".lx"))
        && ewf::is_l01_file(path).unwrap_or(false)
    {
        return Ok(ContainerKind::L01);
    }

    // Check AD1
    if ad1::is_ad1(path)? {
        return Ok(ContainerKind::Ad1);
    }

    // Check archive formats (7z, ZIP, RAR, etc.) - before raw to catch .7z.001 properly
    if archive::is_archive(path).unwrap_or(false) {
        return Ok(ContainerKind::Archive);
    }

    // Check raw disk images (.dd, .raw, .img, .001, .002, etc.)
    if raw::is_raw(path).unwrap_or(false) {
        return Ok(ContainerKind::Raw);
    }

    Err(format!(
        "Unsupported or unrecognized container format: {}\n\
         The file could not be identified as any supported forensic container type.\n\
         Supported formats:\n\
         • Evidence containers: AD1, E01/Ex01 (EWF), L01/Lx01 (logical EWF)\n\
         • Disk images: RAW (.dd, .raw, .img, .001)\n\
         • Archives: 7z, ZIP, RAR, TAR (GZ/BZ2/XZ), DMG, ISO\n\
         • Mobile: UFED (UFD, UFDR, UFDX)\n\
         Please verify the file is not corrupted or in an unsupported format.",
        path
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ContainerKind tests ====================

    #[test]
    fn test_container_kind_variants() {
        // Ensure all variants can be constructed
        let _ = ContainerKind::Ad1;
        let _ = ContainerKind::E01;
        let _ = ContainerKind::L01;
        let _ = ContainerKind::Raw;
        let _ = ContainerKind::Archive;
        let _ = ContainerKind::Ufed;
    }

    // ==================== detect_container tests ====================

    #[test]
    fn test_detect_container_nonexistent_file() {
        let result = detect_container("/nonexistent/path/to/file.e01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_detect_container_unsupported_format() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a file with unsupported extension and non-matching magic bytes
        let temp = NamedTempFile::with_suffix(".xyz").unwrap();
        let mut file = File::create(temp.path()).unwrap();
        // Write non-matching magic bytes to avoid false detection
        file.write_all(b"XYZ random content that doesn't match any known format")
            .unwrap();

        let result = detect_container(temp.path().to_str().unwrap());
        // Should be an error (unsupported format)
        assert!(result.is_err());
    }

    // ==================== info_fast error handling tests ====================

    #[test]
    fn test_info_fast_nonexistent() {
        let result = info_fast("/nonexistent/file.e01");
        assert!(result.is_err());
    }

    #[test]
    fn test_info_nonexistent() {
        let result = info("/nonexistent/file.ad1", false);
        assert!(result.is_err());
    }

    // ==================== verify error handling tests ====================

    #[test]
    fn test_verify_nonexistent() {
        let result = verify("/nonexistent/file.e01", "sha256");
        assert!(result.is_err());
    }

    // ==================== extract error handling tests ====================

    #[test]
    fn test_extract_nonexistent() {
        let result = extract("/nonexistent/file.ad1", "/tmp/output");
        assert!(result.is_err());
    }
}
