// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Utility Functions
//!
//! ## Section Brief
//! Helper functions for AD1 container parsing and data extraction:
//!
//! ### Segment Management
//! - `get_segment_files_with_sizes()` - Discover segment files with sizes
//! - `build_segment_path()` - Build path for segment N (delegates to common)
//! - `segment_span()` - Calculate segment data span
//!
//! ### Input Validation
//! - `validate_input()` - Validate file exists and is readable
//!
//! ### Path Utilities
//! - `join_path()` - Join path components for container paths
//!
//! ### Data Reading
//! - `read_bytes()` - Read bytes at address (cross-segment)
//! - `read_u32_at_addr()`, `read_u64_at_addr()` - Read integers
//! - `read_zlib_string()` - Read zlib-compressed string
//!
//! ### Metadata Extraction
//! - `find_hash()` - Find hash in metadata by type
//! - `find_timestamp()` - Find timestamp in metadata
//! - `parse_volume_info()` - Parse volume information
//! - `parse_companion_log()` - Parse companion log (delegates to shared)
//!
//! ### Tree Building
//! - `collect_tree()` - Build flat tree from items
//! - `collect_children_at_path()` - Collect children at specific path
//!
//! ### Header Parsing Helpers
//! - `segment_header_info()` - Convert internal to public header info
//! - `logical_header_info()` - Convert internal to public header info

use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::time::SystemTime;
use chrono::{Local, NaiveDateTime, TimeZone};
use filetime::FileTime;
use tracing::trace;

use super::types::*;
use crate::containers::ContainerError;
use crate::common::binary::{read_u32_at, read_u64_at, read_string_at};
use crate::common::segments::{discover_ad1_segments, build_ad1_segment_path};
use crate::containers::companion::find_companion_log as find_shared_companion_log;

// =============================================================================
// Segment Management
// =============================================================================

/// Get segment files with their sizes and track missing segments
/// Returns (segment_names, segment_sizes, total_size, missing_segments)
/// 
/// Uses shared segment discovery from common/segments.rs
pub fn get_segment_files_with_sizes(path: &str, segment_count: u32) -> (Vec<String>, Vec<u64>, u64, Vec<String>) {
    let (paths, sizes, missing) = discover_ad1_segments(path, segment_count);
    
    let segment_names: Vec<String> = paths.iter()
        .filter_map(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .collect();
    
    let total_size: u64 = sizes.iter().sum();
    
    (segment_names, sizes, total_size, missing)
}

/// Get detailed segment summary with offset ranges
pub fn get_segment_summary(path: &str, segment_count: u32, fragments_size: u32) -> SegmentSummary {
    let path_obj = Path::new(path);
    let parent = path_obj.parent();
    let stem = path_obj.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let seg_span = segment_span(fragments_size);
    let mut segments = Vec::with_capacity(segment_count as usize);
    let mut total_size = 0u64;
    let mut total_data_size = 0u64;
    let mut found_count = 0u32;
    let mut missing_count = 0u32;
    
    for i in 1..=segment_count {
        let segment_name = format!("{}.ad{}", stem, i);
        let segment_path = if let Some(parent_dir) = parent {
            parent_dir.join(&segment_name).to_string_lossy().to_string()
        } else {
            segment_name.clone()
        };
        
        let seg_path = Path::new(&segment_path);
        let exists = seg_path.exists();
        let size = if exists {
            std::fs::metadata(&segment_path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        
        let data_size = size.saturating_sub(AD1_LOGICAL_MARGIN);
        let offset_start = (i as u64 - 1) * seg_span;
        let offset_end = offset_start + data_size;
        
        if exists {
            found_count += 1;
            total_size += size;
            total_data_size += data_size;
        } else {
            missing_count += 1;
        }
        
        segments.push(SegmentFileInfo {
            number: i,
            path: segment_path,
            filename: segment_name,
            size,
            exists,
            data_size,
            offset_start,
            offset_end,
        });
    }
    
    SegmentSummary {
        expected_count: segment_count,
        found_count,
        missing_count,
        total_size,
        total_data_size,
        segments,
        is_complete: missing_count == 0,
    }
}

/// Validate AD1 file format
/// 
/// # Arguments
/// * `path` - Path to the AD1 file
/// * `check_segments` - If true, verify all segment files exist (strict mode)
/// 
/// # Returns
/// Ok(()) if valid, Err with description if invalid
pub fn validate_ad1(path: &str, check_segments: bool) -> Result<(), ContainerError> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(path.to_string()));
    }

    let mut file = File::open(path_obj)
        .map_err(|e| ContainerError::IoError(format!("Failed to open input file: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read file signature: {e}")))?;
    if &signature[..15] != AD1_SIGNATURE {
        return Err(ContainerError::InvalidFormat("File is not an AD1 segmented image".to_string()));
    }

    let segment_count = read_u32_at(&mut file, 0x1c)?;
    if segment_count == 0 {
        return Err(ContainerError::InvalidFormat("Invalid AD1 segment count".to_string()));
    }

    if check_segments {
        for index in 1..=segment_count {
            let segment_path = build_ad1_segment_path(path, index);
            if !Path::new(&segment_path).exists() {
                return Err(ContainerError::SegmentError(format!("Missing AD1 segment: {segment_path}")));
            }
        }
    }

    Ok(())
}

/// Validate AD1 file format (does not check segments)
/// Convenience wrapper for `validate_ad1(path, false)`
#[inline]
#[allow(dead_code)]
pub fn validate_format(path: &str) -> Result<(), ContainerError> {
    validate_ad1(path, false)
}

/// Build segment file path from base path and segment index
/// Wrapper around shared function for backwards compatibility
pub fn build_segment_path(base: &str, index: u32) -> String {
    build_ad1_segment_path(base, index)
}

/// Read segment header from file
pub fn read_segment_header(file: &mut File) -> Result<SegmentHeader, ContainerError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek segment header: {e}")))?;
    let mut signature = [0u8; 16];
    file.read_exact(&mut signature)
        .map_err(|e| ContainerError::IoError(format!("Failed to read segment signature: {e}")))?;
    if &signature[..15] != AD1_SIGNATURE {
        return Err(ContainerError::InvalidFormat("File is not of AD1 format".to_string()));
    }

    Ok(SegmentHeader {
        signature,
        segment_index: read_u32_at(file, 0x18)?,
        segment_number: read_u32_at(file, 0x1c)?,
        fragments_size: read_u32_at(file, 0x22)?,
        header_size: read_u32_at(file, 0x28)?,
    })
}

/// Read logical header from file
pub fn read_logical_header(file: &mut File) -> Result<LogicalHeader, ContainerError> {
    let signature = read_string_at(file, AD1_LOGICAL_MARGIN, 15)?;
    let image_version = read_u32_at(file, 0x210)?;
    let zlib_chunk_size = read_u32_at(file, 0x218)?;
    let logical_metadata_addr = read_u64_at(file, 0x21c)?;
    let first_item_addr = read_u64_at(file, 0x224)?;
    let data_source_name_length = read_u32_at(file, 0x22c)?;
    let ad_signature = read_string_at(file, 0x230, 3)?;
    let data_source_name_addr = read_u64_at(file, 0x234)?;
    let attrguid_footer_addr = read_u64_at(file, 0x23c)?;
    let locsguid_footer_addr = read_u64_at(file, 0x24c)?;
    let data_source_name = read_string_at(file, 0x25c, data_source_name_length as usize)?;

    Ok(LogicalHeader {
        signature: copy_into_array(&signature, 16)?,
        image_version,
        zlib_chunk_size,
        logical_metadata_addr,
        first_item_addr,
        data_source_name_length,
        ad_signature: copy_into_array(&ad_signature, 4)?,
        data_source_name_addr,
        attrguid_footer_addr,
        locsguid_footer_addr,
        data_source_name,
    })
}

/// Copy string into fixed-size byte array
pub fn copy_into_array<const N: usize>(value: &str, max_len: usize) -> Result<[u8; N], ContainerError> {
    let mut buf = [0u8; N];
    let bytes = value.as_bytes();
    let len = bytes.len().min(max_len).min(N);
    buf[..len].copy_from_slice(&bytes[..len]);
    Ok(buf)
}

/// Calculate segment span from fragments size
pub fn segment_span(fragments_size: u32) -> u64 {
    (fragments_size as u64 * SEGMENT_BLOCK_SIZE).saturating_sub(AD1_LOGICAL_MARGIN)
}

/// Convert bytes to string (stops at null terminator)
/// 
/// # Arguments
/// * `bytes` - Raw byte slice to convert
/// * `trim` - If true, trim leading/trailing whitespace from result
/// 
/// # Returns
/// UTF-8 string up to first null terminator, optionally trimmed
pub fn bytes_to_string(bytes: &[u8], trim: bool) -> String {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let s = String::from_utf8_lossy(&bytes[..end]);
    if trim { s.trim().to_string() } else { s.to_string() }
}

/// Join path components
pub fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else if name.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Convert SegmentHeader to public SegmentHeaderInfo
pub fn segment_header_info(header: &SegmentHeader) -> SegmentHeaderInfo {
    SegmentHeaderInfo {
        signature: bytes_to_string(&header.signature, false),
        segment_index: header.segment_index,
        segment_number: header.segment_number,
        fragments_size: header.fragments_size,
        header_size: header.header_size,
    }
}

/// Convert LogicalHeader to public LogicalHeaderInfo  
pub fn logical_header_info(header: &LogicalHeader) -> LogicalHeaderInfo {
    LogicalHeaderInfo {
        signature: bytes_to_string(&header.signature, false),
        image_version: header.image_version,
        zlib_chunk_size: header.zlib_chunk_size,
        logical_metadata_addr: header.logical_metadata_addr,
        first_item_addr: header.first_item_addr,
        data_source_name_length: header.data_source_name_length,
        ad_signature: bytes_to_string(&header.ad_signature, false),
        data_source_name_addr: header.data_source_name_addr,
        attrguid_footer_addr: header.attrguid_footer_addr,
        locsguid_footer_addr: header.locsguid_footer_addr,
        data_source_name: header.data_source_name.clone(),
    }
}

/// Apply metadata timestamps to extracted file
pub fn apply_metadata(path: &Path, metadata: &[Metadata]) -> Result<(), ContainerError> {
    let mut access_time = None;
    let mut modified_time = None;

    for meta in metadata {
        if meta.category != TIMESTAMP {
            continue;
        }
        let value = bytes_to_string(&meta.data, true);
        match meta.key {
            ACCESS => access_time = parse_timestamp(&value),
            MODIFIED => modified_time = parse_timestamp(&value),
            _ => {}
        }
    }

    if access_time.is_none() && modified_time.is_none() {
        return Ok(());
    }

    let now = FileTime::from_system_time(SystemTime::now());
    let atime = access_time.unwrap_or(now);
    let mtime = modified_time.unwrap_or(atime);
    filetime::set_file_times(path, atime, mtime)
        .map_err(|e| format!("Failed to set file times for {:?}: {e}", path))?;
    Ok(())
}

/// Parse AD1 timestamp string to FileTime
/// AD1 timestamps are in format: %Y%m%dT%H%M%S (e.g., "20240115T143022")
pub fn parse_timestamp(value: &str) -> Option<FileTime> {
    let parsed = parse_ad1_timestamp(value)?;
    let local = Local
        .from_local_datetime(&parsed)
        .single()
        .unwrap_or_else(|| Local.from_utc_datetime(&parsed));
    Some(FileTime::from_unix_time(local.timestamp(), 0))
}

/// Parse AD1 timestamp string to NaiveDateTime
/// Core parsing logic used by both FileTime and ISO conversions
fn parse_ad1_timestamp(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim_matches('\0').trim();
    if trimmed.len() < 15 {
        return None;
    }
    NaiveDateTime::parse_from_str(trimmed, "%Y%m%dT%H%M%S").ok()
}

/// Find hash value in metadata
pub fn find_hash(metadata: &[Metadata], key: u32) -> Option<String> {
    // Debug: log all hash-related metadata entries
    for meta in metadata {
        if meta.category == HASH_INFO {
            let value = bytes_to_string(&meta.data, true);
            trace!(
                category = meta.category,
                key = format!("0x{:04x}", meta.key),
                expected_key = format!("0x{:04x}", key),
                value = %value,
                "Found hash metadata entry"
            );
        }
    }
    
    metadata
        .iter()
        .find(|meta| meta.category == HASH_INFO && meta.key == key)
        .map(|meta| bytes_to_string(&meta.data, true))
        .map(|value| {
            // Clean up the hash value - remove any whitespace or non-hex characters
            let cleaned: String = value.chars()
                .filter(|c| c.is_ascii_hexdigit())
                .collect();
            cleaned.to_lowercase()
        })
}

/// Find SHA256 hash in metadata
#[allow(dead_code)]
pub fn find_sha256_hash(metadata: &[Metadata]) -> Option<String> {
    find_hash_by_key(metadata, SHA256_HASH)
}

/// Find hash by specific key
#[allow(dead_code)]
fn find_hash_by_key(metadata: &[Metadata], key: u32) -> Option<String> {
    metadata
        .iter()
        .find(|meta| meta.category == HASH_INFO && meta.key == key)
        .map(|meta| bytes_to_string(&meta.data, true))
        .map(|value| {
            let cleaned: String = value.chars()
                .filter(|c| c.is_ascii_hexdigit())
                .collect();
            cleaned.to_lowercase()
        })
}

/// Extract timestamp from metadata by key
pub fn find_timestamp(metadata: &[Metadata], key: u32) -> Option<String> {
    metadata
        .iter()
        .find(|meta| meta.category == TIMESTAMP && meta.key == key)
        .and_then(|meta| {
            let value = bytes_to_string(&meta.data, true);
            parse_timestamp_to_iso(&value)
        })
}

/// Parse AD1 timestamp to ISO 8601 format
/// Uses shared parsing logic from parse_ad1_timestamp
fn parse_timestamp_to_iso(value: &str) -> Option<String> {
    let parsed = parse_ad1_timestamp(value)?;
    Some(parsed.format("%Y-%m-%dT%H:%M:%S").to_string())
}

/// Extract file attributes from metadata
fn extract_attributes(metadata: &[Metadata]) -> Option<Vec<String>> {
    let mut attrs = Vec::new();
    
    for meta in metadata {
        if meta.category != ATTRIBUTES {
            continue;
        }
        // Check attribute flags
        let attr_name = match meta.key {
            READONLY => Some("readonly"),
            HIDDEN => Some("hidden"),
            SYSTEM => Some("system"),
            ARCHIVE => Some("archive"),
            ENCRYPTED => Some("encrypted"),
            COMPRESSED => Some("compressed"),
            _ => None,
        };
        
        if let Some(name) = attr_name {
            // Check if the attribute value indicates true (non-zero)
            if !meta.data.is_empty() && meta.data.iter().any(|&b| b != 0) {
                attrs.push(name.to_string());
            }
        }
    }
    
    if attrs.is_empty() {
        None
    } else {
        Some(attrs)
    }
}

// =============================================================================
// Unified Item Finding
// =============================================================================

/// Result of finding an item in the tree
#[derive(Debug)]
pub struct FoundItem<'a> {
    /// The found item
    pub item: &'a Item,
    /// Path to the parent of this item
    pub parent_path: String,
    /// Full path to this item (may be useful for debugging)
    #[allow(dead_code)]
    pub full_path: String,
}

/// Find an item by path in the item tree
/// 
/// # Arguments
/// * `items` - Root items to search
/// * `target_path` - Path to find (e.g., "/folder/file.txt")
/// 
/// # Returns
/// Some(FoundItem) if found, None otherwise
pub fn find_item_by_path<'a>(items: &'a [Item], target_path: &str) -> Option<FoundItem<'a>> {
    fn search<'a>(items: &'a [Item], target: &str, current_path: &str) -> Option<FoundItem<'a>> {
        for item in items {
            let item_path = join_path(current_path, &item.name);
            if item_path == target {
                return Some(FoundItem {
                    item,
                    parent_path: current_path.to_string(),
                    full_path: item_path,
                });
            }
            // If target is deeper, recurse
            if target.starts_with(&item_path) && !item.children.is_empty() {
                if let Some(found) = search(&item.children, target, &item_path) {
                    return Some(found);
                }
            }
        }
        None
    }
    
    // Normalize: strip leading slash since join_path builds paths without leading slashes
    let target = target_path.trim_start_matches('/');
    if target.is_empty() {
        return None;
    }
    search(items, target, "")
}

/// Find children of an item at a specific path
/// 
/// # Arguments
/// * `items` - Root items to search
/// * `parent_path` - Parent path (empty or "/" for root)
/// 
/// # Returns
/// Slice of child items, or root items if parent_path is empty/root
#[allow(dead_code)]
pub fn find_children_at_path<'a>(items: &'a [Item], parent_path: &str) -> Option<&'a [Item]> {
    let normalized = parent_path.trim_matches('/');
    
    if normalized.is_empty() {
        return Some(items);
    }
    
    // Navigate to the parent and return its children
    if let Some(found) = find_item_by_path(items, parent_path) {
        Some(&found.item.children)
    } else {
        None
    }
}

// =============================================================================
// TreeEntry Construction
// =============================================================================

/// Convert an Item to a TreeEntry with full metadata
/// 
/// # Arguments
/// * `item` - The item to convert
/// * `parent_path` - Path to the parent of this item
/// * `include_metadata` - If true, extract hashes and timestamps
pub fn build_tree_entry(item: &Item, parent_path: &str, include_metadata: bool) -> TreeEntry {
    let path = join_path(parent_path, &item.name);
    let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
    let size = if is_dir { 0 } else { item.decompressed_size };
    
    // Address fields for lazy loading and hex view
    let addr = if item.zlib_metadata_addr > 0 { Some(item.zlib_metadata_addr) } else { None };
    let first_child_addr = if is_dir && !item.children.is_empty() { addr } else { None };
    let data_addr = if !is_dir { addr } else { None };
    let child_count = if is_dir { Some(item.children.len()) } else { None };
    
    if include_metadata {
        // Extract metadata (hashes for files, timestamps for both)
        let (md5_hash, sha1_hash, attributes) = if !is_dir {
            (
                find_hash(&item.metadata, MD5_HASH),
                find_hash(&item.metadata, SHA1_HASH),
                extract_attributes(&item.metadata),
            )
        } else {
            (None, None, None)
        };
        
        TreeEntry {
            path,
            name: item.name.clone(),
            is_dir,
            size,
            item_type: item.item_type,
            first_child_addr,
            data_addr,
            item_addr: addr,
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash,
            sha1_hash,
            created: find_timestamp(&item.metadata, CREATED),
            accessed: find_timestamp(&item.metadata, ACCESS),
            modified: find_timestamp(&item.metadata, MODIFIED),
            attributes,
            child_count,
        }
    } else {
        TreeEntry {
            path,
            name: item.name.clone(),
            is_dir,
            size,
            item_type: item.item_type,
            first_child_addr,
            data_addr,
            item_addr: addr,
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: None,
            sha1_hash: None,
            created: None,
            accessed: None,
            modified: None,
            attributes: None,
            child_count,
        }
    }
}

/// Convert an Item from lazy loading to a TreeEntry
/// For lazy-loaded items, zlib_metadata_addr contains first_child_addr for folders
/// 
/// # Arguments
/// * `item` - The item to convert (from read_item_shallow)
/// * `parent_path` - Path to the parent of this item
pub fn build_tree_entry_lazy(item: &Item, parent_path: &str) -> TreeEntry {
    let path = join_path(parent_path, &item.name);
    let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
    let size = if is_dir { 0 } else { item.decompressed_size };
    
    // For lazy-loaded items:
    // - Folders: zlib_metadata_addr holds first_child_addr
    // - Files: zlib_metadata_addr holds actual zlib metadata address
    let addr = if item.zlib_metadata_addr > 0 { Some(item.zlib_metadata_addr) } else { None };
    let first_child_addr = if is_dir { addr } else { None };
    let data_addr = if !is_dir { addr } else { None };
    
    // Extract metadata (hashes for files, timestamps for both)
    let (md5_hash, sha1_hash, attributes) = if !is_dir {
        (
            find_hash(&item.metadata, MD5_HASH),
            find_hash(&item.metadata, SHA1_HASH),
            extract_attributes(&item.metadata),
        )
    } else {
        (None, None, None)
    };
    
    TreeEntry {
        path,
        name: item.name.clone(),
        is_dir,
        size,
        item_type: item.item_type,
        first_child_addr,
        data_addr,
        item_addr: addr,
        compressed_size: None,
        data_end_addr: None,
        metadata_addr: None,
        md5_hash,
        sha1_hash,
        created: find_timestamp(&item.metadata, CREATED),
        accessed: find_timestamp(&item.metadata, ACCESS),
        modified: find_timestamp(&item.metadata, MODIFIED),
        attributes,
        child_count: None, // Unknown for lazy-loaded items
    }
}

// =============================================================================
// Tree Collection
// =============================================================================

/// Collect tree entries recursively with full metadata
pub fn collect_tree(items: &[Item], parent_path: &str, out: &mut Vec<TreeEntry>) {
    for item in items {
        let entry = build_tree_entry(item, parent_path, true);
        let path = entry.path.clone();
        out.push(entry);
        collect_tree(&item.children, &path, out);
    }
}

/// Collect tree entries without metadata (faster, for tree display only)
#[allow(dead_code)]
pub fn collect_tree_simple(items: &[Item], parent_path: &str, out: &mut Vec<TreeEntry>) {
    for item in items {
        let entry = build_tree_entry(item, parent_path, false);
        let path = entry.path.clone();
        out.push(entry);
        collect_tree_simple(&item.children, &path, out);
    }
}

/// Collect children at a specific path (for lazy loading)
/// If parent_path is empty or "/", returns root items
pub fn collect_children_at_path(items: &[Item], target_path: &str, current_path: &str, out: &mut Vec<TreeEntry>) {
    // Normalize target path
    let target = target_path.trim_matches('/');
    
    if target.is_empty() {
        // Return root-level items
        for item in items {
            let path = join_path(current_path, &item.name);
            let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
            let size = if is_dir { 0 } else { item.decompressed_size };
            let addr = if item.zlib_metadata_addr > 0 { Some(item.zlib_metadata_addr) } else { None };
            let first_child_addr = if is_dir && !item.children.is_empty() { addr } else { None };
            let data_addr = if !is_dir { addr } else { None };
            out.push(TreeEntry {
                path,
                name: item.name.clone(),
                is_dir,
                size,
                item_type: item.item_type,
                first_child_addr,
                data_addr,
                item_addr: addr,
                compressed_size: None,
                data_end_addr: None,
                metadata_addr: None,
                md5_hash: find_hash(&item.metadata, MD5_HASH),
                sha1_hash: find_hash(&item.metadata, SHA1_HASH),
                created: find_timestamp(&item.metadata, CREATED),
                accessed: find_timestamp(&item.metadata, ACCESS),
                modified: find_timestamp(&item.metadata, MODIFIED),
                attributes: None,
                child_count: if is_dir { Some(item.children.len()) } else { None },
            });
        }
        return;
    }
    
    // Navigate to the target path and return its children
    for item in items {
        let item_path = join_path(current_path, &item.name);
        let relative = item_path.trim_start_matches('/');
        
        if relative == target {
            // Found the target - return its children
            for child in &item.children {
                let child_path = join_path(&item_path, &child.name);
                let is_dir = child.item_type == AD1_FOLDER_SIGNATURE;
                let size = if is_dir { 0 } else { child.decompressed_size };
                let addr = if child.zlib_metadata_addr > 0 { Some(child.zlib_metadata_addr) } else { None };
                let first_child_addr = if is_dir && !child.children.is_empty() { addr } else { None };
                let data_addr = if !is_dir { addr } else { None };
                out.push(TreeEntry {
                    path: child_path,
                    name: child.name.clone(),
                    is_dir,
                    size,
                    item_type: child.item_type,
                    first_child_addr,
                    data_addr,
                    item_addr: addr,
                    compressed_size: None,
                    data_end_addr: None,
                    metadata_addr: None,
                    md5_hash: find_hash(&child.metadata, MD5_HASH),
                    sha1_hash: find_hash(&child.metadata, SHA1_HASH),
                    created: find_timestamp(&child.metadata, CREATED),
                    accessed: find_timestamp(&child.metadata, ACCESS),
                    modified: find_timestamp(&child.metadata, MODIFIED),
                    attributes: None,
                    child_count: if is_dir { Some(child.children.len()) } else { None },
                });
            }
            return;
        } else if target.starts_with(relative) && !item.children.is_empty() {
            // Target is deeper - recurse
            collect_children_at_path(&item.children, target_path, &item_path, out);
            return;
        }
    }
}

/// Count total files (non-folders) in item tree
pub fn count_files(items: &[Item]) -> u64 {
    items.iter().map(|item| {
        let self_count: u64 = if item.item_type != AD1_FOLDER_SIGNATURE { 1 } else { 0 };
        self_count + count_files(&item.children)
    }).sum()
}

/// Parse volume info from AD1 header region
pub fn parse_volume_info(file: &mut File) -> Option<VolumeInfo> {
    // Volume info is typically at offset 0x2A0+ in the logical header
    // Format: "C:\:NONAME [NTFS]" followed by OS info like "Windows XP (NTFS 3.1)"
    
    let mut info = VolumeInfo::default();
    
    // Read volume label region (around 0x2A0-0x2C0)
    if let Ok(volume_str) = read_string_at(file, 0x2A8, 64) {
        let volume_trimmed = volume_str.trim_matches(char::from(0)).trim();
        if !volume_trimmed.is_empty() && volume_trimmed.contains(':') {
            // Parse "C:\:NONAME [NTFS]" format
            if let Some(bracket_start) = volume_trimmed.find('[') {
                if let Some(bracket_end) = volume_trimmed.find(']') {
                    info.filesystem = Some(volume_trimmed[bracket_start+1..bracket_end].to_string());
                }
                info.volume_label = Some(volume_trimmed[..bracket_start].trim().to_string());
            } else {
                info.volume_label = Some(volume_trimmed.to_string());
            }
        }
    }
    
    // Read OS info region (around 0x370-0x3A0)
    if let Ok(os_str) = read_string_at(file, 0x370, 64) {
        let os_trimmed = os_str.trim_matches(char::from(0)).trim();
        if !os_trimmed.is_empty() && (os_trimmed.contains("Windows") || os_trimmed.contains("NTFS") || os_trimmed.contains("Linux")) {
            info.os_info = Some(os_trimmed.to_string());
        }
    }
    
    // Read block size (typically at 0x2E8)
    if let Ok(block_size_str) = read_string_at(file, 0x2E8, 8) {
        let block_trimmed = block_size_str.trim_matches(char::from(0)).trim();
        if let Ok(block_size) = block_trimmed.parse::<u32>() {
            if block_size > 0 && block_size <= 65536 {
                info.block_size = Some(block_size);
            }
        }
    }
    
    // Only return if we found something useful
    if info.volume_label.is_some() || info.filesystem.is_some() || info.os_info.is_some() {
        Some(info)
    } else {
        None
    }
}

/// Parse companion log file (.ad1.txt, .log, .csv) for case metadata
/// 
/// Uses shared companion log finder from containers/companion.rs for file discovery,
/// then converts to AD1-specific CompanionLogInfo struct.
/// 
/// Supports multiple companion file formats:
/// - `filename.ad1.txt` - Standard FTK companion log
/// - `filename.txt` - Simple text companion  
/// - `filename.ad1.log` - Alternative log format
/// - `filename_log.txt` - FTK log naming convention
/// - `filename.ad1.csv` - CSV export format
pub fn parse_companion_log(ad1_path: &str) -> Option<CompanionLogInfo> {
    // Use shared companion log finder
    let shared_info = find_shared_companion_log(ad1_path)?;
    
    // Convert from shared CompanionLogInfo to AD1-specific format
    // The shared struct has more fields; we extract what AD1 needs
    let mut info = CompanionLogInfo {
        case_number: shared_info.case_number,
        evidence_number: shared_info.evidence_number,
        examiner: shared_info.examiner,
        notes: shared_info.notes,
        md5_hash: None,
        sha1_hash: None,
        sha256_hash: None,
        acquisition_date: shared_info.acquisition_started,
        source_device: shared_info.unique_description,
        source_path: None,
        acquisition_tool: shared_info.created_by,
        total_items: None,
        total_size: None,
        acquisition_method: None,
        organization: None,
    };
    
    // Extract hashes from stored_hashes Vec
    for hash in &shared_info.stored_hashes {
        let algo_lower = hash.algorithm.to_lowercase();
        if algo_lower.contains("md5") {
            info.md5_hash = Some(hash.hash.clone());
        } else if algo_lower.contains("sha1") || algo_lower.contains("sha-1") {
            info.sha1_hash = Some(hash.hash.clone());
        } else if algo_lower.contains("sha256") || algo_lower.contains("sha-256") {
            info.sha256_hash = Some(hash.hash.clone());
        }
    }
    
    Some(info)
}

/// Parse a single field from companion log into CompanionLogInfo
/// NOTE: Kept for backward compatibility but no longer used since we delegate
/// to the shared companion parser from containers/companion.rs
#[allow(dead_code)]
fn parse_companion_field(info: &mut CompanionLogInfo, key: &str, value: &str, notes_lines: &mut Vec<String>) {
    match key {
        // Case identification
        "case number" | "case" | "case #" | "case no" | "case_number" | "casenumber" => {
            info.case_number = Some(value.to_string());
        }
        "evidence number" | "evidence" | "evidence #" | "evidence no" | "evidence_number" | 
        "item" | "item number" | "item #" | "exhibit" => {
            info.evidence_number = Some(value.to_string());
        }
        "examiner name" | "examiner" | "analyst" | "investigator" | "operator" => {
            info.examiner = Some(value.to_string());
        }
        "organization" | "agency" | "department" | "company" => {
            info.organization = Some(value.to_string());
        }
        
        // Hash values
        "md5" | "md5 hash" | "md5 checksum" | "md5_hash" => {
            info.md5_hash = Some(value.to_lowercase());
        }
        "sha1" | "sha1 hash" | "sha-1" | "sha1 checksum" | "sha1_hash" => {
            info.sha1_hash = Some(value.to_lowercase());
        }
        "sha256" | "sha256 hash" | "sha-256" | "sha256 checksum" | "sha256_hash" => {
            info.sha256_hash = Some(value.to_lowercase());
        }
        
        // Acquisition details
        "acquisition date" | "acquired" | "date" | "acquisition_date" | "created" | 
        "start time" | "acquisition time" => {
            info.acquisition_date = Some(value.to_string());
        }
        "source" | "source device" | "device" | "source_device" | "media" | 
        "source media" | "drive" => {
            info.source_device = Some(value.to_string());
        }
        "source path" | "path" | "source_path" | "location" | "source location" => {
            info.source_path = Some(value.to_string());
        }
        "acquisition tool" | "tool" | "acquisition_tool" | "software" | "program" |
        "ftk" | "ftk imager" | "encase" | "axiom" => {
            info.acquisition_tool = Some(value.to_string());
        }
        "acquisition method" | "method" | "acquisition_method" | "type" | "image type" => {
            info.acquisition_method = Some(value.to_string());
        }
        
        // Notes and description
        "notes" | "description" | "comments" | "remarks" => {
            if !value.is_empty() {
                notes_lines.push(value.to_string());
            }
        }
        _ => {}
    }
}

/// Extract hex hash from a line
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_hash(line: &str, expected_len: usize) -> Option<String> {
    // Find consecutive hex string of expected length
    let mut hex_chars = String::new();
    let mut found_start = false;
    
    for c in line.chars() {
        if c.is_ascii_hexdigit() {
            hex_chars.push(c);
            found_start = true;
        } else if found_start && hex_chars.len() >= expected_len {
            // We have enough, stop
            break;
        } else if found_start && !hex_chars.is_empty() {
            // Reset if we hit a non-hex char before getting enough
            if hex_chars.len() < expected_len {
                hex_chars.clear();
                found_start = false;
            }
        }
    }
    
    if hex_chars.len() >= expected_len {
        Some(hex_chars[..expected_len].to_lowercase())
    } else {
        None
    }
}

/// Extract a number from a line (for item counts)
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_number(line: &str) -> Option<u64> {
    // Find first number in the line (ignoring common non-count numbers)
    let digits: String = line.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| c.is_ascii_digit())
        .collect();
    
    if digits.is_empty() {
        return None;
    }
    
    digits.parse().ok()
}

/// Extract size value from a line (handles KB, MB, GB suffixes)
/// NOTE: Kept for reference but no longer used - shared parser handles this
#[allow(dead_code)]
fn extract_size(line: &str) -> Option<u64> {
    // Look for patterns like "1.5 GB", "1024 MB", "1,024,000 bytes"
    let line_lower = line.to_lowercase();
    
    // Find numeric value (including decimals and commas)
    let num_str: String = line.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    
    let num: f64 = num_str.parse().ok()?;
    
    // Determine multiplier based on suffix
    let multiplier = if line_lower.contains("tb") || line_lower.contains("terabyte") {
        1_099_511_627_776u64
    } else if line_lower.contains("gb") || line_lower.contains("gigabyte") {
        1_073_741_824u64
    } else if line_lower.contains("mb") || line_lower.contains("megabyte") {
        1_048_576u64
    } else if line_lower.contains("kb") || line_lower.contains("kilobyte") {
        1_024u64
    } else {
        1u64 // bytes
    };
    
    Some((num * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_segment_path() {
        assert_eq!(build_segment_path("/path/to/file.ad1", 1), "/path/to/file.ad1");
        assert_eq!(build_segment_path("/path/to/file.ad1", 2), "/path/to/file.ad2");
        assert_eq!(build_segment_path("/path/to/file.ad1", 3), "/path/to/file.ad3");
        assert_eq!(build_segment_path("/path/to/file.ad1", 10), "/path/to/file.ad10");
        assert_eq!(build_segment_path("", 1), "");
    }

    #[test]
    fn test_join_path() {
        assert_eq!(join_path("", "file.txt"), "file.txt");
        assert_eq!(join_path("folder", ""), "folder");
        assert_eq!(join_path("folder", "file.txt"), "folder/file.txt");
        assert_eq!(join_path("a/b", "c.txt"), "a/b/c.txt");
    }

    #[test]
    fn test_segment_span() {
        assert_eq!(segment_span(0x10000), SEGMENT_BLOCK_SIZE * 0x10000 - AD1_LOGICAL_MARGIN);
        assert_eq!(segment_span(1), SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN);
        assert_eq!(segment_span(0), 0);
    }

    #[test]
    fn test_copy_into_array() {
        let result: [u8; 4] = copy_into_array("test", 4).unwrap();
        assert_eq!(&result, b"test");

        let result: [u8; 8] = copy_into_array("hi", 8).unwrap();
        assert_eq!(&result[..2], b"hi");
        assert_eq!(&result[2..], &[0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_extract_hash_md5() {
        // Standard MD5 hash line
        let hash = extract_hash("MD5: d41d8cd98f00b204e9800998ecf8427e", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));
        
        // Hash without label
        let hash = extract_hash("d41d8cd98f00b204e9800998ecf8427e", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));
        
        // Hash with spaces
        let hash = extract_hash("MD5 Hash: D41D8CD98F00B204E9800998ECF8427E", 32);
        assert_eq!(hash, Some("d41d8cd98f00b204e9800998ecf8427e".to_string()));
        
        // Too short
        let hash = extract_hash("MD5: d41d8cd98f", 32);
        assert_eq!(hash, None);
    }

    #[test]
    fn test_extract_hash_sha1() {
        // Standard SHA1 hash line
        let hash = extract_hash("SHA1: da39a3ee5e6b4b0d3255bfef95601890afd80709", 40);
        assert_eq!(hash, Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()));
        
        // SHA-1 format
        let hash = extract_hash("SHA-1: DA39A3EE5E6B4B0D3255BFEF95601890AFD80709", 40);
        assert_eq!(hash, Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()));
    }

    #[test]
    fn test_extract_hash_sha256() {
        // Standard SHA256 hash line
        let hash = extract_hash(
            "SHA256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", 64
        );
        assert_eq!(hash, Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()));
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("Total items: 1234"), Some(1234));
        assert_eq!(extract_number("Count: 1,000,000"), Some(1000000));
        assert_eq!(extract_number("Files processed: 42 files"), Some(42));
        assert_eq!(extract_number("No numbers here"), None);
    }

    #[test]
    fn test_extract_size() {
        // Various size formats
        assert_eq!(extract_size("Size: 1024 bytes"), Some(1024));
        assert_eq!(extract_size("Total: 1 KB"), Some(1024));
        assert_eq!(extract_size("Size: 1.5 MB"), Some(1572864)); // 1.5 * 1024 * 1024
        assert_eq!(extract_size("Total: 2 GB"), Some(2147483648));
        assert_eq!(extract_size("Size: 1,000 KB"), Some(1024000));
    }

    #[test]
    fn test_parse_companion_field() {
        let mut info = CompanionLogInfo::default();
        let mut notes = Vec::new();
        
        // Test case number variations
        parse_companion_field(&mut info, "case number", "CASE-001", &mut notes);
        assert_eq!(info.case_number, Some("CASE-001".to_string()));
        
        // Test examiner variations
        let mut info2 = CompanionLogInfo::default();
        parse_companion_field(&mut info2, "analyst", "John Doe", &mut notes);
        assert_eq!(info2.examiner, Some("John Doe".to_string()));
        
        // Test hash fields
        let mut info3 = CompanionLogInfo::default();
        parse_companion_field(&mut info3, "sha256 hash", "abc123", &mut notes);
        assert_eq!(info3.sha256_hash, Some("abc123".to_string()));
        
        // Test source device
        let mut info4 = CompanionLogInfo::default();
        parse_companion_field(&mut info4, "source media", "USB Drive", &mut notes);
        assert_eq!(info4.source_device, Some("USB Drive".to_string()));
    }

    #[test]
    fn test_parse_timestamp_to_iso() {
        // Valid AD1 timestamp format
        let result = parse_timestamp_to_iso("20240115T143022");
        assert_eq!(result, Some("2024-01-15T14:30:22".to_string()));
        
        // With null terminators
        let result = parse_timestamp_to_iso("20240115T143022\0\0\0");
        assert_eq!(result, Some("2024-01-15T14:30:22".to_string()));
        
        // Too short
        let result = parse_timestamp_to_iso("20240115");
        assert_eq!(result, None);
        
        // Empty
        let result = parse_timestamp_to_iso("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_segment_span_calculation() {
        // Test segment span with typical fragment sizes
        // 0x10000 fragments = 65536 * 65536 - 512 = 4294901248
        assert_eq!(segment_span(0x10000), SEGMENT_BLOCK_SIZE * 0x10000 - AD1_LOGICAL_MARGIN);
        
        // Single fragment
        assert_eq!(segment_span(1), SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN);
        
        // 100 fragments
        assert_eq!(segment_span(100), 100 * SEGMENT_BLOCK_SIZE - AD1_LOGICAL_MARGIN);
        
        // Zero fragments
        assert_eq!(segment_span(0), 0);
    }

    #[test]
    fn test_bytes_to_string() {
        // Normal string without trim
        assert_eq!(bytes_to_string(b"hello", false), "hello");
        
        // Null terminated without trim
        assert_eq!(bytes_to_string(b"hello\0world", false), "hello");
        
        // Empty
        assert_eq!(bytes_to_string(b"", false), "");
        
        // All nulls
        assert_eq!(bytes_to_string(&[0, 0, 0], false), "");
    }

    #[test]
    fn test_bytes_to_string_with_trim() {
        // Normal string with trim
        assert_eq!(bytes_to_string(b"value", true), "value");
        
        // With whitespace (trimmed)
        assert_eq!(bytes_to_string(b"  value  ", true), "value");
        
        // Null terminated with whitespace
        assert_eq!(bytes_to_string(b"value\0extra", true), "value");
        
        // Without trim - preserves whitespace
        assert_eq!(bytes_to_string(b"  value  ", false), "  value  ");
    }
}
