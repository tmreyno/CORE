// =============================================================================
// CORE-FFX - Unified Container Handler
// =============================================================================
//!
//! Single unified interface for all forensic container types.
//! This module consolidates all file/folder listing into one consistent API.
//!
//! ## Supported Container Types
//!
//! | Type | Extensions | Handler |
//! |------|------------|---------|
//! | AD1 | .ad1 | Ad1Handler |
//! | E01/EWF | .e01, .l01, .ex01, .lx01 | EwfHandler |
//! | UFED | .ufd, .ufdr, .ufdx, dirs | UfedHandler |
//! | ZIP | .zip | ZipHandler |
//! | 7-Zip | .7z, .7z.001 | SevenZipHandler |
//! | TAR | .tar, .tar.gz, .tgz, etc | TarHandler |
//! | RAR | .rar | RarHandler |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::containers::unified::{get_children, ContainerType};
//!
//! // Auto-detect type and get children
//! let result = get_children("/path/to/container.ad1", "", None, None)?;
//!
//! // Or specify type explicitly
//! let result = get_children_typed(ContainerType::Ad1, "/path/to/file.ad1", "", None, None)?;
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

use crate::common::lazy_loading::{LazyLoadConfig, LazyLoadResult, LazyTreeEntry, ContainerSummary};
use crate::containers::ContainerError;

// =============================================================================
// Helper Functions
// =============================================================================

/// Extract filename from a path string
fn path_to_name(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    trimmed.rsplit('/').next()
        .or_else(|| trimmed.rsplit('\\').next())
        .unwrap_or(trimmed)
        .to_string()
}

/// Check if an entry is at root level (no parent directory)
fn is_root_entry(path: &str) -> bool {
    let trimmed = path.trim_end_matches('/');
    !trimmed.contains('/') && !trimmed.contains('\\')
}

/// Check if an entry is a direct child of the given parent path
fn is_direct_child(entry_path: &str, parent_path: &str) -> bool {
    if parent_path.is_empty() {
        return is_root_entry(entry_path);
    }
    
    let parent_prefix = if parent_path.ends_with('/') { 
        parent_path.to_string() 
    } else { 
        format!("{}/", parent_path) 
    };
    
    if !entry_path.starts_with(&parent_prefix) {
        return false;
    }
    
    let rest = &entry_path[parent_prefix.len()..];
    let trimmed = rest.trim_end_matches('/');
    !trimmed.contains('/') && !trimmed.contains('\\')
}

/// Convert ArchiveEntry to LazyTreeEntry
fn archive_entry_to_lazy(entry: &crate::archive::extraction::ArchiveEntry) -> LazyTreeEntry {
    let name = path_to_name(&entry.path);
    if entry.is_directory {
        LazyTreeEntry::directory(&entry.path, &name, &entry.path)
    } else {
        LazyTreeEntry::file(&entry.path, &name, &entry.path, entry.size)
    }
}

/// Generic helper for get_root_children on archive types
fn archive_get_root_children(
    entries: &[crate::archive::extraction::ArchiveEntry],
    offset: usize,
    limit: usize,
) -> LazyLoadResult {
    let root_entries: Vec<_> = entries.iter()
        .filter(|e| is_root_entry(&e.path))
        .collect();
    
    let total = root_entries.len();
    let items: Vec<LazyTreeEntry> = root_entries.into_iter()
        .skip(offset)
        .take(limit)
        .map(archive_entry_to_lazy)
        .collect();
    
    LazyLoadResult::new(items, total)
}

/// Generic helper for get_children on archive types
fn archive_get_children(
    entries: &[crate::archive::extraction::ArchiveEntry],
    parent_path: &str,
    offset: usize,
    limit: usize,
) -> LazyLoadResult {
    let children: Vec<_> = entries.iter()
        .filter(|e| is_direct_child(&e.path, parent_path))
        .collect();
    
    let total = children.len();
    let items: Vec<LazyTreeEntry> = children.into_iter()
        .skip(offset)
        .take(limit)
        .map(archive_entry_to_lazy)
        .collect();
    
    LazyLoadResult::new(items, total)
}

// =============================================================================
// Container Type Detection
// =============================================================================

/// Supported container types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerType {
    /// AccessData AD1 logical image
    Ad1,
    /// EnCase EWF format (E01, L01, Ex01, Lx01)
    Ewf,
    /// Cellebrite UFED extraction
    Ufed,
    /// ZIP archive
    Zip,
    /// 7-Zip archive
    SevenZip,
    /// TAR archive (including .tar.gz, .tar.bz2, etc)
    Tar,
    /// RAR archive
    Rar,
    /// Raw disk image
    Raw,
    /// Memory dump (RAM capture)
    Memory,
    /// Unknown/unsupported format
    Unknown,
}

impl ContainerType {
    /// Detect container type from file path
    pub fn detect(path: &str) -> Self {
        let lower = path.to_lowercase();
        let path_obj = Path::new(path);
        
        // UFED detection (check first - can be folders or specific files)
        if lower.ends_with(".ufd") || lower.ends_with(".ufdr") || lower.ends_with(".ufdx") {
            return Self::Ufed;
        }
        
        // Check for UFED folder (contains report.xml)
        if path_obj.is_dir()
            && path_obj.join("report.xml").exists() {
                return Self::Ufed;
            }
        
        // Check via UFED module's detection
        if crate::ufed::is_ufed(path) {
            return Self::Ufed;
        }
        
        // AD1
        if lower.ends_with(".ad1") {
            return Self::Ad1;
        }
        
        // EWF formats
        if lower.ends_with(".e01") || lower.ends_with(".l01") || 
           lower.ends_with(".ex01") || lower.ends_with(".lx01") {
            return Self::Ewf;
        }
        
        // Archive formats
        if lower.ends_with(".zip") {
            return Self::Zip;
        }
        
        if lower.ends_with(".7z") || 
           (lower.contains(".7z.") && lower.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
            return Self::SevenZip;
        }
        
        if lower.ends_with(".rar") {
            return Self::Rar;
        }
        
        if lower.ends_with(".tar") || lower.ends_with(".tar.gz") || lower.ends_with(".tgz") ||
           lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") || 
           lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
            return Self::Tar;
        }
        
        // Memory dumps (check before raw images - has more specific patterns)
        if lower.contains("_mem.raw") || lower.ends_with(".mem") || lower.ends_with(".vmem") ||
           lower.ends_with(".dmp") || lower.contains("_memdump") ||
           lower.contains(".hiberfil") || lower.contains(".pagefile") {
            return Self::Memory;
        }
        
        // Raw disk images
        if lower.ends_with(".raw") || lower.ends_with(".img") || lower.ends_with(".dd") ||
           lower.ends_with(".001") || lower.ends_with(".bin") {
            return Self::Raw;
        }
        
        Self::Unknown
    }
    
    /// Get display name for this container type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ad1 => "AD1 Logical Image",
            Self::Ewf => "EnCase EWF Image",
            Self::Ufed => "UFED Extraction",
            Self::Zip => "ZIP Archive",
            Self::SevenZip => "7-Zip Archive",
            Self::Tar => "TAR Archive",
            Self::Rar => "RAR Archive",
            Self::Raw => "Raw Disk Image",
            Self::Memory => "Memory Dump",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for ContainerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// =============================================================================
// Unified Handler Trait
// =============================================================================

/// Unified trait for all container handlers
/// 
/// All container types implement this trait to provide consistent
/// lazy loading behavior. Implementations should use the fastest
/// available method (e.g., ZipIndex for ZIP files).
pub trait UnifiedContainerHandler: Send + Sync {
    /// Get the container type
    fn container_type(&self) -> ContainerType;
    
    /// Get container summary (fast - for UI display)
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError>;
    
    /// Get total entry count (should be fast, may read headers only)
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError>;
    
    /// Get root-level children only
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError>;
    
    /// Get children at a specific parent path
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError>;
}

// =============================================================================
// AD1 Handler
// =============================================================================

pub struct Ad1Handler;

impl UnifiedContainerHandler for Ad1Handler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Ad1
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // AD1: estimate count from file size (~500 bytes per entry average)
        let estimated_count = (total_size / 500) as usize;
        Ok(ContainerSummary::new(path, "ad1", total_size, estimated_count))
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        // AD1 doesn't have a fast entry count - would need to walk tree
        // Use estimate based on file size
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        Ok((total_size / 500) as usize)
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        self.get_children(path, "", offset, limit)
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let children = crate::ad1::get_children(path, parent_path)?;
        let total = children.len();
        
        let entries: Vec<LazyTreeEntry> = children.into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                let id = c.item_addr
                    .map(|a| format!("ad1_{:x}", a))
                    .unwrap_or_else(|| c.path.clone());
                    
                let mut entry = if c.is_dir {
                    LazyTreeEntry::directory(&id, &c.name, &c.path)
                } else {
                    LazyTreeEntry::file(&id, &c.name, &c.path, c.size)
                };
                
                if let Some(modified) = c.modified {
                    entry = entry.with_modified(modified);
                }
                entry
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
}

// =============================================================================
// EWF Handler
// =============================================================================

pub struct EwfHandler;

impl UnifiedContainerHandler for EwfHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Ewf
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        // EWF: always recommend lazy loading for disk images
        let mut summary = ContainerSummary::new(path, "ewf", total_size, 0);
        summary.lazy_loading_recommended = true;
        Ok(summary)
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        // EWF: would need to mount and scan - return 0 and use lazy loading
        let _ = path;
        Ok(0)
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        // EWF requires VFS mounting
        use crate::ewf::EwfVfs;
        use crate::common::vfs::VirtualFileSystem;
        
        let vfs = EwfVfs::open(path)
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        
        let dir_entries = vfs.readdir("/")
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        
        let total = dir_entries.len();
        let entries: Vec<LazyTreeEntry> = dir_entries.into_iter()
            .skip(offset)
            .take(limit)
            .map(|e| {
                // DirEntry has: name, is_directory, inode, file_type (no size)
                if e.is_directory {
                    LazyTreeEntry::directory(&e.name, &e.name, format!("/{}", e.name))
                } else {
                    // VFS DirEntry doesn't include size - set to 0
                    LazyTreeEntry::file(&e.name, &e.name, format!("/{}", e.name), 0)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        use crate::ewf::EwfVfs;
        use crate::common::vfs::VirtualFileSystem;
        
        let vfs = EwfVfs::open(path)
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        
        let dir_entries = vfs.readdir(parent_path)
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        
        let total = dir_entries.len();
        let entries: Vec<LazyTreeEntry> = dir_entries.into_iter()
            .skip(offset)
            .take(limit)
            .map(|e| {
                let full_path = if parent_path == "/" {
                    format!("/{}", e.name)
                } else {
                    format!("{}/{}", parent_path, e.name)
                };
                
                if e.is_directory {
                    LazyTreeEntry::directory(&e.name, &e.name, full_path)
                } else {
                    LazyTreeEntry::file(&e.name, &e.name, full_path, 0)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
}

// =============================================================================
// UFED Handler
// =============================================================================

pub struct UfedHandler;

impl UnifiedContainerHandler for UfedHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Ufed
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let count = crate::ufed::get_entry_count(path).unwrap_or(0);
        Ok(ContainerSummary::new(path, "ufed", total_size, count))
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        crate::ufed::get_entry_count(path)
            .map_err(|e| ContainerError::IoError(e.to_string()))
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let children = crate::ufed::get_root_children(path)?;
        let total = children.len();
        
        let entries: Vec<LazyTreeEntry> = children.into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                if c.is_dir {
                    LazyTreeEntry::directory(&c.path, &c.name, &c.path)
                } else {
                    LazyTreeEntry::file(&c.path, &c.name, &c.path, c.size)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let children = crate::ufed::get_children(path, parent_path)?;
        let total = children.len();
        
        let entries: Vec<LazyTreeEntry> = children.into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                if c.is_dir {
                    LazyTreeEntry::directory(&c.path, &c.name, &c.path)
                } else {
                    LazyTreeEntry::file(&c.path, &c.name, &c.path, c.size)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
}

// =============================================================================
// ZIP Handler (using fast ZipIndex)
// =============================================================================

pub struct ZipHandler;

impl UnifiedContainerHandler for ZipHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Zip
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let count = crate::archive::extraction::get_zip_entry_count(path).unwrap_or(0);
        Ok(ContainerSummary::new(path, "zip", total_size, count))
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        crate::archive::extraction::get_zip_entry_count(path)
            .map_err(|e| ContainerError::IoError(e.to_string()))
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        // Use fast ZipIndex for O(1) lookups
        let index = crate::archive::ZipIndex::get_or_create(path)?;
        let children = index.get_root_entries();
        let total = children.len();
        
        let entries: Vec<LazyTreeEntry> = children.iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                if c.is_directory {
                    LazyTreeEntry::directory(&c.path, &c.name, &c.path)
                } else {
                    LazyTreeEntry::file(&c.path, &c.name, &c.path, c.size)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        // Use fast ZipIndex for O(1) lookups
        let index = crate::archive::ZipIndex::get_or_create(path)?;
        let children = index.get_children(parent_path)
            .cloned()
            .unwrap_or_default();
        
        let total = children.len();
        let entries: Vec<LazyTreeEntry> = children.into_iter()
            .skip(offset)
            .take(limit)
            .map(|c| {
                if c.is_directory {
                    LazyTreeEntry::directory(&c.path, &c.name, &c.path)
                } else {
                    LazyTreeEntry::file(&c.path, &c.name, &c.path, c.size)
                }
            })
            .collect();
        
        Ok(LazyLoadResult::new(entries, total))
    }
}

// =============================================================================
// 7-Zip Handler
// =============================================================================

pub struct SevenZipHandler;

impl UnifiedContainerHandler for SevenZipHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::SevenZip
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let mut summary = ContainerSummary::new(path, "7z", total_size, 0);
        summary.lazy_loading_recommended = true;
        Ok(summary)
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        let entries = crate::archive::sevenz::list_entries(path)?;
        Ok(entries.len())
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::sevenz::list_entries(path)?;
        Ok(archive_get_root_children(&entries, offset, limit))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::sevenz::list_entries(path)?;
        Ok(archive_get_children(&entries, parent_path, offset, limit))
    }
}

// =============================================================================
// TAR Handler
// =============================================================================

pub struct TarHandler;

impl UnifiedContainerHandler for TarHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Tar
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let mut summary = ContainerSummary::new(path, "tar", total_size, 0);
        summary.lazy_loading_recommended = true;
        Ok(summary)
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        let entries = crate::archive::tar::list_entries(path)?;
        Ok(entries.len())
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::tar::list_entries(path)?;
        Ok(archive_get_root_children(&entries, offset, limit))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::tar::list_entries(path)?;
        Ok(archive_get_children(&entries, parent_path, offset, limit))
    }
}

// =============================================================================
// RAR Handler
// =============================================================================

pub struct RarHandler;

impl UnifiedContainerHandler for RarHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Rar
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let mut summary = ContainerSummary::new(path, "rar", total_size, 0);
        summary.lazy_loading_recommended = true;
        Ok(summary)
    }
    
    fn get_entry_count(&self, path: &str) -> Result<usize, ContainerError> {
        let entries = crate::archive::rar::list_entries(path)?;
        Ok(entries.len())
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::rar::list_entries(path)?;
        Ok(archive_get_root_children(&entries, offset, limit))
    }
    
    fn get_children(
        &self, 
        path: &str, 
        parent_path: &str, 
        offset: usize, 
        limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let entries = crate::archive::rar::list_entries(path)?;
        Ok(archive_get_children(&entries, parent_path, offset, limit))
    }
}

// =============================================================================
// Raw Disk Image Handler
// =============================================================================

/// Raw disk image handler
/// 
/// Raw images (.dd, .raw, .img, .001) require partition detection and
/// filesystem mounting before their contents can be listed. Use the
/// VFS commands (vfs_mount_image, vfs_list_dir) for directory browsing.
/// 
/// This handler provides summary info but delegates tree browsing to VFS.
pub struct RawHandler;

impl UnifiedContainerHandler for RawHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Raw
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let total_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let mut summary = ContainerSummary::new(path, "raw", total_size, 0);
        summary.lazy_loading_recommended = true;
        Ok(summary)
    }
    
    fn get_entry_count(&self, _path: &str) -> Result<usize, ContainerError> {
        // Raw: would need to mount and scan
        Ok(0)
    }
    
    fn get_root_children(
        &self, 
        _path: &str, 
        _offset: usize, 
        _limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        // Raw images require partition detection and VFS mounting
        // Return error directing user to use VFS commands
        Err(ContainerError::UnsupportedOperation(
            "Raw disk images require VFS mounting. Use vfs_mount_image first.".to_string()
        ))
    }
    
    fn get_children(
        &self, 
        _path: &str, 
        _parent_path: &str, 
        _offset: usize, 
        _limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        Err(ContainerError::UnsupportedOperation(
            "Raw disk images require VFS mounting. Use vfs_mount_image first.".to_string()
        ))
    }
}

// =============================================================================
// Memory Dump Handler
// =============================================================================

/// Memory dump handler
/// 
/// Memory dumps (_mem.raw, .vmem, .dmp, etc.) are RAM captures that don't 
/// contain filesystems. They display as a single physical raw view with 
/// metadata about the memory capture (size, segments).
/// 
/// Memory dumps support hex viewing and string search but not file browsing.
pub struct MemoryHandler;

impl UnifiedContainerHandler for MemoryHandler {
    fn container_type(&self) -> ContainerType {
        ContainerType::Memory
    }
    
    fn get_summary(&self, path: &str) -> Result<ContainerSummary, ContainerError> {
        let path_obj = std::path::Path::new(path);
        
        // Count segments if this is a multi-segment memory dump
        let total_size = if let Some(parent) = path_obj.parent() {
            if let Some(stem) = path_obj.file_stem().and_then(|s| s.to_str()) {
                // Check for multi-segment pattern (e.g., file.raw.001, file.raw.002)
                let mut size = 0u64;
                
                // Handle patterns like "foo_mem.raw.001" -> base is "foo_mem.raw"
                let base_name = if stem.ends_with(".raw") || stem.ends_with(".mem") {
                    stem.to_string()
                } else if let Some(base) = stem.strip_suffix(".raw") {
                    format!("{}.raw", base)
                } else {
                    stem.to_string()
                };
                
                // Scan for segments
                let mut found_segments = false;
                for i in 1..=999 {
                    let segment_name = format!("{}.{:03}", base_name, i);
                    let segment_path = parent.join(&segment_name);
                    if segment_path.exists() {
                        if let Ok(meta) = std::fs::metadata(&segment_path) {
                            size += meta.len();
                            found_segments = true;
                        }
                    } else {
                        break;
                    }
                }
                
                if found_segments {
                    size
                } else {
                    // Single file
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                }
            } else {
                std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            }
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };
        
        let mut summary = ContainerSummary::new(path, "memory", total_size, 1);
        summary.lazy_loading_recommended = false; // No tree structure
        Ok(summary)
    }
    
    fn get_entry_count(&self, _path: &str) -> Result<usize, ContainerError> {
        // Memory dumps show as single virtual file
        Ok(1)
    }
    
    fn get_root_children(
        &self, 
        path: &str, 
        _offset: usize, 
        _limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        let path_obj = std::path::Path::new(path);
        
        // Count segments for multi-segment memory dumps
        let segment_count = if let Some(parent) = path_obj.parent() {
            if let Some(stem) = path_obj.file_stem().and_then(|s| s.to_str()) {
                let base_name = if stem.ends_with(".raw") || stem.ends_with(".mem") {
                    stem.to_string()
                } else if let Some(base) = stem.strip_suffix(".raw") {
                    format!("{}.raw", base)
                } else {
                    stem.to_string()
                };
                
                let mut count = 0u32;
                for i in 1..=999 {
                    let segment_name = format!("{}.{:03}", base_name, i);
                    if parent.join(&segment_name).exists() {
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count > 0 { count } else { 1 }
            } else {
                1
            }
        } else {
            1
        };
        
        // Get total size from summary
        let summary = self.get_summary(path)?;
        
        let file_name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("memory.raw")
            .to_string();
        
        let mut entry = LazyTreeEntry::new(
            path.to_string(),     // id
            file_name,            // name
            path.to_string(),     // path
            false,                // is_dir
        );
        entry.size = summary.total_size;
        entry.entry_type = "memory".to_string();
        entry.child_count = 0;
        entry.children_loaded = true;
        
        // Store segment info as JSON metadata
        let metadata = serde_json::json!({
            "type": "memory_dump",
            "segments": segment_count,
            "format": "raw"
        });
        entry.metadata = Some(metadata.to_string());
        
        Ok(LazyLoadResult::new(vec![entry], 1))
    }
    
    fn get_children(
        &self, 
        _path: &str, 
        _parent_path: &str, 
        _offset: usize, 
        _limit: usize
    ) -> Result<LazyLoadResult, ContainerError> {
        // Memory dumps have no children - they're single raw files
        Ok(LazyLoadResult::new(vec![], 0))
    }
}

// =============================================================================
// Handler Registry & Dispatch
// =============================================================================

/// Get the appropriate handler for a container type
pub fn get_handler(container_type: ContainerType) -> Box<dyn UnifiedContainerHandler> {
    match container_type {
        ContainerType::Ad1 => Box::new(Ad1Handler),
        ContainerType::Ewf => Box::new(EwfHandler),
        ContainerType::Ufed => Box::new(UfedHandler),
        ContainerType::Zip => Box::new(ZipHandler),
        ContainerType::SevenZip => Box::new(SevenZipHandler),
        ContainerType::Tar => Box::new(TarHandler),
        ContainerType::Rar => Box::new(RarHandler),
        ContainerType::Raw => Box::new(RawHandler),
        ContainerType::Memory => Box::new(MemoryHandler),
        ContainerType::Unknown => Box::new(Ad1Handler), // Fallback
    }
}

/// Get handler by auto-detecting container type from path
pub fn get_handler_for_path(path: &str) -> Box<dyn UnifiedContainerHandler> {
    let container_type = ContainerType::detect(path);
    debug!("get_handler_for_path: {} -> {:?}", path, container_type);
    get_handler(container_type)
}

// =============================================================================
// Public API - Single Entry Points
// =============================================================================

/// Get container summary (auto-detects type)
pub fn get_summary(path: &str) -> Result<ContainerSummary, ContainerError> {
    get_handler_for_path(path).get_summary(path)
}

/// Get entry count (auto-detects type)
pub fn get_entry_count(path: &str) -> Result<usize, ContainerError> {
    get_handler_for_path(path).get_entry_count(path)
}

/// Get root children (auto-detects type)
pub fn get_root_children(
    path: &str, 
    offset: Option<usize>, 
    limit: Option<usize>
) -> Result<LazyLoadResult, ContainerError> {
    let config = LazyLoadConfig::get();
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(config.batch_size);
    
    get_handler_for_path(path).get_root_children(path, offset, limit)
}

/// Get children at parent path (auto-detects type)
pub fn get_children(
    path: &str, 
    parent_path: &str,
    offset: Option<usize>, 
    limit: Option<usize>
) -> Result<LazyLoadResult, ContainerError> {
    let config = LazyLoadConfig::get();
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(config.batch_size);
    
    get_handler_for_path(path).get_children(path, parent_path, offset, limit)
}

/// Get children with explicit container type
pub fn get_children_typed(
    container_type: ContainerType,
    path: &str, 
    parent_path: &str,
    offset: Option<usize>, 
    limit: Option<usize>
) -> Result<LazyLoadResult, ContainerError> {
    let config = LazyLoadConfig::get();
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(config.batch_size);
    
    get_handler(container_type).get_children(path, parent_path, offset, limit)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_type_detection() {
        assert_eq!(ContainerType::detect("/path/to/file.ad1"), ContainerType::Ad1);
        assert_eq!(ContainerType::detect("/path/to/file.AD1"), ContainerType::Ad1);
        assert_eq!(ContainerType::detect("/path/to/file.e01"), ContainerType::Ewf);
        assert_eq!(ContainerType::detect("/path/to/file.E01"), ContainerType::Ewf);
        assert_eq!(ContainerType::detect("/path/to/file.zip"), ContainerType::Zip);
        assert_eq!(ContainerType::detect("/path/to/file.7z"), ContainerType::SevenZip);
        assert_eq!(ContainerType::detect("/path/to/file.tar.gz"), ContainerType::Tar);
        assert_eq!(ContainerType::detect("/path/to/file.rar"), ContainerType::Rar);
        assert_eq!(ContainerType::detect("/path/to/file.ufd"), ContainerType::Ufed);
    }
    
    #[test]
    fn test_handler_dispatch() {
        let handler = get_handler(ContainerType::Ad1);
        assert_eq!(handler.container_type(), ContainerType::Ad1);
        
        let handler = get_handler(ContainerType::Zip);
        assert_eq!(handler.container_type(), ContainerType::Zip);
    }
}
