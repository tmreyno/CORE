// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Error types for report generation

use std::fmt;
use std::io;

/// Result type alias for report operations
pub type ReportResult<T> = Result<T, ReportError>;

/// Errors that can occur during report generation
#[derive(Debug)]
pub enum ReportError {
    /// I/O error (file read/write)
    Io(io::Error),
    /// Template rendering error
    Template(tera::Error),
    /// PDF generation error
    Pdf(String),
    /// DOCX generation error
    Docx(String),
    /// Typst generation/compilation error
    #[cfg(feature = "typst-reports")]
    Typst(String),
    /// Serialization error
    Serialization(serde_json::Error),
    /// Unsupported output format
    UnsupportedFormat(String),
    /// AI assistant not configured
    AiNotConfigured,
    /// AI generation error
    #[cfg(feature = "ai-assistant")]
    AiError(String),
    /// Validation error
    Validation(String),
    /// Missing required field
    MissingField(String),
    /// Invalid data
    InvalidData(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::Io(e) => write!(f, "I/O error: {}", e),
            ReportError::Template(e) => write!(f, "Template error: {}", e),
            ReportError::Pdf(e) => write!(f, "PDF generation error: {}", e),
            ReportError::Docx(e) => write!(f, "DOCX generation error: {}", e),
            #[cfg(feature = "typst-reports")]
            ReportError::Typst(e) => write!(f, "Typst generation error: {}", e),
            ReportError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ReportError::UnsupportedFormat(e) => write!(f, "Unsupported format: {}", e),
            ReportError::AiNotConfigured => write!(f, "AI assistant not configured"),
            #[cfg(feature = "ai-assistant")]
            ReportError::AiError(e) => write!(f, "AI error: {}", e),
            ReportError::Validation(e) => write!(f, "Validation error: {}", e),
            ReportError::MissingField(e) => write!(f, "Missing required field: {}", e),
            ReportError::InvalidData(e) => write!(f, "Invalid data: {}", e),
        }
    }
}

impl std::error::Error for ReportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReportError::Io(e) => Some(e),
            ReportError::Template(e) => Some(e),
            ReportError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ReportError {
    fn from(err: io::Error) -> Self {
        ReportError::Io(err)
    }
}

impl From<tera::Error> for ReportError {
    fn from(err: tera::Error) -> Self {
        ReportError::Template(err)
    }
}

impl From<serde_json::Error> for ReportError {
    fn from(err: serde_json::Error) -> Self {
        ReportError::Serialization(err)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_report_error_io_display() {
        let io_err = IoError::new(ErrorKind::NotFound, "file not found");
        let report_err = ReportError::Io(io_err);
        let display = format!("{}", report_err);
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_report_error_pdf_display() {
        let err = ReportError::Pdf("Failed to render PDF".to_string());
        let display = format!("{}", err);
        assert!(display.contains("PDF generation error"));
        assert!(display.contains("Failed to render PDF"));
    }

    #[test]
    fn test_report_error_docx_display() {
        let err = ReportError::Docx("Invalid DOCX template".to_string());
        let display = format!("{}", err);
        assert!(display.contains("DOCX generation error"));
    }

    #[test]
    fn test_report_error_unsupported_format_display() {
        let err = ReportError::UnsupportedFormat("RTF".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Unsupported format"));
        assert!(display.contains("RTF"));
    }

    #[test]
    fn test_report_error_ai_not_configured_display() {
        let err = ReportError::AiNotConfigured;
        let display = format!("{}", err);
        assert!(display.contains("AI assistant not configured"));
    }

    #[test]
    fn test_report_error_validation_display() {
        let err = ReportError::Validation("Missing case number".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Validation error"));
        assert!(display.contains("Missing case number"));
    }

    #[test]
    fn test_report_error_missing_field_display() {
        let err = ReportError::MissingField("examiner_name".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Missing required field"));
        assert!(display.contains("examiner_name"));
    }

    #[test]
    fn test_report_error_invalid_data_display() {
        let err = ReportError::InvalidData("Hash format invalid".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid data"));
        assert!(display.contains("Hash format invalid"));
    }

    #[test]
    fn test_report_error_from_io_error() {
        let io_err = IoError::new(ErrorKind::PermissionDenied, "access denied");
        let report_err: ReportError = io_err.into();
        
        match report_err {
            ReportError::Io(e) => assert_eq!(e.kind(), ErrorKind::PermissionDenied),
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_report_error_source_io() {
        let io_err = IoError::new(ErrorKind::NotFound, "not found");
        let report_err = ReportError::Io(io_err);
        
        // std::error::Error::source should return the underlying error
        assert!(report_err.source().is_some());
    }

    #[test]
    fn test_report_error_source_pdf() {
        let err = ReportError::Pdf("error".to_string());
        // String-based errors have no source
        assert!(err.source().is_none());
    }

    #[test]
    fn test_report_error_debug() {
        let err = ReportError::Validation("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Validation"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_report_result_ok() {
        let result: ReportResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_report_result_err() {
        let result: ReportResult<i32> = Err(ReportError::AiNotConfigured);
        assert!(result.is_err());
    }
}
