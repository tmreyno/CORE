// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Parser Implementation
//!
//! ## Section Brief
//! Low-level parsing and session management for AD1 containers:
//!
//! ### Session Management
//! - `Session` - Stateful parser with file handles and decompression cache
//! - `CacheEntry` - LRU cache entry for decompressed data
//!
//! ### Header Parsing
//! - `read_segment_header()` - Parse segment header from file
//! - `read_logical_header()` - Parse logical header from file
//!
//! ### Item Parsing
//! - `read_item()` - Parse file/folder item structure
//! - `read_metadata()` - Parse item metadata chain
//!
//! ### Data Decompression
//! - `read_file_data()` - Decompress file content (zlib)
//! - `get_decompressed_data()` - LRU-cached decompression
//! - `decompress_parallel()` - Parallel chunk decompression (rayon)
//!
//! ### Verification & Extraction
//! - `verify_item_with_progress()` - Verify stored hashes
//! - `extract_item_with_progress()` - Extract to filesystem

use flate2::read::ZlibDecoder;
use rayon::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use tracing::{trace, debug, instrument};

use super::types::*;
use super::utils::*;
use crate::common::hash::{HashAlgorithm, compute_hash};
use crate::containers::ContainerError;

// =============================================================================
// Item Structure Offsets
// =============================================================================
// AD1 Item record layout (relative to item start):
//   0x00: next_item_addr (u64)
//   0x08: first_child_addr (u64)
//   0x10: first_metadata_addr (u64)
//   0x18: zlib_metadata_addr (u64)
//   0x20: decompressed_size (u64)
//   0x28: item_type (u32)
//   0x2C: name_length (u32)
//   0x30: name_bytes (variable)

/// Offset to first child item address
const ITEM_FIRST_CHILD_OFFSET: u64 = 0x08;
/// Offset to first metadata address
const ITEM_FIRST_METADATA_OFFSET: u64 = 0x10;
/// Offset to zlib metadata address
const ITEM_ZLIB_METADATA_OFFSET: u64 = 0x18;
/// Offset to decompressed size
const ITEM_DECOMPRESSED_SIZE_OFFSET: u64 = 0x20;
/// Offset to item type
const ITEM_TYPE_OFFSET: u64 = 0x28;
/// Offset to name length
const ITEM_NAME_LENGTH_OFFSET: u64 = 0x2C;
/// Offset to name bytes (variable length)
const ITEM_NAME_OFFSET: u64 = 0x30;

// =============================================================================
// Metadata Structure Offsets
// =============================================================================
// AD1 Metadata record layout (relative to metadata start):
//   0x00: next_metadata_addr (u64)
//   0x08: category (u32)
//   0x0C: key (u32)
//   0x10: data_length (u32)
//   0x14: data (variable)

/// Offset to metadata category
const METADATA_CATEGORY_OFFSET: u64 = 0x08;
/// Offset to metadata key
const METADATA_KEY_OFFSET: u64 = 0x0C;
/// Offset to metadata data length
const METADATA_DATA_LENGTH_OFFSET: u64 = 0x10;
/// Offset to metadata data bytes
const METADATA_DATA_OFFSET: u64 = 0x14;

// =============================================================================
// Zlib Chunk Address Table
// =============================================================================
/// Size of each address entry in the zlib chunk table
const ZLIB_CHUNK_ADDR_SIZE: u64 = 0x08;

// =============================================================================
// Types
// =============================================================================

/// Parameters for verify_item_with_progress to reduce argument count
pub(crate) struct VerifyParams<'a, F> {
    pub algorithm: HashAlgorithm,
    pub out: &'a mut Vec<VerifyEntry>,
    pub current: &'a mut u64,
    pub total: u64,
    pub progress_callback: &'a mut F,
}

// =============================================================================
// Cache Types
// =============================================================================

/// LRU cache entry with access counter
#[derive(Clone)]
pub(crate) struct CacheEntry {
    pub data: Arc<Vec<u8>>,
    pub access_count: u32,
}

// Manual Debug impl for Session since File doesn't impl Debug
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("segment_count", &self.segment_header.segment_number)
            .field("file_count", &self.files.iter().filter(|f| f.is_some()).count())
            .field("missing_segments", &self.missing_segments)
            .field("item_count", &self.item_counter)
            .field("root_items", &self.root_items.len())
            .field("cache_size", &self.cache.len())
            .finish()
    }
}

/// AD1 parsing session - manages file handles, caching, and parsing state
pub(crate) struct Session {
    pub segment_header: SegmentHeader,
    pub logical_header: LogicalHeader,
    /// Segment files - Some(file) if available, None if segment is missing
    pub files: Vec<Option<File>>,
    pub file_sizes: Vec<u64>,
    pub item_counter: u64,
    pub root_items: Vec<Item>,
    cache: HashMap<u64, CacheEntry>,
    /// LRU order tracking - uses VecDeque for O(1) front removal
    cache_order: VecDeque<u64>,
    /// List of missing segment indices (1-based)
    pub missing_segments: Vec<u32>,
}

impl Session {
    /// Open an AD1 container and parse its structure
    /// This is lenient - it will open even if some segments are missing,
    /// allowing navigation of whatever data is available
    #[instrument(skip_all, fields(path))]
    pub fn open(path: &str) -> Result<Self, ContainerError> {
        debug!(path, "Opening AD1 session");
        // Use lenient validation - only check format, not segments
        validate_ad1(path, false)?;
        let mut header_file = File::open(path)
            .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file '{path}': {e}")))?;
        let segment_header = read_segment_header(&mut header_file)?;
        let logical_header = read_logical_header(&mut header_file)?;
        
        debug!(
            segment_count = segment_header.segment_number,
            first_item_addr = logical_header.first_item_addr,
            "AD1 headers parsed"
        );

        let mut files = Vec::new();
        let mut file_sizes = Vec::new();
        let mut missing_segments = Vec::new();
        
        for index in 1..=segment_header.segment_number {
            let segment_path = build_segment_path(path, index);
            trace!(index, segment_path, "Opening segment");
            
            match File::open(&segment_path) {
                Ok(mut file) => {
                    let size = file
                        .seek(SeekFrom::End(0))
                        .map_err(|e| ContainerError::IoError(format!("Failed to seek segment '{segment_path}': {e}")))?;
                    let data_size = size.saturating_sub(AD1_LOGICAL_MARGIN);
                    file.seek(SeekFrom::Start(0))
                        .map_err(|e| ContainerError::IoError(format!("Failed to rewind segment '{segment_path}': {e}")))?;
                    files.push(Some(file));
                    file_sizes.push(data_size);
                }
                Err(_) => {
                    // Segment is missing - track it but continue
                    debug!(index, segment_path, "Segment missing, continuing with available data");
                    missing_segments.push(index);
                    files.push(None);
                    file_sizes.push(0);
                }
            }
        }
        
        // Need at least the first segment to read the tree structure
        if files.is_empty() || files[0].is_none() {
            return Err(ContainerError::SegmentError(
                "First segment is required to read container structure".to_string()
            ));
        }
        
        if !missing_segments.is_empty() {
            debug!(missing = ?missing_segments, "Some segments missing, data may be incomplete");
        }

        let mut session = Session {
            segment_header,
            logical_header,
            files,
            file_sizes,
            item_counter: 0,
            root_items: Vec::new(),
            cache: HashMap::with_capacity(CACHE_SIZE),
            cache_order: VecDeque::with_capacity(CACHE_SIZE),
            missing_segments,
        };

        let root_items = session.read_item_chain(session.logical_header.first_item_addr)?;
        debug!(root_item_count = root_items.len(), "Parsed root items");
        session.root_items = root_items;

        Ok(session)
    }

    /// Open an AD1 container in lazy mode - only opens files and parses headers
    /// Does NOT parse the item tree - use read_children_lazy() for on-demand loading
    /// This is much faster for large containers as it avoids loading everything into memory
    #[instrument(skip_all, fields(path))]
    pub fn open_lazy(path: &str) -> Result<Self, ContainerError> {
        debug!(path, "Opening AD1 session (lazy mode)");
        // Use lenient validation - only check format, not segments
        validate_ad1(path, false)?;
        let mut header_file = File::open(path)
            .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file '{path}': {e}")))?;
        let segment_header = read_segment_header(&mut header_file)?;
        let logical_header = read_logical_header(&mut header_file)?;
        
        debug!(
            segment_count = segment_header.segment_number,
            first_item_addr = logical_header.first_item_addr,
            "AD1 headers parsed (lazy)"
        );

        let mut files = Vec::new();
        let mut file_sizes = Vec::new();
        let mut missing_segments = Vec::new();
        
        for index in 1..=segment_header.segment_number {
            let segment_path = build_segment_path(path, index);
            trace!(index, segment_path, "Opening segment");
            
            match File::open(&segment_path) {
                Ok(mut file) => {
                    let size = file
                        .seek(SeekFrom::End(0))
                        .map_err(|e| ContainerError::IoError(format!("Failed to seek segment '{segment_path}': {e}")))?;
                    let data_size = size.saturating_sub(AD1_LOGICAL_MARGIN);
                    file.seek(SeekFrom::Start(0))
                        .map_err(|e| ContainerError::IoError(format!("Failed to rewind segment '{segment_path}': {e}")))?;
                    files.push(Some(file));
                    file_sizes.push(data_size);
                }
                Err(_) => {
                    // Segment is missing - track it but continue
                    debug!(index, segment_path, "Segment missing, continuing with available data");
                    missing_segments.push(index);
                    files.push(None);
                    file_sizes.push(0);
                }
            }
        }
        
        // Need at least the first segment to read the tree structure
        if files.is_empty() || files[0].is_none() {
            return Err(ContainerError::SegmentError(
                "First segment is required to read container structure".to_string()
            ));
        }
        
        if !missing_segments.is_empty() {
            debug!(missing = ?missing_segments, "Some segments missing, data may be incomplete");
        }

        let session = Session {
            segment_header,
            logical_header,
            files,
            file_sizes,
            item_counter: 0,
            root_items: Vec::new(),  // Not populated in lazy mode
            cache: HashMap::with_capacity(CACHE_SIZE),
            cache_order: VecDeque::with_capacity(CACHE_SIZE),
            missing_segments,
        };

        Ok(session)
    }

    /// Get the first item address from the logical header
    pub fn first_item_addr(&self) -> u64 {
        self.logical_header.first_item_addr
    }

    /// Read immediate children at a given address without recursing into grandchildren
    /// This is the key method for lazy/on-demand loading
    /// Returns items with empty children vectors - use first_child_addr for subsequent loading
    pub fn read_children_lazy(&mut self, offset: u64) -> Result<Vec<Item>, ContainerError> {
        let mut items = Vec::new();
        let mut next_addr = offset;
        
        while next_addr != 0 {
            match self.read_item_shallow(next_addr) {
                Ok((item, next)) => {
                    items.push(item);
                    next_addr = next;
                }
                Err(e) => {
                    if items.is_empty() {
                        return Err(ContainerError::ParseError(e));
                    }
                    debug!("Stopping lazy item chain at offset 0x{:x}: {}", next_addr, e);
                    break;
                }
            }
        }
        Ok(items)
    }

    /// Read a single item without recursing into children
    /// Returns the item and next_item_addr, with first_child_addr stored in the item
    /// for lazy loading later
    fn read_item_shallow(&mut self, offset: u64) -> Result<(Item, u64), String> {
        // Read core item fields
        let next_item_addr = self.read_u64(offset)
            .map_err(|e| format!("Failed to read next_item_addr at 0x{:x}: {}", offset, e))?;
        let first_child_addr = self.read_u64(offset + ITEM_FIRST_CHILD_OFFSET)
            .map_err(|e| format!("Failed to read first_child_addr at 0x{:x}: {}", offset + ITEM_FIRST_CHILD_OFFSET, e))?;
        let first_metadata_addr = self.read_u64(offset + ITEM_FIRST_METADATA_OFFSET)
            .map_err(|e| format!("Failed to read first_metadata_addr at 0x{:x}: {}", offset + ITEM_FIRST_METADATA_OFFSET, e))?;
        let zlib_metadata_addr = self.read_u64(offset + ITEM_ZLIB_METADATA_OFFSET)
            .map_err(|e| format!("Failed to read zlib_metadata_addr at 0x{:x}: {}", offset + ITEM_ZLIB_METADATA_OFFSET, e))?;
        let decompressed_size = self.read_u64(offset + ITEM_DECOMPRESSED_SIZE_OFFSET)
            .map_err(|e| format!("Failed to read decompressed_size at 0x{:x}: {}", offset + ITEM_DECOMPRESSED_SIZE_OFFSET, e))?;
        let item_type = self.read_u32(offset + ITEM_TYPE_OFFSET)
            .map_err(|e| format!("Failed to read item_type at 0x{:x}: {}", offset + ITEM_TYPE_OFFSET, e))?;
        let name_length = self.read_u32(offset + ITEM_NAME_LENGTH_OFFSET)
            .map_err(|e| format!("Failed to read name_length at 0x{:x}: {}", offset + ITEM_NAME_LENGTH_OFFSET, e))? as usize;
        let name_bytes = self.read_bytes(offset + ITEM_NAME_OFFSET, name_length)
            .map_err(|e| format!("Failed to read name at 0x{:x}: {}", offset + ITEM_NAME_OFFSET, e))?;
        let mut name = bytes_to_string(&name_bytes, false);
        name = name.replace('/', "_");

        // Try to read metadata - gracefully handle missing segments
        let metadata = if first_metadata_addr != 0 {
            match self.read_metadata_list(first_metadata_addr) {
                Ok(meta) => meta,
                Err(e) => {
                    trace!("Skipping metadata for '{}' due to: {}", name, e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Store first_child_addr for lazy loading but don't recurse
        // We use zlib_metadata_addr to store our item reference, but we need
        // to communicate first_child_addr for later loading
        self.item_counter += 1;
        let item = Item {
            id: self.item_counter,
            name,
            item_type,
            decompressed_size,
            // Store first_child_addr in zlib_metadata_addr for folders,
            // actual zlib_metadata_addr for files
            zlib_metadata_addr: if item_type == AD1_FOLDER_SIGNATURE { 
                first_child_addr 
            } else { 
                zlib_metadata_addr 
            },
            metadata,
            children: Vec::new(),  // Empty - will be loaded on-demand
        };

        Ok((item, next_item_addr))
    }

    /// Read a chain of items starting at the given offset
    /// Gracefully handles missing segments by stopping when data is unavailable
    pub fn read_item_chain(&mut self, offset: u64) -> Result<Vec<Item>, ContainerError> {
        let mut items = Vec::new();
        let mut next_addr = offset;
        while next_addr != 0 {
            match self.read_item(next_addr) {
                Ok((item, next)) => {
                    items.push(item);
                    next_addr = next;
                }
                Err(e) => {
                    // If we can't read an item (likely due to missing segment),
                    // return what we have so far
                    if items.is_empty() {
                        // If we couldn't read even the first item, propagate the error
                        return Err(ContainerError::from(e));
                    }
                    debug!("Stopping item chain read at offset 0x{:x}: {}", next_addr, e);
                    break;
                }
            }
        }
        Ok(items)
    }

    /// Read a single item at the given offset
    /// Returns error string on failure (for graceful handling in read_item_chain)
    fn read_item(&mut self, offset: u64) -> Result<(Item, u64), String> {
        // Read core item fields - these are mandatory
        let next_item_addr = self.read_u64(offset)
            .map_err(|e| format!("Failed to read next_item_addr at 0x{:x}: {}", offset, e))?;
        let first_child_addr = self.read_u64(offset + ITEM_FIRST_CHILD_OFFSET)
            .map_err(|e| format!("Failed to read first_child_addr at 0x{:x}: {}", offset + ITEM_FIRST_CHILD_OFFSET, e))?;
        let first_metadata_addr = self.read_u64(offset + ITEM_FIRST_METADATA_OFFSET)
            .map_err(|e| format!("Failed to read first_metadata_addr at 0x{:x}: {}", offset + ITEM_FIRST_METADATA_OFFSET, e))?;
        let zlib_metadata_addr = self.read_u64(offset + ITEM_ZLIB_METADATA_OFFSET)
            .map_err(|e| format!("Failed to read zlib_metadata_addr at 0x{:x}: {}", offset + ITEM_ZLIB_METADATA_OFFSET, e))?;
        let decompressed_size = self.read_u64(offset + ITEM_DECOMPRESSED_SIZE_OFFSET)
            .map_err(|e| format!("Failed to read decompressed_size at 0x{:x}: {}", offset + ITEM_DECOMPRESSED_SIZE_OFFSET, e))?;
        let item_type = self.read_u32(offset + ITEM_TYPE_OFFSET)
            .map_err(|e| format!("Failed to read item_type at 0x{:x}: {}", offset + ITEM_TYPE_OFFSET, e))?;
        let name_length = self.read_u32(offset + ITEM_NAME_LENGTH_OFFSET)
            .map_err(|e| format!("Failed to read name_length at 0x{:x}: {}", offset + ITEM_NAME_LENGTH_OFFSET, e))? as usize;
        let name_bytes = self.read_bytes(offset + ITEM_NAME_OFFSET, name_length)
            .map_err(|e| format!("Failed to read name at 0x{:x}: {}", offset + ITEM_NAME_OFFSET, e))?;
        let mut name = bytes_to_string(&name_bytes, false);
        name = name.replace('/', "_");

        // Try to read metadata - gracefully handle missing segments
        let metadata = if first_metadata_addr != 0 {
            match self.read_metadata_list(first_metadata_addr) {
                Ok(meta) => meta,
                Err(e) => {
                    trace!("Skipping metadata for '{}' due to: {}", name, e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Try to read children - gracefully handle missing segments
        let children = if first_child_addr != 0 {
            match self.read_item_chain(first_child_addr) {
                Ok(kids) => kids,
                Err(e) => {
                    trace!("Skipping children for '{}' due to: {}", name, e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        self.item_counter += 1;
        let item = Item {
            id: self.item_counter,
            name,
            item_type,
            decompressed_size,
            zlib_metadata_addr,
            metadata,
            children,
        };

        Ok((item, next_item_addr))
    }

    /// Read metadata list starting at the given offset
    fn read_metadata_list(&mut self, offset: u64) -> Result<Vec<Metadata>, ContainerError> {
        let mut list = Vec::new();
        let mut next_addr = offset;
        while next_addr != 0 {
            let meta = self.read_metadata(next_addr)?;
            next_addr = meta.next_metadata_addr;
            list.push(meta);
        }
        Ok(list)
    }

    /// Read a single metadata entry
    fn read_metadata(&mut self, offset: u64) -> Result<Metadata, ContainerError> {
        let next_metadata_addr = self.read_u64(offset)?;
        let category = self.read_u32(offset + METADATA_CATEGORY_OFFSET)?;
        let key = self.read_u32(offset + METADATA_KEY_OFFSET)?;
        let data_length = self.read_u32(offset + METADATA_DATA_LENGTH_OFFSET)? as usize;
        let data = self.read_bytes(offset + METADATA_DATA_OFFSET, data_length)?;

        Ok(Metadata {
            next_metadata_addr,
            category,
            key,
            data,
        })
    }

    /// Read u32 at offset
    pub fn read_u32(&mut self, offset: u64) -> Result<u32, ContainerError> {
        let bytes = self.read_bytes(offset, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read u64 at offset
    pub fn read_u64(&mut self, offset: u64) -> Result<u64, ContainerError> {
        let bytes = self.read_bytes(offset, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read bytes at offset
    pub fn read_bytes(&mut self, offset: u64, length: usize) -> Result<Vec<u8>, ContainerError> {
        if length == 0 {
            return Ok(Vec::new());
        }
        let mut buf = vec![0u8; length];
        self.read_into(offset, &mut buf)?;
        Ok(buf)
    }

    /// Read into buffer at offset (handles multi-segment reads)
    fn read_into(&mut self, offset: u64, buf: &mut [u8]) -> Result<(), ContainerError> {
        if buf.is_empty() {
            return Ok(());
        }

        let seg_span = segment_span(self.segment_header.fragments_size);
        if seg_span == 0 {
            return Err(ContainerError::ParseError(
                "Invalid AD1 fragment size: fragments_size is 0, cannot calculate segment span".to_string()
            ));
        }
        let mut remaining = buf.len() as u64;
        let mut buf_cursor = 0usize;
        let mut file_cursor = (offset / seg_span) as usize;
        let mut data_cursor = offset - (seg_span * file_cursor as u64);

        // Pre-compute counts for error messages to avoid borrow conflicts
        let segment_count = self.file_sizes.len();
        let file_handle_count = self.files.len();

        while remaining > 0 {
            let file_size = self
                .file_sizes
                .get(file_cursor)
                .copied()
                .ok_or_else(|| ContainerError::SegmentError(format!(
                    "AD1 offset 0x{:x} out of range: requires segment {} but only {} segments available",
                    offset, file_cursor + 1, segment_count
                )))?;
            let mut to_read = remaining;
            if data_cursor + to_read > file_size {
                to_read = file_size.saturating_sub(data_cursor);
            }
            if to_read == 0 {
                return Err(ContainerError::ParseError(format!(
                    "AD1 read failed: offset 0x{:x} exceeds segment {} data size ({} bytes)",
                    offset, file_cursor + 1, file_size
                )));
            }

            let file_opt = self
                .files
                .get_mut(file_cursor)
                .ok_or_else(|| ContainerError::SegmentError(format!(
                    "AD1 segment index {} out of range: only {} segment handles open",
                    file_cursor, file_handle_count
                )))?;
            
            // Handle missing segment - return error with helpful message
            let file = file_opt.as_mut().ok_or_else(|| ContainerError::SegmentError(format!(
                "AD1 segment {} is missing - data at offset 0x{:x} is unavailable",
                file_cursor + 1, offset
            )))?;
            
            file.seek(SeekFrom::Start(data_cursor + AD1_LOGICAL_MARGIN))
                .map_err(|e| ContainerError::IoError(format!(
                    "Failed to seek to offset 0x{:x} in segment {}: {}",
                    data_cursor + AD1_LOGICAL_MARGIN, file_cursor + 1, e
                )))?;
            file.read_exact(&mut buf[buf_cursor..buf_cursor + to_read as usize])
                .map_err(|e| ContainerError::IoError(format!(
                    "Failed to read {} bytes at offset 0x{:x} in segment {}: {}",
                    to_read, data_cursor + AD1_LOGICAL_MARGIN, file_cursor + 1, e
                )))?;

            buf_cursor += to_read as usize;
            remaining -= to_read;
            data_cursor = 0;
            file_cursor += 1;
        }

        Ok(())
    }

    /// Read and decompress file data for an item
    pub fn read_file_data(&mut self, item: &Item) -> Result<Arc<Vec<u8>>, ContainerError> {
        if item.decompressed_size == 0 {
            return Ok(Arc::new(Vec::new()));
        }
        if let Some(data) = self.search_cache(item.id) {
            return Ok(data);
        }
        if item.zlib_metadata_addr == 0 {
            return Err(ContainerError::ParseError(format!(
                "Item '{}' (id={}) has invalid zlib metadata address: 0x0",
                item.name, item.id
            )));
        }

        let chunk_count = self.read_u64(item.zlib_metadata_addr)?;
        let mut addresses = Vec::with_capacity(chunk_count as usize + 1);
        for index in 0..=chunk_count {
            let addr = self.read_u64(item.zlib_metadata_addr + ((index + 1) * ZLIB_CHUNK_ADDR_SIZE))?;
            addresses.push(addr);
        }

        // For small files (< 4 chunks), use sequential decompression
        // For larger files, use parallel decompression
        let data = if chunk_count < 4 {
            self.decompress_sequential(&addresses, item.decompressed_size as usize)?
        } else {
            self.decompress_parallel(&addresses, item.decompressed_size as usize)?
        };

        let data = Arc::new(data);
        self.cache_data(item.id, data.clone());
        Ok(data)
    }

    /// Sequential decompression for small files
    fn decompress_sequential(&mut self, addresses: &[u64], decompressed_size: usize) -> Result<Vec<u8>, ContainerError> {
        let chunk_count = addresses.len() - 1;
        let mut output = vec![0u8; decompressed_size];
        let mut data_index = 0usize;
        
        for index in 0..chunk_count {
            let start = addresses[index];
            let end = addresses[index + 1];
            let compressed_len = end.saturating_sub(start) as usize;
            if compressed_len == 0 {
                continue;
            }
            let compressed = self.read_bytes(start, compressed_len)?;
            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut chunk = Vec::new();
            decoder
                .read_to_end(&mut chunk)
                .map_err(|e| ContainerError::IoError(format!("Zlib inflate error: {e}")))?;
            let end_index = (data_index + chunk.len()).min(output.len());
            output[data_index..end_index].copy_from_slice(&chunk[..end_index - data_index]);
            data_index = end_index;
        }
        
        Ok(output)
    }

    /// Parallel decompression for large files
    fn decompress_parallel(&mut self, addresses: &[u64], decompressed_size: usize) -> Result<Vec<u8>, ContainerError> {
        let chunk_count = addresses.len() - 1;
        
        // Pre-read all compressed chunks sequentially (I/O bound)
        let mut compressed_chunks: Vec<(usize, Vec<u8>)> = Vec::with_capacity(chunk_count);
        for index in 0..chunk_count {
            let start = addresses[index];
            let end = addresses[index + 1];
            let compressed_len = end.saturating_sub(start) as usize;
            if compressed_len == 0 {
                continue;
            }
            let compressed = self.read_bytes(start, compressed_len)?;
            compressed_chunks.push((index, compressed));
        }
        
        // Decompress in parallel (CPU bound)
        let decompressed_chunks: Vec<Result<(usize, Vec<u8>), String>> = compressed_chunks
            .par_iter()
            .map(|(index, compressed)| {
                let mut decoder = ZlibDecoder::new(&compressed[..]);
                let mut chunk = Vec::new();
                decoder
                    .read_to_end(&mut chunk)
                    .map_err(|e| ContainerError::IoError(format!("Zlib inflate error: {e}")))?;
                Ok((*index, chunk))
            })
            .collect();
        
        // Assemble output in order
        let mut output = vec![0u8; decompressed_size];
        let mut data_index = 0usize;
        
        // Sort by index to maintain order
        let mut sorted_chunks: Vec<(usize, Vec<u8>)> = Vec::with_capacity(decompressed_chunks.len());
        for result in decompressed_chunks {
            sorted_chunks.push(result?);
        }
        sorted_chunks.sort_by_key(|(idx, _)| *idx);
        
        for (_, chunk) in sorted_chunks {
            let end_index = (data_index + chunk.len()).min(output.len());
            output[data_index..end_index].copy_from_slice(&chunk[..end_index - data_index]);
            data_index = end_index;
        }
        
        Ok(output)
    }

    /// O(1) cache lookup using HashMap
    fn search_cache(&mut self, item_id: u64) -> Option<Arc<Vec<u8>>> {
        if let Some(entry) = self.cache.get_mut(&item_id) {
            entry.access_count = entry.access_count.saturating_add(1);
            return Some(entry.data.clone());
        }
        None
    }

    /// LRU cache insertion with eviction
    fn cache_data(&mut self, item_id: u64, data: Arc<Vec<u8>>) {
        if self.cache.contains_key(&item_id) {
            return;
        }
        
        // Evict oldest entry if cache is full - O(1) with VecDeque
        if self.cache.len() >= CACHE_SIZE {
            if let Some(oldest_id) = self.cache_order.pop_front() {
                self.cache.remove(&oldest_id);
            }
        }
        
        self.cache.insert(item_id, CacheEntry {
            data,
            access_count: 1,
        });
        self.cache_order.push_back(item_id);
    }

    /// Verify item hash with progress callback
    pub fn verify_item_with_progress<F>(
        &mut self,
        item: &Item,
        parent_path: &str,
        params: &mut VerifyParams<'_, F>,
    ) -> Result<(), ContainerError>
    where
        F: FnMut(u64, u64)
    {
        let path = join_path(parent_path, &item.name);
        if item.item_type != AD1_FOLDER_SIGNATURE {
            let stored = match params.algorithm {
                HashAlgorithm::Md5 => find_hash(&item.metadata, MD5_HASH),
                HashAlgorithm::Sha1 => find_hash(&item.metadata, SHA1_HASH),
                HashAlgorithm::Sha256 | HashAlgorithm::Sha512 | 
                HashAlgorithm::Blake3 | HashAlgorithm::Blake2 |
                HashAlgorithm::Xxh3 | HashAlgorithm::Xxh64 | HashAlgorithm::Crc32 => None,
            };
            
            let data = self.read_file_data(item)?;
            let computed = compute_hash(&data, params.algorithm);
            
            let (status, stored_for_output) = match &stored {
                Some(stored_hash) => {
                    // Compare hashes case-insensitively (both should be lowercase, but be safe)
                    let matches = stored_hash.eq_ignore_ascii_case(&computed);
                    if matches {
                        (VerifyStatus::Ok, Some(stored_hash.clone()))
                    } else {
                        debug!(
                            path = %path,
                            stored = %stored_hash,
                            computed = %computed,
                            size = item.decompressed_size,
                            "Hash mismatch"
                        );
                        (VerifyStatus::Nok, Some(stored_hash.clone()))
                    }
                }
                None => {
                    trace!(path = %path, "No stored hash, computed only");
                    (VerifyStatus::Computed, None)
                }
            };

            params.out.push(VerifyEntry {
                path: path.clone(),
                status,
                algorithm: Some(params.algorithm.name().to_string()),
                computed: Some(computed),
                stored: stored_for_output,
                size: Some(item.decompressed_size),
            });
            
            *params.current += 1;
            (params.progress_callback)(*params.current, params.total);
        }

        for child in &item.children {
            self.verify_item_with_progress(child, &path, params)?;
        }

        Ok(())
    }

    /// Extract item with progress callback
    pub fn extract_item_with_progress<F>(
        &mut self,
        item: &Item,
        output_dir: &Path,
        current: &mut u64,
        total: u64,
        progress_callback: &mut F,
    ) -> Result<(), ContainerError>
    where
        F: FnMut(u64, u64)
    {
        let item_path = output_dir.join(&item.name);
        if item.item_type == AD1_FOLDER_SIGNATURE {
            fs::create_dir_all(&item_path)
                .map_err(|e| format!("Failed to create directory {:?}: {e}", item_path))?;
        } else if item.item_type == 0 {
            if let Some(parent) = item_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {:?}: {e}", parent)
                })?;
            }
            let data = self.read_file_data(item)?;
            let mut file = File::create(&item_path)
                .map_err(|e| format!("Failed to create file {:?}: {e}", item_path))?;
            file.write_all(&data)
                .map_err(|e| format!("Failed to write file {:?}: {e}", item_path))?;
            
            *current += 1;
            progress_callback(*current, total);
        }

        for child in &item.children {
            self.extract_item_with_progress(child, &item_path, current, total, progress_callback)?;
        }

        apply_metadata(&item_path, &item.metadata)?;
        Ok(())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a minimal (but incomplete) AD1 file for testing error paths
    #[allow(dead_code)]
    fn create_minimal_ad1(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        
        // Write AD1 segment header signature: "ADSEGMENTEDFILE\0"
        file.write_all(b"ADSEGMENTEDFILE\0").unwrap();
        // Fill header with some data (segment header is 512 bytes)
        file.write_all(&[0u8; 496]).unwrap();
        
        path
    }

    #[test]
    fn test_session_open_nonexistent() {
        let result = Session::open("/nonexistent/path/file.ad1");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_open_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.ad1");
        File::create(&path).unwrap();
        
        let result = Session::open(path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_session_open_invalid_signature() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.ad1");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOT A VALID AD1 SIGNATURE____").unwrap();
        file.write_all(&[0u8; 1000]).unwrap(); // Pad it
        
        let result = Session::open(path.to_str().unwrap());
        assert!(result.is_err());
        // The error should mention signature or validation
        let err = result.unwrap_err().to_string();
        assert!(err.contains("AD1") || err.contains("signature") || err.contains("Invalid") || err.contains("format"));
    }

    #[test]
    fn test_cache_entry_clone() {
        let entry = CacheEntry {
            data: Arc::new(vec![1, 2, 3]),
            access_count: 5,
        };
        let cloned = entry.clone();
        
        assert_eq!(*cloned.data, vec![1, 2, 3]);
        assert_eq!(cloned.access_count, 5);
        // Should share the same Arc
        assert!(Arc::ptr_eq(&entry.data, &cloned.data));
    }

    #[test]
    fn test_verify_params_structure() {
        let mut results: Vec<VerifyEntry> = vec![];
        let mut current: u64 = 0;
        
        {
            let params = VerifyParams {
                algorithm: HashAlgorithm::Md5,
                out: &mut results,
                current: &mut current,
                total: 10,
                progress_callback: &mut |_curr: usize, _tot: usize| {},
            };
            // Just test that params can be created
            assert_eq!(params.total, 10);
        }
    }

    #[test]
    fn test_verify_status_ok() {
        let status = VerifyStatus::Ok;
        assert!(status.is_ok());
        assert!(!status.is_error());
        assert_eq!(status.to_string(), "ok");
    }

    #[test]
    fn test_verify_status_nok() {
        let status = VerifyStatus::Nok;
        assert!(!status.is_ok());
        assert!(status.is_error());
        assert_eq!(status.to_string(), "nok");
    }

    #[test]
    fn test_verify_status_computed() {
        let status = VerifyStatus::Computed;
        assert!(status.is_ok()); // computed is still ok (no mismatch)
        assert!(!status.is_error());
        assert_eq!(status.to_string(), "computed");
    }

    #[test]
    fn test_verify_status_equality() {
        assert_eq!(VerifyStatus::Ok, VerifyStatus::Ok);
        assert_ne!(VerifyStatus::Ok, VerifyStatus::Nok);
        assert_ne!(VerifyStatus::Nok, VerifyStatus::Computed);
    }
}
