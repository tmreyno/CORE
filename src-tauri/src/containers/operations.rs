// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Core container operations (info, verify, extract)
//!
//! This module provides the main entry points for working with forensic containers.

use tracing::debug;
use std::path::Path;

use crate::ad1;
use crate::archive;
use crate::common::audit::{log_evidence_access, log_data_export};
use crate::ewf;
use crate::raw;
use crate::ufed;

use super::types::{ContainerInfo, ContainerKind, StoredHash, VerifyEntry};
use super::companion::find_companion_log;

/// Get only stored hashes from a container - minimal parsing
/// This is the fastest option for just extracting hash values.
/// Returns empty vec if no stored hashes are found.
pub fn get_stored_hashes_only(path: &str) -> Result<Vec<StoredHash>, String> {
    debug!("get_stored_hashes_only: {}", path);
    
    // First, try companion log - this is always fast (just text file parsing)
    if let Some(companion) = find_companion_log(path) {
        if !companion.stored_hashes.is_empty() {
            debug!("Found {} hashes in companion log", companion.stored_hashes.len());
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
                    let hashes: Vec<StoredHash> = info.stored_hashes.iter().map(|h| {
                        StoredHash {
                            algorithm: h.algorithm.clone(),
                            hash: h.hash.clone(),
                            verified: h.verified,
                            timestamp: None,
                            source: Some("container".to_string()),
                            offset: None,
                            size: None,
                        }
                    }).collect();
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
                        let hashes: Vec<StoredHash> = stored.iter().map(|h| {
                            StoredHash {
                                algorithm: h.algorithm.clone(),
                                hash: h.hash.clone(),
                                verified: None, // UFED doesn't track verified state
                                timestamp: h.timestamp.clone(),
                                source: Some("container".to_string()),
                                offset: None,
                                size: None,
                            }
                        }).collect();
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
/// Use this for quick container listing/display
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
            let info = ewf::info(path)?;
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
            let info = ewf::info(path)?;
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
            Ok(ad1_results.into_iter().map(|entry| VerifyEntry {
                path: Some(entry.path),
                chunk_index: None,
                status: entry.status.to_string(),
                message: None,
            }).collect())
        }
        ContainerKind::E01 => {
            let ewf_results = ewf::verify_chunks(path, algorithm)?;
            Ok(ewf_results.into_iter().map(|entry| VerifyEntry {
                path: None,
                chunk_index: Some(entry.chunk_index),
                status: entry.status,
                message: entry.message,
            }).collect())
        }
        ContainerKind::L01 => {
            // L01 uses the same EWF format - use ewf::verify_chunks
            let ewf_results = ewf::verify_chunks(path, algorithm)?;
            Ok(ewf_results.into_iter().map(|entry| VerifyEntry {
                path: None,
                chunk_index: Some(entry.chunk_index),
                status: entry.status,
                message: entry.message,
            }).collect())
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
pub fn verify_with_progress<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<Vec<VerifyEntry>, String>
where
    F: FnMut(u64, u64),
{
    log_evidence_access("verify_with_progress", Path::new(path), Some(algorithm), None);
    
    match detect_container(path)? {
        ContainerKind::Ad1 => {
            // Wrap the u64 callback for AD1's usize signature
            let ad1_results = ad1::verify_with_progress(path, algorithm, |a, b| {
                progress_callback(a, b)
            }).map_err(|e| e.to_string())?;
            Ok(ad1_results.into_iter().map(|entry| VerifyEntry {
                path: Some(entry.path),
                chunk_index: None,
                status: entry.status.to_string(),
                message: None,
            }).collect())
        }
        ContainerKind::E01 | ContainerKind::L01 => {
            // Wrap the u64 callback for EWF's usize signature
            let hash = ewf::verify_with_progress(path, algorithm, |a, b| {
                progress_callback(a, b)
            })?;
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
pub fn extract_with_progress<F>(path: &str, output_dir: &str, mut progress_callback: F) -> Result<(), String>
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
            ad1::extract_with_progress(path, output_dir, |a, b| {
                progress_callback(a, b)
            }).map_err(|e| e.to_string())
        }
        ContainerKind::E01 | ContainerKind::L01 => ewf::extract_with_progress(path, output_dir, progress_callback).map_err(|e| e.to_string()),
        ContainerKind::Raw => raw::extract_with_progress(path, output_dir, progress_callback).map_err(|e| e.to_string()),
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
            let segment_count = info.segment_files.as_ref().map(|f| f.len() as u32).unwrap_or(1);
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
                container_type: if path.to_lowercase().contains(".l01") { "L01" } else { "E01" }.to_string(),
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
        ContainerKind::E01 | ContainerKind::L01 => ewf::get_segment_paths(path).map_err(|e| e.to_string()),
        ContainerKind::Raw => raw::get_segment_paths(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Archives typically don't have segments, return single path
            Ok(vec![std::path::PathBuf::from(path)])
        }
        ContainerKind::Ufed => ufed::get_segment_paths(path).map_err(|e| e.to_string()),
    }
}

// =============================================================================
// Metadata Export Functions
// =============================================================================

/// Export container metadata as JSON
pub fn export_metadata_json(path: &str) -> Result<String, String> {
    debug!(path = %path, "Exporting container metadata as JSON");
    
    match detect_container(path)? {
        ContainerKind::Ad1 => ad1::export_tree_json(path).map_err(|e| e.to_string()),
        ContainerKind::E01 | ContainerKind::L01 => ewf::export_metadata_json(path).map_err(|e| e.to_string()),
        ContainerKind::Raw => raw::export_metadata_json(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Serialize archive info as JSON
            let info = archive::info(path)?;
            serde_json::to_string_pretty(&info)
                .map_err(|e| format!("Failed to serialize: {}", e))
        }
        ContainerKind::Ufed => {
            // Serialize UFED info as JSON
            let info = ufed::info(path)?;
            serde_json::to_string_pretty(&info)
                .map_err(|e| format!("Failed to serialize: {}", e))
        }
    }
}

/// Export container metadata as CSV
pub fn export_metadata_csv(path: &str) -> Result<String, String> {
    debug!(path = %path, "Exporting container metadata as CSV");
    
    match detect_container(path)? {
        ContainerKind::Ad1 => ad1::export_tree_csv(path).map_err(|e| e.to_string()),
        ContainerKind::E01 | ContainerKind::L01 => ewf::export_metadata_csv(path).map_err(|e| e.to_string()),
        ContainerKind::Raw => raw::export_metadata_csv(path).map_err(|e| e.to_string()),
        ContainerKind::Archive => {
            // Generate CSV from archive entries
            let entries = archive::list_zip_entries(path)?;
            let mut csv = String::new();
            csv.push_str("Index,Path,Is Directory,Size,Compressed Size,CRC32,Last Modified\n");
            for entry in entries {
                csv.push_str(&format!(
                    "{},\"{}\",{},{},{},{},\"{}\"\n",
                    entry.index, entry.path, entry.is_directory, entry.size,
                    entry.compressed_size, entry.crc32, entry.last_modified
                ));
            }
            Ok(csv)
        }
        ContainerKind::Ufed => {
            // Generate CSV from UFED info
            let info = ufed::info(path)?;
            let mut csv = String::new();
            csv.push_str("# UFED Container Metadata\n");
            csv.push_str(&format!("Format,{}\n", info.format));
            csv.push_str(&format!("Size,{}\n", info.size));
            if let Some(ref hint) = info.device_hint {
                csv.push_str(&format!("Device Hint,\"{}\"\n", hint));
            }
            csv.push_str("\n# Associated Files\n");
            csv.push_str("Filename,Size,Type,Hash\n");
            for file in &info.associated_files {
                csv.push_str(&format!(
                    "\"{}\",{},\"{}\",\"{}\"\n",
                    file.filename,
                    file.size,
                    file.file_type,
                    file.stored_hash.as_deref().unwrap_or("")
                ));
            }
            Ok(csv)
        }
    }
}

// =============================================================================
// Search Operations
// =============================================================================

use super::types::{SearchQuery, ContainerSearchResult, SearchMatchType};

/// Search for files within a container
/// 
/// This unified search API works across all container types that support
/// file trees (AD1, L01, Archive). For disk image containers (E01, Raw),
/// this function will return an error as they don't have file tree structures.
/// 
/// # Arguments
/// * `path` - Path to the container file
/// * `query` - Search query specifying what to find
/// 
/// # Returns
/// A vector of search results matching the query
/// 
/// # Example
/// ```rust,ignore
/// use crate::containers::{search, SearchQuery};
/// 
/// // Find all .exe files
/// let query = SearchQuery::new().with_extension("exe");
/// let results = search("/evidence/disk.ad1", query)?;
/// 
/// // Find files by name pattern
/// let query = SearchQuery::new().with_name("*.log");
/// let results = search("/evidence/disk.ad1", query)?;
/// ```
pub fn search(path: &str, query: SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    debug!(path = %path, "Searching container");
    
    let kind = detect_container(path)?;
    
    match kind {
        ContainerKind::Ad1 => search_ad1(path, &query),
        ContainerKind::L01 => search_l01(path, &query),
        ContainerKind::Archive => search_archive(path, &query),
        ContainerKind::E01 => Err("E01 containers are disk images without file tree structure. Use filesystem tools after extraction.".to_string()),
        ContainerKind::Raw => Err("Raw containers are disk images without file tree structure. Use filesystem tools after extraction.".to_string()),
        ContainerKind::Ufed => Err("UFED search not yet implemented. Search associated files directly.".to_string()),
    }
}

/// Search within an AD1 container
fn search_ad1(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    let mut results = Vec::new();
    
    // Search by name pattern
    if let Some(ref pattern) = query.name_pattern {
        let matches = ad1::find_by_name(path, pattern).map_err(|e| e.to_string())?;
        for m in matches {
            if should_include_ad1(&m.entry, query) {
                // Extract name from path
                let name = std::path::Path::new(&m.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| m.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: m.entry.path.clone(),
                    name,
                    is_directory: m.entry.is_dir,
                    size: m.entry.size,
                    match_type: SearchMatchType::Name,
                    depth: m.depth,
                    md5: m.entry.md5_hash.clone(),
                    sha1: m.entry.sha1_hash.clone(),
                    created: m.entry.created.clone(),
                    modified: m.entry.modified.clone(),
                });
            }
        }
    }
    
    // Search by extension
    if let Some(ref ext) = query.extension {
        let matches = ad1::find_by_extension(path, ext).map_err(|e| e.to_string())?;
        for m in matches {
            // Avoid duplicates if both name and extension are specified
            if query.name_pattern.is_some() && results.iter().any(|r| r.entry_path == m.entry.path) {
                continue;
            }
            if should_include_ad1(&m.entry, query) {
                let name = std::path::Path::new(&m.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| m.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: m.entry.path.clone(),
                    name,
                    is_directory: m.entry.is_dir,
                    size: m.entry.size,
                    match_type: SearchMatchType::Extension,
                    depth: m.depth,
                    md5: m.entry.md5_hash.clone(),
                    sha1: m.entry.sha1_hash.clone(),
                    created: m.entry.created.clone(),
                    modified: m.entry.modified.clone(),
                });
            }
        }
    }
    
    // Search by hash
    if let Some(ref hash) = query.hash {
        if let Some(found) = ad1::find_by_hash(path, hash).map_err(|e| e.to_string())? {
            if should_include_ad1(&found.entry, query) {
                let name = std::path::Path::new(&found.entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| found.entry.path.clone());
                
                results.push(ContainerSearchResult {
                    container_path: path.to_string(),
                    container_type: "AD1".to_string(),
                    entry_path: found.entry.path.clone(),
                    name,
                    is_directory: found.entry.is_dir,
                    size: found.entry.size,
                    match_type: SearchMatchType::Hash,
                    depth: found.depth,
                    md5: found.entry.md5_hash.clone(),
                    sha1: found.entry.sha1_hash.clone(),
                    created: found.entry.created.clone(),
                    modified: found.entry.modified.clone(),
                });
            }
        }
    }
    
    // Limit results if specified
    if let Some(max) = query.max_results {
        results.truncate(max);
    }
    
    Ok(results)
}

/// Search within an L01 container (logical EWF)
fn search_l01(path: &str, _query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    // L01 containers have similar structure to AD1 for logical evidence
    // For now, return an informative error until L01 tree parsing is implemented
    Err(format!(
        "L01 file tree search not yet implemented for: {}. Use ewf::info() to get metadata.",
        path
    ))
}

/// Search within an archive container
fn search_archive(path: &str, query: &SearchQuery) -> Result<Vec<ContainerSearchResult>, String> {
    let entries = archive::list_zip_entries(path)?;
    let mut results = Vec::new();
    
    for entry in entries {
        let name = std::path::Path::new(&entry.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| entry.path.clone());
        
        let matches = check_entry_matches(&name, &entry.path, entry.size, entry.is_directory, query);
        
        if let Some(match_type) = matches {
            results.push(ContainerSearchResult {
                container_path: path.to_string(),
                container_type: "Archive".to_string(),
                entry_path: entry.path.clone(),
                name,
                is_directory: entry.is_directory,
                size: entry.size,
                match_type,
                depth: entry.path.matches('/').count() as u32,
                md5: None,
                sha1: None,
                created: None,
                modified: Some(entry.last_modified),
            });
            
            // Check result limit
            if let Some(max) = query.max_results {
                if results.len() >= max {
                    break;
                }
            }
        }
    }
    
    Ok(results)
}

/// Check if an entry matches the query criteria
fn check_entry_matches(
    name: &str,
    path: &str,
    size: u64,
    is_directory: bool,
    query: &SearchQuery,
) -> Option<SearchMatchType> {
    // Check directory/file filters
    if query.directories_only && !is_directory {
        return None;
    }
    if query.files_only && is_directory {
        return None;
    }
    
    // Check size filters
    if let Some(min) = query.min_size {
        if size < min {
            return None;
        }
    }
    if let Some(max) = query.max_size {
        if size > max {
            return None;
        }
    }
    
    let mut matched = false;
    let mut match_type = SearchMatchType::Name;
    
    // Check name pattern
    if let Some(ref pattern) = query.name_pattern {
        if matches_pattern(name, pattern) || matches_pattern(path, pattern) {
            matched = true;
            match_type = SearchMatchType::Name;
        }
    }
    
    // Check extension
    if let Some(ref ext) = query.extension {
        let lower_name = name.to_lowercase();
        let lower_ext = ext.to_lowercase();
        if lower_name.ends_with(&format!(".{}", lower_ext)) {
            if matched {
                match_type = SearchMatchType::Multiple;
            } else {
                matched = true;
                match_type = SearchMatchType::Extension;
            }
        }
    }
    
    // If no search criteria specified, match all (filtered by dir/file/size above)
    if query.name_pattern.is_none() && query.extension.is_none() && query.hash.is_none() {
        return Some(SearchMatchType::Name);
    }
    
    if matched {
        Some(match_type)
    } else {
        None
    }
}

/// Simple wildcard pattern matching (* and ?)
fn matches_pattern(text: &str, pattern: &str) -> bool {
    let lower_text = text.to_lowercase();
    let lower_pattern = pattern.to_lowercase();
    
    // Convert glob pattern to simple matching
    if lower_pattern.contains('*') || lower_pattern.contains('?') {
        // Simple wildcard matching
        let parts: Vec<&str> = lower_pattern.split('*').collect();
        if parts.len() == 1 {
            // No wildcards, exact match
            lower_text == lower_pattern
        } else {
            // Check if text contains all parts in order
            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                if i == 0 {
                    // First part must match at start
                    if !lower_text.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == parts.len() - 1 && !pattern.ends_with('*') {
                    // Last part must match at end
                    if !lower_text.ends_with(part) {
                        return false;
                    }
                } else {
                    // Middle part must exist somewhere after current position
                    if let Some(found_pos) = lower_text[pos..].find(part) {
                        pos += found_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        }
    } else {
        // No wildcards - check for substring
        lower_text.contains(&lower_pattern)
    }
}

/// Helper to check if an AD1 tree entry should be included based on query filters
fn should_include_ad1(entry: &ad1::TreeEntry, query: &SearchQuery) -> bool {
    // Check directory/file filters
    if query.directories_only && !entry.is_dir {
        return false;
    }
    if query.files_only && entry.is_dir {
        return false;
    }
    
    // Check size filters
    if let Some(min) = query.min_size {
        if entry.size < min {
            return false;
        }
    }
    if let Some(max) = query.max_size {
        if entry.size > max {
            return false;
        }
    }
    
    true
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
    if lower.ends_with(".e01") || lower.ends_with(".ex01") || lower.ends_with(".ewf") 
        || lower.contains(".e0") || lower.contains(".ex")
    {
        debug!("Checking E01 signature for: {}", path);
        if ewf::is_e01(path).unwrap_or(false) {
            return Ok(ContainerKind::E01);
        } else {
            debug!("E01 signature check failed for: {}", path);
        }
    }
    
    // Check L01/Lx01 - use ewf::is_l01_file for proper LVF signature detection
    if (lower.ends_with(".l01") || lower.ends_with(".lx01") 
        || lower.contains(".l0") || lower.contains(".lx"))
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

    Err(format!("Unsupported or unrecognized logical container: {}\nSupported formats: AD1, E01/EWF, L01, RAW (.dd, .raw, .img, .001), Archives (7z, ZIP, RAR), UFED (UFD, UFDR, UFDX)", path))
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
        file.write_all(b"XYZ random content that doesn't match any known format").unwrap();
        
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
