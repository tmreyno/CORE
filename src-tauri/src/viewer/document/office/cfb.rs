// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! CFB/OLE2 Format Handling (Legacy DOC, PPT)
//!
//! Extracts text from legacy Microsoft Office binary formats using the `cfb`
//! crate to open OLE2 compound file streams.

use std::io::{Cursor, Read};
use std::path::Path;

use super::{OfficeParagraph, OfficeTextSection};
use crate::viewer::document::error::{DocumentError, DocumentResult};

const MAX_CFB_TEXT_STREAM_BYTES: u64 = 32 * 1024 * 1024;
const MAX_CFB_SOURCE_BYTES: u64 = 100 * 1024 * 1024;

fn ensure_cfb_source_size_allowed(size: u64, format: &str) -> DocumentResult<()> {
    if size > MAX_CFB_SOURCE_BYTES {
        return Err(DocumentError::Parse(format!(
            "{} CFB source exceeds {} MiB limit",
            format,
            MAX_CFB_SOURCE_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

fn read_cfb_text_stream<R: Read>(reader: R, stream_name: &str) -> DocumentResult<Vec<u8>> {
    let mut data = Vec::new();
    reader
        .take(MAX_CFB_TEXT_STREAM_BYTES + 1)
        .read_to_end(&mut data)?;

    if data.len() as u64 > MAX_CFB_TEXT_STREAM_BYTES {
        return Err(DocumentError::Parse(format!(
            "CFB stream '{}' exceeds {} MiB text extraction limit",
            stream_name,
            MAX_CFB_TEXT_STREAM_BYTES / (1024 * 1024)
        )));
    }

    Ok(data)
}

// =============================================================================
// Legacy DOC Text Extraction (OLE2 / CFB)
// =============================================================================

/// Extract text from a legacy .doc file using the CFB crate.
///
/// Legacy Word Binary Format stores text in the "WordDocument" stream
/// with complex encoding. We attempt a best-effort text extraction:
/// 1. Try to read the "WordDocument" stream
/// 2. Extract printable text runs (UTF-16LE or ASCII)
pub(crate) fn extract_doc_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    ensure_cfb_source_size_allowed(std::fs::metadata(path)?.len(), "DOC")?;
    let data = std::fs::read(path)?;
    extract_doc_text_from_bytes(&data)
}

pub(crate) fn extract_doc_text_from_bytes(data: &[u8]) -> DocumentResult<Vec<OfficeTextSection>> {
    ensure_cfb_source_size_allowed(data.len() as u64, "DOC")?;
    // Validate the file is a valid OLE2/CFB container
    let _comp = ::cfb::CompoundFile::open(Cursor::new(data))
        .map_err(|e| DocumentError::Parse(format!("Not a valid OLE2/DOC file: {}", e)))?;

    // Try common stream names for text content
    let stream_names = ["/WordDocument", "/1Table", "/0Table"];

    let mut all_text = String::new();

    // Re-open for each stream read (cfb borrows mutably)
    for stream_name in &stream_names {
        if let Ok(mut comp) = ::cfb::CompoundFile::open(Cursor::new(data)) {
            if comp.is_stream(stream_name) {
                if let Ok(mut stream) = comp.open_stream(stream_name) {
                    match read_cfb_text_stream(&mut stream, stream_name) {
                        Ok(data) => {
                            // Extract printable text runs from binary data
                            let text = extract_printable_text(&data);
                            if !text.is_empty() {
                                if !all_text.is_empty() {
                                    all_text.push('\n');
                                }
                                all_text.push_str(&text);
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }
    }

    if all_text.is_empty() {
        // Fallback: scan the entire file for printable text runs
        all_text = extract_printable_text(data);
    }

    if all_text.is_empty() {
        return Err(DocumentError::Parse(
            "No readable text found in .doc file".to_string(),
        ));
    }

    let paragraphs: Vec<OfficeParagraph> = all_text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(OfficeParagraph::normal)
        .collect();

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

// =============================================================================
// Legacy PPT Text Extraction (OLE2 / CFB)
// =============================================================================

/// Extract text from a legacy .ppt file.
///
/// PPT files store slide text in the "PowerPoint Document" stream.
/// We extract printable text runs as a best-effort approach.
pub(crate) fn extract_ppt_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    ensure_cfb_source_size_allowed(std::fs::metadata(path)?.len(), "PPT")?;
    let data = std::fs::read(path)?;
    extract_ppt_text_from_bytes(&data)
}

pub(crate) fn extract_ppt_text_from_bytes(data: &[u8]) -> DocumentResult<Vec<OfficeTextSection>> {
    ensure_cfb_source_size_allowed(data.len() as u64, "PPT")?;
    let mut comp = ::cfb::CompoundFile::open(Cursor::new(data))
        .map_err(|e| DocumentError::Parse(format!("Not a valid OLE2/PPT file: {}", e)))?;

    let mut all_text = String::new();

    // The main content stream in PPT files
    if comp.is_stream("/PowerPoint Document") {
        if let Ok(mut stream) = comp.open_stream("/PowerPoint Document") {
            let data = read_cfb_text_stream(&mut stream, "/PowerPoint Document")?;
            all_text = extract_printable_text(&data);
        }
    }

    if all_text.is_empty() {
        // Fallback: scan entire file
        all_text = extract_printable_text(data);
    }

    if all_text.is_empty() {
        return Err(DocumentError::Parse(
            "No readable text found in .ppt file".to_string(),
        ));
    }

    let paragraphs: Vec<OfficeParagraph> = all_text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(OfficeParagraph::normal)
        .collect();

    Ok(vec![OfficeTextSection {
        label: Some("Presentation".to_string()),
        paragraphs,
    }])
}

// =============================================================================
// Text Extraction Helpers
// =============================================================================

/// Extract printable text runs from binary data.
///
/// Scans for sequences of printable ASCII characters (min 4 chars).
/// Also attempts UTF-16LE decoding for Unicode documents.
fn extract_printable_text(data: &[u8]) -> String {
    let mut result = String::new();

    // Try UTF-16LE first (common in Word docs)
    let utf16_text = extract_utf16le_text(data);
    if utf16_text.len() > result.len() {
        result = utf16_text;
    }

    // Also try ASCII extraction if UTF-16 didn't yield much
    if result.len() < 100 {
        let ascii_text = extract_ascii_text(data);
        if ascii_text.len() > result.len() {
            result = ascii_text;
        }
    }

    result
}

/// Extract text from UTF-16LE encoded data
fn extract_utf16le_text(data: &[u8]) -> String {
    if data.len() < 2 {
        return String::new();
    }

    let mut result = Vec::new();
    let mut current_run = String::new();

    for chunk in data.chunks_exact(2) {
        let code_unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if let Some(ch) = char::from_u32(code_unit as u32) {
            if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_ascii_punctuation() {
                current_run.push(ch);
            } else if !current_run.is_empty() {
                if current_run.trim().len() >= 4 {
                    result.push(current_run.trim().to_string());
                }
                current_run.clear();
            }
        }
    }
    if current_run.trim().len() >= 4 {
        result.push(current_run.trim().to_string());
    }

    result.join("\n")
}

/// Extract ASCII text runs from binary data
fn extract_ascii_text(data: &[u8]) -> String {
    let mut result = Vec::new();
    let mut current_run = String::new();

    for &byte in data {
        if (0x20..0x7F).contains(&byte) || byte == b'\n' || byte == b'\r' || byte == b'\t' {
            current_run.push(byte as char);
        } else if !current_run.is_empty() {
            if current_run.trim().len() >= 4 {
                result.push(current_run.trim().to_string());
            }
            current_run.clear();
        }
    }
    if current_run.trim().len() >= 4 {
        result.push(current_run.trim().to_string());
    }

    result.join("\n")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_cfb_text_stream_rejects_oversized_stream() {
        let oversized = std::io::repeat(b'x').take(MAX_CFB_TEXT_STREAM_BYTES + 1);

        let err = read_cfb_text_stream(oversized, "/WordDocument").unwrap_err();

        assert!(err
            .to_string()
            .contains("exceeds 32 MiB text extraction limit"));
    }

    #[test]
    fn test_cfb_source_size_rejects_oversized_source() {
        let err = ensure_cfb_source_size_allowed(MAX_CFB_SOURCE_BYTES + 1, "DOC").unwrap_err();

        assert!(err.to_string().contains("CFB source exceeds"));
    }

    #[test]
    fn test_extract_doc_text_rejects_sparse_oversized_source_before_read() {
        let file = tempfile::Builder::new().suffix(".doc").tempfile().unwrap();
        file.as_file().set_len(MAX_CFB_SOURCE_BYTES + 1).unwrap();

        let err = extract_doc_text(file.path()).unwrap_err();

        assert!(err.to_string().contains("CFB source exceeds"));
    }

    #[test]
    fn test_extract_ascii_text() {
        let data = b"Hello World\x00\x01\x02This is text\x00\x00end";
        let text = extract_ascii_text(data);
        assert!(text.contains("Hello World"), "Got: {}", text);
        assert!(text.contains("This is text"), "Got: {}", text);
    }

    #[test]
    fn test_extract_ascii_short_runs_filtered() {
        // Runs shorter than 4 chars should be filtered out
        let data = b"Hi\x00\x00Long enough text here";
        let text = extract_ascii_text(data);
        assert!(!text.contains("Hi"));
        assert!(text.contains("Long enough text here"));
    }

    #[test]
    fn test_extract_utf16le_text() {
        // "Hello World" in UTF-16LE
        let data: Vec<u8> = "Hello World"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let text = extract_utf16le_text(&data);
        assert!(text.contains("Hello World"), "Got: {}", text);
    }
}
