// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Document handling error types

use std::io;
use std::fmt;

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;

/// Errors that can occur during document operations
#[derive(Debug)]
pub enum DocumentError {
    /// I/O error
    Io(io::Error),

    /// PDF-specific error
    Pdf(String),

    /// DOCX-specific error
    Docx(String),

    /// HTML parsing error
    Html(String),

    /// Markdown parsing error
    Markdown(String),

    /// Unsupported format
    UnsupportedFormat(String),

    /// Invalid document structure
    InvalidDocument(String),

    /// Encoding error
    Encoding(String),

    /// Font error
    Font(String),

    /// Password protected document
    PasswordProtected,

    /// Corrupted document
    Corrupted(String),

    /// Generic parse error
    Parse(String),

    /// Resource not found
    NotFound(String),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Pdf(s) => write!(f, "PDF error: {}", s),
            Self::Docx(s) => write!(f, "DOCX error: {}", s),
            Self::Html(s) => write!(f, "HTML error: {}", s),
            Self::Markdown(s) => write!(f, "Markdown error: {}", s),
            Self::UnsupportedFormat(s) => write!(f, "Unsupported document format: {}", s),
            Self::InvalidDocument(s) => write!(f, "Invalid document: {}", s),
            Self::Encoding(s) => write!(f, "Encoding error: {}", s),
            Self::Font(s) => write!(f, "Font error: {}", s),
            Self::PasswordProtected => write!(f, "Document is password protected"),
            Self::Corrupted(s) => write!(f, "Document appears to be corrupted: {}", s),
            Self::Parse(s) => write!(f, "Parse error: {}", s),
            Self::NotFound(s) => write!(f, "Not found: {}", s),
        }
    }
}

impl std::error::Error for DocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DocumentError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for DocumentError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Encoding(err.to_string())
    }
}
