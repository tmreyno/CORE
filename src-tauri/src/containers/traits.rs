// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence Container Traits
//!
//! This module defines the core traits that all evidence container parsers must
//! implement. These traits provide a unified interface for working with different
//! forensic container formats (AD1, E01, L01, Raw, UFED, Archives).
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
use std::fmt;
use serde::Serialize;

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
#[derive(Debug, Clone)]
pub enum ContainerError {
    /// File or directory not found at the specified path.
    ///
    /// Contains the path that was not found. For container operations,
    /// this typically means the primary container file is missing.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::FileNotFound("/path/to/evidence.E01".to_string())
    /// ```
    FileNotFound(String),

    /// Container format is invalid, unrecognized, or corrupted.
    ///
    /// Returned when magic bytes don't match, required headers are
    /// missing, or the format version is unsupported.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::InvalidFormat("Missing EVF signature at offset 0".to_string())
    /// ```
    InvalidFormat(String),

    /// Requested operation is not supported for this container type.
    ///
    /// Some operations (like chunk verification) may not apply to all
    /// container formats (e.g., raw images don't have embedded chunks).
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::UnsupportedOperation("RAW images do not support chunk verification".to_string())
    /// ```
    UnsupportedOperation(String),

    /// I/O error during file operations.
    ///
    /// Wraps underlying filesystem errors like permission denied,
    /// disk full, or unexpected EOF during reads.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::IoError("Failed to read at offset 0x1000: unexpected EOF".to_string())
    /// ```
    IoError(String),

    /// Error parsing container internal structure.
    ///
    /// Returned when container data is syntactically invalid:
    /// malformed headers, invalid offsets, truncated records, etc.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::ParseError("Invalid chunk table at offset 0x5000: expected 1024 entries, found 512".to_string())
    /// ```
    ParseError(String),

    /// Computed hash does not match expected/stored hash.
    ///
    /// Indicates potential data tampering or corruption. Both the
    /// expected and actual hash values are preserved for reporting.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::HashMismatch {
    ///     expected: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
    ///     actual: "098f6bcd4621d373cade4e832627b4f6".to_string(),
    /// }
    /// ```
    HashMismatch {
        /// The expected hash value (from container metadata or companion file)
        expected: String,
        /// The actually computed hash value
        actual: String,
    },

    /// Error related to multi-segment containers.
    ///
    /// Common causes: missing segment files (.E02, .ad2), sequence
    /// gaps, or segment header mismatches.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::SegmentError("Missing segment 3: expected /path/evidence.E03".to_string())
    /// ```
    SegmentError(String),

    /// Error during content extraction to filesystem.
    ///
    /// Covers failures when writing extracted files: permission
    /// denied, disk full, path too long, etc.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::ExtractionError("Failed to create directory: /output/Documents".to_string())
    /// ```
    ExtractionError(String),

    /// Decompression failure for compressed container data.
    ///
    /// E01 and AD1 use zlib compression; this error indicates
    /// corrupted compressed blocks or invalid compression headers.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::CompressionError("zlib inflate failed at chunk 42: invalid block type".to_string())
    /// ```
    CompressionError(String),

    /// General verification failure (not hash-specific).
    ///
    /// Used for structural verification failures like CRC mismatches
    /// on section headers, invalid checksums, etc.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::VerificationError("Section CRC mismatch at offset 0x8000".to_string())
    /// ```
    VerificationError(String),

    /// Invalid configuration or parameter value.
    ///
    /// Returned when user-provided parameters are invalid: unknown
    /// hash algorithm, invalid path patterns, etc.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::ConfigError("Unknown hash algorithm: 'sha512'".to_string())
    /// ```
    ConfigError(String),

    /// Error during search operations within containers.
    ///
    /// Covers regex compilation failures, search timeout, or
    /// other search-specific errors.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::SearchError("Invalid regex pattern: unclosed group".to_string())
    /// ```
    SearchError(String),

    /// Serialization/deserialization error for JSON, CSV, etc.
    ///
    /// Returned when exporting container metadata fails due to
    /// serialization issues (invalid UTF-8, schema mismatches).
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::SerializationError("Failed to serialize tree: invalid UTF-8 in filename".to_string())
    /// ```
    SerializationError(String),

    /// Entry (file/folder) not found within container.
    ///
    /// Distinct from `FileNotFound` - this means the container
    /// exists but the requested internal path doesn't.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::EntryNotFound("/Documents/missing.txt".to_string())
    /// ```
    EntryNotFound(String),

    /// Internal error indicating unexpected program state.
    ///
    /// Used for conditions that "shouldn't happen" - typically
    /// indicates a bug in the parser implementation.
    ///
    /// # Example
    /// ```rust,ignore
    /// ContainerError::InternalError("Chunk index out of bounds: 999 >= 100".to_string())
    /// ```
    InternalError(String),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "File not found: {}", path),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {}, got {}", expected, actual)
            }
            Self::SegmentError(msg) => write!(f, "Segment error: {}", msg),
            Self::ExtractionError(msg) => write!(f, "Extraction error: {}", msg),
            Self::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            Self::VerificationError(msg) => write!(f, "Verification error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::SearchError(msg) => write!(f, "Search error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::EntryNotFound(path) => write!(f, "Entry not found: {}", path),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ContainerError {}

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
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

/// Container verification result
#[derive(Debug, Clone, Serialize)]
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

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VerifyStatus {
    /// All hashes verified successfully
    Verified,
    /// Computed hash (no stored hash to compare)
    Computed,
    /// Hash mismatch detected
    Mismatch,
    /// Verification not supported for this format
    NotSupported,
    /// Verification failed due to error
    Error,
}

/// Per-chunk verification result
#[derive(Debug, Clone, Serialize)]
pub struct ChunkVerifyResult {
    /// Chunk index
    pub index: usize,
    /// Status ("ok", "mismatch", "error")
    pub status: String,
    /// Error message if any
    pub message: Option<String>,
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
#[derive(Debug, Clone, Serialize)]
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

/// Stored hash information
#[derive(Debug, Clone, Serialize)]
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

/// Case metadata from container
#[derive(Debug, Clone, Serialize)]
pub struct CaseMetadata {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquisition_date: Option<String>,
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
#[derive(Debug, Clone, Serialize)]
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

    #[test]
    fn test_lifecycle_stages() {
        assert_eq!(LifecycleStage::Discovered as u8, 0);
    }

    #[test]
    fn test_verify_status_serialization() {
        let status = VerifyStatus::Verified;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Verified"));
    }
}
