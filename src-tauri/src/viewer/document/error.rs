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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn io_error_display() {
        let err = DocumentError::Io(io::Error::new(io::ErrorKind::NotFound, "file missing"));
        assert!(err.to_string().contains("I/O error"));
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn pdf_error_display() {
        let err = DocumentError::Pdf("bad header".to_string());
        assert_eq!(err.to_string(), "PDF error: bad header");
    }

    #[test]
    fn docx_error_display() {
        let err = DocumentError::Docx("corrupt zip".to_string());
        assert_eq!(err.to_string(), "DOCX error: corrupt zip");
    }

    #[test]
    fn html_error_display() {
        let err = DocumentError::Html("malformed".to_string());
        assert_eq!(err.to_string(), "HTML error: malformed");
    }

    #[test]
    fn markdown_error_display() {
        let err = DocumentError::Markdown("parse fail".to_string());
        assert_eq!(err.to_string(), "Markdown error: parse fail");
    }

    #[test]
    fn unsupported_format_display() {
        let err = DocumentError::UnsupportedFormat("rtf".to_string());
        assert_eq!(err.to_string(), "Unsupported document format: rtf");
    }

    #[test]
    fn invalid_document_display() {
        let err = DocumentError::InvalidDocument("no header".to_string());
        assert_eq!(err.to_string(), "Invalid document: no header");
    }

    #[test]
    fn encoding_error_display() {
        let err = DocumentError::Encoding("invalid utf-8".to_string());
        assert_eq!(err.to_string(), "Encoding error: invalid utf-8");
    }

    #[test]
    fn font_error_display() {
        let err = DocumentError::Font("missing font".to_string());
        assert_eq!(err.to_string(), "Font error: missing font");
    }

    #[test]
    fn password_protected_display() {
        let err = DocumentError::PasswordProtected;
        assert_eq!(err.to_string(), "Document is password protected");
    }

    #[test]
    fn corrupted_display() {
        let err = DocumentError::Corrupted("bad magic".to_string());
        assert_eq!(
            err.to_string(),
            "Document appears to be corrupted: bad magic"
        );
    }

    #[test]
    fn parse_error_display() {
        let err = DocumentError::Parse("syntax error".to_string());
        assert_eq!(err.to_string(), "Parse error: syntax error");
    }

    #[test]
    fn not_found_display() {
        let err = DocumentError::NotFound("resource.png".to_string());
        assert_eq!(err.to_string(), "Not found: resource.png");
    }

    #[test]
    fn from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "no access");
        let doc_err: DocumentError = io_err.into();
        assert!(matches!(doc_err, DocumentError::Io(_)));
    }

    #[test]
    fn from_utf8_error() {
        let bad_bytes = vec![0xFF, 0xFE];
        let utf8_err = String::from_utf8(bad_bytes).unwrap_err();
        let doc_err: DocumentError = utf8_err.into();
        assert!(matches!(doc_err, DocumentError::Encoding(_)));
    }
}
