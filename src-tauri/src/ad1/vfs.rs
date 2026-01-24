// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Virtual Filesystem Implementation
//!
//! ## Section Brief
//! Read-only virtual filesystem implementation for AD1 containers.
//! Provides safe, corruption-proof access to container contents.
//!
//! ### Key Types
//! - `Ad1Vfs` - Virtual filesystem for AD1 containers
//! - `Ad1VfsEntry` - Cached entry metadata
//!
//! ### Features
//! - Read-only access prevents container corruption
//! - Lazy loading of directory contents
//! - Metadata caching for performance
//! - Path traversal prevention
//!
//! ### Usage
//! ```rust,ignore
//! use crate::ad1::vfs::Ad1Vfs;
//! use crate::common::vfs::VirtualFileSystem;
//!
//! let vfs = Ad1Vfs::open("/path/to/container.ad1")?;
//!
//! // List root directory
//! let entries = vfs.readdir("/")?;
//!
//! // Read a file
//! let data = vfs.read("/Documents/file.txt", 0, 1024)?;
//! ```

use std::collections::HashMap;
use parking_lot::RwLock;

use crate::common::vfs::{VirtualFileSystem, VfsError, FileAttr, DirEntry, normalize_path, join_path};
use super::parser::Session;
use super::types::{Item, AD1_FOLDER_SIGNATURE, CREATED, ACCESS, MODIFIED};
use super::utils::find_timestamp;

// =============================================================================
// AD1 Virtual Filesystem
// =============================================================================

/// Virtual filesystem implementation for AD1 containers
/// 
/// Provides read-only access to AD1 container contents through a 
/// filesystem-like interface. All operations are safe and cannot
/// modify the underlying container.
pub struct Ad1Vfs {
    /// Container path
    path: String,
    /// Cached session (lazily opened)
    session: RwLock<Option<Session>>,
    /// Cached entry tree (path -> entry info)
    entry_cache: RwLock<HashMap<String, Ad1VfsEntry>>,
    /// Next synthetic inode number
    next_inode: RwLock<u64>,
}

/// Cached entry information
#[derive(Clone)]
struct Ad1VfsEntry {
    /// File attributes
    attr: FileAttr,
    /// Children paths (for directories)
    children: Option<Vec<String>>,
    /// Original Item reference data for reading
    #[allow(dead_code)]
    item_data: Option<ItemRef>,
}

/// Reference to item data for reading file contents
#[derive(Clone)]
#[allow(dead_code)]
struct ItemRef {
    /// Address for zlib data
    zlib_addr: u64,
    /// Decompressed size
    size: u64,
}

impl Ad1Vfs {
    /// Open an AD1 container for virtual filesystem access
    pub fn open(path: &str) -> Result<Self, VfsError> {
        // Verify file exists
        if !std::path::Path::new(path).exists() {
            return Err(VfsError::NotFound(path.to_string()));
        }
        
        let vfs = Self {
            path: path.to_string(),
            session: RwLock::new(None),
            entry_cache: RwLock::new(HashMap::new()),
            next_inode: RwLock::new(2), // 1 is reserved for root
        };
        
        // Initialize root entry
        vfs.init_root()?;
        
        Ok(vfs)
    }

    /// Initialize the root directory entry
    fn init_root(&self) -> Result<(), VfsError> {
        let root_attr = FileAttr {
            size: 0,
            is_directory: true,
            permissions: 0o555,
            nlink: 2,
            inode: 1,
            ..Default::default()
        };
        
        let root_entry = Ad1VfsEntry {
            attr: root_attr,
            children: None, // Will be loaded on first readdir
            item_data: None,
        };
        
        self.entry_cache.write().insert("/".to_string(), root_entry);
        
        Ok(())
    }

    /// Get or open session
    fn get_session(&self) -> Result<(), VfsError> {
        let needs_open = self.session.read().is_none();
        
        if needs_open {
            let new_session = Session::open(&self.path)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            *self.session.write() = Some(new_session);
        }
        
        Ok(())
    }

    /// Allocate a new inode number
    fn alloc_inode(&self) -> u64 {
        let mut next = self.next_inode.write();
        let inode = *next;
        *next += 1;
        inode
    }

    /// Load children for a directory path
    fn load_children(&self, path: &str) -> Result<Vec<String>, VfsError> {
        self.get_session()?;
        
        let session_guard = self.session.read();
        let session = session_guard.as_ref()
            .ok_or(VfsError::NotMounted)?;
        
        // Find items at this path
        let normalized = normalize_path(path);
        let items = if normalized == "/" {
            &session.root_items
        } else {
            // Find the parent item
            self.find_items_at_path(session, &normalized)?
        };
        
        // Build child entries - collect data first to avoid nested lock
        let child_data: Vec<_> = items.iter().map(|item| {
            let child_path = join_path(&normalized, &item.name);
            let inode = self.alloc_inode();
            let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
            let attr = self.item_to_attr(item, inode);
            let entry = Ad1VfsEntry {
                attr,
                children: if is_dir { None } else { Some(Vec::new()) },
                item_data: if is_dir {
                    None
                } else {
                    Some(ItemRef {
                        zlib_addr: item.zlib_metadata_addr,
                        size: item.decompressed_size,
                    })
                },
            };
            (child_path, entry)
        }).collect();
        
        // Drop session guard before getting write lock on cache
        drop(session_guard);
        
        let mut cache = self.entry_cache.write();
        let mut child_paths = Vec::new();
        
        for (child_path, entry) in child_data {
            cache.insert(child_path.clone(), entry);
            child_paths.push(child_path);
        }
        
        // Update parent's children list
        if let Some(parent) = cache.get_mut(&normalized) {
            parent.children = Some(child_paths.clone());
        }
        
        Ok(child_paths)
    }

    /// Find items at a specific path in the container
    fn find_items_at_path<'a>(&self, session: &'a Session, path: &str) -> Result<&'a [Item], VfsError> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        
        let mut current_items = session.root_items.as_slice();
        
        for (i, part) in parts.iter().enumerate() {
            let found = current_items.iter().find(|item| item.name == *part);
            
            match found {
                Some(item) => {
                    if i == parts.len() - 1 {
                        // This is the target - return its children
                        return Ok(&item.children);
                    } else {
                        // Navigate deeper
                        current_items = &item.children;
                    }
                }
                None => {
                    return Err(VfsError::NotFound(path.to_string()));
                }
            }
        }
        
        Ok(current_items)
    }

    /// Convert Item to FileAttr
    fn item_to_attr(&self, item: &Item, inode: u64) -> FileAttr {
        let is_dir = item.item_type == AD1_FOLDER_SIGNATURE;
        let size = if is_dir { 0 } else { item.decompressed_size };
        
        let created = find_timestamp(&item.metadata, CREATED)
            .and_then(|s| parse_timestamp(&s));
        let accessed = find_timestamp(&item.metadata, ACCESS)
            .and_then(|s| parse_timestamp(&s));
        let modified = find_timestamp(&item.metadata, MODIFIED)
            .and_then(|s| parse_timestamp(&s));
        
        FileAttr {
            size,
            is_directory: is_dir,
            created,
            modified,
            accessed,
            permissions: if is_dir { 0o555 } else { 0o444 },
            nlink: if is_dir { 2 } else { 1 },
            uid: 0,
            gid: 0,
            inode,
        }
    }

    /// Read file data from container
    fn read_file_data(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        // Get the cached entry
        let entry = self.entry_cache.read().get(path).cloned();
        
        let entry = entry.ok_or_else(|| VfsError::NotFound(path.to_string()))?;
        
        if entry.attr.is_directory {
            return Err(VfsError::NotAFile(path.to_string()));
        }
        
        // Use the session to read file data
        self.get_session()?;
        
        // We need to find the item and read its data
        let session_guard = self.session.read();
        let session = session_guard.as_ref()
            .ok_or(VfsError::NotMounted)?;
        
        // Find the item by path
        let item = self.find_item_by_path(session, path)?;
        
        // Need mutable session for reading
        drop(session_guard);
        
        let mut session_guard = self.session.write();
        let session = session_guard.as_mut()
            .ok_or(VfsError::NotMounted)?;
        
        // Read the file data
        let full_data = session.read_file_data(&item)
            .map_err(|e| VfsError::IoError(e.to_string()))?;
        
        // Apply offset and size
        let start = offset as usize;
        if start >= full_data.len() {
            return Ok(Vec::new());
        }
        
        let end = (start + size).min(full_data.len());
        Ok(full_data[start..end].to_vec())
    }

    /// Find an item by its path
    fn find_item_by_path(&self, session: &Session, path: &str) -> Result<Item, VfsError> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        
        if parts.is_empty() {
            return Err(VfsError::NotAFile(path.to_string()));
        }
        
        let mut current_items = &session.root_items;
        
        for (i, part) in parts.iter().enumerate() {
            let found = current_items.iter().find(|item| item.name == *part);
            
            match found {
                Some(item) => {
                    if i == parts.len() - 1 {
                        // This is the target
                        return Ok(item.clone());
                    } else {
                        // Navigate deeper
                        current_items = &item.children;
                    }
                }
                None => {
                    return Err(VfsError::NotFound(path.to_string()));
                }
            }
        }
        
        Err(VfsError::NotFound(path.to_string()))
    }
}

impl VirtualFileSystem for Ad1Vfs {
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);
        
        // Check cache
        if let Some(entry) = self.entry_cache.read().get(&normalized) {
            return Ok(entry.attr.clone());
        }
        
        // Not in cache - need to load parent directory first
        if let Some(parent) = crate::common::vfs::parent_path(&normalized) {
            // Load parent's children, which will cache this entry
            self.load_children(&parent)?;
            
            // Now try cache again
            if let Some(entry) = self.entry_cache.read().get(&normalized) {
                return Ok(entry.attr.clone());
            }
        }
        
        Err(VfsError::NotFound(normalized))
    }

    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);
        
        // Check if we have cached children
        let cached_children = {
            let cache = self.entry_cache.read();
            
            if let Some(entry) = cache.get(&normalized) {
                if !entry.attr.is_directory {
                    return Err(VfsError::NotADirectory(normalized));
                }
                entry.children.clone()
            } else {
                None
            }
        };
        
        // Load children if not cached
        let child_paths = match cached_children {
            Some(paths) => paths,
            None => self.load_children(&normalized)?,
        };
        
        // Build DirEntry list from cached entries
        let cache = self.entry_cache.read();
        
        let mut entries = Vec::new();
        for child_path in &child_paths {
            if let Some(entry) = cache.get(child_path) {
                let name = crate::common::vfs::filename(child_path).to_string();
                entries.push(DirEntry {
                    name,
                    is_directory: entry.attr.is_directory,
                    inode: entry.attr.inode,
                    file_type: if entry.attr.is_directory { 4 } else { 8 },
                });
            }
        }
        
        Ok(entries)
    }

    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        self.read_file_data(&normalized, offset, size)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse timestamp string to nanoseconds since epoch
fn parse_timestamp(s: &str) -> Option<i64> {
    use chrono::{DateTime, NaiveDateTime};
    
    // Try RFC 3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_nanos_opt().unwrap_or(0));
    }
    
    // Try common ISO formats
    let formats = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
    ];
    
    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt.and_utc().timestamp_nanos_opt().unwrap_or(0));
        }
    }
    
    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        let ts = parse_timestamp("2024-01-15T10:30:00Z");
        assert!(ts.is_some());
        
        let ts = parse_timestamp("2024-01-15 10:30:00");
        assert!(ts.is_some());
    }
}
