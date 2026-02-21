// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Error types for libewf-ffi

use std::fmt;

/// Error type for libewf operations
#[derive(Debug)]
pub enum Error {
    /// libewf C library returned an error
    Libewf(String),
    /// Invalid parameter passed to API
    InvalidParam(String),
    /// I/O error
    Io(std::io::Error),
    /// UTF-8 conversion error
    Utf8(std::str::Utf8Error),
    /// Null pointer encountered
    NullPointer(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Libewf(msg) => write!(f, "libewf error: {}", msg),
            Error::InvalidParam(msg) => write!(f, "invalid parameter: {}", msg),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Utf8(e) => write!(f, "UTF-8 error: {}", e),
            Error::NullPointer(msg) => write!(f, "null pointer: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Utf8(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Extract error message from libewf_error_t pointer and free it
///
/// # Safety
/// The error pointer must be a valid libewf_error_t pointer or null
pub(crate) unsafe fn extract_error(error: *mut crate::ffi::libewf_error_t) -> String {
    if error.is_null() {
        return "unknown libewf error (null error pointer)".to_string();
    }

    let mut buf = vec![0u8; 2048];
    let result = crate::ffi::libewf_error_sprint(
        error,
        buf.as_mut_ptr() as *mut std::os::raw::c_char,
        buf.len(),
    );

    let msg = if result > 0 {
        // Find the null terminator
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        "unknown libewf error (sprint failed)".to_string()
    };

    // Free the error
    let mut err_ptr = error;
    crate::ffi::libewf_error_free(&mut err_ptr);

    msg
}
