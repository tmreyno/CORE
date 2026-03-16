// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Standardized Lazy Loading for Forensic Containers
//!
//! This module provides a unified lazy loading interface for all forensic
//! container types (AD1, E01, UFED, Archives). Large forensic images can
//! contain millions of files, so lazy loading is essential for responsive UI.
//!
//! ## Key Features
//!
//! - **Configurable batch sizes**: User-adjustable via settings
//! - **Entry count pre-fetch**: Fast count without loading all entries
//! - **On-demand expansion**: Load children only when folders are expanded
//! - **Unified interface**: Same API for all container types
//!
//! ## Configuration
//!
//! Settings can be adjusted through the app config:
//!
//! ```json
//! {
//!   "lazy_loading": {
//!     "enabled": true,
//!     "batch_size": 100,
//!     "auto_expand_threshold": 1000,
//!     "large_container_threshold": 10000
//!   }
//! }
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::common::lazy_loading::{LazyLoadConfig, LazyTreeEntry};
//!
//! // Get current config
//! let config = LazyLoadConfig::default();
//!
//! // Load root entries (lazy)
//! let root = container.get_lazy_root_children(&config)?;
//!
//! // Load children on expansion
//! let children = container.get_lazy_children_at_path(path, &config)?;
//! ```

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// =============================================================================
// Configuration
// =============================================================================

/// Global lazy loading configuration
///
/// This can be updated at runtime through settings
static LAZY_LOAD_CONFIG: RwLock<LazyLoadConfig> = RwLock::new(LazyLoadConfig::DEFAULT);

/// Configuration for lazy loading behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LazyLoadConfig {
    /// Whether lazy loading is enabled (true by default)
    pub enabled: bool,

    /// Maximum number of entries to load per batch/level
    /// Default: 100
    pub batch_size: usize,

    /// Threshold for auto-expanding directories
    /// Directories with fewer children than this will auto-expand
    /// Default: 50
    pub auto_expand_threshold: usize,

    /// Entry count threshold for switching to lazy loading
    /// Containers with more entries than this use lazy loading
    /// Default: 10,000
    pub large_container_threshold: usize,

    /// Maximum entries to show before pagination
    /// If a directory has more children than this, paginate
    /// Default: 500
    pub pagination_threshold: usize,

    /// Whether to show entry count before loading
    /// Default: true
    pub show_entry_count: bool,

    /// Timeout for entry count operations (milliseconds)
    /// Default: 5000 (5 seconds)
    pub count_timeout_ms: u64,

    /// Timeout for children loading operations (milliseconds)
    /// Default: 30000 (30 seconds)
    pub load_timeout_ms: u64,
}

impl LazyLoadConfig {
    /// Default configuration
    pub const DEFAULT: Self = Self {
        enabled: true,
        batch_size: 100,
        auto_expand_threshold: 50,
        large_container_threshold: 10_000,
        pagination_threshold: 500,
        show_entry_count: true,
        count_timeout_ms: 5_000,
        load_timeout_ms: 30_000,
    };

    /// Get the current global configuration
    pub fn get() -> Self {
        *LAZY_LOAD_CONFIG.read()
    }

    /// Update the global configuration
    pub fn set(config: Self) {
        *LAZY_LOAD_CONFIG.write() = config;
    }

    /// Check if lazy loading should be used for a container with this many entries
    pub fn should_use_lazy_loading(&self, entry_count: usize) -> bool {
        self.enabled && entry_count > self.large_container_threshold
    }

    /// Check if a directory should auto-expand
    pub fn should_auto_expand(&self, child_count: usize) -> bool {
        !self.enabled || child_count <= self.auto_expand_threshold
    }

    /// Check if pagination is needed
    pub fn needs_pagination(&self, child_count: usize) -> bool {
        child_count > self.pagination_threshold
    }
}

impl Default for LazyLoadConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// =============================================================================
// Free Functions for Config Access
// =============================================================================

/// Get the current lazy loading configuration
///
/// This is a convenience wrapper around `LazyLoadConfig::get()`.
pub fn get_config() -> LazyLoadConfig {
    LazyLoadConfig::get()
}

/// Update the lazy loading configuration
///
/// This is a convenience wrapper around `LazyLoadConfig::set()`.
pub fn update_config(config: LazyLoadConfig) {
    LazyLoadConfig::set(config)
}

// =============================================================================
// Unified Tree Entry Type
// =============================================================================

/// Unified tree entry for all container types
///
/// This provides a consistent interface for the UI regardless of whether
/// the source is AD1, E01, UFED, ZIP, or any other container format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyTreeEntry {
    /// Unique identifier for this entry (format-specific)
    /// For AD1: address as hex string
    /// For ZIP: entry index
    /// For E01: inode or path
    pub id: String,

    /// Display name
    pub name: String,

    /// Full path within the container
    pub path: String,

    /// Whether this is a directory/folder
    pub is_dir: bool,

    /// File size in bytes (0 for directories)
    pub size: u64,

    /// Entry type (file, folder, ad1, e01, zip, etc.)
    pub entry_type: String,

    /// Number of children (if known, -1 if unknown)
    /// Used to show expandable indicator before loading
    pub child_count: i64,

    /// Whether children have been loaded
    pub children_loaded: bool,

    /// Hash value if available
    pub hash: Option<String>,

    /// Last modified timestamp if available
    pub modified: Option<String>,

    /// Container-specific metadata (JSON)
    pub metadata: Option<String>,
}

impl LazyTreeEntry {
    /// Create a new lazy tree entry
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
        is_dir: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            is_dir,
            size: 0,
            entry_type: if is_dir {
                "folder".to_string()
            } else {
                "file".to_string()
            },
            child_count: if is_dir { -1 } else { 0 },
            children_loaded: false,
            hash: None,
            modified: None,
            metadata: None,
        }
    }

    /// Create a directory entry
    pub fn directory(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::new(id, name, path, true)
    }

    /// Create a file entry
    pub fn file(
        id: impl Into<String>,
        name: impl Into<String>,
        path: impl Into<String>,
        size: u64,
    ) -> Self {
        let mut entry = Self::new(id, name, path, false);
        entry.size = size;
        entry.child_count = 0;
        entry
    }

    /// Set the child count
    pub fn with_child_count(mut self, count: i64) -> Self {
        self.child_count = count;
        self
    }

    /// Set the entry type
    pub fn with_entry_type(mut self, entry_type: impl Into<String>) -> Self {
        self.entry_type = entry_type.into();
        self
    }

    /// Set the hash
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }

    /// Set the modified timestamp
    pub fn with_modified(mut self, modified: impl Into<String>) -> Self {
        self.modified = Some(modified.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: impl Into<String>) -> Self {
        self.metadata = Some(metadata.into());
        self
    }

    /// Mark children as loaded
    pub fn mark_children_loaded(mut self) -> Self {
        self.children_loaded = true;
        self
    }
}

// =============================================================================
// Lazy Loading Results
// =============================================================================

/// Result of a lazy loading operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LazyLoadResult {
    /// Loaded entries
    pub entries: Vec<LazyTreeEntry>,

    /// Total number of entries at this level
    pub total_count: usize,

    /// Whether there are more entries to load
    pub has_more: bool,

    /// Offset for pagination (next page starts here)
    pub next_offset: usize,

    /// Whether lazy loading was used
    pub lazy_loaded: bool,

    /// Configuration that was applied
    pub config: LazyLoadConfig,
}

impl LazyLoadResult {
    /// Create a new lazy load result
    pub fn new(entries: Vec<LazyTreeEntry>, total_count: usize) -> Self {
        let entry_count = entries.len();
        Self {
            entries,
            total_count,
            has_more: entry_count < total_count,
            next_offset: entry_count,
            lazy_loaded: true,
            config: LazyLoadConfig::get(),
        }
    }

    /// Create a result for fully loaded entries (no lazy loading)
    pub fn fully_loaded(entries: Vec<LazyTreeEntry>) -> Self {
        let count = entries.len();
        Self {
            entries,
            total_count: count,
            has_more: false,
            next_offset: count,
            lazy_loaded: false,
            config: LazyLoadConfig::get(),
        }
    }
}

// =============================================================================
// Container Info for Lazy Loading
// =============================================================================

/// Summary information for lazy loading decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    /// Path to the container file
    pub path: String,

    /// Container type (ad1, e01, zip, etc.)
    pub container_type: String,

    /// Total file size
    pub total_size: u64,

    /// Total entry count (files + directories)
    pub entry_count: usize,

    /// Root-level entry count
    pub root_entry_count: usize,

    /// Whether lazy loading is recommended
    pub lazy_loading_recommended: bool,

    /// Estimated time to load all entries (ms)
    pub estimated_load_time_ms: Option<u64>,
}

impl ContainerSummary {
    /// Create a new container summary
    pub fn new(
        path: impl Into<String>,
        container_type: impl Into<String>,
        total_size: u64,
        entry_count: usize,
    ) -> Self {
        let config = LazyLoadConfig::get();
        Self {
            path: path.into(),
            container_type: container_type.into(),
            total_size,
            entry_count,
            root_entry_count: 0,
            lazy_loading_recommended: config.should_use_lazy_loading(entry_count),
            estimated_load_time_ms: None,
        }
    }

    /// Set root entry count
    pub fn with_root_count(mut self, count: usize) -> Self {
        self.root_entry_count = count;
        self
    }

    /// Set estimated load time
    pub fn with_estimated_time(mut self, ms: u64) -> Self {
        self.estimated_load_time_ms = Some(ms);
        self
    }
}

// =============================================================================
// Trait for Lazy-Loadable Containers
// =============================================================================

/// Trait for containers that support lazy loading
///
/// All forensic container implementations should implement this trait
/// to provide consistent lazy loading behavior.
pub trait LazyLoadable {
    /// Get the container summary (fast operation)
    fn get_summary(&self) -> Result<ContainerSummary, String>;

    /// Get the total entry count (should be fast, may read header only)
    fn get_entry_count(&self) -> Result<usize, String>;

    /// Get root-level children only
    fn get_root_children(&self, config: &LazyLoadConfig) -> Result<LazyLoadResult, String>;

    /// Get children at a specific path or address
    fn get_children_at(
        &self,
        parent_id: &str,
        config: &LazyLoadConfig,
    ) -> Result<LazyLoadResult, String>;

    /// Get children with pagination
    fn get_children_paged(
        &self,
        parent_id: &str,
        offset: usize,
        limit: usize,
        config: &LazyLoadConfig,
    ) -> Result<LazyLoadResult, String>;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if lazy loading should be used based on entry count
pub fn should_lazy_load(entry_count: usize) -> bool {
    LazyLoadConfig::get().should_use_lazy_loading(entry_count)
}

/// Get the current batch size setting
pub fn get_batch_size() -> usize {
    LazyLoadConfig::get().batch_size
}

/// Get the current pagination threshold
pub fn get_pagination_threshold() -> usize {
    LazyLoadConfig::get().pagination_threshold
}

/// Update lazy loading settings
pub fn update_settings(config: LazyLoadConfig) {
    LazyLoadConfig::set(config);
}

/// Get current lazy loading settings
pub fn get_settings() -> LazyLoadConfig {
    LazyLoadConfig::get()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LazyLoadConfig::default();
        assert!(config.enabled);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.large_container_threshold, 10_000);
    }

    #[test]
    fn test_should_use_lazy_loading() {
        let config = LazyLoadConfig::default();
        assert!(!config.should_use_lazy_loading(5_000));
        assert!(config.should_use_lazy_loading(15_000));
    }

    #[test]
    fn test_should_auto_expand() {
        let config = LazyLoadConfig::default();
        assert!(config.should_auto_expand(10));
        assert!(config.should_auto_expand(50));
        assert!(!config.should_auto_expand(100));
    }

    #[test]
    fn test_lazy_tree_entry_builder() {
        let entry = LazyTreeEntry::file("1", "test.txt", "/test.txt", 1024)
            .with_hash("abc123")
            .with_modified("2024-01-01");

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.size, 1024);
        assert!(!entry.is_dir);
        assert_eq!(entry.hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_lazy_load_result() {
        let entries = vec![
            LazyTreeEntry::directory("1", "folder1", "/folder1"),
            LazyTreeEntry::file("2", "file.txt", "/file.txt", 100),
        ];

        let result = LazyLoadResult::new(entries, 10);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);
        assert_eq!(result.next_offset, 2);
    }

    #[test]
    fn test_container_summary() {
        let summary = ContainerSummary::new("/path/to/file.ad1", "ad1", 1024 * 1024 * 100, 50_000)
            .with_root_count(10);

        assert!(summary.lazy_loading_recommended);
        assert_eq!(summary.root_entry_count, 10);
    }
}
