// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File viewer commands for hex/text viewing.

use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{
    analyze_byte_source, read_all_with_limit, read_range_fully, EvidenceByteSource,
    EvidenceSourceError, LocalFileByteSource, SourceAnalysis, SourceAnalysisOptions,
};
use crate::viewer;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;

const MAX_INLINE_BINARY_BASE64_BYTES: u64 = 100 * 1024 * 1024;
const MAX_BINARY_BASE64_CHUNK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerBinaryInfo {
    pub path: String,
    pub size: u64,
    pub max_inline_bytes: u64,
    pub supports_range_reads: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerBinaryBase64Chunk {
    pub path: String,
    pub offset: u64,
    pub bytes_read: usize,
    pub total_size: u64,
    pub eof: bool,
    pub data: String,
}

fn binary_info_for_source(
    source_id: String,
    source: &dyn EvidenceByteSource,
) -> Result<ViewerBinaryInfo, String> {
    let size = source.len().map_err(source_error_to_string)?;
    Ok(ViewerBinaryInfo {
        path: source_id,
        size,
        max_inline_bytes: MAX_INLINE_BINARY_BASE64_BYTES,
        supports_range_reads: true,
    })
}

fn validate_binary_base64_chunk_size(size: usize) -> Result<(), String> {
    if size == 0 {
        return Err("Binary chunk request size must be greater than zero".to_string());
    }
    if size > MAX_BINARY_BASE64_CHUNK_BYTES {
        return Err(format!(
            "Binary chunk request is too large: {size} bytes > {MAX_BINARY_BASE64_CHUNK_BYTES} bytes"
        ));
    }
    Ok(())
}

fn read_binary_base64_chunk_for_source(
    source_id: String,
    source: &dyn EvidenceByteSource,
    offset: u64,
    size: usize,
) -> Result<ViewerBinaryBase64Chunk, String> {
    validate_binary_base64_chunk_size(size)?;
    let total_size = source.len().map_err(source_error_to_string)?;
    if offset > total_size {
        return Err(format!(
            "Binary chunk offset is beyond EOF for {source_id}: offset {offset} > size {total_size}"
        ));
    }
    let actual_offset = offset;
    let remaining = total_size - actual_offset;
    let read_size = usize::try_from(remaining.min(size as u64))
        .map_err(|_| "Binary chunk read size does not fit this platform".to_string())?;
    let data = if read_size == 0 {
        Vec::new()
    } else {
        read_range_fully(source, actual_offset, read_size).map_err(|e| {
            binary_chunk_source_error_to_string(e, &source_id, total_size, actual_offset, read_size)
        })?
    };
    let bytes_read = data.len();
    let eof = actual_offset.saturating_add(bytes_read as u64) >= total_size;

    Ok(ViewerBinaryBase64Chunk {
        path: source_id,
        offset: actual_offset,
        bytes_read,
        total_size,
        eof,
        data: STANDARD.encode(&data),
    })
}

fn binary_chunk_source_error_to_string(
    error: EvidenceSourceError,
    source_id: &str,
    total_size: u64,
    actual_offset: u64,
    read_size: usize,
) -> String {
    match error {
        EvidenceSourceError::ShortRead { actual, .. } => format!(
            "Short binary chunk read for {source_id}: source reported {total_size} bytes but read {actual} of {read_size} requested bytes at offset {actual_offset}"
        ),
        EvidenceSourceError::OversizedRead { actual, .. } => format!(
            "Binary chunk source returned too many bytes: requested {read_size}, received {actual}"
        ),
        other => source_error_to_string(other),
    }
}

/// Read a chunk of a file for hex viewing
#[tauri::command]
pub async fn viewer_read_chunk(
    path: String,
    offset: u64,
    size: Option<usize>,
) -> Result<viewer::FileChunk, String> {
    tauri::async_runtime::spawn_blocking(move || {
        viewer::read_file_chunk(&path, offset, size).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Failed to join file chunk read task: {e}"))?
}

/// Detect file type from magic bytes and extension
#[tauri::command]
pub async fn viewer_detect_type(path: String) -> Result<viewer::FileTypeInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        viewer::detect_file_type(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Failed to join file type detection task: {e}"))?
}

/// Detect source type from magic bytes and source extension.
#[tauri::command]
pub async fn viewer_detect_type_source(
    source: HashSourceInput,
) -> Result<viewer::FileTypeInfo, String> {
    let extension = source_extension(&source);
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        detect_type_for_source(byte_source.as_ref(), &extension)
    })
    .await
    .map_err(|e| format!("Failed to join source type detection task: {e}"))?
}

fn detect_type_for_source(
    source: &dyn EvidenceByteSource,
    extension: &str,
) -> Result<viewer::FileTypeInfo, String> {
    let total_size = source.len().map_err(source_error_to_string)?;
    let read_size = total_size.min(32) as usize;
    let magic = if read_size == 0 {
        Vec::new()
    } else {
        read_range_fully(source, 0, read_size).map_err(source_error_to_string)?
    };
    Ok(viewer::detect_file_type_bytes(&magic, extension))
}

/// Parse file header and extract metadata with regions for hex highlighting
#[tauri::command]
pub async fn viewer_parse_header(path: String) -> Result<viewer::ParsedMetadata, String> {
    tauri::async_runtime::spawn_blocking(move || {
        viewer::parse_file_header(&path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Failed to join file header parse task: {e}"))?
}

/// Parse source header and extract metadata with regions for hex highlighting.
#[tauri::command]
pub async fn viewer_parse_header_source(
    source: HashSourceInput,
) -> Result<viewer::ParsedMetadata, String> {
    let extension = source_extension(&source);
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        parse_header_for_source(byte_source.as_ref(), &extension)
    })
    .await
    .map_err(|e| format!("Failed to join source header parse task: {e}"))?
}

fn parse_header_for_source(
    source: &dyn EvidenceByteSource,
    extension: &str,
) -> Result<viewer::ParsedMetadata, String> {
    let total_size = source.len().map_err(source_error_to_string)?;
    let read_size = total_size.min(512) as usize;
    let header = if read_size == 0 {
        Vec::new()
    } else {
        read_range_fully(source, 0, read_size).map_err(source_error_to_string)?
    };
    viewer::parse_file_header_bytes(&header, extension, total_size).map_err(|e| e.to_string())
}

/// Analyze a local file for hex/data review.
#[tauri::command]
pub async fn viewer_analyze_path(
    path: String,
    options: Option<SourceAnalysisOptions>,
) -> Result<SourceAnalysis, String> {
    let source = HashSourceInput {
        path: Some(path),
        container_path: None,
        entry_path: None,
        nested_archive_path: None,
        container_type: Some("disk".to_string()),
        size: None,
        data_addr: None,
        item_addr: None,
    };
    viewer_analyze_source(source, options).await
}

/// Analyze a local file or supported container entry through the shared
/// byte-source layer.
#[tauri::command]
pub async fn viewer_analyze_source(
    source: HashSourceInput,
    options: Option<SourceAnalysisOptions>,
) -> Result<SourceAnalysis, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        analyze_byte_source(byte_source.as_ref(), options.unwrap_or_default())
            .map_err(source_error_to_string)
    })
    .await
    .map_err(|e| format!("Failed to join source analysis task: {e}"))?
}

/// Read file as text for text viewer
#[tauri::command]
pub async fn viewer_read_text(
    path: String,
    offset: u64,
    max_chars: usize,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        viewer::read_file_text(&path, offset, max_chars).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Failed to join text read task: {e}"))?
}

/// Get binary viewer metadata before choosing full or ranged reads.
#[tauri::command]
pub async fn viewer_get_binary_info(path: String) -> Result<ViewerBinaryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = LocalFileByteSource::new(&path);
        binary_info_for_source(path, &source)
    })
    .await
    .map_err(|e| format!("Failed to join binary info task: {e}"))?
}

/// Get binary viewer metadata for a local file or supported container entry.
#[tauri::command]
pub async fn viewer_get_binary_info_source(
    source: HashSourceInput,
) -> Result<ViewerBinaryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        binary_info_for_source(source_id, byte_source.as_ref())
    })
    .await
    .map_err(|e| format!("Failed to join source info task: {e}"))?
}

/// Read entire file as base64 for PDF/binary viewing
/// Returns the file content as a base64-encoded string
#[tauri::command]
pub async fn viewer_read_binary_base64(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = LocalFileByteSource::new(&path);
        let data = read_all_with_limit(&source, MAX_INLINE_BINARY_BASE64_BYTES)
            .map_err(source_error_to_string)?;
        Ok(STANDARD.encode(&data))
    })
    .await
    .map_err(|e| format!("Failed to join binary read task: {e}"))?
}

/// Read a local file or supported container entry as base64 for image/PDF/binary viewing.
#[tauri::command]
pub async fn viewer_read_binary_source_base64(source: HashSourceInput) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let data = read_all_with_limit(byte_source.as_ref(), MAX_INLINE_BINARY_BASE64_BYTES)
            .map_err(source_error_to_string)?;
        Ok(STANDARD.encode(&data))
    })
    .await
    .map_err(|e| format!("Failed to join source read task: {e}"))?
}

/// Read a file range as base64 for large binary/PDF viewers.
#[tauri::command]
pub async fn viewer_read_binary_base64_chunk(
    path: String,
    offset: u64,
    size: usize,
) -> Result<ViewerBinaryBase64Chunk, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = LocalFileByteSource::new(&path);
        read_binary_base64_chunk_for_source(path, &source, offset, size)
    })
    .await
    .map_err(|e| format!("Failed to join binary chunk read task: {e}"))?
}

/// Read a source range as base64 for large binary/PDF viewers.
#[tauri::command]
pub async fn viewer_read_binary_source_base64_chunk(
    source: HashSourceInput,
    offset: u64,
    size: usize,
) -> Result<ViewerBinaryBase64Chunk, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        read_binary_base64_chunk_for_source(source_id, byte_source.as_ref(), offset, size)
    })
    .await
    .map_err(|e| format!("Failed to join source chunk read task: {e}"))?
}

fn source_error_to_string(error: EvidenceSourceError) -> String {
    match error {
        EvidenceSourceError::TooLarge { size, max_size } => format!(
            "File is too large for inline binary loading ({size} bytes > {max_size} bytes). Use ranged binary reads instead."
        ),
        other => other.to_string(),
    }
}

fn source_extension(source: &HashSourceInput) -> String {
    let candidate = source
        .entry_path
        .as_deref()
        .or(source.path.as_deref())
        .or(source.nested_archive_path.as_deref())
        .or(source.container_path.as_deref())
        .unwrap_or_default();

    Path::new(candidate)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EvidenceSourceRef, EvidenceSourceResult};

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
            let start = usize::try_from(offset).unwrap_or(usize::MAX);
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let requested = size.min(self.max_chunk);
            let end = start.saturating_add(requested).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    struct OverreturningByteSource {
        source_ref: EvidenceSourceRef,
        len: u64,
    }

    impl OverreturningByteSource {
        fn new(path: &str, len: u64) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                len,
            }
        }
    }

    struct EmptyBeforeEofByteSource {
        source_ref: EvidenceSourceRef,
        len: u64,
    }

    impl EmptyBeforeEofByteSource {
        fn new(path: &str, len: u64) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                len,
            }
        }
    }

    impl EvidenceByteSource for EmptyBeforeEofByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.len)
        }

        fn read_range(&self, _offset: u64, _size: usize) -> EvidenceSourceResult<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    impl EvidenceByteSource for OverreturningByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.len)
        }

        fn read_range(&self, _offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            Ok(vec![0u8; size.saturating_add(1)])
        }
    }

    #[test]
    fn detect_type_for_source_assembles_chunked_magic() {
        let source = ChunkedByteSource::new("case.ad1", b"ADSEGMEN\x00extra", 3);

        let info = detect_type_for_source(&source, "ad1").unwrap();

        assert_eq!(info.description, "AD1 Forensic Container");
        assert!(info.is_forensic_format);
        assert_eq!(info.magic_hex, "41 44 53 45 47 4D 45 4E 00 65 78 74 72 61");
    }

    #[test]
    fn parse_header_for_source_assembles_chunked_header() {
        let source = ChunkedByteSource::new("archive.zip", b"PK\x03\x04chunked zip data", 2);

        let metadata = parse_header_for_source(&source, "zip").unwrap();

        assert_eq!(metadata.format, "ZIP Archive");
        assert!(metadata
            .regions
            .iter()
            .any(|region| region.name == "Signature"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_reads_bounded_chunk() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let chunk =
            read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 2, 3).unwrap();

        assert_eq!(chunk.path, "sample.bin");
        assert_eq!(chunk.offset, 2);
        assert_eq!(chunk.bytes_read, 3);
        assert_eq!(chunk.total_size, 6);
        assert!(!chunk.eof);
        assert_eq!(chunk.data, STANDARD.encode(b"cde"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_assembles_chunked_source() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 2);

        let chunk =
            read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 1, 5).unwrap();

        assert_eq!(chunk.offset, 1);
        assert_eq!(chunk.bytes_read, 5);
        assert!(chunk.eof);
        assert_eq!(chunk.data, STANDARD.encode(b"bcdef"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_marks_eof() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let chunk =
            read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 4, 8).unwrap();

        assert_eq!(chunk.bytes_read, 2);
        assert!(chunk.eof);
        assert_eq!(chunk.data, STANDARD.encode(b"ef"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_rejects_offset_past_eof() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let err =
            match read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 99, 8) {
                Ok(_) => panic!("expected offset past EOF to fail"),
                Err(err) => err,
            };

        assert!(err.contains("offset 99 > size 6"), "unexpected: {err}");
    }

    #[test]
    fn read_binary_base64_chunk_for_source_allows_offset_at_eof() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let chunk =
            read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 6, 8).unwrap();

        assert_eq!(chunk.offset, 6);
        assert_eq!(chunk.bytes_read, 0);
        assert_eq!(chunk.total_size, 6);
        assert!(chunk.eof);
        assert_eq!(chunk.data, "");
    }

    #[test]
    fn read_binary_base64_chunk_for_source_rejects_oversize_request() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let result = read_binary_base64_chunk_for_source(
            "sample.bin".to_string(),
            &source,
            0,
            MAX_BINARY_BASE64_CHUNK_BYTES + 1,
        );
        let err = match result {
            Ok(_) => panic!("expected oversized binary chunk request to fail"),
            Err(err) => err,
        };

        assert!(err.contains("Binary chunk request is too large"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_rejects_zero_size_request() {
        let source = ChunkedByteSource::new("sample.bin", b"abcdef", 16);

        let err = match read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 0, 0)
        {
            Ok(_) => panic!("expected zero-size binary chunk request to fail"),
            Err(err) => err,
        };

        assert!(err.contains("greater than zero"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_rejects_oversized_source_return() {
        let source = OverreturningByteSource::new("sample.bin", 6);

        let err = match read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 0, 3)
        {
            Ok(_) => panic!("expected oversized source return to fail"),
            Err(err) => err,
        };

        assert!(err.contains("Binary chunk source returned too many bytes"));
    }

    #[test]
    fn read_binary_base64_chunk_for_source_rejects_empty_read_before_eof() {
        let source = EmptyBeforeEofByteSource::new("sample.bin", 6);

        let err = match read_binary_base64_chunk_for_source("sample.bin".to_string(), &source, 2, 3)
        {
            Ok(_) => panic!("expected empty source read before EOF to fail"),
            Err(err) => err,
        };

        assert!(err.contains("Short binary chunk read for sample.bin"));
        assert!(err.contains("offset 2"));
    }
}
