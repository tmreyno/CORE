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

use std::path::Path;
use serde::Serialize;
use thiserror::Error;

use crate::formats::{FormatCategory, ContainerFormat};

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Unified error type for all forensic container operations.
///
/// `ContainerError` provides structured, contextual error information for
/// failures during container parsing, verification, and extraction. Each
/// variant captures relevant context (file paths, offsets, etc.) to aid
/// in debugging and user-facing error messages.
///
/// # Error Categories
///
/// | Category | Variants | Common Causes |
/// |----------|----------|---------------|
/// | **I/O** | `FileNotFound`, `IoError` | Missing files, permissions, disk errors |
/// | **Format** | `InvalidFormat`, `ParseError` | Corrupted data, unsupported versions |
/// | **Integrity** | `HashMismatch`, `VerificationError` | Tampered or damaged containers |
/// | **Segments** | `SegmentError` | Missing .E02/.ad2 files, sequence gaps |
/// | **Data** | `CompressionError`, `ExtractionError` | Decompression failures, write errors |
/// | **Config** | `ConfigError`, `UnsupportedOperation` | Invalid parameters, unsupported features |
///
/// # Example
///
/// ```rust,ignore
/// use crate::containers::ContainerError;
///
/// fn open_container(path: &str) -> Result<(), ContainerError> {
///     if !Path::new(path).exists() {
///         return Err(ContainerError::FileNotFound(format!(
///             "Container not found: {}", path
///         )));
///     }
///     // ... parsing logic
///     Ok(())
/// }
/// ```
///
/// # Conversion
///
/// `ContainerError` implements `From` for common error types:
/// - `std::io::Error` → `ContainerError::IoError`
/// - `serde_json::Error` → `ContainerError::SerializationError`
/// - `rusqlite::Error` → `ContainerError::ParseError`
/// - `zip::ZipError` → `ContainerError::ParseError`
///
/// It also implements `Into<String>` for Tauri command compatibility.
#[derive(Debug, Clone, Error)]
pub enum ContainerError {
    /// File or directory not found at the specified path.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Container format is invalid, unrecognized, or corrupted.
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Requested operation is not supported for this container type.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Error parsing container internal structure.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Computed hash does not match expected/stored hash.
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        /// The expected hash value (from container metadata or companion file)
        expected: String,
        /// The actually computed hash value
        actual: String,
    },

    /// Error related to multi-segment containers.
    #[error("Segment error: {0}")]
    SegmentError(String),

    /// Error during content extraction to filesystem.
    #[error("Extraction error: {0}")]
    ExtractionError(String),

    /// Decompression failure for compressed container data.
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// General verification failure (not hash-specific).
    #[error("Verification error: {0}")]
    VerificationError(String),

    /// Invalid configuration or parameter value.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error during search operations within containers.
    #[error("Search error: {0}")]
    SearchError(String),

    /// Serialization/deserialization error for JSON, CSV, etc.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Entry (file/folder) not found within container.
    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    /// Internal error indicating unexpected program state.
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for ContainerError {
    fn from(err: std::io::Error) -> Self {
        ContainerError::IoError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ContainerError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ContainerError::ParseError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for ContainerError {
    fn from(err: std::num::ParseIntError) -> Self {
        ContainerError::ParseError(err.to_string())
    }
}

impl From<ContainerError> for String {
    fn from(err: ContainerError) -> String {
        err.to_string()
    }
}

impl From<String> for ContainerError {
    fn from(msg: String) -> Self {
        ContainerError::InternalError(msg)
    }
}

impl From<&str> for ContainerError {
    fn from(msg: &str) -> Self {
        ContainerError::InternalError(msg.to_string())
    }
}

impl From<serde_json::Error> for ContainerError {
    fn from(err: serde_json::Error) -> Self {
        ContainerError::SerializationError(err.to_string())
    }
}

impl From<regex::Error> for ContainerError {
    fn from(err: regex::Error) -> Self {
        ContainerError::ParseError(format!("Regex error: {}", err))
    }
}

impl From<rusqlite::Error> for ContainerError {
    fn from(err: rusqlite::Error) -> Self {
        ContainerError::ParseError(format!("Database error: {}", err))
    }
}

impl From<zip::result::ZipError> for ContainerError {
    fn from(err: zip::result::ZipError) -> Self {
        ContainerError::ParseError(format!("ZIP error: {}", err))
    }
}

impl From<std::str::Utf8Error> for ContainerError {
    fn from(err: std::str::Utf8Error) -> Self {
        ContainerError::ParseError(format!("UTF-8 error: {}", err))
    }
}

impl From<std::num::TryFromIntError> for ContainerError {
    fn from(err: std::num::TryFromIntError) -> Self {
        ContainerError::ParseError(format!("Integer conversion error: {}", err))
    }
}

impl From<std::array::TryFromSliceError> for ContainerError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        ContainerError::ParseError(format!("Slice conversion error: {}", err))
    }
}

// =============================================================================
// LIFECYCLE STAGES
// =============================================================================

/// Evidence container lifecycle stages
/// 
/// Represents the stages a container goes through during analysis:
/// 
/// ```text
/// Discovered → Detected → Opened → Verified → Extracted
///     ↓           ↓          ↓         ↓          ↓
/// scan_dir   detect()   info()   verify()   extract()
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, strum::Display, strum::EnumString, strum::AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum LifecycleStage {
    /// Container file discovered during directory scan
    Discovered,
    /// Format detected (magic bytes verified)
    Detected,
    /// Container opened and metadata parsed
    Opened,
    /// Integrity verified (hashes checked)
    Verified,
    /// Contents extracted to output
    Extracted,
    /// Error state
    Error,
}

// =============================================================================
// CONTAINER METADATA
// =============================================================================

/// Basic information about a container format
#[derive(Debug, Clone, Serialize)]
pub struct FormatInfo {
    /// Unique identifier (e.g., "e01", "ad1")
    pub id: &'static str,
    /// Display name (e.g., "Expert Witness Format")
    pub name: &'static str,
    /// File extensions (lowercase, no dot)
    pub extensions: &'static [&'static str],
    /// Format category
    pub category: FormatCategory,
    /// Whether format supports segmentation
    pub supports_segments: bool,
    /// Whether format stores hashes internally
    pub stores_hashes: bool,
    /// Whether format contains file/folder tree
    pub has_file_tree: bool,
}

/// Segment information for multi-segment containers
#[derive(Debug, Clone, Serialize)]
pub struct SegmentInfo {
    /// Total number of segments
    pub count: u32,
    /// Paths to all segment files
    pub files: Vec<String>,
    /// Size of each segment in bytes
    pub sizes: Vec<u64>,
    /// Total size of all segments
    pub total_size: u64,
    /// Missing segment files (gaps)
    pub missing: Vec<String>,
}

/// Hash verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct HashResult {
    /// Hash algorithm used
    pub algorithm: String,
    /// Computed hash value
    pub computed: String,
    /// Expected hash value (if stored)
    pub expected: Option<String>,
    /// Whether hashes match
    pub verified: Option<bool>,
    /// Time taken to compute (seconds)
    pub duration_secs: f64,
}

impl HashResult {
    /// Create a new HashResult
    #[inline]
    pub fn new(algorithm: impl Into<String>, computed: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            computed: computed.into(),
            ..Default::default()
        }
    }

    /// Set expected hash value
    #[inline]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set verification result
    #[inline]
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = Some(verified);
        self
    }

    /// Set duration
    #[inline]
    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Mark as verified (hashes match)
    #[inline]
    pub fn verified(algorithm: impl Into<String>, hash: impl Into<String>, duration: f64) -> Self {
        let hash_str = hash.into();
        Self {
            algorithm: algorithm.into(),
            computed: hash_str.clone(),
            expected: Some(hash_str),
            verified: Some(true),
            duration_secs: duration,
        }
    }

    /// Mark as mismatch
    #[inline]
    pub fn mismatch(algorithm: impl Into<String>, computed: impl Into<String>, expected: impl Into<String>, duration: f64) -> Self {
        Self {
            algorithm: algorithm.into(),
            computed: computed.into(),
            expected: Some(expected.into()),
            verified: Some(false),
            duration_secs: duration,
        }
    }
}

/// Container verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct VerifyResult {
    /// Overall verification status
    pub status: VerifyStatus,
    /// Hash verification results
    pub hashes: Vec<HashResult>,
    /// Per-chunk/block verification (if applicable)
    pub chunks: Vec<ChunkVerifyResult>,
    /// Verification messages/warnings
    pub messages: Vec<String>,
}

impl VerifyResult {
    /// Create a new VerifyResult with a status
    #[inline]
    pub fn new(status: VerifyStatus) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }

    /// Create a verified result
    #[inline]
    pub fn verified() -> Self {
        Self::new(VerifyStatus::Verified)
    }

    /// Create a computed (no comparison) result
    #[inline]
    pub fn computed() -> Self {
        Self::new(VerifyStatus::Computed)
    }

    /// Create a mismatch result
    #[inline]
    pub fn mismatched() -> Self {
        Self::new(VerifyStatus::Mismatch)
    }

    /// Add a hash result
    #[inline]
    pub fn with_hash(mut self, hash: HashResult) -> Self {
        self.hashes.push(hash);
        self
    }

    /// Add a chunk verification result
    #[inline]
    pub fn with_chunk(mut self, chunk: ChunkVerifyResult) -> Self {
        self.chunks.push(chunk);
        self
    }

    /// Add a message
    #[inline]
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum VerifyStatus {
    /// All hashes verified successfully
    Verified,
    /// Computed hash (no stored hash to compare)
    #[default]
    Computed,
    /// Hash mismatch detected
    Mismatch,
    /// Verification not supported for this format
    NotSupported,
    /// Verification failed due to error
    Error,
}

/// Per-chunk verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChunkVerifyResult {
    /// Chunk index
    pub index: usize,
    /// Status ("ok", "mismatch", "error")
    pub status: String,
    /// Error message if any
    pub message: Option<String>,
}

impl ChunkVerifyResult {
    /// Create a successful chunk result
    #[inline]
    pub fn ok(index: usize) -> Self {
        Self {
            index,
            status: "ok".to_string(),
            message: None,
        }
    }

    /// Create a mismatch chunk result
    #[inline]
    pub fn mismatch(index: usize) -> Self {
        Self {
            index,
            status: "mismatch".to_string(),
            message: None,
        }
    }

    /// Create an error chunk result
    #[inline]
    pub fn error(index: usize, msg: impl Into<String>) -> Self {
        Self {
            index,
            status: "error".to_string(),
            message: Some(msg.into()),
        }
    }

    /// Add a message
    #[inline]
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

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

/// Generic container metadata (returned by info())
#[derive(Debug, Clone, Serialize, Default)]
pub struct ContainerMetadata {
    /// Format identifier
    pub format: String,
    /// Format version (if applicable)
    pub version: Option<String>,
    /// Total logical size of data
    pub total_size: u64,
    /// Segment information
    pub segments: Option<SegmentInfo>,
    /// Stored hashes
    pub stored_hashes: Vec<StoredHashInfo>,
    /// Case/evidence metadata
    pub case_info: Option<CaseMetadata>,
    /// Additional format-specific data (serialized as JSON)
    pub format_specific: Option<serde_json::Value>,
}

impl ContainerMetadata {
    /// Create new ContainerMetadata with format
    #[inline]
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            ..Default::default()
        }
    }

    /// Set format version
    #[inline]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set total size
    #[inline]
    pub fn with_size(mut self, size: u64) -> Self {
        self.total_size = size;
        self
    }

    /// Set segment info
    #[inline]
    pub fn with_segments(mut self, segments: SegmentInfo) -> Self {
        self.segments = Some(segments);
        self
    }

    /// Add a stored hash
    #[inline]
    pub fn add_hash(mut self, hash: StoredHashInfo) -> Self {
        self.stored_hashes.push(hash);
        self
    }

    /// Set case metadata
    #[inline]
    pub fn with_case_info(mut self, case_info: CaseMetadata) -> Self {
        self.case_info = Some(case_info);
        self
    }

    /// Set format-specific data
    #[inline]
    pub fn with_format_data(mut self, data: serde_json::Value) -> Self {
        self.format_specific = Some(data);
        self
    }
}

/// Stored hash information
#[derive(Debug, Clone, Serialize, Default)]
pub struct StoredHashInfo {
    /// Algorithm (e.g., "MD5", "SHA1")
    pub algorithm: String,
    /// Hash value (hex string)
    pub hash: String,
    /// Hash source ("container", "companion", "computed")
    pub source: String,
    /// Verification status
    pub verified: Option<bool>,
}

impl StoredHashInfo {
    /// Create new StoredHashInfo
    #[inline]
    pub fn new(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            hash: hash.into(),
            source: "container".to_string(),
            verified: None,
        }
    }

    /// Set source type
    #[inline]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Create from companion file
    #[inline]
    pub fn from_companion(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(algorithm, hash).with_source("companion")
    }

    /// Create from computed value
    #[inline]
    pub fn from_computed(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(algorithm, hash).with_source("computed")
    }

    /// Mark as verified
    #[inline]
    pub fn verified(mut self, is_verified: bool) -> Self {
        self.verified = Some(is_verified);
        self
    }
}

/// Case metadata from container
#[derive(Debug, Clone, Serialize, Default)]
pub struct CaseMetadata {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquisition_date: Option<String>,
}

impl CaseMetadata {
    /// Create empty CaseMetadata
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case number
    #[inline]
    pub fn with_case_number(mut self, case_number: impl Into<String>) -> Self {
        self.case_number = Some(case_number.into());
        self
    }

    /// Set evidence number
    #[inline]
    pub fn with_evidence_number(mut self, evidence_number: impl Into<String>) -> Self {
        self.evidence_number = Some(evidence_number.into());
        self
    }

    /// Set examiner name
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner_name = Some(examiner.into());
        self
    }

    /// Set description
    #[inline]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set acquisition date
    #[inline]
    pub fn with_acquisition_date(mut self, date: impl Into<String>) -> Self {
        self.acquisition_date = Some(date.into());
        self
    }

    /// Check if any case info is present
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.case_number.is_none()
            && self.evidence_number.is_none()
            && self.examiner_name.is_none()
            && self.description.is_none()
            && self.notes.is_none()
            && self.acquisition_date.is_none()
    }
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

/// Metadata for a single segment
#[derive(Debug, Clone, Serialize)]
pub struct SegmentMetadata {
    pub index: u32,
    pub path: String,
    pub size: u64,
    pub hash: Option<String>,
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

/// Tree entry information
#[derive(Debug, Clone, Serialize, Default)]
pub struct TreeEntryInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub hash: Option<String>,
}

impl TreeEntryInfo {
    /// Create new TreeEntryInfo for a file
    #[inline]
    pub fn file(path: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_directory: false,
            size,
            ..Default::default()
        }
    }

    /// Create new TreeEntryInfo for a directory
    #[inline]
    pub fn directory(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_directory: true,
            size: 0,
            ..Default::default()
        }
    }

    /// Set created timestamp
    #[inline]
    pub fn with_created(mut self, created: impl Into<String>) -> Self {
        self.created = Some(created.into());
        self
    }

    /// Set modified timestamp
    #[inline]
    pub fn with_modified(mut self, modified: impl Into<String>) -> Self {
        self.modified = Some(modified.into());
        self
    }

    /// Set accessed timestamp
    #[inline]
    pub fn with_accessed(mut self, accessed: impl Into<String>) -> Self {
        self.accessed = Some(accessed.into());
        self
    }

    /// Set all timestamps at once
    #[inline]
    pub fn with_timestamps(
        mut self,
        created: Option<String>,
        modified: Option<String>,
        accessed: Option<String>,
    ) -> Self {
        self.created = created;
        self.modified = modified;
        self.accessed = accessed;
        self
    }

    /// Set hash value
    #[inline]
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }
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

use crate::common::vfs::{VirtualFileSystem, VfsError};

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

impl From<VfsError> for ContainerError {
    fn from(err: VfsError) -> Self {
        match err {
            VfsError::NotFound(path) => ContainerError::FileNotFound(path),
            VfsError::IoError(e) => ContainerError::IoError(e.to_string()),
            VfsError::InvalidPath(path) => ContainerError::ParseError(format!("Invalid path: {}", path)),
            VfsError::Internal(msg) => ContainerError::IoError(msg),
            VfsError::NotADirectory(path) => ContainerError::ParseError(format!("Not a directory: {}", path)),
            VfsError::NotAFile(path) => ContainerError::ParseError(format!("Not a file: {}", path)),
            VfsError::NotMounted => ContainerError::UnsupportedOperation("Container not mounted".to_string()),
            VfsError::PermissionDenied(path) => ContainerError::IoError(format!("Permission denied: {}", path)),
            VfsError::OutOfBounds { offset, size } => {
                ContainerError::IoError(format!("Read out of bounds: offset={}, size={}", offset, size))
            }
        }
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ContainerError variants & Display
    // =========================================================================

    #[test]
    fn test_container_error_display_messages() {
        assert_eq!(
            ContainerError::FileNotFound("test.ad1".into()).to_string(),
            "File not found: test.ad1"
        );
        assert_eq!(
            ContainerError::InvalidFormat("bad header".into()).to_string(),
            "Invalid format: bad header"
        );
        assert_eq!(
            ContainerError::UnsupportedOperation("mount".into()).to_string(),
            "Unsupported operation: mount"
        );
        assert_eq!(
            ContainerError::IoError("disk full".into()).to_string(),
            "I/O error: disk full"
        );
        assert_eq!(
            ContainerError::ParseError("offset".into()).to_string(),
            "Parse error: offset"
        );
        assert_eq!(
            ContainerError::SegmentError("missing .E02".into()).to_string(),
            "Segment error: missing .E02"
        );
        assert_eq!(
            ContainerError::ExtractionError("write fail".into()).to_string(),
            "Extraction error: write fail"
        );
        assert_eq!(
            ContainerError::CompressionError("zlib".into()).to_string(),
            "Compression error: zlib"
        );
        assert_eq!(
            ContainerError::VerificationError("crc".into()).to_string(),
            "Verification error: crc"
        );
        assert_eq!(
            ContainerError::ConfigError("param".into()).to_string(),
            "Configuration error: param"
        );
        assert_eq!(
            ContainerError::SearchError("regex".into()).to_string(),
            "Search error: regex"
        );
        assert_eq!(
            ContainerError::SerializationError("json".into()).to_string(),
            "Serialization error: json"
        );
        assert_eq!(
            ContainerError::EntryNotFound("/path".into()).to_string(),
            "Entry not found: /path"
        );
        assert_eq!(
            ContainerError::InternalError("panic".into()).to_string(),
            "Internal error: panic"
        );
    }

    #[test]
    fn test_container_error_hash_mismatch_display() {
        let err = ContainerError::HashMismatch {
            expected: "abc123".into(),
            actual: "def456".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

    // =========================================================================
    // ContainerError From implementations
    // =========================================================================

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ContainerError::from(io_err);
        match err {
            ContainerError::IoError(msg) => assert!(msg.contains("file missing")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_from_string() {
        let err = ContainerError::from("something broke".to_string());
        match err {
            ContainerError::InternalError(msg) => assert_eq!(msg, "something broke"),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_from_str() {
        let err = ContainerError::from("literal error");
        match err {
            ContainerError::InternalError(msg) => assert_eq!(msg, "literal error"),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_into_string() {
        let err = ContainerError::FileNotFound("test.e01".into());
        let s: String = err.into();
        assert!(s.contains("test.e01"));
    }

    #[test]
    fn test_from_utf8_error() {
        let bytes = vec![0xff, 0xfe];
        let err = String::from_utf8(bytes).unwrap_err();
        let container_err = ContainerError::from(err);
        match container_err {
            ContainerError::ParseError(msg) => assert!(msg.contains("invalid utf-8")),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_from_parse_int_error() {
        let err = "not_a_number".parse::<i32>().unwrap_err();
        let container_err = ContainerError::from(err);
        match container_err {
            ContainerError::ParseError(_) => {}
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let err = serde_json::from_str::<serde_json::Value>("{{bad json").unwrap_err();
        let container_err = ContainerError::from(err);
        match container_err {
            ContainerError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    // =========================================================================
    // LifecycleStage
    // =========================================================================

    #[test]
    fn test_lifecycle_stage_display() {
        assert_eq!(LifecycleStage::Discovered.to_string(), "discovered");
        assert_eq!(LifecycleStage::Detected.to_string(), "detected");
        assert_eq!(LifecycleStage::Opened.to_string(), "opened");
        assert_eq!(LifecycleStage::Verified.to_string(), "verified");
        assert_eq!(LifecycleStage::Extracted.to_string(), "extracted");
        assert_eq!(LifecycleStage::Error.to_string(), "error");
    }

    #[test]
    fn test_lifecycle_stage_from_string() {
        use std::str::FromStr;
        assert_eq!(LifecycleStage::from_str("discovered").unwrap(), LifecycleStage::Discovered);
        assert_eq!(LifecycleStage::from_str("verified").unwrap(), LifecycleStage::Verified);
        assert_eq!(LifecycleStage::from_str("error").unwrap(), LifecycleStage::Error);
    }

    #[test]
    fn test_lifecycle_stage_as_ref() {
        assert_eq!(LifecycleStage::Discovered.as_ref(), "discovered");
        assert_eq!(LifecycleStage::Extracted.as_ref(), "extracted");
    }

    #[test]
    fn test_lifecycle_stage_equality() {
        assert_eq!(LifecycleStage::Verified, LifecycleStage::Verified);
        assert_ne!(LifecycleStage::Discovered, LifecycleStage::Detected);
    }

    #[test]
    fn test_lifecycle_stage_serialization() {
        let json = serde_json::to_string(&LifecycleStage::Opened).unwrap();
        assert!(json.contains("Opened"));
    }

    // =========================================================================
    // HashResult
    // =========================================================================

    #[test]
    fn test_hash_result_new() {
        let hr = HashResult::new("SHA-256", "abcdef1234567890");
        assert_eq!(hr.algorithm, "SHA-256");
        assert_eq!(hr.computed, "abcdef1234567890");
        assert!(hr.expected.is_none());
        assert!(hr.verified.is_none());
        assert_eq!(hr.duration_secs, 0.0);
    }

    #[test]
    fn test_hash_result_builder_chain() {
        let hr = HashResult::new("MD5", "abc")
            .with_expected("abc")
            .with_verified(true)
            .with_duration(1.5);

        assert_eq!(hr.expected.unwrap(), "abc");
        assert_eq!(hr.verified, Some(true));
        assert_eq!(hr.duration_secs, 1.5);
    }

    #[test]
    fn test_hash_result_verified_constructor() {
        let hr = HashResult::verified("SHA-1", "deadbeef", 2.0);
        assert_eq!(hr.algorithm, "SHA-1");
        assert_eq!(hr.computed, "deadbeef");
        assert_eq!(hr.expected.unwrap(), "deadbeef");
        assert_eq!(hr.verified, Some(true));
        assert_eq!(hr.duration_secs, 2.0);
    }

    #[test]
    fn test_hash_result_mismatch_constructor() {
        let hr = HashResult::mismatch("MD5", "aaa", "bbb", 0.5);
        assert_eq!(hr.computed, "aaa");
        assert_eq!(hr.expected.unwrap(), "bbb");
        assert_eq!(hr.verified, Some(false));
    }

    // =========================================================================
    // VerifyResult
    // =========================================================================

    #[test]
    fn test_verify_result_constructors() {
        assert_eq!(VerifyResult::verified().status, VerifyStatus::Verified);
        assert_eq!(VerifyResult::computed().status, VerifyStatus::Computed);
        assert_eq!(VerifyResult::mismatched().status, VerifyStatus::Mismatch);
    }

    #[test]
    fn test_verify_result_default_status() {
        let vr = VerifyResult::default();
        assert_eq!(vr.status, VerifyStatus::Computed); // Computed is #[default]
    }

    #[test]
    fn test_verify_result_builder() {
        let vr = VerifyResult::verified()
            .with_hash(HashResult::new("SHA-256", "abc"))
            .with_chunk(ChunkVerifyResult::ok(0))
            .with_message("All chunks verified");

        assert_eq!(vr.hashes.len(), 1);
        assert_eq!(vr.chunks.len(), 1);
        assert_eq!(vr.messages.len(), 1);
        assert_eq!(vr.messages[0], "All chunks verified");
    }

    #[test]
    fn test_verify_status_equality() {
        assert_eq!(VerifyStatus::Verified, VerifyStatus::Verified);
        assert_ne!(VerifyStatus::Verified, VerifyStatus::Mismatch);
        assert_ne!(VerifyStatus::NotSupported, VerifyStatus::Error);
    }

    // =========================================================================
    // ChunkVerifyResult
    // =========================================================================

    #[test]
    fn test_chunk_verify_result_ok() {
        let chunk = ChunkVerifyResult::ok(0);
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.status, "ok");
        assert!(chunk.message.is_none());
    }

    #[test]
    fn test_chunk_verify_result_mismatch() {
        let chunk = ChunkVerifyResult::mismatch(5);
        assert_eq!(chunk.index, 5);
        assert_eq!(chunk.status, "mismatch");
    }

    #[test]
    fn test_chunk_verify_result_error() {
        let chunk = ChunkVerifyResult::error(10, "CRC failed");
        assert_eq!(chunk.index, 10);
        assert_eq!(chunk.status, "error");
        assert_eq!(chunk.message.unwrap(), "CRC failed");
    }

    #[test]
    fn test_chunk_verify_result_with_message() {
        let chunk = ChunkVerifyResult::ok(0).with_message("sector 0");
        assert_eq!(chunk.message.unwrap(), "sector 0");
    }

    // =========================================================================
    // ContainerMetadata
    // =========================================================================

    #[test]
    fn test_container_metadata_new() {
        let meta = ContainerMetadata::new("e01");
        assert_eq!(meta.format, "e01");
        assert_eq!(meta.total_size, 0);
        assert!(meta.version.is_none());
        assert!(meta.segments.is_none());
    }

    #[test]
    fn test_container_metadata_builder_chain() {
        let meta = ContainerMetadata::new("ad1")
            .with_version("3.0")
            .with_size(1024 * 1024)
            .with_case_info(CaseMetadata::new().with_case_number("CASE-1"))
            .with_format_data(serde_json::json!({"compression": "zlib"}));

        assert_eq!(meta.version.unwrap(), "3.0");
        assert_eq!(meta.total_size, 1024 * 1024);
        assert!(meta.case_info.is_some());
        assert!(meta.format_specific.is_some());
    }

    #[test]
    fn test_container_metadata_add_hash() {
        let meta = ContainerMetadata::new("e01")
            .add_hash(StoredHashInfo::new("MD5", "abc123"))
            .add_hash(StoredHashInfo::new("SHA-1", "def456"));

        assert_eq!(meta.stored_hashes.len(), 2);
    }

    // =========================================================================
    // StoredHashInfo
    // =========================================================================

    #[test]
    fn test_stored_hash_info_new() {
        let h = StoredHashInfo::new("MD5", "abcdef");
        assert_eq!(h.algorithm, "MD5");
        assert_eq!(h.hash, "abcdef");
        assert_eq!(h.source, "container");
        assert!(h.verified.is_none());
    }

    #[test]
    fn test_stored_hash_info_from_companion() {
        let h = StoredHashInfo::from_companion("SHA-256", "deadbeef");
        assert_eq!(h.source, "companion");
    }

    #[test]
    fn test_stored_hash_info_from_computed() {
        let h = StoredHashInfo::from_computed("BLAKE3", "cafebabe");
        assert_eq!(h.source, "computed");
    }

    #[test]
    fn test_stored_hash_info_verified() {
        let h = StoredHashInfo::new("MD5", "abc").verified(true);
        assert_eq!(h.verified, Some(true));

        let h2 = StoredHashInfo::new("MD5", "abc").verified(false);
        assert_eq!(h2.verified, Some(false));
    }

    // =========================================================================
    // CaseMetadata
    // =========================================================================

    #[test]
    fn test_case_metadata_new_is_empty() {
        let meta = CaseMetadata::new();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_case_metadata_builder_chain() {
        let meta = CaseMetadata::new()
            .with_case_number("C-100")
            .with_evidence_number("E-001")
            .with_examiner("Dr. Jones")
            .with_description("Test case")
            .with_notes("Important notes")
            .with_acquisition_date("2026-01-15");

        assert_eq!(meta.case_number.as_deref(), Some("C-100"));
        assert_eq!(meta.evidence_number.as_deref(), Some("E-001"));
        assert_eq!(meta.examiner_name.as_deref(), Some("Dr. Jones"));
        assert_eq!(meta.description.as_deref(), Some("Test case"));
        assert_eq!(meta.notes.as_deref(), Some("Important notes"));
        assert_eq!(meta.acquisition_date.as_deref(), Some("2026-01-15"));
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_case_metadata_is_empty_with_one_field() {
        let meta = CaseMetadata::new().with_case_number("X");
        assert!(!meta.is_empty());
    }

    // =========================================================================
    // TreeEntryInfo
    // =========================================================================

    #[test]
    fn test_tree_entry_info_file() {
        let entry = TreeEntryInfo::file("/docs/readme.txt", "readme.txt", 1024);
        assert_eq!(entry.path, "/docs/readme.txt");
        assert_eq!(entry.name, "readme.txt");
        assert!(!entry.is_directory);
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_tree_entry_info_directory() {
        let entry = TreeEntryInfo::directory("/docs", "docs");
        assert_eq!(entry.path, "/docs");
        assert!(entry.is_directory);
        assert_eq!(entry.size, 0);
    }

    #[test]
    fn test_tree_entry_info_with_timestamps() {
        let entry = TreeEntryInfo::file("/a.txt", "a.txt", 10)
            .with_created("2026-01-01T00:00:00Z")
            .with_modified("2026-01-02T00:00:00Z")
            .with_accessed("2026-01-03T00:00:00Z");

        assert_eq!(entry.created.unwrap(), "2026-01-01T00:00:00Z");
        assert_eq!(entry.modified.unwrap(), "2026-01-02T00:00:00Z");
        assert_eq!(entry.accessed.unwrap(), "2026-01-03T00:00:00Z");
    }

    #[test]
    fn test_tree_entry_info_with_timestamps_batch() {
        let entry = TreeEntryInfo::file("/b.txt", "b.txt", 20)
            .with_timestamps(
                Some("2026-01-01".into()),
                Some("2026-01-02".into()),
                None,
            );

        assert!(entry.created.is_some());
        assert!(entry.modified.is_some());
        assert!(entry.accessed.is_none());
    }

    #[test]
    fn test_tree_entry_info_with_hash() {
        let entry = TreeEntryInfo::file("/c.dat", "c.dat", 100)
            .with_hash("abcdef1234567890");
        assert_eq!(entry.hash.unwrap(), "abcdef1234567890");
    }

    // =========================================================================
    // VfsError → ContainerError conversion
    // =========================================================================

    #[test]
    fn test_vfs_error_not_found() {
        let vfs_err = VfsError::NotFound("/missing".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::FileNotFound(p) => assert_eq!(p, "/missing"),
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_vfs_error_not_a_directory() {
        let vfs_err = VfsError::NotADirectory("/file.txt".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::ParseError(msg) => assert!(msg.contains("Not a directory")),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_vfs_error_not_a_file() {
        let vfs_err = VfsError::NotAFile("/dir".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::ParseError(msg) => assert!(msg.contains("Not a file")),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_vfs_error_not_mounted() {
        let vfs_err = VfsError::NotMounted;
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::UnsupportedOperation(msg) => assert!(msg.contains("not mounted")),
            _ => panic!("Expected UnsupportedOperation"),
        }
    }

    #[test]
    fn test_vfs_error_permission_denied() {
        let vfs_err = VfsError::PermissionDenied("/secret".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::IoError(msg) => assert!(msg.contains("Permission denied")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_vfs_error_out_of_bounds() {
        let vfs_err = VfsError::OutOfBounds { offset: 100, size: 50 };
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::IoError(msg) => {
                assert!(msg.contains("100"));
                assert!(msg.contains("50"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_vfs_error_invalid_path() {
        let vfs_err = VfsError::InvalidPath("../../../etc/passwd".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::ParseError(msg) => assert!(msg.contains("Invalid path")),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_vfs_error_internal() {
        let vfs_err = VfsError::Internal("unexpected state".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::IoError(msg) => assert_eq!(msg, "unexpected state"),
            _ => panic!("Expected IoError"),
        }
    }

    // =========================================================================
    // FormatInfo
    // =========================================================================

    #[test]
    fn test_format_info_fields() {
        let info = FormatInfo {
            id: "e01",
            name: "Expert Witness Format",
            extensions: &["e01", "ex01"],
            category: FormatCategory::ForensicContainer,
            supports_segments: true,
            stores_hashes: true,
            has_file_tree: false,
        };
        assert_eq!(info.id, "e01");
        assert!(info.supports_segments);
        assert!(info.stores_hashes);
        assert!(!info.has_file_tree);
    }

    // =========================================================================
    // SegmentInfo
    // =========================================================================

    #[test]
    fn test_segment_info() {
        let seg = SegmentInfo {
            count: 3,
            files: vec!["a.E01".into(), "a.E02".into(), "a.E03".into()],
            sizes: vec![100, 200, 150],
            total_size: 450,
            missing: vec![],
        };
        assert_eq!(seg.count, 3);
        assert_eq!(seg.total_size, 450);
        assert!(seg.missing.is_empty());
    }

    // =========================================================================
    // SegmentMetadata
    // =========================================================================

    #[test]
    fn test_segment_metadata() {
        let sm = SegmentMetadata {
            index: 0,
            path: "/data/image.E01".into(),
            size: 1_000_000,
            hash: Some("abc123".into()),
        };
        assert_eq!(sm.index, 0);
        assert_eq!(sm.size, 1_000_000);
        assert!(sm.hash.is_some());
    }
}
