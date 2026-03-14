//! Shared error types for CORE-FFX forensic modules.
//!
//! This crate provides `ContainerError` and `VfsError`, the two core
//! error types used across forensic container operations and the virtual
//! filesystem layer.
//!
//! ## Feature Flags
//!
//! Optional `From` implementations for third-party error types are gated
//! behind features to keep the dependency footprint minimal when not needed:
//!
//! - `serde_json` — `From<serde_json::Error> for ContainerError`
//! - `regex` — `From<regex::Error> for ContainerError`
//! - `rusqlite` — `From<rusqlite::Error> for ContainerError`
//! - `zip` — `From<zip::result::ZipError> for ContainerError`
//! - `all` — enables all of the above

use thiserror::Error;

// =============================================================================
// VfsError
// =============================================================================

/// Virtual filesystem errors.
///
/// Used by all VFS implementations (EWF, Raw, NTFS, FAT, etc.) to report
/// read-only filesystem access failures.
#[derive(Debug, Clone, Error)]
pub enum VfsError {
    /// Entry not found at path
    #[error("Not found: {0}")]
    NotFound(String),
    /// Path is not a directory
    #[error("Not a directory: {0}")]
    NotADirectory(String),
    /// Path is not a file
    #[error("Not a file: {0}")]
    NotAFile(String),
    /// Permission denied (shouldn't happen in read-only)
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
    /// Invalid path (traversal attempt, invalid characters)
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    /// Container not mounted
    #[error("Container not mounted")]
    NotMounted,
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    /// Read out of bounds
    #[error("Read out of bounds: offset={offset}, size={size}")]
    OutOfBounds { offset: u64, size: usize },
}

impl From<std::io::Error> for VfsError {
    fn from(err: std::io::Error) -> Self {
        VfsError::IoError(err.to_string())
    }
}

// =============================================================================
// ContainerError
// =============================================================================

/// Unified error type for all forensic container operations.
///
/// Provides structured, contextual error information for failures during
/// container parsing, verification, and extraction. Each variant captures
/// relevant context (file paths, offsets, etc.) to aid in debugging and
/// user-facing error messages.
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

// =============================================================================
// From implementations — standard library types (always available)
// =============================================================================

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
// From implementations — feature-gated third-party types
// =============================================================================

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for ContainerError {
    fn from(err: serde_json::Error) -> Self {
        ContainerError::SerializationError(err.to_string())
    }
}

#[cfg(feature = "regex")]
impl From<regex::Error> for ContainerError {
    fn from(err: regex::Error) -> Self {
        ContainerError::ParseError(format!("Regex error: {}", err))
    }
}

#[cfg(feature = "rusqlite")]
impl From<rusqlite::Error> for ContainerError {
    fn from(err: rusqlite::Error) -> Self {
        ContainerError::ParseError(format!("Database error: {}", err))
    }
}

#[cfg(feature = "zip")]
impl From<zip::result::ZipError> for ContainerError {
    fn from(err: zip::result::ZipError) -> Self {
        ContainerError::ParseError(format!("ZIP error: {}", err))
    }
}

// =============================================================================
// VfsError → ContainerError conversion
// =============================================================================

impl From<VfsError> for ContainerError {
    fn from(err: VfsError) -> Self {
        match err {
            VfsError::NotFound(path) => ContainerError::FileNotFound(path),
            VfsError::IoError(e) => ContainerError::IoError(e),
            VfsError::InvalidPath(path) => {
                ContainerError::ParseError(format!("Invalid path: {}", path))
            }
            VfsError::Internal(msg) => ContainerError::IoError(msg),
            VfsError::NotADirectory(path) => {
                ContainerError::ParseError(format!("Not a directory: {}", path))
            }
            VfsError::NotAFile(path) => {
                ContainerError::ParseError(format!("Not a file: {}", path))
            }
            VfsError::NotMounted => {
                ContainerError::UnsupportedOperation("Container not mounted".to_string())
            }
            VfsError::PermissionDenied(path) => {
                ContainerError::IoError(format!("Permission denied: {}", path))
            }
            VfsError::OutOfBounds { offset, size } => ContainerError::IoError(format!(
                "Read out of bounds: offset={}, size={}",
                offset, size
            )),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
            ContainerError::IoError("disk full".into()).to_string(),
            "I/O error: disk full"
        );
        assert_eq!(
            ContainerError::ParseError("offset".into()).to_string(),
            "Parse error: offset"
        );
    }

    #[test]
    fn test_container_error_hash_mismatch() {
        let err = ContainerError::HashMismatch {
            expected: "abc123".into(),
            actual: "def456".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("abc123"));
        assert!(msg.contains("def456"));
    }

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
    fn test_vfs_error_not_found() {
        let vfs_err = VfsError::NotFound("/missing".into());
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::FileNotFound(p) => assert_eq!(p, "/missing"),
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_vfs_error_not_mounted() {
        let vfs_err = VfsError::NotMounted;
        let err: ContainerError = vfs_err.into();
        match err {
            ContainerError::UnsupportedOperation(_) => {}
            _ => panic!("Expected UnsupportedOperation"),
        }
    }

    #[test]
    fn test_vfs_error_out_of_bounds() {
        let vfs_err = VfsError::OutOfBounds {
            offset: 100,
            size: 50,
        };
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
    fn test_vfs_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "disk error");
        let vfs_err = VfsError::from(io_err);
        match vfs_err {
            VfsError::IoError(msg) => assert!(msg.contains("disk error")),
            _ => panic!("Expected VfsError::IoError"),
        }
    }
}
