// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File info extraction and content reading utilities.
//!
//! All operations are read-only — forensic integrity is preserved.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;

use super::{UniversalFormat, ViewerType};
use crate::viewer::document::error::{DocumentError, DocumentResult};

const MAX_DATA_URL_SOURCE_BYTES: u64 = 100 * 1024 * 1024;

fn read_limited_prefix<R: Read>(reader: R, max_bytes: u64) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(max_bytes).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn read_with_limit<R: Read>(reader: R, max_bytes: u64) -> std::io::Result<(Vec<u8>, bool)> {
    let read_limit = max_bytes.saturating_add(1);
    let mut bytes = read_limited_prefix(reader, read_limit)?;
    let truncated = bytes.len() as u64 > max_bytes;
    if truncated {
        bytes.truncate(max_bytes as usize);
    }
    Ok((bytes, truncated))
}

fn ensure_data_url_size_allowed(size: u64) -> DocumentResult<()> {
    if size > MAX_DATA_URL_SOURCE_BYTES {
        return Err(DocumentError::Parse(format!(
            "File too large for data URL rendering ({:.1} MiB, max {} MiB)",
            size as f64 / (1024.0 * 1024.0),
            MAX_DATA_URL_SOURCE_BYTES / (1024 * 1024)
        )));
    }
    Ok(())
}

fn truncate_to_utf8_boundary(bytes: &mut Vec<u8>) {
    if let Err(err) = std::str::from_utf8(bytes) {
        if err.error_len().is_none() {
            bytes.truncate(err.valid_up_to());
        }
    }
}

// =============================================================================
// FILE INFO (READ-ONLY)
// =============================================================================

/// File information (read-only metadata extraction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub format: UniversalFormat,
    pub viewer_type: ViewerType,
    pub mime_type: String,
    pub description: String,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub is_readable: bool,
    pub is_binary: bool,
}

impl FileInfo {
    /// Get file info without reading content (fast)
    pub fn from_path(path: impl AsRef<Path>) -> DocumentResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(DocumentError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )));
        }

        let meta = fs::metadata(path)?;
        let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);

        // Check if binary by reading first bytes
        let is_binary = Self::check_binary(path);

        Ok(Self {
            path: path.to_string_lossy().to_string(),
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            format,
            viewer_type: format.viewer_type(),
            mime_type: format.mime_type().to_string(),
            description: format.description().to_string(),
            size: meta.len(),
            created: meta
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format_timestamp(d.as_secs())),
            modified: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format_timestamp(d.as_secs())),
            is_readable: path.is_file(),
            is_binary,
        })
    }

    /// Quick check if file appears to be binary
    fn check_binary(path: &Path) -> bool {
        if let Ok(mut file) = fs::File::open(path) {
            let mut buffer = [0u8; 8192];
            if let Ok(n) = file.read(&mut buffer) {
                // Count null bytes and non-printable chars
                let null_count = buffer[..n].iter().filter(|&&b| b == 0).count();
                let non_printable = buffer[..n]
                    .iter()
                    .filter(|&&b| b < 0x09 || (b > 0x0D && b < 0x20 && b != 0x1B))
                    .count();

                // If more than 10% null or non-printable, likely binary
                return null_count > n / 10 || non_printable > n / 10;
            }
        }
        true // Default to binary if can't read
    }
}

fn format_timestamp(secs: u64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

// =============================================================================
// CONTENT READING (READ-ONLY)
// =============================================================================

/// Read file as base64 data URL (for images)
pub fn read_as_data_url(path: impl AsRef<Path>) -> DocumentResult<String> {
    let path = path.as_ref();
    ensure_data_url_size_allowed(fs::metadata(path)?.len())?;
    let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);
    let file = fs::File::open(path)?;
    let (data, exceeded_limit) = read_with_limit(file, MAX_DATA_URL_SOURCE_BYTES)?;
    if exceeded_limit {
        return Err(DocumentError::Parse(format!(
            "File too large for data URL rendering (actual read exceeded max {} MiB)",
            MAX_DATA_URL_SOURCE_BYTES / (1024 * 1024)
        )));
    }
    let mime = format.mime_type();
    Ok(format!("data:{};base64,{}", mime, BASE64.encode(&data)))
}

/// Read file as text (with size limit)
pub fn read_as_text(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(String, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    let file = fs::File::open(path)?;
    let (mut buffer, read_truncated) = read_with_limit(file, max_bytes as u64)?;
    truncate_to_utf8_boundary(&mut buffer);
    let text = String::from_utf8_lossy(&buffer).to_string();
    Ok((text, meta.len() > max_bytes as u64 || read_truncated))
}

/// Read file bytes (with size limit)
pub fn read_bytes(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(Vec<u8>, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    let file = fs::File::open(path)?;
    let (data, read_truncated) = read_with_limit(file, max_bytes as u64)?;
    Ok((data, meta.len() > max_bytes as u64 || read_truncated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn read_as_data_url_encodes_small_file() {
        let mut file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        file.write_all(b"png-bytes").unwrap();

        let data_url = read_as_data_url(file.path()).unwrap();

        assert!(data_url.starts_with("data:image/png;base64,"));
        assert!(data_url.ends_with("cG5nLWJ5dGVz"));
    }

    #[test]
    fn read_as_data_url_rejects_sparse_oversized_file() {
        let file = tempfile::Builder::new().suffix(".jpg").tempfile().unwrap();
        file.as_file()
            .set_len(MAX_DATA_URL_SOURCE_BYTES + 1)
            .unwrap();

        let err = read_as_data_url(file.path()).unwrap_err();

        assert!(err.to_string().contains("File too large for data URL"));
    }

    #[test]
    fn read_with_limit_reports_truncation_from_actual_bytes_read() {
        let (bytes, truncated) = read_with_limit(&b"abcdef"[..], 3).unwrap();

        assert_eq!(bytes, b"abc");
        assert!(truncated);
    }

    #[test]
    fn read_as_text_enforces_actual_read_limit() {
        let mut file = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
        file.write_all("abcdef".as_bytes()).unwrap();

        let (text, truncated) = read_as_text(file.path(), 3).unwrap();

        assert_eq!(text, "abc");
        assert!(truncated);
    }

    #[test]
    fn read_bytes_enforces_actual_read_limit() {
        let mut file = tempfile::Builder::new().suffix(".bin").tempfile().unwrap();
        file.write_all(b"abcdef").unwrap();

        let (bytes, truncated) = read_bytes(file.path(), 3).unwrap();

        assert_eq!(bytes, b"abc");
        assert!(truncated);
    }
}
