// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified error type for all forensic container operations.

use thiserror::Error;

use crate::common::vfs::VfsError;

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

// =============================================================================
// FROM IMPLEMENTATIONS
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
}
