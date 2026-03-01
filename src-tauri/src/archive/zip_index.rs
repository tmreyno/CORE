// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Fast ZIP Index for Large Archives
//!
//! ## Problem
//! Standard ZIP parsing iterates through every entry in the Central Directory,
//! which is O(n) for n entries. For 100K+ entry archives, this is slow.
//!
//! ## Solution
//! Build and cache a tree-structured index that maps paths to children,
//! allowing O(1) lookups for any directory listing.
//!
//! ## Index Format (serialized to disk)
//! ```text
//! [Magic: "ZIDX"][Version: u8][Entry Count: u32]
//! [Tree Nodes...]
//! [String Table (deduplicated paths)]
//! ```
//!
//! ## Performance
//! - First load: O(n) to build index, cached to disk
//! - Subsequent loads: O(1) index load from cache (+ in-memory LRU)
//! - Directory listing: O(children) instead of O(total_entries)

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::containers::ContainerError;

// =============================================================================
// In-Memory Cache (LRU-style, max 8 indices)
// =============================================================================

const MAX_CACHED_INDICES: usize = 8;

/// Type alias for the index cache
type ZipIndexCache = Vec<(String, Arc<ZipIndex>)>;

/// Type alias for parsed directory result
type ParsedDirectory = (Vec<ZipIndexEntry>, HashMap<String, Vec<ZipIndexEntry>>);

/// In-memory cache for ZipIndex instances
static INDEX_CACHE: LazyLock<Mutex<ZipIndexCache>> =
    LazyLock::new(|| Mutex::new(Vec::with_capacity(MAX_CACHED_INDICES)));

fn cache_get(path: &str) -> Option<Arc<ZipIndex>> {
    let mut cache = INDEX_CACHE.lock().ok()?;
    // Move to front if found (LRU behavior)
    if let Some(pos) = cache.iter().position(|(p, _)| p == path) {
        let item = cache.remove(pos);
        let index = item.1.clone();
        cache.push(item);
        Some(index)
    } else {
        None
    }
}

fn cache_put(path: &str, index: ZipIndex) -> Arc<ZipIndex> {
    let arc = Arc::new(index);
    if let Ok(mut cache) = INDEX_CACHE.lock() {
        // Evict oldest if at capacity
        if cache.len() >= MAX_CACHED_INDICES {
            cache.remove(0);
        }
        cache.push((path.to_string(), arc.clone()));
    }
    arc
}

// =============================================================================
// Index Types
// =============================================================================

/// Cached ZIP index for fast tree navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipIndex {
    /// Version for cache invalidation
    pub version: u8,
    /// Source file size (for cache invalidation)
    pub source_size: u64,
    /// Source file modification time (for cache invalidation)
    pub source_mtime: u64,
    /// Total entry count
    pub entry_count: u32,
    /// Root entries (direct children of "/")
    pub root_entries: Vec<ZipIndexEntry>,
    /// Directory children map: dir_path -> children
    pub children: HashMap<String, Vec<ZipIndexEntry>>,
}

/// Single entry in the ZIP index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipIndexEntry {
    /// Entry name (not full path, just the filename/dirname)
    pub name: String,
    /// Full path within the archive
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Uncompressed size
    pub size: u64,
    /// Compressed size
    pub compressed_size: u64,
    /// CRC32 checksum
    pub crc32: u32,
    /// Index in the archive's central directory
    pub index: u32,
}

const INDEX_VERSION: u8 = 1;
const INDEX_MAGIC: &[u8] = b"ZIDX";

impl ZipIndex {
    /// Build index by parsing the ZIP central directory directly
    ///
    /// This bypasses the `zip` crate for maximum speed, reading the
    /// Central Directory in a single pass and building the tree.
    pub fn build(archive_path: &str) -> Result<Self, ContainerError> {
        debug!(path = %archive_path, "Building ZIP index");

        let path = Path::new(archive_path);
        let metadata = path
            .metadata()
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let source_size = metadata.len();
        let source_mtime = metadata
            .modified()
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
            .unwrap_or(0);

        let mut file = File::open(path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        // Find EOCD
        let (entry_count, cd_offset, cd_size) = Self::find_eocd(&mut file, source_size)?;

        debug!(entries = entry_count, cd_offset, cd_size, "Found EOCD");

        // Read Central Directory in one shot
        let mut cd_buf = vec![0u8; cd_size as usize];
        file.seek(SeekFrom::Start(cd_offset))
            .map_err(|e| format!("Failed to seek to CD: {}", e))?;
        file.read_exact(&mut cd_buf)
            .map_err(|e| format!("Failed to read CD: {}", e))?;

        // Parse entries and build tree
        let (root_entries, children) = Self::parse_central_directory(&cd_buf, entry_count)?;

        debug!(
            root_count = root_entries.len(),
            dir_count = children.len(),
            "ZIP index built"
        );

        Ok(Self {
            version: INDEX_VERSION,
            source_size,
            source_mtime,
            entry_count,
            root_entries,
            children,
        })
    }

    /// Find End of Central Directory record
    fn find_eocd(file: &mut File, file_size: u64) -> Result<(u32, u64, u64), ContainerError> {
        // Search last 65KB for EOCD signature (max ZIP comment is 65535 bytes)
        let search_size = file_size.min(65557) as usize;
        let mut buf = vec![0u8; search_size];

        let seek_pos = file_size - search_size as u64;
        file.seek(SeekFrom::Start(seek_pos))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        file.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read: {}", e))?;

        // Find EOCD signature (PK\x05\x06)
        let eocd_sig = [0x50, 0x4B, 0x05, 0x06];

        // EOCD is 22 bytes minimum. Search from end of buffer backwards.
        // Range: 0 to buf.len()-21 (inclusive), since we need at least 22 bytes for EOCD
        let max_offset = buf.len().saturating_sub(21);

        let eocd_pos = (0..max_offset)
            .rev()
            .find(|&i| buf[i..i + 4] == eocd_sig)
            .ok_or_else(|| ContainerError::InvalidFormat("ZIP EOCD not found".to_string()))?;

        // Check for ZIP64 EOCD locator (PK\x06\x07) right before EOCD
        let is_zip64 =
            eocd_pos >= 20 && buf[eocd_pos - 20..eocd_pos - 16] == [0x50, 0x4B, 0x06, 0x07];

        if is_zip64 {
            // ZIP64: Get EOCD64 location from locator
            let eocd64_offset = u64::from_le_bytes([
                buf[eocd_pos - 12],
                buf[eocd_pos - 11],
                buf[eocd_pos - 10],
                buf[eocd_pos - 9],
                buf[eocd_pos - 8],
                buf[eocd_pos - 7],
                buf[eocd_pos - 6],
                buf[eocd_pos - 5],
            ]);

            // Read ZIP64 EOCD
            let mut eocd64_buf = [0u8; 56];
            file.seek(SeekFrom::Start(eocd64_offset))
                .map_err(|e| format!("Failed to seek to EOCD64: {}", e))?;
            file.read_exact(&mut eocd64_buf)
                .map_err(|e| format!("Failed to read EOCD64: {}", e))?;

            // Verify signature (PK\x06\x06)
            if eocd64_buf[0..4] != [0x50, 0x4B, 0x06, 0x06] {
                return Err(ContainerError::InvalidFormat(
                    "Invalid ZIP64 EOCD signature".to_string(),
                ));
            }

            let entry_count = u64::from_le_bytes([
                eocd64_buf[32],
                eocd64_buf[33],
                eocd64_buf[34],
                eocd64_buf[35],
                eocd64_buf[36],
                eocd64_buf[37],
                eocd64_buf[38],
                eocd64_buf[39],
            ]) as u32;

            let cd_size = u64::from_le_bytes([
                eocd64_buf[40],
                eocd64_buf[41],
                eocd64_buf[42],
                eocd64_buf[43],
                eocd64_buf[44],
                eocd64_buf[45],
                eocd64_buf[46],
                eocd64_buf[47],
            ]);

            let cd_offset = u64::from_le_bytes([
                eocd64_buf[48],
                eocd64_buf[49],
                eocd64_buf[50],
                eocd64_buf[51],
                eocd64_buf[52],
                eocd64_buf[53],
                eocd64_buf[54],
                eocd64_buf[55],
            ]);

            Ok((entry_count, cd_offset, cd_size))
        } else {
            // Standard ZIP
            let entry_count = u16::from_le_bytes([buf[eocd_pos + 10], buf[eocd_pos + 11]]) as u32;
            let cd_size = u32::from_le_bytes([
                buf[eocd_pos + 12],
                buf[eocd_pos + 13],
                buf[eocd_pos + 14],
                buf[eocd_pos + 15],
            ]) as u64;
            let cd_offset = u32::from_le_bytes([
                buf[eocd_pos + 16],
                buf[eocd_pos + 17],
                buf[eocd_pos + 18],
                buf[eocd_pos + 19],
            ]) as u64;

            Ok((entry_count, cd_offset, cd_size))
        }
    }

    /// Parse Central Directory and build tree structure
    fn parse_central_directory(
        cd_buf: &[u8],
        entry_count: u32,
    ) -> Result<ParsedDirectory, ContainerError> {
        let mut root_entries: Vec<ZipIndexEntry> = Vec::new();
        let mut children: HashMap<String, Vec<ZipIndexEntry>> = HashMap::new();
        let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut pos = 0usize;
        let mut index = 0u32;

        while pos + 46 <= cd_buf.len() && index < entry_count {
            // Verify signature (PK\x01\x02)
            if cd_buf[pos..pos + 4] != [0x50, 0x4B, 0x01, 0x02] {
                break;
            }

            // Parse entry fields
            let compressed_size = u32::from_le_bytes([
                cd_buf[pos + 20],
                cd_buf[pos + 21],
                cd_buf[pos + 22],
                cd_buf[pos + 23],
            ]) as u64;
            let uncompressed_size = u32::from_le_bytes([
                cd_buf[pos + 24],
                cd_buf[pos + 25],
                cd_buf[pos + 26],
                cd_buf[pos + 27],
            ]) as u64;
            let filename_len = u16::from_le_bytes([cd_buf[pos + 28], cd_buf[pos + 29]]) as usize;
            let extra_len = u16::from_le_bytes([cd_buf[pos + 30], cd_buf[pos + 31]]) as usize;
            let comment_len = u16::from_le_bytes([cd_buf[pos + 32], cd_buf[pos + 33]]) as usize;
            let crc32 = u32::from_le_bytes([
                cd_buf[pos + 16],
                cd_buf[pos + 17],
                cd_buf[pos + 18],
                cd_buf[pos + 19],
            ]);

            // Get filename
            if pos + 46 + filename_len > cd_buf.len() {
                break;
            }
            let filename =
                String::from_utf8_lossy(&cd_buf[pos + 46..pos + 46 + filename_len]).to_string();
            let is_directory = filename.ends_with('/');
            let normalized_path = filename.trim_end_matches('/').to_string();

            // Determine parent and name
            let (parent_path, name) = if let Some(last_slash) = normalized_path.rfind('/') {
                let parent = &normalized_path[..last_slash];
                let name = &normalized_path[last_slash + 1..];
                (parent.to_string(), name.to_string())
            } else {
                (String::new(), normalized_path.clone())
            };

            let entry = ZipIndexEntry {
                name: name.clone(),
                path: normalized_path.clone(),
                is_directory,
                size: uncompressed_size,
                compressed_size,
                crc32,
                index,
            };

            // Add to appropriate parent
            if parent_path.is_empty() {
                root_entries.push(entry);
            } else {
                // Ensure all parent directories exist in the tree
                Self::ensure_parent_dirs(
                    &mut root_entries,
                    &mut children,
                    &mut seen_dirs,
                    &parent_path,
                );

                children.entry(parent_path.clone()).or_default().push(entry);
            }

            // Mark directory as seen if it's an explicit directory entry
            if is_directory {
                seen_dirs.insert(normalized_path);
            }

            // Move to next entry
            pos += 46 + filename_len + extra_len + comment_len;
            index += 1;
        }

        Ok((root_entries, children))
    }

    /// Ensure all parent directories exist in the tree (for implicit directories)
    fn ensure_parent_dirs(
        root_entries: &mut Vec<ZipIndexEntry>,
        children: &mut HashMap<String, Vec<ZipIndexEntry>>,
        seen_dirs: &mut std::collections::HashSet<String>,
        path: &str,
    ) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current = String::new();

        for (i, part) in parts.iter().enumerate() {
            let parent = current.clone();
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(part);

            if !seen_dirs.contains(&current) {
                seen_dirs.insert(current.clone());

                let dir_entry = ZipIndexEntry {
                    name: part.to_string(),
                    path: current.clone(),
                    is_directory: true,
                    size: 0,
                    compressed_size: 0,
                    crc32: 0,
                    index: 0, // Implicit directory, no real index
                };

                if i == 0 {
                    // First level goes in root
                    if !root_entries.iter().any(|e| e.path == current) {
                        root_entries.push(dir_entry);
                    }
                } else {
                    // Deeper levels go in children
                    let parent_children = children.entry(parent).or_default();
                    if !parent_children.iter().any(|e| e.path == current) {
                        parent_children.push(dir_entry);
                    }
                }
            }
        }
    }

    /// Get cache file path for a given archive
    pub fn cache_path(archive_path: &str) -> PathBuf {
        let archive_path = Path::new(archive_path);
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("core-ffx")
            .join("zip-index");

        // Create cache directory if needed
        let _ = std::fs::create_dir_all(&cache_dir);

        // Generate cache filename from archive path hash
        let hash = blake3::hash(archive_path.to_string_lossy().as_bytes());
        cache_dir.join(format!("{}.zidx", hex::encode(&hash.as_bytes()[..8])))
    }

    /// Save index to disk cache
    pub fn save(&self, cache_path: &Path) -> Result<(), ContainerError> {
        let file =
            File::create(cache_path).map_err(|e| format!("Failed to create cache file: {}", e))?;
        let mut writer = BufWriter::new(file);

        // Write magic
        writer
            .write_all(INDEX_MAGIC)
            .map_err(|e| format!("Failed to write magic: {}", e))?;

        // Serialize with bincode for speed
        bincode::serialize_into(&mut writer, self)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;

        debug!(path = %cache_path.display(), "Saved ZIP index to cache");
        Ok(())
    }

    /// Load index from disk cache
    pub fn load(cache_path: &Path) -> Result<Self, ContainerError> {
        let file =
            File::open(cache_path).map_err(|e| format!("Failed to open cache file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Verify magic
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| format!("Failed to read magic: {}", e))?;

        if magic != INDEX_MAGIC {
            return Err(ContainerError::InvalidFormat(
                "Invalid index magic".to_string(),
            ));
        }

        // Deserialize
        let index: Self = bincode::deserialize_from(&mut reader)
            .map_err(|e| format!("Failed to deserialize index: {}", e))?;

        debug!(path = %cache_path.display(), entries = index.entry_count, "Loaded ZIP index from cache");
        Ok(index)
    }

    /// Check if cached index is still valid
    pub fn is_valid_for(&self, archive_path: &str) -> bool {
        if let Ok(metadata) = Path::new(archive_path).metadata() {
            let size = metadata.len();
            let mtime = metadata
                .modified()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);

            self.source_size == size && self.source_mtime == mtime && self.version == INDEX_VERSION
        } else {
            false
        }
    }

    /// Get or create index (loads from cache if valid, builds otherwise)
    pub fn get_or_create(archive_path: &str) -> Result<Self, ContainerError> {
        // Check in-memory cache first
        if let Some(cached) = cache_get(archive_path) {
            if cached.is_valid_for(archive_path) {
                debug!(path = %archive_path, "Using in-memory cached ZIP index");
                // Clone from Arc - this is relatively cheap for the index
                return Ok((*cached).clone());
            }
        }

        let cache_path = Self::cache_path(archive_path);

        // Try to load from disk cache
        if cache_path.exists() {
            if let Ok(index) = Self::load(&cache_path) {
                if index.is_valid_for(archive_path) {
                    debug!(path = %archive_path, "Using disk-cached ZIP index");
                    // Store in memory cache
                    cache_put(archive_path, index.clone());
                    return Ok(index);
                }
            }
        }

        // Build new index
        debug!(path = %archive_path, "Building new ZIP index (this may take a moment for large archives)");
        let index = Self::build(archive_path)?;

        // Save to disk cache (ignore errors)
        if let Err(e) = index.save(&cache_path) {
            debug!(error = %e, "Failed to save ZIP index to cache");
        }

        // Store in memory cache
        cache_put(archive_path, index.clone());

        Ok(index)
    }

    /// Get root entries (fast - O(1))
    pub fn get_root_entries(&self) -> &[ZipIndexEntry] {
        &self.root_entries
    }

    /// Get children of a directory (fast - O(1))
    pub fn get_children(&self, path: &str) -> Option<&Vec<ZipIndexEntry>> {
        let normalized = path.trim_start_matches('/').trim_end_matches('/');
        self.children.get(normalized)
    }

    /// Get total entry count
    pub fn len(&self) -> usize {
        self.entry_count as usize
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_path_generation() {
        let path1 = ZipIndex::cache_path("/path/to/archive.zip");
        let path2 = ZipIndex::cache_path("/path/to/archive.zip");
        let path3 = ZipIndex::cache_path("/other/archive.zip");

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }
}
