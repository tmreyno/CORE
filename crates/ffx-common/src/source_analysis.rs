// =============================================================================
// ffx-common — Common Utilities for Forensic Container Parsers
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Source-aware byte analysis for hex/data review.
//!
//! This module intentionally works on [`EvidenceByteSource`] so local files,
//! container entries, and VFS-backed entries get the same classification and
//! byte statistics.

use serde::{Deserialize, Serialize};

use crate::evidence_source::{
    bounded_read_size, read_range_fully, EvidenceByteSource, EvidenceSourceRef,
    EvidenceSourceResult,
};
use crate::hex::format_hex_inline;
use crate::magic::{detect_file_type, Confidence, FileCategory};

const DEFAULT_ANALYSIS_BYTES: usize = 64 * 1024;
const MAX_ANALYSIS_BYTES: usize = 1024 * 1024;
const DEFAULT_ENTROPY_WINDOW_BYTES: usize = 4096;
const MAX_ENTROPY_WINDOWS: usize = 256;
const ASCII_PREVIEW_LIMIT: usize = 512;
const SIGNATURE_READ_BYTES: usize = 512;
const MAX_SIGNATURES: usize = 64;
const MAX_INDICATORS: usize = 128;
const MAX_INDICATOR_VALUE_BYTES: usize = 256;

/// Options for bounded source analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAnalysisOptions {
    /// Starting offset to analyze.
    #[serde(default)]
    pub offset: u64,
    /// Requested analysis length. Clamped to a conservative maximum.
    #[serde(default)]
    pub length: Option<usize>,
    /// Entropy window size. Set to 0 to disable per-window entropy.
    #[serde(default)]
    pub entropy_window_bytes: Option<usize>,
}

impl Default for SourceAnalysisOptions {
    fn default() -> Self {
        Self {
            offset: 0,
            length: Some(DEFAULT_ANALYSIS_BYTES),
            entropy_window_bytes: Some(DEFAULT_ENTROPY_WINDOW_BYTES),
        }
    }
}

/// A detected magic signature for a source or analyzed range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSignature {
    pub offset: u64,
    pub description: String,
    pub mime_type: String,
    pub extensions: Vec<String>,
    pub category: String,
    pub confidence: String,
    pub magic_hex: String,
}

/// Entropy for one contiguous analyzed window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntropyWindow {
    pub offset: u64,
    pub length: usize,
    pub entropy: f64,
}

/// Text-like indicator extracted from analyzed bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceIndicator {
    pub indicator_type: String,
    pub value: String,
    pub offset: u64,
    pub length: usize,
    pub confidence: String,
}

/// Result of source-aware byte analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAnalysis {
    pub source_ref: EvidenceSourceRef,
    pub source_id: String,
    pub total_size: u64,
    pub offset: u64,
    pub bytes_analyzed: usize,
    pub magic_hex: String,
    pub signatures: Vec<SourceSignature>,
    pub entropy: f64,
    pub entropy_windows: Vec<EntropyWindow>,
    pub histogram: Vec<u64>,
    pub printable_bytes: usize,
    pub nul_bytes: usize,
    pub high_bit_bytes: usize,
    pub printable_ratio: f64,
    pub is_likely_text: bool,
    pub indicators: Vec<SourceIndicator>,
    pub ascii_preview: String,
}

/// Analyze a local file, container entry, or VFS entry through the shared source
/// abstraction.
pub fn analyze_byte_source(
    source: &dyn EvidenceByteSource,
    options: SourceAnalysisOptions,
) -> EvidenceSourceResult<SourceAnalysis> {
    let source_ref = source.source_ref();
    let total_size = source.len()?;
    let offset = options.offset;
    let requested = options
        .length
        .unwrap_or(DEFAULT_ANALYSIS_BYTES)
        .min(MAX_ANALYSIS_BYTES);
    let read_len = bounded_read_size(&source_ref, total_size, offset, requested)?;
    let data = read_range_fully(source, offset, read_len)?;

    let mut signature_data = data.clone();
    if offset != 0 {
        let signature_len = bounded_read_size(&source_ref, total_size, 0, SIGNATURE_READ_BYTES)?;
        signature_data = read_range_fully(source, 0, signature_len)?;
    }

    let histogram = byte_histogram(&data);
    let printable_bytes = data.iter().filter(|byte| is_printable_byte(**byte)).count();
    let nul_bytes = data.iter().filter(|byte| **byte == 0).count();
    let high_bit_bytes = data.iter().filter(|byte| **byte >= 0x80).count();
    let printable_ratio = ratio(printable_bytes, data.len());
    let entropy = shannon_entropy_from_histogram(&histogram, data.len());
    let entropy_windows = entropy_windows(
        &data,
        offset,
        options
            .entropy_window_bytes
            .unwrap_or(DEFAULT_ENTROPY_WINDOW_BYTES),
    );
    let is_likely_text = is_likely_text(&data, printable_ratio, nul_bytes);

    let mut signatures = detect_signatures_at_offset(&signature_data, 0);
    if offset != 0 {
        merge_signatures(&mut signatures, detect_signatures_at_offset(&data, offset));
    }
    let magic_hex = format_hex_inline(&signature_data[..signature_data.len().min(16)], true);
    let indicators = extract_source_indicators(&data, offset);

    Ok(SourceAnalysis {
        source_id: source_ref.display_id(),
        source_ref,
        total_size,
        offset,
        bytes_analyzed: data.len(),
        magic_hex,
        signatures,
        entropy,
        entropy_windows,
        histogram,
        printable_bytes,
        nul_bytes,
        high_bit_bytes,
        printable_ratio,
        is_likely_text,
        indicators,
        ascii_preview: ascii_preview(&data),
    })
}

fn detect_signatures_at_offset(header: &[u8], base_offset: u64) -> Vec<SourceSignature> {
    let mut signatures = Vec::new();

    if let Some(file_type) = detect_file_type(header) {
        signatures.push(SourceSignature {
            offset: base_offset,
            description: file_type.description,
            mime_type: file_type.mime,
            extensions: file_type.extensions,
            category: category_name(file_type.category).to_string(),
            confidence: confidence_name(file_type.confidence).to_string(),
            magic_hex: format_hex_inline(&header[..header.len().min(16)], true),
        });
    }

    for known in EMBEDDED_SIGNATURES {
        for relative_offset in signature_offsets(
            header,
            known.pattern,
            MAX_SIGNATURES.saturating_sub(signatures.len()),
        ) {
            let Some(offset) = checked_source_offset(base_offset, relative_offset) else {
                continue;
            };
            if relative_offset == 0 && signatures.iter().any(|signature| signature.offset == 0) {
                continue;
            }
            signatures.push(SourceSignature {
                offset,
                description: known.description.to_string(),
                mime_type: known.mime_type.to_string(),
                extensions: known
                    .extensions
                    .iter()
                    .map(|ext| (*ext).to_string())
                    .collect(),
                category: known.category.to_string(),
                confidence: "high".to_string(),
                magic_hex: format_hex_inline(signature_preview(header, relative_offset), true),
            });
            if signatures.len() >= MAX_SIGNATURES {
                signatures.sort_by(|left, right| {
                    left.offset
                        .cmp(&right.offset)
                        .then_with(|| left.description.cmp(&right.description))
                });
                return signatures;
            }
        }
    }

    signatures.sort_by(|left, right| {
        left.offset
            .cmp(&right.offset)
            .then_with(|| left.description.cmp(&right.description))
    });
    signatures
}

fn merge_signatures(signatures: &mut Vec<SourceSignature>, additional: Vec<SourceSignature>) {
    for signature in additional {
        if signatures.len() >= MAX_SIGNATURES {
            break;
        }
        if signatures.iter().any(|existing| {
            existing.offset == signature.offset && existing.description == signature.description
        }) {
            continue;
        }
        signatures.push(signature);
    }
    signatures.sort_by(|left, right| {
        left.offset
            .cmp(&right.offset)
            .then_with(|| left.description.cmp(&right.description))
    });
}

struct EmbeddedSignature {
    pattern: &'static [u8],
    description: &'static str,
    mime_type: &'static str,
    extensions: &'static [&'static str],
    category: &'static str,
}

static EMBEDDED_SIGNATURES: &[EmbeddedSignature] = &[
    EmbeddedSignature {
        pattern: b"%PDF",
        description: "PDF Document",
        mime_type: "application/pdf",
        extensions: &["pdf"],
        category: "document",
    },
    EmbeddedSignature {
        pattern: b"\x89PNG\r\n\x1a\n",
        description: "PNG Image",
        mime_type: "image/png",
        extensions: &["png"],
        category: "image",
    },
    EmbeddedSignature {
        pattern: &[0xff, 0xd8, 0xff],
        description: "JPEG Image",
        mime_type: "image/jpeg",
        extensions: &["jpg", "jpeg"],
        category: "image",
    },
    EmbeddedSignature {
        pattern: b"GIF87a",
        description: "GIF Image",
        mime_type: "image/gif",
        extensions: &["gif"],
        category: "image",
    },
    EmbeddedSignature {
        pattern: b"GIF89a",
        description: "GIF Image",
        mime_type: "image/gif",
        extensions: &["gif"],
        category: "image",
    },
    EmbeddedSignature {
        pattern: b"PK\x03\x04",
        description: "ZIP Archive",
        mime_type: "application/zip",
        extensions: &["zip", "docx", "xlsx", "pptx"],
        category: "archive",
    },
    EmbeddedSignature {
        pattern: b"7z\xbc\xaf\x27\x1c",
        description: "7-Zip Archive",
        mime_type: "application/x-7z-compressed",
        extensions: &["7z"],
        category: "archive",
    },
    EmbeddedSignature {
        pattern: b"Rar!\x1a\x07\x00",
        description: "RAR Archive",
        mime_type: "application/vnd.rar",
        extensions: &["rar"],
        category: "archive",
    },
    EmbeddedSignature {
        pattern: b"SQLite format 3\0",
        description: "SQLite Database",
        mime_type: "application/vnd.sqlite3",
        extensions: &["sqlite", "sqlite3", "db"],
        category: "database",
    },
    EmbeddedSignature {
        pattern: &[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1],
        description: "OLE Compound Document",
        mime_type: "application/x-ole-storage",
        extensions: &["doc", "xls", "ppt", "msg"],
        category: "document",
    },
    EmbeddedSignature {
        pattern: b"MZ",
        description: "DOS/Windows Executable",
        mime_type: "application/x-msdownload",
        extensions: &["exe", "dll"],
        category: "executable",
    },
    EmbeddedSignature {
        pattern: b"\x7fELF",
        description: "ELF Executable",
        mime_type: "application/x-elf",
        extensions: &["elf", "so"],
        category: "executable",
    },
    EmbeddedSignature {
        pattern: b"bplist00",
        description: "Binary Property List",
        mime_type: "application/x-plist",
        extensions: &["plist"],
        category: "system",
    },
    EmbeddedSignature {
        pattern: b"regf",
        description: "Windows Registry Hive",
        mime_type: "application/x-windows-registry",
        extensions: &["dat"],
        category: "system",
    },
];

fn signature_offsets(header: &[u8], pattern: &[u8], limit: usize) -> Vec<usize> {
    if limit == 0 || pattern.is_empty() || header.len() < pattern.len() {
        return Vec::new();
    }

    header
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == pattern).then_some(offset))
        .take(limit)
        .collect()
}

fn signature_preview(header: &[u8], relative_offset: usize) -> &[u8] {
    let end = relative_offset.saturating_add(16).min(header.len());
    header.get(relative_offset..end).unwrap_or_default()
}

fn byte_histogram(data: &[u8]) -> Vec<u64> {
    let mut histogram = vec![0u64; 256];
    for byte in data {
        histogram[*byte as usize] += 1;
    }
    histogram
}

fn shannon_entropy_from_histogram(histogram: &[u64], len: usize) -> f64 {
    if len == 0 {
        return 0.0;
    }

    let len = len as f64;
    histogram
        .iter()
        .copied()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = count as f64 / len;
            -probability * probability.log2()
        })
        .sum()
}

fn entropy_windows(data: &[u8], base_offset: u64, window_size: usize) -> Vec<EntropyWindow> {
    if data.is_empty() || window_size == 0 {
        return Vec::new();
    }

    data.chunks(window_size)
        .take(MAX_ENTROPY_WINDOWS)
        .enumerate()
        .filter_map(|(index, chunk)| {
            let relative_offset = index.checked_mul(window_size)?;
            let offset = checked_source_offset(base_offset, relative_offset)?;
            let histogram = byte_histogram(chunk);
            Some(EntropyWindow {
                offset,
                length: chunk.len(),
                entropy: shannon_entropy_from_histogram(&histogram, chunk.len()),
            })
        })
        .collect()
}

fn ascii_preview(data: &[u8]) -> String {
    data.iter()
        .take(ASCII_PREVIEW_LIMIT)
        .map(|byte| match *byte {
            b'\r' => ' ',
            b'\n' | b'\t' => *byte as char,
            0x20..=0x7e => *byte as char,
            _ => '.',
        })
        .collect()
}

fn is_likely_text(data: &[u8], printable_ratio: f64, nul_bytes: usize) -> bool {
    !data.is_empty() && nul_bytes == 0 && printable_ratio >= 0.85
}

fn is_printable_byte(byte: u8) -> bool {
    (0x20..=0x7e).contains(&byte) || matches!(byte, b'\n' | b'\r' | b'\t')
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

pub(crate) fn extract_source_indicators(data: &[u8], base_offset: u64) -> Vec<SourceIndicator> {
    let mut indicators = Vec::new();
    extract_url_indicators(data, base_offset, &mut indicators);
    extract_email_indicators(data, base_offset, &mut indicators);
    extract_ipv4_indicators(data, base_offset, &mut indicators);
    extract_windows_path_indicators(data, base_offset, &mut indicators);
    indicators.sort_by(|left, right| {
        left.offset
            .cmp(&right.offset)
            .then_with(|| left.indicator_type.cmp(&right.indicator_type))
            .then_with(|| left.value.cmp(&right.value))
    });
    indicators
}

fn extract_url_indicators(data: &[u8], base_offset: u64, indicators: &mut Vec<SourceIndicator>) {
    for prefix in [
        b"http://".as_slice(),
        b"https://".as_slice(),
        b"file://".as_slice(),
        b"www.".as_slice(),
    ] {
        if indicators.len() >= MAX_INDICATORS {
            return;
        }

        for offset in find_ascii_pattern_offsets(
            data,
            prefix,
            MAX_INDICATORS.saturating_sub(indicators.len()),
        ) {
            if indicators.len() >= MAX_INDICATORS {
                return;
            }
            let end = scan_until_delimiter(data, offset, is_url_delimiter);
            push_indicator(indicators, "url", data, base_offset, offset, end, "medium");
        }
    }
}

fn extract_email_indicators(data: &[u8], base_offset: u64, indicators: &mut Vec<SourceIndicator>) {
    let mut index = 0;
    while index < data.len() {
        if indicators.len() >= MAX_INDICATORS {
            return;
        }

        if data[index] != b'@' {
            index += 1;
            continue;
        }

        let mut start = index;
        while start > 0 && is_email_char(data[start - 1]) {
            start -= 1;
        }
        let Some(mut end) = index.checked_add(1) else {
            break;
        };
        while end < data.len() && is_email_char(data[end]) {
            end += 1;
        }

        if valid_email_candidate(&data[start..end]) {
            push_indicator(indicators, "email", data, base_offset, start, end, "medium");
        }
        index = advance_indicator_index(index, end);
    }
}

fn extract_ipv4_indicators(data: &[u8], base_offset: u64, indicators: &mut Vec<SourceIndicator>) {
    let mut index = 0;
    while index < data.len() {
        if indicators.len() >= MAX_INDICATORS {
            return;
        }

        if !data[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        if index > 0 && is_ipv4_boundary_char(data[index - 1]) {
            index += 1;
            continue;
        }

        let start = index;
        let mut end = index;
        while end < data.len() && (data[end].is_ascii_digit() || data[end] == b'.') {
            end += 1;
        }

        if valid_ipv4_candidate(&data[start..end])
            && (end == data.len() || !is_ipv4_boundary_char(data[end]))
        {
            push_indicator(indicators, "ipv4", data, base_offset, start, end, "medium");
        }
        index = advance_indicator_index(index, end);
    }
}

fn extract_windows_path_indicators(
    data: &[u8],
    base_offset: u64,
    indicators: &mut Vec<SourceIndicator>,
) {
    let mut index = 0;
    while has_bytes_from(data, index, 3) {
        if indicators.len() >= MAX_INDICATORS {
            return;
        }

        if (index == 0 || !data[index - 1].is_ascii_alphanumeric())
            && data[index].is_ascii_alphabetic()
            && data[index + 1] == b':'
            && is_path_separator(data[index + 2])
        {
            let end = scan_until_delimiter(data, index, is_path_delimiter);
            push_indicator(
                indicators,
                "windows_path",
                data,
                base_offset,
                index,
                end,
                "medium",
            );
            index = advance_indicator_index(index, end);
            continue;
        }

        if has_bytes_from(data, index, 4)
            && (index == 0 || data[index - 1] != b':')
            && is_path_separator(data[index])
            && is_path_separator(data[index + 1])
            && is_path_component_char(data[index + 2])
        {
            let end = scan_until_delimiter(data, index, is_path_delimiter);
            if data[index + 2..end]
                .iter()
                .any(|byte| is_path_separator(*byte))
            {
                push_indicator(
                    indicators,
                    "unc_path",
                    data,
                    base_offset,
                    index,
                    end,
                    "medium",
                );
            }
            index = advance_indicator_index(index, end);
            continue;
        }

        index += 1;
    }
}

fn find_ascii_pattern_offsets(data: &[u8], pattern: &[u8], limit: usize) -> Vec<usize> {
    if limit == 0 || pattern.is_empty() || data.len() < pattern.len() {
        return Vec::new();
    }

    data.windows(pattern.len())
        .enumerate()
        .filter_map(|(offset, window)| window.eq_ignore_ascii_case(pattern).then_some(offset))
        .take(limit)
        .collect()
}

fn scan_until_delimiter(data: &[u8], start: usize, is_delimiter: fn(u8) -> bool) -> usize {
    let mut end = start;
    while end < data.len() && end - start < MAX_INDICATOR_VALUE_BYTES && !is_delimiter(data[end]) {
        end += 1;
    }
    end
}

fn has_bytes_from(data: &[u8], index: usize, needed: usize) -> bool {
    data.len().saturating_sub(index) >= needed
}

fn advance_indicator_index(index: usize, end: usize) -> usize {
    end.max(index.saturating_add(1))
}

fn push_indicator(
    indicators: &mut Vec<SourceIndicator>,
    indicator_type: &str,
    data: &[u8],
    base_offset: u64,
    start: usize,
    end: usize,
    confidence: &str,
) {
    if indicators.len() >= MAX_INDICATORS || end <= start {
        return;
    }
    let Some(value) = std::str::from_utf8(&data[start..end])
        .ok()
        .map(|value| value.trim_matches(trim_indicator_edge).to_string())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(offset) = checked_source_offset(base_offset, start) else {
        return;
    };
    if indicators.iter().any(|existing| {
        existing.indicator_type == indicator_type
            && existing.value == value
            && existing.offset == offset
    }) {
        return;
    }

    indicators.push(SourceIndicator {
        indicator_type: indicator_type.to_string(),
        length: value.len(),
        value,
        offset,
        confidence: confidence.to_string(),
    });
}

fn checked_source_offset(base_offset: u64, relative_offset: usize) -> Option<u64> {
    let relative_offset = u64::try_from(relative_offset).ok()?;
    base_offset.checked_add(relative_offset)
}

fn valid_email_candidate(bytes: &[u8]) -> bool {
    let Ok(value) = std::str::from_utf8(bytes) else {
        return false;
    };
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .rsplit('.')
            .next()
            .is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
}

fn valid_ipv4_candidate(bytes: &[u8]) -> bool {
    let Ok(value) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut parts = value.split('.');
    (0..4).all(|_| {
        parts
            .next()
            .filter(|part| !part.is_empty() && part.len() <= 3)
            .and_then(|part| part.parse::<u8>().ok())
            .is_some()
    }) && parts.next().is_none()
}

fn is_email_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'%' | b'+' | b'-')
}

fn is_url_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b')' | b']' | b'}')
}

fn is_path_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'|' | b'*' | b'?')
}

fn is_path_separator(byte: u8) -> bool {
    matches!(byte, b'\\' | b'/')
}

fn is_path_component_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'$')
}

fn is_ipv4_boundary_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'.'
}

fn trim_indicator_edge(ch: char) -> bool {
    matches!(ch, '.' | ',' | ';' | ':' | ')' | ']' | '}' | '"' | '\'')
}

fn category_name(category: FileCategory) -> &'static str {
    match category {
        FileCategory::Image => "image",
        FileCategory::Document => "document",
        FileCategory::Archive => "archive",
        FileCategory::Executable => "executable",
        FileCategory::Audio => "audio",
        FileCategory::Video => "video",
        FileCategory::Database => "database",
        FileCategory::Forensic => "forensic",
        FileCategory::System => "system",
        FileCategory::Text => "text",
        FileCategory::Unknown => "unknown",
    }
}

fn confidence_name(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_source::{
        EvidenceByteSource, EvidenceSourceRef, EvidenceSourceResult, LocalFileByteSource,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    struct ChunkedByteSource {
        source_ref: EvidenceSourceRef,
        data: Vec<u8>,
        max_chunk: usize,
    }

    impl ChunkedByteSource {
        fn new(path: &str, data: &[u8], max_chunk: usize) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                data: data.to_vec(),
                max_chunk,
            }
        }
    }

    impl EvidenceByteSource for ChunkedByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.data.len() as u64)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let requested = size.min(self.max_chunk);
            let end = start.saturating_add(requested).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    #[test]
    fn analyze_byte_source_detects_pdf_signature_and_text_stats() {
        let fixture = b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>\n";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(fixture).unwrap();
        let source = LocalFileByteSource::new(file.path());

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                length: Some(64),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(analysis.total_size, fixture.len() as u64);
        assert_eq!(analysis.bytes_analyzed, fixture.len());
        assert!(analysis.is_likely_text);
        assert_eq!(analysis.signatures.len(), 1);
        assert_eq!(analysis.signatures[0].description, "PDF Document");
        assert_eq!(analysis.signatures[0].category, "document");
        assert_eq!(analysis.histogram[b'%' as usize], 1);
        assert!(analysis.entropy > 0.0);
    }

    #[test]
    fn analyze_byte_source_assembles_chunked_reads() {
        let fixture = b"%PDF-1.7\nContact admin@example.com\n";
        let source = ChunkedByteSource::new("chunked.pdf", fixture, 3);

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                length: Some(128),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(analysis.bytes_analyzed, fixture.len());
        assert_eq!(analysis.signatures[0].description, "PDF Document");
        assert!(analysis.indicators.iter().any(|indicator| {
            indicator.indicator_type == "email" && indicator.value == "admin@example.com"
        }));
    }

    #[test]
    fn analyze_byte_source_reports_embedded_signature_offsets() {
        let mut fixture = vec![0u8; 32];
        fixture.extend_from_slice(b"%PDF-1.7\n");
        fixture.extend_from_slice(&[0u8; 17]);
        let png_offset = fixture.len();
        fixture.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        fixture.extend_from_slice(&[0u8; 16]);
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&fixture).unwrap();
        let source = LocalFileByteSource::new(file.path());

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                length: Some(128),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(analysis.signatures.iter().any(|signature| {
            signature.offset == 32 && signature.description == "PDF Document"
        }));
        assert!(analysis.signatures.iter().any(|signature| {
            signature.offset == png_offset as u64 && signature.description == "PNG Image"
        }));
    }

    #[test]
    fn analyze_byte_source_reports_signatures_in_nonzero_range() {
        let mut fixture = vec![0u8; 1024];
        fixture.extend_from_slice(b"%PDF-1.7\nrange document");
        let source = ChunkedByteSource::new("range.bin", &fixture, 7);

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                offset: 1024,
                length: Some(64),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(analysis.offset, 1024);
        assert!(analysis.signatures.iter().any(|signature| {
            signature.offset == 1024 && signature.description == "PDF Document"
        }));
    }

    #[test]
    fn analyze_byte_source_extracts_text_indicators() {
        let fixture = b"Contact admin@example.com from 192.168.1.10 and visit https://example.com/login or C:\\Users\\Alice\\NTUSER.DAT";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(fixture).unwrap();
        let source = LocalFileByteSource::new(file.path());

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                length: Some(256),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(analysis.indicators.iter().any(|indicator| {
            indicator.indicator_type == "email"
                && indicator.value == "admin@example.com"
                && indicator.offset == 8
        }));
        assert!(analysis.indicators.iter().any(|indicator| {
            indicator.indicator_type == "ipv4" && indicator.value == "192.168.1.10"
        }));
        assert!(analysis.indicators.iter().any(|indicator| {
            indicator.indicator_type == "url" && indicator.value == "https://example.com/login"
        }));
        assert!(analysis.indicators.iter().any(|indicator| {
            indicator.indicator_type == "windows_path"
                && indicator.value == r"C:\Users\Alice\NTUSER.DAT"
        }));
    }

    #[test]
    fn source_indicators_preserve_repeated_values_at_distinct_offsets() {
        let fixture =
            b"admin@example.com admin@example.com https://example.test https://example.test";

        let indicators = extract_source_indicators(fixture, 0);

        let email_offsets: Vec<u64> = indicators
            .iter()
            .filter(|indicator| {
                indicator.indicator_type == "email" && indicator.value == "admin@example.com"
            })
            .map(|indicator| indicator.offset)
            .collect();
        let url_offsets: Vec<u64> = indicators
            .iter()
            .filter(|indicator| {
                indicator.indicator_type == "url" && indicator.value == "https://example.test"
            })
            .map(|indicator| indicator.offset)
            .collect();

        assert_eq!(email_offsets, vec![0, 18]);
        assert_eq!(url_offsets, vec![36, 57]);
    }

    #[test]
    fn pattern_offset_helpers_honor_limits_before_collecting() {
        let repeated = b"%PDF %PDF %PDF %PDF";
        assert_eq!(signature_offsets(repeated, b"%PDF", 0), Vec::<usize>::new());
        assert_eq!(signature_offsets(repeated, b"%PDF", 2), vec![0, 5]);

        let repeated_urls = b"http://a http://b http://c";
        assert_eq!(
            find_ascii_pattern_offsets(repeated_urls, b"http://", 2),
            vec![0, 9]
        );
    }

    #[test]
    fn analyze_byte_source_caps_repeated_url_indicators() {
        let mut fixture = Vec::new();
        for index in 0..(MAX_INDICATORS + 32) {
            fixture.extend_from_slice(format!("https://example{index}.com/path\n").as_bytes());
        }
        let source = ChunkedByteSource::new("urls.txt", &fixture, 31);

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                length: Some(fixture.len()),
                ..Default::default()
            },
        )
        .unwrap();

        let url_count = analysis
            .indicators
            .iter()
            .filter(|indicator| indicator.indicator_type == "url")
            .count();
        assert_eq!(url_count, MAX_INDICATORS);
        assert_eq!(analysis.indicators.len(), MAX_INDICATORS);
    }

    #[test]
    fn indicator_extractors_stop_when_budget_is_full() {
        fn capped_indicators() -> Vec<SourceIndicator> {
            (0..MAX_INDICATORS)
                .map(|index| SourceIndicator {
                    indicator_type: "existing".to_string(),
                    value: format!("value-{index}"),
                    offset: index as u64,
                    length: 1,
                    confidence: "low".to_string(),
                })
                .collect()
        }

        let fixture = b"admin@example.com 192.168.1.10 C:\\Users\\Alice \\server\\share";

        let mut email_indicators = capped_indicators();
        extract_email_indicators(fixture, 0, &mut email_indicators);
        assert_eq!(email_indicators.len(), MAX_INDICATORS);

        let mut ipv4_indicators = capped_indicators();
        extract_ipv4_indicators(fixture, 0, &mut ipv4_indicators);
        assert_eq!(ipv4_indicators.len(), MAX_INDICATORS);

        let mut path_indicators = capped_indicators();
        extract_windows_path_indicators(fixture, 0, &mut path_indicators);
        assert_eq!(path_indicators.len(), MAX_INDICATORS);
    }

    #[test]
    fn signature_preview_saturates_truncated_ranges() {
        assert_eq!(signature_preview(b"abcdef", 4), b"ef");
        assert_eq!(signature_preview(b"abcdef", usize::MAX), b"");
    }

    #[test]
    fn short_windows_path_candidates_do_not_index_past_input() {
        for fixture in [b"C".as_slice(), b"C:", b"\\", b"\\\\s"] {
            let mut indicators = Vec::new();
            extract_windows_path_indicators(fixture, 0, &mut indicators);
            assert!(indicators.is_empty());
        }
    }

    #[test]
    fn source_indicators_cap_extracted_value_length() {
        let mut fixture = b"https://".to_vec();
        fixture.extend(std::iter::repeat(b'a').take(MAX_INDICATOR_VALUE_BYTES + 32));

        let indicators = extract_source_indicators(&fixture, 0);

        let url = indicators
            .iter()
            .find(|indicator| indicator.indicator_type == "url")
            .unwrap();
        assert_eq!(url.length, MAX_INDICATOR_VALUE_BYTES);
        assert_eq!(url.value.len(), MAX_INDICATOR_VALUE_BYTES);
    }

    #[test]
    fn indicator_index_helpers_saturate_boundary_arithmetic() {
        assert!(!has_bytes_from(b"ab", usize::MAX, 1));
        assert!(has_bytes_from(b"abc", 0, 3));
        assert_eq!(advance_indicator_index(usize::MAX, 0), usize::MAX);
    }

    #[test]
    fn source_offset_helper_rejects_overflow() {
        assert_eq!(checked_source_offset(u64::MAX, 0), Some(u64::MAX));
        assert_eq!(checked_source_offset(u64::MAX, 1), None);
    }

    #[test]
    fn embedded_signatures_skip_unrepresentable_offsets() {
        let mut fixture = vec![0u8];
        fixture.extend_from_slice(b"%PDF-1.7\n");

        let signatures = detect_signatures_at_offset(&fixture, u64::MAX);

        assert!(signatures.is_empty());
    }

    #[test]
    fn entropy_windows_skip_unrepresentable_offsets() {
        let windows = entropy_windows(&[1, 2], u64::MAX, 1);

        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].offset, u64::MAX);
    }

    #[test]
    fn source_indicators_skip_unrepresentable_offsets() {
        let indicators = extract_source_indicators(b"x admin@example.com", u64::MAX);

        assert!(indicators.is_empty());
    }

    #[test]
    fn analyze_byte_source_clamps_range_to_eof() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0, 1, 2, 3]).unwrap();
        let source = LocalFileByteSource::new(file.path());

        let analysis = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                offset: 2,
                length: Some(16),
                entropy_window_bytes: Some(1),
            },
        )
        .unwrap();

        assert_eq!(analysis.offset, 2);
        assert_eq!(analysis.bytes_analyzed, 2);
        assert_eq!(analysis.histogram[2], 1);
        assert_eq!(analysis.histogram[3], 1);
        assert_eq!(analysis.entropy_windows.len(), 2);
    }

    #[test]
    fn analyze_byte_source_rejects_offset_past_eof() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0, 1, 2, 3]).unwrap();
        let source = LocalFileByteSource::new(file.path());

        let err = analyze_byte_source(
            &source,
            SourceAnalysisOptions {
                offset: 5,
                length: Some(1),
                ..Default::default()
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            crate::evidence_source::EvidenceSourceError::InvalidRange {
                offset: 5,
                size: 4,
                ..
            }
        ));
    }

    #[test]
    fn shannon_entropy_reports_zero_for_repeated_bytes() {
        let histogram = byte_histogram(&[0xAA; 32]);
        assert_eq!(shannon_entropy_from_histogram(&histogram, 32), 0.0);
    }
}
