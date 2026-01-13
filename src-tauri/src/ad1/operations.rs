// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Container Operations
//!
//! ## Section Brief
//! Public API for AD1 (AccessData Logical Image) container operations:
//!
//! ### Container Information
//! - `info_fast` - Quick info (headers only, no tree parsing)
//! - `info` - Full container info with optional tree
//! - `is_ad1` - Format detection by signature
//! - `get_stats` - Container statistics (files, folders, sizes)
//! - `get_segment_paths` - List all segment file paths
//!
//! ### Tree Navigation
//! - `get_tree` - Get complete file tree
//! - `get_children` - Get children at path (lazy loading)
//! - `get_children_at_addr` - Get children by address (efficient)
//! - `get_entry_info` - Get metadata for single entry
//!
//! ### Search Functions
//! - `find_by_name` - Search by filename pattern
//! - `find_by_extension` - Search by file extension
//! - `find_by_hash` - Search by MD5/SHA1 hash
//!
//! ### Data Operations
//! - `read_entry_data` - Read file by path
//! - `read_entry_data_by_addr` - Read file by address (hex viewer)
//! - `read_entry_chunk` - Read partial file (streaming)
//!
//! ### Verification & Extraction
//! - `verify` / `verify_with_progress` - Verify stored hashes
//! - `verify_chunks` - Chunk-level verification results
//! - `verify_against_log` - Verify against companion log
//! - `extract` / `extract_with_progress` - Extract to disk
//! - `hash_segments` / `hash_segments_with_progress` - Hash image
//!
//! ### Export Functions
//! - `export_tree_json` - Export tree to JSON
//! - `export_tree_csv` - Export tree to CSV

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::{debug, trace, instrument};

use super::types::{
    Ad1Info, VerifyEntry, VerifyStatus, AD1_SIGNATURE,
    TreeEntry, Item, AD1_FOLDER_SIGNATURE,
    Ad1Stats, SearchResult, ChunkVerifyResult,
    HASH_INFO, MD5_HASH, SHA1_HASH,
};
use super::parser::Session;
use super::utils::*;
use crate::common::hash::{HashAlgorithm, StreamingHasher};
use crate::containers::ContainerError;

// =============================================================================
// Container Information
// =============================================================================

/// Fast info - only reads headers, doesn't parse full item tree
/// Use this for quick container detection/display
/// This uses lenient validation - will return info even with missing segments
#[must_use = "this returns the AD1 info, which should be used"]
#[instrument]
pub fn info_fast(path: &str) -> Result<Ad1Info, ContainerError> {
    debug!("Getting fast AD1 info (headers only)");
    validate_ad1(path, false)?;  // Only validate format, not segments
    
    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file '{path}': {e}")))?;
    
    let segment_header = read_segment_header(&mut file)?;
    let logical_header = read_logical_header(&mut file)?;
    
    // Parse volume info from header
    let volume = parse_volume_info(&mut file);
    
    // Parse companion log file for case metadata
    let companion_log = parse_companion_log(path);
    
    // Get segment files with sizes (includes missing segments)
    let (segment_files, segment_sizes, total_size, missing_segments) = 
        get_segment_files_with_sizes(path, segment_header.segment_number);
    
    // Get detailed segment summary
    let segment_summary = get_segment_summary(
        path,
        segment_header.segment_number,
        segment_header.fragments_size,
    );
    
    let missing = if missing_segments.is_empty() {
        None
    } else {
        Some(missing_segments)
    };
    
    Ok(Ad1Info {
        segment: segment_header_info(&segment_header),
        logical: logical_header_info(&logical_header),
        item_count: 0, // Not parsed in fast mode
        tree: None,
        segment_files: Some(segment_files),
        segment_sizes: Some(segment_sizes),
        total_size: Some(total_size),
        missing_segments: missing,
        segment_summary: Some(segment_summary),
        volume,
        companion_log,
    })
}

/// Get full AD1 container information
/// Note: This still requires all segments to be present (strict validation via Session::open)
#[must_use = "this returns the AD1 info, which should be used"]
#[instrument]
pub fn info(path: &str, include_tree: bool) -> Result<Ad1Info, ContainerError> {
    debug!("Getting AD1 info, include_tree={}", include_tree);
    let session = Session::open(path)?;
    
    let tree = if include_tree {
        let mut entries = Vec::new();
        collect_tree(&session.root_items, "", &mut entries);
        Some(entries)
    } else {
        None
    };
    
    // Get segment files with sizes
    let (segment_files, segment_sizes, total_size, missing_segments) = 
        get_segment_files_with_sizes(path, session.segment_header.segment_number);
    
    // Get detailed segment summary
    let segment_summary = get_segment_summary(
        path,
        session.segment_header.segment_number,
        session.segment_header.fragments_size,
    );
    
    let missing = if missing_segments.is_empty() {
        None
    } else {
        Some(missing_segments)
    };
    
    // Parse volume info from the first segment file
    let volume = {
        let mut file = File::open(path)
            .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file for volume info: {e}")))?;
        parse_volume_info(&mut file)
    };
    
    // Parse companion log file for case metadata
    let companion_log = parse_companion_log(path);
    
    Ok(Ad1Info {
        segment: segment_header_info(&session.segment_header),
        logical: logical_header_info(&session.logical_header),
        item_count: session.item_counter,
        tree,
        segment_files: Some(segment_files),
        segment_sizes: Some(segment_sizes),
        total_size: Some(total_size),
        missing_segments: missing,
        segment_summary: Some(segment_summary),
        volume,
        companion_log,
    })
}

// =============================================================================
// Verification Functions
// =============================================================================

/// Verify file hashes in the container
#[must_use = "this returns verification results, which should be used"]
pub fn verify(path: &str, algorithm: &str) -> Result<Vec<VerifyEntry>, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Verify with progress callback
#[must_use = "this returns verification results, which should be used"]
pub fn verify_with_progress<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<Vec<VerifyEntry>, ContainerError>
where
    F: FnMut(usize, usize)
{
    let mut session = Session::open(path)?;
    let algo: HashAlgorithm = algorithm.parse()?;
    let mut results = Vec::new();
    
    // Count total files for progress
    let total = count_files(&session.root_items);
    let mut current = 0;
    
    // Clone root_items to avoid borrow checker issues
    let root_items = session.root_items.clone();
    
    // Create params struct for verify function
    let mut params = super::parser::VerifyParams {
        algorithm: algo,
        out: &mut results,
        current: &mut current,
        total,
        progress_callback: &mut progress_callback,
    };
    
    for item in &root_items {
        session.verify_item_with_progress(item, "", &mut params)?;
    }
    
    Ok(results)
}

// =============================================================================
// Extraction Functions
// =============================================================================

/// Extract container contents to output directory
#[must_use = "this returns a Result that should be checked for errors"]
pub fn extract(path: &str, output_dir: &str) -> Result<(), ContainerError> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract with progress callback
#[must_use = "this returns a Result that should be checked for errors"]
pub fn extract_with_progress<F>(path: &str, output_dir: &str, mut progress_callback: F) -> Result<(), ContainerError>
where
    F: FnMut(usize, usize)
{
    let mut session = Session::open(path)?;
    let output_path = Path::new(output_dir);
    
    // Count total files for progress
    let total = count_files(&session.root_items);
    let mut current = 0;
    
    // Clone root_items to avoid borrow checker issues
    let root_items = session.root_items.clone();
    
    for item in &root_items {
        session.extract_item_with_progress(item, output_path, &mut current, total, &mut progress_callback)?;
    }
    
    Ok(())
}

// =============================================================================
// Format Detection
// =============================================================================

/// Check if file is an AD1 container
#[must_use = "this returns whether the file is AD1, which should be used"]
pub fn is_ad1(path: &str) -> Result<bool, ContainerError> {
    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open input file: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read file signature: {e}")))?;
    let is_ad1 = &signature[..15] == AD1_SIGNATURE;
    trace!(path, is_ad1, "AD1 signature check");
    Ok(is_ad1)
}

// =============================================================================
// Segment Hashing Functions
// =============================================================================

/// Hash AD1 segment files (image-level hash)
/// This hashes all segment files sequentially to produce a single hash
/// that can be compared against the stored hash in the companion log
#[must_use = "this returns the hash, which should be used"]
pub fn hash_segments(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    hash_segments_with_progress(path, algorithm, |_, _| {})
}

/// Hash AD1 segments with progress callback
#[must_use = "this returns the hash, which should be used"]
pub fn hash_segments_with_progress<F>(path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    validate_ad1(path, true)?;  // Validate format and segments
    
    let algo: HashAlgorithm = algorithm.parse()?;
    
    // Get segment info
    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file: {e}")))?;
    let segment_header = read_segment_header(&mut file)?;
    drop(file);
    
    let segment_count = segment_header.segment_number;
    
    // Calculate total size for progress
    let mut total_size: u64 = 0;
    let mut segment_paths = Vec::with_capacity(segment_count as usize);
    
    for i in 1..=segment_count {
        let segment_path = build_segment_path(path, i);
        let seg_path = Path::new(&segment_path);
        if !seg_path.exists() {
            return Err(ContainerError::SegmentError(format!("Missing segment: {}", segment_path)));
        }
        let size = std::fs::metadata(&segment_path)
            .map(|m| m.len())
            .map_err(|e| ContainerError::IoError(format!("Failed to get segment size: {e}")))?;
        total_size += size;
        segment_paths.push(segment_path);
    }
    
    debug!(segment_count, total_size, "Hashing AD1 segments");
    
    // Hash all segments sequentially
    let mut hasher = StreamingHasher::new(algo);
    let mut bytes_processed: u64 = 0;
    let buffer_size = 1024 * 1024; // 1MB buffer
    
    for segment_path in &segment_paths {
        let file = File::open(segment_path)
            .map_err(|e| ContainerError::IoError(format!("Failed to open segment {}: {e}", segment_path)))?;
        let mut reader = BufReader::with_capacity(buffer_size, file);
        let mut buffer = vec![0u8; buffer_size];
        
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| ContainerError::IoError(format!("Failed to read segment: {e}")))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            bytes_processed += bytes_read as u64;
            progress_callback(bytes_processed, total_size);
        }
    }
    
    let hash = hasher.finalize();
    debug!(hash = %hash, "AD1 segment hash complete");
    Ok(hash)
}

// =============================================================================
// Tree Navigation Functions
// =============================================================================

/// Get the full tree of items in the container
#[must_use = "this returns the tree entries, which should be used"]
pub fn get_tree(path: &str) -> Result<Vec<TreeEntry>, ContainerError> {
    let session = Session::open(path)?;
    let mut entries = Vec::new();
    collect_tree(&session.root_items, "", &mut entries);
    Ok(entries)
}

/// Get children at a specific path in the container
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children(path: &str, parent_path: &str) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!("get_children: path={}, parent_path={}", path, parent_path);
    let session = Session::open(path)?;
    let mut entries = Vec::new();
    collect_children_at_path(&session.root_items, parent_path, "", &mut entries);
    tracing::debug!("get_children: found {} entries", entries.len());
    Ok(entries)
}

/// Get children at a specific address using lazy loading
/// This is FAST - it only reads the items at the specified address without
/// loading the entire container tree into memory
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children_at_addr_lazy(path: &str, addr: u64, parent_path: &str) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!("get_children_at_addr_lazy: path={}, addr=0x{:x}, parent_path={}", path, addr, parent_path);
    
    let mut session = Session::open_lazy(path)?;
    
    let target_addr = if addr == 0 {
        // Return root items
        session.first_item_addr()
    } else {
        addr
    };
    
    let items = session.read_children_lazy(target_addr)?;
    tracing::debug!("get_children_at_addr_lazy: found {} items", items.len());
    
    let entries: Vec<_> = items.iter()
        .map(|item| build_tree_entry_lazy(item, parent_path))
        .collect();
    
    Ok(entries)
}

/// Get children at a specific address (for lazy loading)
/// This is more efficient than path-based lookup when navigating the tree
#[must_use = "this returns the children entries, which should be used"]
pub fn get_children_at_addr(path: &str, addr: u64) -> Result<Vec<TreeEntry>, ContainerError> {
    tracing::debug!("get_children_at_addr: path={}, addr={}", path, addr);
    let session = Session::open(path)?;
    
    if addr == 0 {
        // Return root items
        let entries: Vec<_> = session.root_items.iter()
            .map(|item| build_tree_entry(item, "", true))
            .collect();
        tracing::debug!("get_children_at_addr: addr=0, returning {} root items", entries.len());
        return Ok(entries);
    }
    
    // Find item at address using utility function
    fn find_by_addr<'a>(items: &'a [Item], addr: u64, parent_path: &str) -> Option<(&'a Item, String)> {
        for item in items {
            let item_path = join_path(parent_path, &item.name);
            if item.zlib_metadata_addr == addr {
                return Some((item, item_path));
            }
            if let Some(found) = find_by_addr(&item.children, addr, &item_path) {
                return Some(found);
            }
        }
        None
    }
    
    let (item, item_path) = find_by_addr(&session.root_items, addr, "")
        .ok_or_else(|| ContainerError::EntryNotFound(format!("Item not found at address {}", addr)))?;
    
    Ok(item.children.iter()
        .map(|child| build_tree_entry(child, &item_path, true))
        .collect())
}

// =============================================================================
// Data Reading Functions
// =============================================================================

/// Read file data by path
#[must_use = "this returns the file data, which should be used"]
pub fn read_entry_data(path: &str, entry_path: &str) -> Result<Vec<u8>, ContainerError> {
    let mut session = Session::open(path)?;
    
    // Use unified item finder and clone to avoid borrow issues
    let found = find_item_by_path(&session.root_items, entry_path)
        .ok_or_else(|| ContainerError::EntryNotFound(entry_path.to_string()))?;
    let item = found.item.clone();
    
    let data = session.read_file_data(&item)?;
    Ok((*data).clone())
}

/// Read file data by address (for hex viewer)
#[must_use = "this returns the file data, which should be used"]
pub fn read_entry_data_by_addr(path: &str, data_addr: u64, size: u64) -> Result<Vec<u8>, ContainerError> {
    let mut session = Session::open(path)?;
    
    // Create a temporary item with the address info
    let temp_item = Item {
        id: 0,
        name: String::new(),
        item_type: 0,
        decompressed_size: size,
        zlib_metadata_addr: data_addr,
        metadata: Vec::new(),
        children: Vec::new(),
    };
    
    let data = session.read_file_data(&temp_item)?;
    Ok((*data).clone())
}

/// Read a chunk of file data (for large files / streaming)
#[must_use = "this returns the file data chunk, which should be used"]
pub fn read_entry_chunk(path: &str, entry_path: &str, offset: u64, size: usize) -> Result<Vec<u8>, ContainerError> {
    let data = read_entry_data(path, entry_path)?;
    let start = offset as usize;
    let end = start + size;
    
    if start >= data.len() {
        return Ok(Vec::new());
    }
    
    let actual_end = end.min(data.len());
    Ok(data[start..actual_end].to_vec())
}

/// Get entry information (metadata) by path
#[must_use = "this returns the entry info, which should be used"]
pub fn get_entry_info(path: &str, entry_path: &str) -> Result<TreeEntry, ContainerError> {
    let session = Session::open(path)?;
    
    // Use unified item finder
    let found = find_item_by_path(&session.root_items, entry_path)
        .ok_or_else(|| ContainerError::EntryNotFound(entry_path.to_string()))?;
    
    Ok(build_tree_entry(found.item, &found.parent_path, true))
}

// =============================================================================
// Segment Path Functions
// =============================================================================

/// Get all segment file paths for an AD1 container
/// Returns paths for .ad1, .ad2, .ad3, etc.
#[must_use = "this returns the segment paths, which should be used"]
pub fn get_segment_paths(path: &str) -> Result<Vec<std::path::PathBuf>, ContainerError> {
    validate_ad1(path, false)?;
    
    let mut file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file: {e}")))?;
    let segment_header = read_segment_header(&mut file)?;
    
    let mut paths = Vec::with_capacity(segment_header.segment_number as usize);
    for i in 1..=segment_header.segment_number {
        let segment_path = build_segment_path(path, i);
        paths.push(std::path::PathBuf::from(segment_path));
    }
    
    Ok(paths)
}

// =============================================================================
// Statistics Functions
// =============================================================================

/// Get container statistics
#[must_use = "this returns the statistics, which should be used"]
pub fn get_stats(path: &str) -> Result<Ad1Stats, ContainerError> {
    let session = Session::open(path)?;
    let mut stats = Ad1Stats::default();
    
    // Get compressed size from segments
    let segment_paths = get_segment_paths(path)?;
    for seg_path in &segment_paths {
        if let Ok(meta) = std::fs::metadata(seg_path) {
            stats.compressed_size += meta.len();
        }
    }
    
    // Recursively gather stats from items
    fn gather_stats(items: &[Item], stats: &mut Ad1Stats, depth: u32) {
        stats.max_depth = stats.max_depth.max(depth);
        
        for item in items {
            stats.total_items += 1;
            
            if item.item_type == AD1_FOLDER_SIGNATURE {
                stats.total_folders += 1;
            } else {
                stats.total_files += 1;
                stats.total_size += item.decompressed_size;
                
                // Track largest file
                if item.decompressed_size > stats.largest_file_size {
                    stats.largest_file_size = item.decompressed_size;
                    stats.largest_file_path = Some(item.name.clone());
                }
                
                // Check for hashes
                let has_md5 = item.metadata.iter().any(|m| m.category == HASH_INFO && m.key == MD5_HASH);
                let has_sha1 = item.metadata.iter().any(|m| m.category == HASH_INFO && m.key == SHA1_HASH);
                if has_md5 { stats.files_with_md5 += 1; }
                if has_sha1 { stats.files_with_sha1 += 1; }
            }
            
            gather_stats(&item.children, stats, depth + 1);
        }
    }
    
    gather_stats(&session.root_items, &mut stats, 0);
    
    // Calculate compression ratio
    if stats.total_size > 0 {
        stats.compression_ratio = stats.compressed_size as f64 / stats.total_size as f64;
    }
    
    Ok(stats)
}

// =============================================================================
// Chunk Verification Functions
// =============================================================================

/// Verify container and return chunk-level results (for parity with EWF)
#[must_use = "this returns the verification results, which should be used"]
pub fn verify_chunks(path: &str, algorithm: &str) -> Result<Vec<ChunkVerifyResult>, ContainerError> {
    let results = verify(path, algorithm)?;
    
    Ok(results.into_iter().enumerate().map(|(i, entry)| {
        ChunkVerifyResult {
            index: i as u64,
            status: entry.status.to_string(),
            message: entry.computed,
            path: Some(entry.path),
        }
    }).collect())
}

// =============================================================================
// Search Functions
// =============================================================================

/// Find entries by name pattern (case-insensitive substring match)
#[must_use = "this returns the search results, which should be used"]
pub fn find_by_name(path: &str, pattern: &str) -> Result<Vec<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();
    
    fn search_items(items: &[Item], pattern: &str, parent_path: &str, depth: u32, results: &mut Vec<SearchResult>) {
        for item in items {
            let item_path = join_path(parent_path, &item.name);
            
            if item.name.to_lowercase().contains(pattern) {
                results.push(SearchResult {
                    entry: build_tree_entry(item, parent_path, true),
                    match_type: "name".to_string(),
                    depth,
                });
            }
            
            search_items(&item.children, pattern, &item_path, depth + 1, results);
        }
    }
    
    search_items(&session.root_items, &pattern_lower, "", 0, &mut results);
    Ok(results)
}

/// Find entries by file extension
#[must_use = "this returns the search results, which should be used"]
pub fn find_by_extension(path: &str, extension: &str) -> Result<Vec<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let ext_lower = extension.to_lowercase().trim_start_matches('.').to_string();
    let mut results = Vec::new();
    
    fn search_items(items: &[Item], ext: &str, parent_path: &str, depth: u32, results: &mut Vec<SearchResult>) {
        for item in items {
            let item_path = join_path(parent_path, &item.name);
            
            // Check extension
            if item.item_type != AD1_FOLDER_SIGNATURE {
                if let Some(item_ext) = item.name.rsplit('.').next() {
                    if item_ext.to_lowercase() == ext {
                        results.push(SearchResult {
                            entry: build_tree_entry(item, parent_path, true),
                            match_type: "extension".to_string(),
                            depth,
                        });
                    }
                }
            }
            
            search_items(&item.children, ext, &item_path, depth + 1, results);
        }
    }
    
    search_items(&session.root_items, &ext_lower, "", 0, &mut results);
    Ok(results)
}

/// Find entry by hash value (searches MD5 and SHA1)
#[must_use = "this returns the search result, which should be used"]
pub fn find_by_hash(path: &str, hash: &str) -> Result<Option<SearchResult>, ContainerError> {
    let session = Session::open(path)?;
    let hash_lower = hash.to_lowercase();
    
    fn search_items(items: &[Item], hash: &str, parent_path: &str, depth: u32) -> Option<SearchResult> {
        for item in items {
            let item_path = join_path(parent_path, &item.name);
            
            // Check MD5 and SHA1 hashes
            let entry = build_tree_entry(item, parent_path, true);
            if let Some(ref md5) = entry.md5_hash {
                if md5.to_lowercase() == hash {
                    return Some(SearchResult {
                        entry,
                        match_type: "md5".to_string(),
                        depth,
                    });
                }
            }
            if let Some(ref sha1) = entry.sha1_hash {
                if sha1.to_lowercase() == hash {
                    return Some(SearchResult {
                        entry,
                        match_type: "sha1".to_string(),
                        depth,
                    });
                }
            }
            
            if let Some(found) = search_items(&item.children, hash, &item_path, depth + 1) {
                return Some(found);
            }
        }
        None
    }
    
    Ok(search_items(&session.root_items, &hash_lower, "", 0))
}

// =============================================================================
// Export Functions
// =============================================================================

/// Export tree structure to JSON string
#[must_use = "this returns the JSON string, which should be used"]
pub fn export_tree_json(path: &str) -> Result<String, ContainerError> {
    let entries = get_tree(path)?;
    serde_json::to_string_pretty(&entries)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize tree to JSON: {e}")))
}

/// Export tree structure to CSV string
#[must_use = "this returns the CSV string, which should be used"]
pub fn export_tree_csv(path: &str) -> Result<String, ContainerError> {
    let entries = get_tree(path)?;
    let mut csv = String::from("path,is_dir,size,item_type,md5_hash,sha1_hash,created,accessed,modified\n");
    
    for entry in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            escape_csv(&entry.path),
            entry.is_dir,
            entry.size,
            entry.item_type,
            entry.md5_hash.as_deref().unwrap_or(""),
            entry.sha1_hash.as_deref().unwrap_or(""),
            entry.created.as_deref().unwrap_or(""),
            entry.accessed.as_deref().unwrap_or(""),
            entry.modified.as_deref().unwrap_or(""),
        ));
    }
    
    Ok(csv)
}

/// Export container metadata as JSON (container-level info, not file tree)
#[must_use = "this returns the JSON string, which should be used"]
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    let info = info(path, false)?; // Don't include tree for metadata export
    let stats = get_stats(path)?;
    
    #[derive(serde::Serialize)]
    struct Ad1Metadata {
        format: String,
        segment_info: SegmentInfo,
        logical_info: LogicalInfo,
        statistics: Statistics,
        companion_log: Option<CompanionLogSummary>,
    }
    
    #[derive(serde::Serialize)]
    struct SegmentInfo {
        segment_count: u32,
        fragments_size: u32,
        segment_files: Vec<String>,
        segment_sizes: Vec<u64>,
        total_size: u64,
        total_size_formatted: String,
        missing_segments: Option<Vec<String>>,
    }
    
    #[derive(serde::Serialize)]
    struct LogicalInfo {
        image_version: u32,
        zlib_chunk_size: u32,
        data_source_name: String,
    }
    
    #[derive(serde::Serialize)]
    struct Statistics {
        total_items: u64,
        total_files: u64,
        total_folders: u64,
        total_size: u64,
        compressed_size: u64,
        compression_ratio: f64,
        max_depth: u32,
        files_with_md5: u64,
        files_with_sha1: u64,
        largest_file_size: u64,
        largest_file_path: Option<String>,
    }
    
    #[derive(serde::Serialize)]
    struct CompanionLogSummary {
        case_number: Option<String>,
        evidence_number: Option<String>,
        examiner: Option<String>,
        md5_hash: Option<String>,
        sha1_hash: Option<String>,
        sha256_hash: Option<String>,
    }
    
    let total_size = info.total_size.unwrap_or(0);
    
    let metadata = Ad1Metadata {
        format: "AD1".to_string(),
        segment_info: SegmentInfo {
            segment_count: info.segment.segment_number,
            fragments_size: info.segment.fragments_size,
            segment_files: info.segment_files.unwrap_or_default(),
            segment_sizes: info.segment_sizes.unwrap_or_default(),
            total_size,
            total_size_formatted: crate::common::format_size(total_size),
            missing_segments: info.missing_segments,
        },
        logical_info: LogicalInfo {
            image_version: info.logical.image_version,
            zlib_chunk_size: info.logical.zlib_chunk_size,
            data_source_name: info.logical.data_source_name.clone(),
        },
        statistics: Statistics {
            total_items: stats.total_items,
            total_files: stats.total_files,
            total_folders: stats.total_folders,
            total_size: stats.total_size,
            compressed_size: stats.compressed_size,
            compression_ratio: stats.compression_ratio,
            max_depth: stats.max_depth,
            files_with_md5: stats.files_with_md5,
            files_with_sha1: stats.files_with_sha1,
            largest_file_size: stats.largest_file_size,
            largest_file_path: stats.largest_file_path.clone(),
        },
        companion_log: info.companion_log.as_ref().map(|log| CompanionLogSummary {
            case_number: log.case_number.clone(),
            evidence_number: log.evidence_number.clone(),
            examiner: log.examiner.clone(),
            md5_hash: log.md5_hash.clone(),
            sha1_hash: log.sha1_hash.clone(),
            sha256_hash: log.sha256_hash.clone(),
        }),
    };
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize metadata to JSON: {e}")))
}

/// Export container metadata as CSV (container-level info, not file tree)
#[must_use = "this returns the CSV string, which should be used"]
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    let info = info(path, false)?;
    let stats = get_stats(path)?;
    
    let mut csv = String::new();
    let total_size = info.total_size.unwrap_or(0);
    
    // Header section
    csv.push_str("# AD1 Container Metadata\n");
    csv.push_str("Format,AD1\n");
    csv.push_str(&format!("Segment Count,{}\n", info.segment.segment_number));
    csv.push_str(&format!("Fragments Size,{}\n", info.segment.fragments_size));
    csv.push_str(&format!("Total Size,{}\n", total_size));
    csv.push_str(&format!("Total Size (Formatted),\"{}\"\n", crate::common::format_size(total_size)));
    csv.push('\n');
    
    // Logical header info
    csv.push_str("# Logical Header\n");
    csv.push_str(&format!("Image Version,{}\n", info.logical.image_version));
    csv.push_str(&format!("zlib Chunk Size,{}\n", info.logical.zlib_chunk_size));
    csv.push_str(&format!("Data Source Name,\"{}\"\n", escape_csv(&info.logical.data_source_name)));
    csv.push('\n');
    
    // Statistics
    csv.push_str("# Statistics\n");
    csv.push_str(&format!("Total Items,{}\n", stats.total_items));
    csv.push_str(&format!("Total Files,{}\n", stats.total_files));
    csv.push_str(&format!("Total Folders,{}\n", stats.total_folders));
    csv.push_str(&format!("Compressed Size,{}\n", stats.compressed_size));
    csv.push_str(&format!("Compression Ratio,{:.2}\n", stats.compression_ratio));
    csv.push_str(&format!("Max Depth,{}\n", stats.max_depth));
    csv.push_str(&format!("Files With MD5,{}\n", stats.files_with_md5));
    csv.push_str(&format!("Files With SHA1,{}\n", stats.files_with_sha1));
    if let Some(ref largest_path) = stats.largest_file_path {
        csv.push_str(&format!("Largest File,\"{}\"\n", escape_csv(largest_path)));
        csv.push_str(&format!("Largest File Size,{}\n", stats.largest_file_size));
    }
    csv.push('\n');
    
    // Companion log info if available
    if let Some(ref log) = info.companion_log {
        csv.push_str("# Companion Log\n");
        if let Some(ref case) = log.case_number {
            csv.push_str(&format!("Case Number,\"{}\"\n", escape_csv(case)));
        }
        if let Some(ref evidence) = log.evidence_number {
            csv.push_str(&format!("Evidence Number,\"{}\"\n", escape_csv(evidence)));
        }
        if let Some(ref examiner) = log.examiner {
            csv.push_str(&format!("Examiner,\"{}\"\n", escape_csv(examiner)));
        }
        if let Some(ref md5) = log.md5_hash {
            csv.push_str(&format!("MD5 Hash,{}\n", md5));
        }
        if let Some(ref sha1) = log.sha1_hash {
            csv.push_str(&format!("SHA1 Hash,{}\n", sha1));
        }
        if let Some(ref sha256) = log.sha256_hash {
            csv.push_str(&format!("SHA256 Hash,{}\n", sha256));
        }
        csv.push('\n');
    }
    
    // Segment details
    csv.push_str("# Segment Details\n");
    csv.push_str("Index,Filename,Size,Size (Formatted)\n");
    if let (Some(files), Some(sizes)) = (&info.segment_files, &info.segment_sizes) {
        for (i, (name, &size)) in files.iter().zip(sizes.iter()).enumerate() {
            csv.push_str(&format!(
                "{},\"{}\",{},\"{}\"\n",
                i + 1,
                escape_csv(name),
                size,
                crate::common::format_size(size)
            ));
        }
    }
    
    Ok(csv)
}

/// Hash a single AD1 segment file
#[must_use = "this returns the hash, which should be used"]
pub fn hash_single_segment<F>(segment_path: &str, algorithm: &str, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    let algo: HashAlgorithm = algorithm.parse()?;
    let path = Path::new(segment_path);
    
    if !path.exists() {
        return Err(ContainerError::FileNotFound(segment_path.to_string()));
    }
    
    let file_size = std::fs::metadata(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to get segment size: {e}")))?
        .len();
    
    let file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open segment: {e}")))?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    
    let mut hasher = StreamingHasher::new(algo);
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut bytes_processed: u64 = 0;
    
    loop {
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| ContainerError::IoError(format!("Failed to read segment: {e}")))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        bytes_processed += bytes_read as u64;
        progress_callback(bytes_processed, file_size);
    }
    
    Ok(hasher.finalize())
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
// Companion Log Functions
// =============================================================================

/// Verify container against companion log file
/// Compares computed hash with hash stored in log file
#[must_use = "this returns the verification result, which should be used"]
pub fn verify_against_log(path: &str, algorithm: &str) -> Result<VerifyEntry, ContainerError> {
    // Parse companion log (returns Option, so provide error if not found)
    let log_info = parse_companion_log(path)
        .ok_or_else(|| ContainerError::FileNotFound(format!("No companion log file found for: {}", path)))?;
    
    // Get expected hash from log
    let expected_hash = match algorithm.to_lowercase().as_str() {
        "md5" => log_info.md5_hash,
        "sha1" => log_info.sha1_hash,
        "sha256" => log_info.sha256_hash,
        _ => return Err(ContainerError::ConfigError(format!("Unsupported algorithm: {}", algorithm))),
    };
    
    // Compute actual hash
    let computed_hash = hash_segments(path, algorithm)?;
    
    // Compare
    let status = match &expected_hash {
        Some(expected) => {
            if expected.to_lowercase() == computed_hash.to_lowercase() {
                VerifyStatus::Ok
            } else {
                VerifyStatus::Nok
            }
        }
        None => VerifyStatus::Computed,
    };
    
    Ok(VerifyEntry {
        path: path.to_string(),
        status,
        algorithm: Some(algorithm.to_string()),
        computed: Some(computed_hash),
        stored: expected_hash,
        size: None,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a minimal AD1 file that passes signature validation
    /// Note: This creates a structurally invalid but signature-valid file
    fn create_test_ad1(dir: &Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        
        // Write AD1 segment header signature: "ADSEGMENTEDFILE\0"
        file.write_all(b"ADSEGMENTEDFILE\0").unwrap();
        // Pad to header size (512 bytes minimum)
        file.write_all(&[0u8; 496]).unwrap();
        
        path
    }

    #[test]
    fn test_is_ad1_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        
        let result = is_ad1(ad1_path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_ad1_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("not_ad1.bin");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOT AN AD1 FILE!").unwrap(); // 16 bytes
        
        let result = is_ad1(path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_ad1_nonexistent_file() {
        let result = is_ad1("/nonexistent/path/file.ad1");
        // Returns Err for nonexistent files (can't open)
        assert!(result.is_err());
    }

    #[test]
    fn test_is_ad1_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.ad1");
        File::create(&path).unwrap();
        
        let result = is_ad1(path.to_str().unwrap());
        // Returns Err for empty files (can't read 16 bytes)
        assert!(result.is_err());
    }

    #[test]
    fn test_info_fast_nonexistent() {
        let result = info_fast("/nonexistent/path/file.ad1");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found") || err.contains("No such file") || err.contains("Failed"));
    }

    #[test]
    fn test_info_nonexistent() {
        let result = info("/nonexistent/path/file.ad1", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_nonexistent() {
        let result = verify("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_algorithm() {
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        
        // File exists but algorithm parsing happens after file validation
        // So this tests the format validation path
        let result = verify(ad1_path.to_str().unwrap(), "md5");
        // Will fail on parsing (incomplete AD1), but the algorithm is valid
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_nonexistent() {
        let result = extract("/nonexistent/path/file.ad1", "/tmp/output");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_tree_nonexistent() {
        let result = get_tree("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_children_nonexistent() {
        let result = get_children("/nonexistent/path/file.ad1", "/");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_data_nonexistent() {
        let result = read_entry_data("/nonexistent/path/file.ad1", "/some/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_chunk_nonexistent() {
        let result = read_entry_chunk("/nonexistent/path/file.ad1", "/some/file.txt", 0, 1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_segments_nonexistent() {
        let result = hash_segments("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_tree_entry_file() {
        let item = Item {
            id: 1,
            name: "test.txt".to_string(),
            item_type: 0, // File
            decompressed_size: 1024,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        
        let entry = build_tree_entry(&item, "/documents", true);
        
        assert_eq!(entry.path, "/documents/test.txt");
        assert!(!entry.is_dir);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.item_type, 0);
    }

    #[test]
    fn test_build_tree_entry_folder() {
        let item = Item {
            id: 2,
            name: "folder".to_string(),
            item_type: AD1_FOLDER_SIGNATURE, // Folder
            decompressed_size: 0,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        
        let entry = build_tree_entry(&item, "", true);
        
        assert_eq!(entry.path, "folder");
        assert!(entry.is_dir);
        assert_eq!(entry.size, 0);
        assert_eq!(entry.item_type, AD1_FOLDER_SIGNATURE);
    }

    // Tests for new functions

    #[test]
    fn test_get_segment_paths_nonexistent() {
        let result = get_segment_paths("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_stats_nonexistent() {
        let result = get_stats("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_chunks_nonexistent() {
        let result = verify_chunks("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_name_nonexistent() {
        let result = find_by_name("/nonexistent/path/file.ad1", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_extension_nonexistent() {
        let result = find_by_extension("/nonexistent/path/file.ad1", "txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_hash_nonexistent() {
        let result = find_by_hash("/nonexistent/path/file.ad1", "abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_tree_json_nonexistent() {
        let result = export_tree_json("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_tree_csv_nonexistent() {
        let result = export_tree_csv("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_against_log_nonexistent() {
        let result = verify_against_log("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_ad1_stats_default() {
        let stats = Ad1Stats::default();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_folders, 0);
        assert_eq!(stats.compression_ratio, 0.0);
    }

    // =============================================================================
    // Edge Case Tests - Error Paths
    // =============================================================================

    #[test]
    fn test_verify_status_display() {
        assert_eq!(VerifyStatus::Ok.to_string(), "ok");
        assert_eq!(VerifyStatus::Nok.to_string(), "nok");
        assert_eq!(VerifyStatus::Computed.to_string(), "computed");
        assert_eq!(VerifyStatus::Skipped.to_string(), "skipped");
    }

    #[test]
    fn test_verify_status_is_ok() {
        assert!(VerifyStatus::Ok.is_ok());
        assert!(VerifyStatus::Computed.is_ok());
        assert!(!VerifyStatus::Nok.is_ok());
        assert!(!VerifyStatus::Skipped.is_ok());
    }

    #[test]
    fn test_verify_status_is_error() {
        assert!(VerifyStatus::Nok.is_error());
        assert!(!VerifyStatus::Ok.is_error());
        assert!(!VerifyStatus::Computed.is_error());
        assert!(!VerifyStatus::Skipped.is_error());
    }

    #[test]
    fn test_is_ad1_partial_signature() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("partial.ad1");
        let mut file = File::create(&path).unwrap();
        // Write partial signature - starts right but not complete
        file.write_all(b"ADSEGMENTED").unwrap(); // Only 11 bytes
        
        let result = is_ad1(path.to_str().unwrap());
        // Should fail - can't read full signature
        assert!(result.is_err());
    }

    #[test]
    fn test_read_entry_data_invalid_path_format() {
        // Test with empty path which should be handled gracefully
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        
        // Session will fail to open properly, but we test the error path
        let result = read_entry_data(ad1_path.to_str().unwrap(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_info_nonexistent() {
        let result = get_entry_info("/nonexistent/path/file.ad1", "/some/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_children_at_addr_nonexistent() {
        let result = get_children_at_addr("/nonexistent/path/file.ad1", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_json_nonexistent() {
        let result = export_metadata_json("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_metadata_csv_nonexistent() {
        let result = export_metadata_csv("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_single_segment_nonexistent() {
        let result = hash_single_segment("/nonexistent/path/file.ad1", "md5", |_, _| {});
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("File not found"));
    }

    #[test]
    fn test_hash_single_segment_invalid_algorithm() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.ad1");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"test content").unwrap();
        
        let result = hash_single_segment(path.to_str().unwrap(), "invalid_algo", |_, _| {});
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unsupported") || err_msg.contains("algorithm"));
    }

    #[test]
    fn test_verify_against_log_invalid_algorithm() {
        // Even with nonexistent file, should check algorithm validity
        let result = verify_against_log("/nonexistent/path/file.ad1", "sha3"); // SHA3 not supported
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_csv_empty_string() {
        assert_eq!(escape_csv(""), "");
    }

    #[test]
    fn test_escape_csv_unicode() {
        assert_eq!(escape_csv("文件.txt"), "文件.txt");
        assert_eq!(escape_csv("文件,名称"), "\"文件,名称\"");
    }

    #[test]
    fn test_escape_csv_long_string() {
        let long_string = "a".repeat(10000);
        assert_eq!(escape_csv(&long_string), long_string);
    }

    #[test]
    fn test_build_tree_entry_root_path() {
        let item = Item {
            id: 1,
            name: "root_file.txt".to_string(),
            item_type: 0,
            decompressed_size: 512,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        
        // Empty parent path
        let entry = build_tree_entry(&item, "", true);
        assert_eq!(entry.path, "root_file.txt");
    }

    #[test]
    fn test_build_tree_entry_nested_path() {
        let item = Item {
            id: 1,
            name: "deep.txt".to_string(),
            item_type: 0,
            decompressed_size: 256,
            zlib_metadata_addr: 0,
            metadata: vec![],
            children: vec![],
        };
        
        // Deeply nested parent path
        let entry = build_tree_entry(&item, "/a/b/c/d/e", true);
        assert_eq!(entry.path, "/a/b/c/d/e/deep.txt");
    }

    #[test]
    fn test_tree_entry_serialization() {
        let entry = TreeEntry {
            path: "/test.txt".to_string(),
            name: "test.txt".to_string(),
            is_dir: false,
            size: 1024,
            item_type: 0,
            first_child_addr: None,
            data_addr: Some(12345),
            item_addr: Some(12345),
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: None,
            created: Some("2024-01-01T00:00:00Z".to_string()),
            accessed: None,
            modified: None,
            attributes: None,
            child_count: None,
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test.txt"));
        assert!(json.contains("d41d8cd98f00b204e9800998ecf8427e"));
        // Optional None fields should be skipped
        assert!(!json.contains("sha1_hash"));
    }

    #[test]
    fn test_verify_entry_serialization() {
        let entry = VerifyEntry {
            path: "/file.txt".to_string(),
            status: VerifyStatus::Ok,
            algorithm: Some("md5".to_string()),
            computed: Some("abc123".to_string()),
            stored: Some("abc123".to_string()),
            size: Some(1024),
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("ok"));
        assert!(json.contains("abc123"));
    }

    /// Integration test with real AD1 file (only runs if file exists)
    #[test]
    fn test_real_ad1_get_children() {
        // Use single-segment AD1 file that has all data
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_401358/02606-0900_1E_401358_img1.ad1";
        if !std::path::Path::new(path).exists() {
            println!("Skipping test - file not found: {}", path);
            return;
        }
        
        println!("Testing with real AD1 file: {}", path);
        
        // First verify it's a valid AD1
        let is_valid = is_ad1(path);
        println!("is_ad1 result: {:?}", is_valid);
        assert!(is_valid.is_ok(), "is_ad1 should succeed");
        assert!(is_valid.unwrap(), "File should be recognized as AD1");
        
        // Now get root children
        let result = get_children(path, "");
        println!("get_children result: {:?}", result.as_ref().map(|v| v.len()));
        
        match result {
            Ok(entries) => {
                println!("SUCCESS: Found {} root entries", entries.len());
                for (i, e) in entries.iter().enumerate().take(10) {
                    println!("  [{}] {} (dir={}, size={}, item_addr={:?})", 
                        i, e.name, e.is_dir, e.size, e.item_addr);
                }
                // Should have at least some entries
                // Note: Empty containers are valid, so we just verify no error
            }
            Err(e) => {
                panic!("get_children failed: {}", e);
            }
        }
    }
}
