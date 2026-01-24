// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Document handling error types

use std::io;
use thiserror::Error;

/// Result type for document operations
pub type DocumentResult<T> = Result<T, DocumentError>;

/// Errors that can occur during document operations
#[derive(Debug, Error)]
pub enum DocumentError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// PDF-specific error
    #[error("PDF error: {0}")]
    Pdf(String),

    /// DOCX-specific error
    #[error("DOCX error: {0}")]
    Docx(String),

    /// HTML parsing error
    #[error("HTML error: {0}")]
    Html(String),

    /// Markdown parsing error
    #[error("Markdown error: {0}")]
    Markdown(String),

    /// Unsupported format
    #[error("Unsupported document format: {0}")]
    UnsupportedFormat(String),

    /// Invalid document structure
    #[error("Invalid document: {0}")]
    InvalidDocument(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Font error
    #[error("Font error: {0}")]
    Font(String),

    /// Password protected document
    #[error("Document is password protected")]
    PasswordProtected,

    /// Corrupted document
    #[error("Document appears to be corrupted: {0}")]
    Corrupted(String),

    /// Generic parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<std::string::FromUtf8Error> for DocumentError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Encoding(err.to_string())
    }
}
