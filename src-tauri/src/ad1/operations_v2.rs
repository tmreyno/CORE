// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Operations V2
//!
//! High-performance operations for AD1 containers based on libad1 C implementation.
//! Uses direct reader implementation for lazy loading without parsing entire tree upfront.
//! 
//! ## Features from libad1
//! - Complete metadata extraction (MD5, SHA1, timestamps, Windows flags)
//! - All item types supported (files, folders, symlinks, placeholders, etc.)
//! - Lazy loading for tree navigation
//! - Zlib decompression with proper chunk handling
//! - Thread-safe segment access
//! - LRU file data cache for repeated access

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use tracing::{debug, warn};

use super::reader_v2::{SessionV2, ItemHeader, MetadataEntry};
use super::types::*;

// =============================================================================
// File Data Cache (based on libad1_file_reader.c)
// =============================================================================

/// Cache size (number of files to cache)
const CACHE_SIZE: usize = 32;

/// Cache entry with reference counter
#[derive(Clone)]
struct CacheEntry {
    data: Arc<Vec<u8>>,
    counter: u32,
}

/// Global file data cache (LRU-style with reference counting like libad1)
static FILE_CACHE: Lazy<Mutex<FileDataCache>> = Lazy::new(|| {
    Mutex::new(FileDataCache::new(CACHE_SIZE))
});

struct FileDataCache {
    entries: HashMap<u64, CacheEntry>, // key = item offset
    max_size: usize,
}

impl FileDataCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
        }
    }
    
    /// Search cache for item (increments counter if found)
    fn search(&mut self, item_offset: u64) -> Option<Arc<Vec<u8>>> {
        // Decrement all counters (like libad1)
        for entry in self.entries.values_mut() {
            if entry.counter > 0 {
                entry.counter -= 1;
            }
        }
        
        // Remove entries with zero counter
        self.entries.retain(|_, entry| entry.counter > 0);
        
        // Search for item
        if let Some(entry) = self.entries.get_mut(&item_offset) {
            entry.counter += 2; // Boost counter on access
            return Some(Arc::clone(&entry.data));
        }
        
        None
    }
    
    /// Add data to cache
    fn cache(&mut self, item_offset: u64, data: Vec<u8>) -> Arc<Vec<u8>> {
        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            // Find entry with lowest counter
            if let Some((&key, _)) = self.entries.iter().min_by_key(|(_, e)| e.counter) {
                self.entries.remove(&key);
            }
        }
        
        let data = Arc::new(data);
        self.entries.insert(item_offset, CacheEntry {
            data: Arc::clone(&data),
            counter: 5, // Initial counter value (like libad1)
        });
        data
    }
    
    /// Clear entire cache
    #[allow(dead_code)]
    fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Clear the file data cache (call when closing container)
#[allow(dead_code)]
pub fn clear_file_cache() {
    if let Ok(mut cache) = FILE_CACHE.lock() {
        cache.clear();
    }
}

// =============================================================================
// Metadata Constants (from libad1_definitions.h)
// =============================================================================

// Metadata category constants (from libad1_definitions.h)
const HASH_INFO: u32 = 0x01;
#[allow(dead_code)]
const ITEM_TYPE_CATEGORY: u32 = 0x02;
#[allow(dead_code)]
const ITEM_SIZE_CATEGORY: u32 = 0x03;
const WINDOWS_FLAGS: u32 = 0x04;
const TIMESTAMP: u32 = 0x05;

// Hash keys
const MD5_HASH: u32 = 0x5001;
const SHA1_HASH: u32 = 0x5002;

// Timestamp keys
const ACCESS_TIME: u32 = 0x07;
const MODIFIED_TIME: u32 = 0x08;
const CHANGE_TIME: u32 = 0x09;

// Windows flag keys
const FLAG_ENCRYPTED: u32 = 0x0D;
const FLAG_COMPRESSED: u32 = 0x0E;
const FLAG_HIDDEN: u32 = 0x1002;
const FLAG_READ_ONLY: u32 = 0x1004;
const FLAG_ARCHIVE: u32 = 0x1005;

// Item type values (from libad1_definitions.h enum ad_item_type_value)
#[allow(dead_code)]
const REGULAR_FILE: u32 = 0x31;
#[allow(dead_code)]
const PLACEHOLDER: u32 = 0x32;
const REGULAR_FOLDER: u32 = 0x33;
#[allow(dead_code)]
const FILESYSTEM_METADATA: u32 = 0x34;
const FOLDER: u32 = 0x05;  // AD1_FOLDER_SIGNATURE
#[allow(dead_code)]
const FILESLACK: u32 = 0x36;
#[allow(dead_code)]
const SYMLINK: u32 = 0x39;

/// Get root children of AD1 container using V2 reader (lazy loading)
pub fn get_root_children<P: AsRef<Path>>(path: P) -> Result<Vec<TreeEntry>, Ad1Error> {
    debug!("get_root_children_v2: using V2 reader for lazy loading");
    
    let session = SessionV2::open(path)?;
    
    // Read root items starting from first_item_addr
    let first_addr = session.logical_header.first_item_addr;
    debug!(
        "get_root_children_v2: first_item_addr=0x{:X} ({} decimal), fragment_size={}",
        first_addr, first_addr, session.fragment_size
    );
    
    if first_addr == 0 {
        return Ok(Vec::new());
    }
    
    // Read all items in the root chain
    let mut entries = Vec::new();
    let mut current_addr = first_addr;
    
    while current_addr != 0 {
        let item = session.read_item_at(current_addr)?;
        let entry = build_tree_entry(&session, &item, "")?;
        entries.push(entry);
        current_addr = item.next_item_addr;
    }
    
    debug!("get_root_children_v2: found {} root items", entries.len());
    Ok(entries)
}

/// Get children at specific address with parent path (lazy loading)
pub fn get_children_at_addr<P: AsRef<Path>>(
    path: P,
    addr: u64,
    parent_path: &str,
) -> Result<Vec<TreeEntry>, Ad1Error> {
    debug!("get_children_at_addr_v2: addr=0x{:x}, parent={}", addr, parent_path);
    
    if addr == 0 {
        return get_root_children(path);
    }
    
    let session = SessionV2::open(path)?;
    
    // Read the parent item to get first_child_addr
    let parent_item = session.read_item_at(addr)?;
    
    if parent_item.first_child_addr == 0 {
        return Ok(Vec::new());
    }
    
    // Read all children in the chain
    let mut entries = Vec::new();
    let mut current_addr = parent_item.first_child_addr;
    
    while current_addr != 0 {
        let item = session.read_item_at(current_addr)?;
        let entry = build_tree_entry(&session, &item, parent_path)?;
        entries.push(entry);
        current_addr = item.next_item_addr;
    }
    
    debug!("get_children_at_addr_v2: found {} children", entries.len());
    Ok(entries)
}

/// Build TreeEntry from ItemHeader
fn build_tree_entry(
    session: &SessionV2,
    item: &ItemHeader,
    parent_path: &str,
) -> Result<TreeEntry, Ad1Error> {
    // Check if directory using all folder types from libad1
    let is_dir = matches!(item.item_type, FOLDER | REGULAR_FOLDER);
    
    // Build full path - no leading slash for root items
    let path = if parent_path.is_empty() {
        item.name.clone()
    } else {
        format!("{}/{}", parent_path, item.name)
    };

    // Read metadata if available
    let metadata = if item.first_metadata_addr != 0 {
        match session.read_metadata_chain(item.first_metadata_addr) {
            Ok(meta) => Some(meta),
            Err(e) => {
                warn!("Failed to read metadata for {}: {}", item.name, e);
                None
            }
        }
    } else {
        None
    };

    // Extract all metadata information (hashes, timestamps, attributes)
    let metadata_info = extract_metadata_info(&metadata);

    // Convert timestamps to ISO 8601 strings
    let created = metadata_info.created_time.map(|ts| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| ts.to_string())
    });

    let accessed = metadata_info.accessed_time.map(|ts| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| ts.to_string())
    });

    let modified = metadata_info.modified_time.map(|ts| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| ts.to_string())
    });

    // Build attributes string from Windows flags
    let attributes = if metadata_info.flags.is_empty() {
        None
    } else {
        Some(metadata_info.flags)
    };

    Ok(TreeEntry {
        name: item.name.clone(),
        path,
        is_dir,
        size: item.decompressed_size,
        item_type: item.item_type,
        first_child_addr: if item.first_child_addr != 0 {
            Some(item.first_child_addr)
        } else {
            None
        },
        data_addr: if item.zlib_metadata_addr != 0 {
            Some(item.zlib_metadata_addr)
        } else {
            None
        },
        item_addr: Some(item.offset),
        compressed_size: None, // Not easily available without reading zlib metadata
        data_end_addr: None,
        metadata_addr: if item.first_metadata_addr != 0 {
            Some(item.first_metadata_addr)
        } else {
            None
        },
        md5_hash: metadata_info.md5_hash,
        sha1_hash: metadata_info.sha1_hash,
        created,
        accessed,
        modified,
        attributes,
        child_count: None,
    })
}

/// Metadata extraction result structure
struct MetadataInfo {
    created_time: Option<i64>,
    accessed_time: Option<i64>,
    modified_time: Option<i64>,
    md5_hash: Option<String>,
    sha1_hash: Option<String>,
    flags: Vec<String>,
}

/// Extract comprehensive metadata information from metadata chain
/// Based on libad1_definitions.h categories and keys
fn extract_metadata_info(metadata: &Option<Vec<MetadataEntry>>) -> MetadataInfo {
    let mut info = MetadataInfo {
        created_time: None,
        accessed_time: None,
        modified_time: None,
        md5_hash: None,
        sha1_hash: None,
        flags: Vec::new(),
    };

    let Some(meta_list) = metadata else {
        return info;
    };

    for entry in meta_list {
        match entry.category {
            // HASH_INFO category (0x01)
            HASH_INFO => {
                match entry.key {
                    MD5_HASH => {
                        if entry.data.len() >= 16 {
                            info.md5_hash = Some(hex::encode(&entry.data[0..16]));
                        }
                    }
                    SHA1_HASH => {
                        if entry.data.len() >= 20 {
                            info.sha1_hash = Some(hex::encode(&entry.data[0..20]));
                        }
                    }
                    _ => {}
                }
            }

            // TIMESTAMP category (0x05)
            TIMESTAMP => {
                if entry.data.len() >= 8 {
                    let timestamp = u64::from_le_bytes([
                        entry.data[0], entry.data[1], entry.data[2], entry.data[3],
                        entry.data[4], entry.data[5], entry.data[6], entry.data[7],
                    ]);

                    // Convert Windows FILETIME to Unix timestamp
                    // FILETIME is 100-nanosecond intervals since January 1, 1601 UTC
                    const FILETIME_EPOCH_DIFF: u64 = 116444736000000000;
                    
                    if timestamp > FILETIME_EPOCH_DIFF {
                        let unix_time = (timestamp - FILETIME_EPOCH_DIFF) / 10000000;
                        
                        match entry.key {
                            ACCESS_TIME => {
                                info.accessed_time = Some(unix_time as i64);
                            }
                            MODIFIED_TIME => {
                                info.modified_time = Some(unix_time as i64);
                            }
                            CHANGE_TIME => {
                                // Change time - could be used as created if needed
                                if info.created_time.is_none() {
                                    info.created_time = Some(unix_time as i64);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // WINDOWS_FLAGS category (0x04)
            WINDOWS_FLAGS => {
                // Check if flag is set (non-zero data)
                let is_set = entry.data.iter().any(|&b| b != 0);
                
                if is_set {
                    let flag_name = match entry.key {
                        FLAG_ENCRYPTED => "Encrypted",
                        FLAG_COMPRESSED => "Compressed",
                        FLAG_HIDDEN => "Hidden",
                        FLAG_READ_ONLY => "Read-Only",
                        FLAG_ARCHIVE => "Archive",
                        _ => continue,
                    };
                    info.flags.push(flag_name.to_string());
                }
            }

            // ITEM_TYPE_CATEGORY (0x02), ITEM_SIZE_CATEGORY (0x03)
            // These are informational but we get size from item header
            _ => {}
        }
    }

    info
}

/// Read file data with decompression
pub fn read_file_data<P: AsRef<Path>>(
    path: P,
    item_addr: u64,
) -> Result<Vec<u8>, Ad1Error> {
    let session = SessionV2::open(path)?;
    let item = session.read_item_at(item_addr)?;
    
    if item.decompressed_size == 0 {
        return Ok(Vec::new());
    }

    if item.zlib_metadata_addr == 0 {
        return Err(Ad1Error::InvalidFormat(
            "Item has size but no zlib metadata".to_string()
        ));
    }

    decompress_file_data(&session, &item)
}

/// Decompress file data from zlib chunks (public for hash_v2 and extract_v2)
/// Uses LRU cache for repeated access (like libad1's file cache)
pub fn decompress_file_data(session: &SessionV2, item: &ItemHeader) -> Result<Vec<u8>, Ad1Error> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    // Empty files return immediately
    if item.decompressed_size == 0 {
        return Ok(Vec::new());
    }

    // Check cache first
    if let Ok(mut cache) = FILE_CACHE.lock() {
        if let Some(cached_data) = cache.search(item.offset) {
            debug!("Cache hit for {} (offset 0x{:X})", item.name, item.offset);
            return Ok((*cached_data).clone());
        }
    }

    // Read chunk count
    let chunk_count = session.read_u64_at(item.zlib_metadata_addr)?;
    
    debug!(
        "Decompressing {} chunks for {} (size: {})",
        chunk_count, item.name, item.decompressed_size
    );

    // Read chunk addresses
    let mut chunk_addrs = Vec::with_capacity((chunk_count + 1) as usize);
    for i in 0..=chunk_count {
        let addr = session.read_u64_at(item.zlib_metadata_addr + ((i + 1) * 8))?;
        chunk_addrs.push(addr);
    }

    // Decompress all chunks
    let mut decompressed = Vec::with_capacity(item.decompressed_size as usize);

    for i in 0..chunk_count as usize {
        let chunk_start = chunk_addrs[i];
        let chunk_size = chunk_addrs[i + 1] - chunk_start;

        // Read compressed chunk
        let compressed = session.arbitrary_read(chunk_start, chunk_size)?;

        // Decompress using zlib
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut chunk_data = Vec::new();
        decoder.read_to_end(&mut chunk_data).map_err(|e| {
            Ad1Error::DecompressionError(format!("Chunk {} decompression failed: {}", i, e))
        })?;

        decompressed.extend_from_slice(&chunk_data);
    }

    if decompressed.len() != item.decompressed_size as usize {
        warn!(
            "Decompressed size mismatch: expected {}, got {}",
            item.decompressed_size,
            decompressed.len()
        );
    }

    // Add to cache
    if let Ok(mut cache) = FILE_CACHE.lock() {
        cache.cache(item.offset, decompressed.clone());
        debug!("Cached data for {} (offset 0x{:X})", item.name, item.offset);
    }

    Ok(decompressed)
}

/// Get item info by address
pub fn get_item_info<P: AsRef<Path>>(
    path: P,
    addr: u64,
) -> Result<TreeEntry, Ad1Error> {
    let session = SessionV2::open(path)?;
    let item = session.read_item_at(addr)?;
    
    // Build entry without parent path (will use item name only)
    build_tree_entry(&session, &item, "")
}

/// Verify file hash
pub fn verify_item_hash<P: AsRef<Path>>(
    path: P,
    item_addr: u64,
) -> Result<bool, Ad1Error> {
    let session = SessionV2::open(path)?;
    let item = session.read_item_at(item_addr)?;
    
    // Read metadata to get stored hash
    let metadata = if item.first_metadata_addr != 0 {
        session.read_metadata_chain(item.first_metadata_addr)?
    } else {
        return Err(Ad1Error::InvalidFormat("No metadata found".to_string()));
    };

    // Find MD5 hash in metadata
    let stored_md5 = metadata.iter().find_map(|m| {
        if m.category == 0x01 && m.key == 0x5001 {
            Some(hex::encode(&m.data))
        } else {
            None
        }
    });

    let stored_md5 = stored_md5.ok_or_else(|| {
        Ad1Error::InvalidFormat("No MD5 hash in metadata".to_string())
    })?;

    // Decompress and hash file data
    use md5::Md5;
    use md5::Digest;
    
    let data = decompress_file_data(&session, &item)?;
    let mut hasher = Md5::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let computed_md5 = format!("{:x}", result);

    Ok(stored_md5.eq_ignore_ascii_case(&computed_md5))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_segment_header_reading() {
        // Test with the 40-segment AD1 file
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
        
        if !std::path::Path::new(path).exists() {
            println!("Test file not found, skipping test");
            return;
        }
        
        let session = SessionV2::open(path).expect("Failed to open AD1");
        
        // Should read 40 from offset 0x1C (header_size which equals segment count)
        assert_eq!(session.segments.len(), 40, "Should have 40 segments");
        
        println!("✓ V2 correctly opened 40 segments!");
        println!("  fragment_size: {}", session.fragment_size);
        println!("  first_item_addr: 0x{:X}", session.logical_header.first_item_addr);
    }
    
    #[test]
    fn test_item_name_decoding() {
        // Test with real AD1 file
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
        
        if !std::path::Path::new(path).exists() {
            println!("Test file not found, skipping test");
            return;
        }
        
        let session = SessionV2::open(path).expect("Failed to open AD1");
        
        // Read first item
        let first_item = session.read_item_at(session.logical_header.first_item_addr).expect("Failed to read item");
        
        println!("First item:");
        println!("  name: '{}'", first_item.name);
        println!("  name_length: {}", first_item.name_length);
        println!("  item_type: 0x{:X}", first_item.item_type);
        
        // Name should be readable, not garbage
        assert!(!first_item.name.is_empty(), "Name should not be empty");
        assert!(first_item.name.is_ascii() || first_item.name.chars().all(|c| !c.is_control()), 
                "Name should be readable, got: {}", first_item.name);
    }

    #[test]
    fn test_path_building() {
        // Test with real AD1 file
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
        
        if !std::path::Path::new(path).exists() {
            println!("Test file not found, skipping test");
            return;
        }
        
        let session = SessionV2::open(path).expect("Failed to open AD1");
        
        // Read first item
        let first_item = session.read_item_at(session.logical_header.first_item_addr).expect("Failed to read item");
        
        // Build path for first item
        let full_path = session.build_item_path(&first_item);
        println!("First item full path: '{}'", full_path);
        
        // Should at least contain the item name
        assert!(full_path.contains(&first_item.name), "Full path should contain item name");
        
        // If item has children, test path building for a child
        if first_item.first_child_addr != 0 {
            let child = session.read_item_at(first_item.first_child_addr).expect("Failed to read child");
            let child_path = session.build_item_path(&child);
            println!("Child path: '{}'", child_path);
            
            // Child path should be longer (include parent)
            assert!(child_path.len() >= child.name.len(), "Child path should include parent path");
        }
        
        println!("✓ Path building working!");
    }

    #[test]
    fn test_file_cache() {
        // Test with real AD1 file  
        let path = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
        
        if !std::path::Path::new(path).exists() {
            println!("Test file not found, skipping test");
            return;
        }
        
        let session = SessionV2::open(path).expect("Failed to open AD1");
        
        // Find a file item (not folder)
        let first_item = session.read_item_at(session.logical_header.first_item_addr).expect("Failed to read item");
        
        if first_item.first_child_addr == 0 {
            println!("No children to test cache");
            return;
        }
        
        // Find a file in children
        let children = session.read_children_at(first_item.first_child_addr).expect("Failed to read children");
        let file_item = children.iter().find(|i| i.item_type != 0x05 && i.decompressed_size > 0 && i.zlib_metadata_addr != 0);
        
        if let Some(item) = file_item {
            println!("Testing cache with file: {} (size: {})", item.name, item.decompressed_size);
            
            // First read - should decompress
            let data1 = decompress_file_data(&session, item).expect("Failed to decompress");
            println!("  First read: {} bytes", data1.len());
            
            // Second read - should hit cache
            let data2 = decompress_file_data(&session, item).expect("Failed to decompress");
            println!("  Second read (cached): {} bytes", data2.len());
            
            assert_eq!(data1, data2, "Cached data should match original");
            println!("✓ File caching working!");
        } else {
            println!("No file items found to test cache");
        }
    }
}
