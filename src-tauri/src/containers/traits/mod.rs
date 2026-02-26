// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence Container Traits and Error Types
//!
//! This module defines traits and types for evidence container parsers.
//!
//! # Active Components
//!
//! The following types are actively used throughout the codebase:
//! - `ContainerError` - Unified error type for container operations
//! - `LifecycleStage` - Evidence lifecycle tracking
//!
//! # Deprecated Components
//!
//! The trait-based parser system is **DEPRECATED** and not used in production.
//! The application uses direct module calls via `operations.rs` instead.
//! These traits are preserved for potential future plugin system development:
//! - `EvidenceContainer`, `SegmentedContainer`, `TreeContainer`, `HashableContainer`
//! - `MountableContainer`, `FormatInfo`, `ContainerMetadata`, etc.
//!
//! For current container operations, use:
//! - `containers::info()`, `containers::verify()`, `containers::extract()` from `operations.rs`
//! - `unified::get_children()`, `unified::get_summary()` from `unified.rs`
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    EvidenceContainer Trait                      │
//! │  - format_info() → FormatInfo                                   │
//! │  - detect() → bool                                              │
//! │  - info() → ContainerInfo                                       │
//! │  - verify() → VerifyResult                                      │
//! │  - extract() → ()                                               │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Optional Extensions                                            │
//! │  - SegmentedContainer: Multi-segment support                    │
//! │  - TreeContainer: File/folder tree support                      │
//! │  - HashableContainer: Stored hash verification                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Implementing a New Format
//!
//! To add support for a new format:
//!
//! 1. Create format module under `src-tauri/src/` (e.g., `myformat/`)
//! 2. Implement the `EvidenceContainer` trait
//! 3. Register format in `formats.rs`
//! 4. Add detection in `containers/operations.rs`
//! 5. Add TypeScript types in `src/types.ts`
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::containers::traits::{EvidenceContainer, FormatInfo};
//!
//! pub struct MyFormatParser;
//!
//! impl EvidenceContainer for MyFormatParser {
//!     fn format_info(&self) -> FormatInfo {
//!         FormatInfo {
//!             id: "myformat",
//!             name: "My Format",
//!             extensions: &["myf", "myf2"],
//!             category: FormatCategory::ForensicContainer,
//!         }
//!     }
//!     
//!     fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
//!         // Check magic bytes or signature
//!     }
//!     
//!     // ... implement other methods
//! }
//! ```

mod error;
mod results;

// Re-export everything for backwards compatibility
pub use error::*;
pub use results::*;

use std::path::Path;

use crate::formats::ContainerFormat;
use crate::common::vfs::VirtualFileSystem;

// =============================================================================
// CORE TRAIT
// =============================================================================

/// Core trait for all evidence container parsers
/// 
/// This trait defines the unified interface that all format-specific parsers
/// must implement. It provides the foundation for the evidence lifecycle:
/// detection → parsing → verification → extraction.
pub trait EvidenceContainer: Send + Sync {
    /// Get format information for this container type
    fn format_info(&self) -> FormatInfo;
    
    /// Detect if the given path is this container format
    /// 
    /// Should check magic bytes/signatures, not just file extension.
    fn detect(&self, path: &Path) -> Result<bool, ContainerError>;
    
    /// Parse container and return metadata
    /// 
    /// # Arguments
    /// * `path` - Path to the container file
    /// * `include_tree` - Whether to parse the full file/folder tree
    fn info(&self, path: &Path, include_tree: bool) -> Result<ContainerMetadata, ContainerError>;
    
    /// Fast info - only parse headers, skip tree
    fn info_fast(&self, path: &Path) -> Result<ContainerMetadata, ContainerError> {
        self.info(path, false)
    }
    
    /// Verify container integrity
    /// 
    /// # Arguments
    /// * `path` - Path to the container file
    /// * `algorithm` - Hash algorithm to use (e.g., "sha256")
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError>;
    
    /// Extract container contents to output directory
    fn extract(&self, path: &Path, output_dir: &Path) -> Result<(), ContainerError>;
}

// =============================================================================
// OPTIONAL EXTENSION TRAITS
// =============================================================================

/// Extension trait for segmented containers (E01, AD1, Raw .001)
pub trait SegmentedContainer: EvidenceContainer {
    /// Discover all segments for this container
    fn discover_segments(&self, path: &Path) -> Result<SegmentInfo, ContainerError>;
    
    /// Get info for a specific segment
    fn segment_info(&self, path: &Path, index: u32) -> Result<SegmentMetadata, ContainerError>;
}

/// Extension trait for containers with file/folder trees (AD1, L01)
pub trait TreeContainer: EvidenceContainer {
    /// List entries in the container's file tree
    fn list_entries(&self, path: &Path) -> Result<Vec<TreeEntryInfo>, ContainerError>;
    
    /// Get info for a specific entry
    fn entry_info(&self, container_path: &Path, entry_path: &str) -> Result<TreeEntryInfo, ContainerError>;
    
    /// Extract a specific entry
    fn extract_entry(&self, container_path: &Path, entry_path: &str, output_path: &Path) -> Result<(), ContainerError>;
}

/// Extension trait for containers with embedded hashes
pub trait HashableContainer: EvidenceContainer {
    /// Get all stored hashes from the container
    fn stored_hashes(&self, path: &Path) -> Result<Vec<StoredHashInfo>, ContainerError>;
    
    /// Verify stored hashes against computed values
    fn verify_stored_hashes(&self, path: &Path) -> Result<Vec<HashResult>, ContainerError>;
}

// =============================================================================
// VIRTUAL FILESYSTEM TRAIT
// =============================================================================

/// Extension trait for containers that support virtual filesystem mounting
/// 
/// This trait enables read-only, corruption-safe access to container contents
/// through a filesystem-like interface. Implementations must ensure:
/// 
/// - All operations are strictly read-only
/// - No container data can be modified through the VFS
/// - Path traversal attacks are prevented
/// - Operations are thread-safe
/// 
/// # Example
/// 
/// ```rust,ignore
/// use crate::containers::traits::MountableContainer;
/// use crate::common::vfs::VirtualFileSystem;
/// 
/// let parser = Ad1Parser;
/// let vfs = parser.mount(Path::new("/path/to/container.ad1"))?;
/// 
/// // List root directory
/// let entries = vfs.readdir("/")?;
/// 
/// // Read a file
/// let data = vfs.read("/Documents/file.txt", 0, 1024)?;
/// ```
pub trait MountableContainer: EvidenceContainer {
    /// Mount the container as a virtual filesystem
    /// 
    /// Returns a read-only virtual filesystem interface for accessing
    /// container contents. The returned VFS handle should be used for
    /// all file access operations.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the container file(s)
    /// 
    /// # Returns
    /// 
    /// A boxed VirtualFileSystem implementation, or an error if mounting fails.
    fn mount(&self, path: &Path) -> Result<Box<dyn VirtualFileSystem>, ContainerError>;
    
    /// Check if the container supports virtual filesystem mounting
    /// 
    /// Some containers may not support VFS access (e.g., disk images without
    /// filesystem support). This method allows checking before attempting mount.
    fn supports_mount(&self) -> bool {
        true
    }
}

// =============================================================================
// REGISTRY TRAIT
// =============================================================================

/// Trait for format registry (used by extension system)
/// 
/// This trait is defined for future plugin/extension architecture where
/// third-party format parsers can be registered dynamically.
#[allow(dead_code)]
pub trait FormatRegistry {
    /// Get all registered formats
    fn formats(&self) -> &[&'static ContainerFormat];
    
    /// Detect format for a given path
    fn detect_format(&self, path: &Path) -> Option<&'static ContainerFormat>;
    
    /// Get parser for a format
    fn get_parser(&self, format_id: &str) -> Option<Box<dyn EvidenceContainer>>;
    
    /// Register a new format parser
    fn register_parser(&mut self, parser: Box<dyn EvidenceContainer>);
}
