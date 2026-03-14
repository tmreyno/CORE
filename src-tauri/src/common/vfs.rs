// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Virtual Filesystem Module
//!
//! ## Section Brief
//! Provides read-only virtual filesystem abstraction for forensic containers.
//! This prevents accidental corruption by ensuring all access is read-only.
//!
//! ### Core Types
//! - `FileAttr` - File/directory attributes (size, timestamps, permissions)
//! - `DirEntry` - Directory entry (name, type)
//! - `VfsError` - Virtual filesystem errors
//!
//! ### Core Traits
//! - `VirtualFileSystem` - Read-only filesystem operations
//! - `MountHandle` - Mounted container handle with caching
//!
//! ### Design Principles
//! 1. **Read-Only by Design** - No write operations exposed
//! 2. **Safe Access** - Path traversal prevention built-in
//! 3. **Lazy Loading** - Children loaded on demand
//! 4. **Caching** - Metadata cached for performance
//!
//! Inspired by libewf's mount_file_system and mount_file_entry patterns.

use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Error Types — re-exported from ffx-errors
// =============================================================================

pub use ffx_errors::VfsError;

// =============================================================================
// File Attributes
// =============================================================================

/// File/directory attributes (similar to struct stat)
/// All timestamps are in nanoseconds since Unix epoch
#[derive(Debug, Clone, Serialize)]
pub struct FileAttr {
    /// Size in bytes (0 for directories)
    pub size: u64,
    /// True if this is a directory
    pub is_directory: bool,
    /// Creation time (nanoseconds since epoch)
    pub created: Option<i64>,
    /// Modification time (nanoseconds since epoch)
    pub modified: Option<i64>,
    /// Access time (nanoseconds since epoch)
    pub accessed: Option<i64>,
    /// Unix permissions (e.g., 0o755)
    pub permissions: u16,
    /// Number of hard links (1 for files, 2+ for directories)
    pub nlink: u32,
    /// User ID (owner)
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// Inode number (synthetic)
    pub inode: u64,
}

impl Default for FileAttr {
    fn default() -> Self {
        Self {
            size: 0,
            is_directory: false,
            created: None,
            modified: None,
            accessed: None,
            permissions: 0o444, // Read-only by default
            nlink: 1,
            uid: 0,
            gid: 0,
            inode: 0,
        }
    }
}

impl FileAttr {
    /// Create attributes for a regular file
    pub fn file(size: u64) -> Self {
        Self {
            size,
            is_directory: false,
            permissions: 0o444, // r--r--r--
            nlink: 1,
            ..Default::default()
        }
    }

    /// Create attributes for a directory
    pub fn directory() -> Self {
        Self {
            size: 0,
            is_directory: true,
            permissions: 0o555, // r-xr-xr-x
            nlink: 2,
            ..Default::default()
        }
    }

    /// Set timestamps from ISO 8601 strings
    pub fn with_timestamps(
        mut self,
        created: Option<&str>,
        modified: Option<&str>,
        accessed: Option<&str>,
    ) -> Self {
        self.created = created.and_then(parse_iso_timestamp);
        self.modified = modified.and_then(parse_iso_timestamp);
        self.accessed = accessed.and_then(parse_iso_timestamp);
        self
    }

    /// Set timestamps from Unix timestamps (seconds)
    pub fn with_unix_timestamps(
        mut self,
        created: Option<i64>,
        modified: Option<i64>,
        accessed: Option<i64>,
    ) -> Self {
        // Convert seconds to nanoseconds
        self.created = created.map(|t| t * 1_000_000_000);
        self.modified = modified.map(|t| t * 1_000_000_000);
        self.accessed = accessed.map(|t| t * 1_000_000_000);
        self
    }

    /// Get file mode for FUSE (combines type and permissions)
    pub fn mode(&self) -> u32 {
        if self.is_directory {
            0o40000 | (self.permissions as u32)
        } else {
            0o100000 | (self.permissions as u32)
        }
    }
}

/// Parse ISO 8601 timestamp to nanoseconds since epoch
fn parse_iso_timestamp(s: &str) -> Option<i64> {
    // Try parsing common ISO formats
    use chrono::{DateTime, NaiveDateTime};

    // Try full ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp_nanos_opt().unwrap_or(0));
    }

    // Try without timezone (assume UTC)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.and_utc().timestamp_nanos_opt().unwrap_or(0));
    }

    // Try date only
    if let Ok(dt) = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", s), "%Y-%m-%dT%H:%M:%S") {
        return Some(dt.and_utc().timestamp_nanos_opt().unwrap_or(0));
    }

    None
}

// =============================================================================
// Directory Entry
// =============================================================================

/// Directory entry returned by readdir
#[derive(Debug, Clone, Serialize)]
pub struct DirEntry {
    /// Entry name (filename only, not full path)
    pub name: String,
    /// True if this entry is a directory
    pub is_directory: bool,
    /// Inode number (synthetic)
    pub inode: u64,
    /// File type for FUSE (DT_DIR=4, DT_REG=8)
    pub file_type: u8,
}

impl DirEntry {
    /// Create a directory entry
    pub fn new(name: impl Into<String>, is_directory: bool) -> Self {
        Self {
            name: name.into(),
            is_directory,
            inode: 0,
            file_type: if is_directory { 4 } else { 8 }, // DT_DIR or DT_REG
        }
    }

    /// Create with inode
    pub fn with_inode(mut self, inode: u64) -> Self {
        self.inode = inode;
        self
    }
}

// =============================================================================
// Virtual Filesystem Trait
// =============================================================================

/// Read-only virtual filesystem interface
///
/// This trait provides FUSE-like operations for accessing container contents
/// without risk of modification. All operations are read-only by design.
///
/// # Example
/// ```rust,ignore
/// let vfs = container.mount(path)?;
///
/// // Get file attributes
/// let attr = vfs.getattr("/")?;
/// assert!(attr.is_directory);
///
/// // List directory contents
/// let entries = vfs.readdir("/")?;
/// for entry in entries {
///     println!("{}: {}", if entry.is_directory { "DIR" } else { "FILE" }, entry.name);
/// }
///
/// // Read file data
/// let data = vfs.read("/path/to/file.txt", 0, 1024)?;
/// ```
pub trait VirtualFileSystem: Send + Sync {
    /// Get file/directory attributes
    ///
    /// Returns attributes for the entry at the given path.
    /// Path should be absolute within the container (e.g., "/folder/file.txt").
    fn getattr(&self, path: &str) -> Result<FileAttr, VfsError>;

    /// List directory contents
    ///
    /// Returns all entries in the directory at the given path.
    /// Does not include "." and ".." entries.
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError>;

    /// Read file data
    ///
    /// Reads up to `size` bytes starting at `offset` from the file at `path`.
    /// Returns the data read (may be less than `size` if EOF reached).
    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError>;

    /// Check if path exists
    fn exists(&self, path: &str) -> bool {
        self.getattr(path).is_ok()
    }

    /// Check if path is a directory
    fn is_dir(&self, path: &str) -> Result<bool, VfsError> {
        Ok(self.getattr(path)?.is_directory)
    }

    /// Check if path is a file
    fn is_file(&self, path: &str) -> Result<bool, VfsError> {
        Ok(!self.getattr(path)?.is_directory)
    }

    /// Get file size
    fn file_size(&self, path: &str) -> Result<u64, VfsError> {
        Ok(self.getattr(path)?.size)
    }

    /// Read entire file (convenience method)
    ///
    /// Warning: Only use for reasonably sized files!
    fn read_all(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let size = self.file_size(path)?;
        if size > 100 * 1024 * 1024 {
            return Err(VfsError::IoError(
                "File too large for read_all (>100MB), use read() instead".to_string(),
            ));
        }
        self.read(path, 0, size as usize)
    }
}

// =============================================================================
// Mount Handle
// =============================================================================

/// Handle for a mounted container with metadata caching
///
/// This provides a cached view of the container's virtual filesystem.
/// Metadata is cached to avoid repeated parsing for frequently accessed entries.
pub struct MountHandle<T: VirtualFileSystem> {
    /// The underlying virtual filesystem
    vfs: T,
    /// Cached attributes (path -> FileAttr)
    attr_cache: RwLock<HashMap<String, FileAttr>>,
    /// Cached directory listings (path -> `Vec<DirEntry>`)
    dir_cache: RwLock<HashMap<String, Vec<DirEntry>>>,
    /// Mount timestamp (nanoseconds since epoch)
    mounted_at: i64,
    /// Path prefix for mounted paths
    path_prefix: String,
}

impl<T: VirtualFileSystem> MountHandle<T> {
    /// Create a new mount handle wrapping a virtual filesystem
    pub fn new(vfs: T) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        Self {
            vfs,
            attr_cache: RwLock::new(HashMap::new()),
            dir_cache: RwLock::new(HashMap::new()),
            mounted_at: now,
            path_prefix: String::new(),
        }
    }

    /// Create with a path prefix
    pub fn with_prefix(vfs: T, prefix: impl Into<String>) -> Self {
        let mut handle = Self::new(vfs);
        handle.path_prefix = prefix.into();
        handle
    }

    /// Get mount timestamp
    pub fn mounted_at(&self) -> i64 {
        self.mounted_at
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.attr_cache.write().clear();
        self.dir_cache.write().clear();
    }

    /// Get attributes with caching
    pub fn getattr_cached(&self, path: &str) -> Result<FileAttr, VfsError> {
        let normalized = normalize_path(path);

        // Check cache first
        if let Some(attr) = self.attr_cache.read().get(&normalized) {
            return Ok(attr.clone());
        }

        // Fetch and cache
        let attr = self.vfs.getattr(&normalized)?;
        self.attr_cache.write().insert(normalized, attr.clone());
        Ok(attr)
    }

    /// List directory with caching
    pub fn readdir_cached(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
        let normalized = normalize_path(path);

        // Check cache first
        if let Some(entries) = self.dir_cache.read().get(&normalized) {
            return Ok(entries.clone());
        }

        // Fetch and cache
        let entries = self.vfs.readdir(&normalized)?;
        self.dir_cache.write().insert(normalized, entries.clone());
        Ok(entries)
    }

    /// Read data (not cached - too large)
    pub fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path);
        self.vfs.read(&normalized, offset, size)
    }

    /// Get underlying VFS reference
    pub fn vfs(&self) -> &T {
        &self.vfs
    }
}

// =============================================================================
// Path Utilities
// =============================================================================

/// Normalize a path for consistent lookup
/// - Removes trailing slashes (except for root)
/// - Ensures leading slash
/// - Prevents path traversal
pub fn normalize_path(path: &str) -> String {
    let path = path.trim();

    // Empty or root
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }

    // Build normalized path
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                // Prevent traversal - just ignore ..
                parts.pop();
            }
            _ => parts.push(part),
        }
    }

    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

/// Check if a path is safe (no traversal attempts)
pub fn is_safe_path(path: &str) -> bool {
    !path.contains("..") && !path.contains('\0')
}

// =============================================================================
// Mounted Partition Support
// =============================================================================

use super::filesystem::{FilesystemDriver, PartitionEntry};

/// A mounted partition with its filesystem driver
///
/// This struct is used by VFS implementations that support mounting partitions
/// from disk images (e.g., RAW, E01). It associates a partition's metadata with
/// its filesystem driver and a human-readable mount point name.
pub struct MountedPartition {
    /// Partition metadata from the partition table
    pub entry: PartitionEntry,
    /// Filesystem driver for reading partition contents
    pub fs: Box<dyn FilesystemDriver>,
    /// Mount point name (e.g., "Partition_1_NTFS", "Volume_2_HFS+")
    pub mount_name: String,
}

impl MountedPartition {
    /// Create a new mounted partition
    pub fn new(entry: PartitionEntry, fs: Box<dyn FilesystemDriver>, mount_name: String) -> Self {
        Self {
            entry,
            fs,
            mount_name,
        }
    }

    /// Create a mount name from partition number and filesystem type
    pub fn generate_mount_name(number: u32, fs_type: Option<&str>) -> String {
        match fs_type {
            Some(fs) => format!("Partition_{}_{}", number, fs),
            None => format!("Partition_{}", number),
        }
    }
}

/// Find a mounted partition by path and return the remaining sub-path
///
/// Given a path like "/Partition_1_NTFS/Users/Documents", this function
/// finds the partition matching "Partition_1_NTFS" and returns the remaining
/// path "/Users/Documents" for the filesystem driver to handle.
///
/// # Arguments
/// * `partitions` - Slice of mounted partitions to search
/// * `path` - The full path to search (should be normalized)
///
/// # Returns
/// * `Some((partition, remaining_path))` - The matched partition and sub-path
/// * `None` - No partition matches the path
///
/// # Example
/// ```rust,ignore
/// let partitions = vec![/* mounted partitions */];
/// if let Some((partition, sub_path)) = find_partition(&partitions, "/Partition_1_NTFS/file.txt") {
///     // partition.mount_name == "Partition_1_NTFS"
///     // sub_path == "/file.txt"
///     partition.fs.read(&sub_path, 0, 1024)?;
/// }
/// ```
pub fn find_partition<'a>(
    partitions: &'a [MountedPartition],
    path: &str,
) -> Option<(&'a MountedPartition, String)> {
    // Strip leading slash for matching
    let path_trimmed = path.trim_start_matches('/');

    for partition in partitions {
        // Check for exact mount point match
        if path_trimmed == partition.mount_name {
            return Some((partition, "/".to_string()));
        }

        // Check for path within partition
        let prefix = format!("{}/", partition.mount_name);
        if path_trimmed.starts_with(&prefix) {
            let remaining = format!("/{}", &path_trimmed[prefix.len()..]);
            return Some((partition, remaining));
        }
    }
    None
}

/// Check if a path is a partition mount point
pub fn is_partition_mount_point(partitions: &[MountedPartition], path: &str) -> bool {
    let path_trimmed = path.trim_start_matches('/');
    partitions.iter().any(|p| p.mount_name == path_trimmed)
}

/// Join two paths safely
pub fn join_path(base: &str, name: &str) -> String {
    let base = normalize_path(base);
    let name = name.trim_matches('/');

    if base == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", base, name)
    }
}

/// Get parent path
pub fn parent_path(path: &str) -> Option<String> {
    let normalized = normalize_path(path);
    if normalized == "/" {
        return None;
    }

    if let Some(pos) = normalized.rfind('/') {
        if pos == 0 {
            Some("/".to_string())
        } else {
            Some(normalized[..pos].to_string())
        }
    } else {
        Some("/".to_string())
    }
}

/// Get filename from path
pub fn filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(""), "/");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path("/foo"), "/foo");
        assert_eq!(normalize_path("/foo/"), "/foo");
        assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("/foo/../bar"), "/bar");
        assert_eq!(normalize_path("/foo/./bar"), "/foo/bar");
        assert_eq!(normalize_path("/../../../etc/passwd"), "/etc/passwd");
    }

    #[test]
    fn test_join_path() {
        assert_eq!(join_path("/", "foo"), "/foo");
        assert_eq!(join_path("/foo", "bar"), "/foo/bar");
        assert_eq!(join_path("/foo/", "bar"), "/foo/bar");
    }

    #[test]
    fn test_parent_path() {
        assert_eq!(parent_path("/"), None);
        assert_eq!(parent_path("/foo"), Some("/".to_string()));
        assert_eq!(parent_path("/foo/bar"), Some("/foo".to_string()));
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename("/foo/bar.txt"), "bar.txt");
        assert_eq!(filename("/foo/bar/"), "");
        assert_eq!(filename("file.txt"), "file.txt");
    }

    #[test]
    fn test_file_attr_mode() {
        let file = FileAttr::file(1024);
        assert_eq!(file.mode() & 0o170000, 0o100000); // Regular file

        let dir = FileAttr::directory();
        assert_eq!(dir.mode() & 0o170000, 0o40000); // Directory
    }
}
