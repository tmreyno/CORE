//! Error types and result handling
//!
//! Provides safe Rust error types that wrap the C library error codes.

use std::fmt;
use std::error::Error as StdError;
use crate::ffi::SevenZipErrorCode;

/// Result type for 7z operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for 7z operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Failed to open file
    OpenFile(String),
    /// Invalid or corrupted archive
    InvalidArchive(String),
    /// Memory allocation failure
    Memory(String),
    /// Extraction failed
    Extract(String),
    /// Compression failed
    Compress(String),
    /// Invalid parameter
    InvalidParameter(String),
    /// Feature not implemented
    NotImplemented(String),
    /// Unknown or unspecified error
    Unknown(String),
    /// IO error
    Io(String),
    /// Encryption failed
    EncryptionError(String),
    /// Decryption failed (wrong password or corrupted data)
    DecryptionError(String),
}

impl Error {
    /// Convert from C error code to Rust Error
    pub(crate) fn from_code(code: SevenZipErrorCode) -> Self {
        match code {
            SevenZipErrorCode::SEVENZIP_OK => {
                Error::Unknown("Unexpected OK status treated as error".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_OPEN_FILE => {
                Error::OpenFile("Failed to open file".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_INVALID_ARCHIVE => {
                Error::InvalidArchive("Invalid or corrupted archive".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_MEMORY => {
                Error::Memory("Memory allocation failed".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_EXTRACT => {
                Error::Extract("Extraction failed".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_COMPRESS => {
                Error::Compress("Compression failed".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_INVALID_PARAM => {
                Error::InvalidParameter("Invalid parameter".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_NOT_IMPLEMENTED => {
                Error::NotImplemented("Feature not implemented".to_string())
            }
            SevenZipErrorCode::SEVENZIP_ERROR_UNKNOWN => {
                Error::Unknown("Unknown error".to_string())
            }
        }
    }

    /// Create an error with a custom message
    pub fn with_message(self, message: impl Into<String>) -> Self {
        let msg = message.into();
        match self {
            Error::OpenFile(_) => Error::OpenFile(msg),
            Error::InvalidArchive(_) => Error::InvalidArchive(msg),
            Error::Memory(_) => Error::Memory(msg),
            Error::Extract(_) => Error::Extract(msg),
            Error::Compress(_) => Error::Compress(msg),
            Error::InvalidParameter(_) => Error::InvalidParameter(msg),
            Error::NotImplemented(_) => Error::NotImplemented(msg),
            Error::Unknown(_) => Error::Unknown(msg),
            Error::Io(_) => Error::Io(msg),
            Error::EncryptionError(_) => Error::EncryptionError(msg),
            Error::DecryptionError(_) => Error::DecryptionError(msg),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OpenFile(msg) => write!(f, "Failed to open file: {}", msg),
            Error::InvalidArchive(msg) => write!(f, "Invalid archive: {}", msg),
            Error::Memory(msg) => write!(f, "Memory error: {}", msg),
            Error::Extract(msg) => write!(f, "Extraction failed: {}", msg),
            Error::Compress(msg) => write!(f, "Compression failed: {}", msg),
            Error::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            Error::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            Error::Unknown(msg) => write!(f, "Unknown error: {}", msg),
            Error::Io(msg) => write!(f, "IO error: {}", msg),
            Error::EncryptionError(msg) => write!(f, "Encryption failed: {}", msg),
            Error::DecryptionError(msg) => write!(f, "Decryption failed: {}", msg),
        }
    }
}

impl StdError for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Error::InvalidParameter(format!("String contains null byte: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::OpenFile("test.7z".to_string());
        assert_eq!(err.to_string(), "Failed to open file: test.7z");

        let err = Error::InvalidArchive("corrupted".to_string());
        assert_eq!(err.to_string(), "Invalid archive: corrupted");
    }

    #[test]
    fn test_error_from_code() {
        let err = Error::from_code(SevenZipErrorCode::SEVENZIP_ERROR_MEMORY);
        match err {
            Error::Memory(_) => (),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_with_message() {
        let err = Error::Extract("original".to_string());
        let err = err.with_message("new message");
        assert_eq!(err.to_string(), "Extraction failed: new message");
    }
}
