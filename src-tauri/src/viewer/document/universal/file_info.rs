// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File info extraction and content reading utilities.
//!
//! All operations are read-only — forensic integrity is preserved.

use std::path::Path;
use std::fs;
use std::io::Read;
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::viewer::document::error::{DocumentError, DocumentResult};
use super::{UniversalFormat, ViewerType};

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
                format!("File not found: {}", path.display())
            )));
        }
        
        let meta = fs::metadata(path)?;
        let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);
        
        // Check if binary by reading first bytes
        let is_binary = Self::check_binary(path);
        
        Ok(Self {
            path: path.to_string_lossy().to_string(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            format,
            viewer_type: format.viewer_type(),
            mime_type: format.mime_type().to_string(),
            description: format.description().to_string(),
            size: meta.len(),
            created: meta.created().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| format_timestamp(d.as_secs())),
            modified: meta.modified().ok()
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
                let non_printable = buffer[..n].iter()
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
    let format = UniversalFormat::from_path(path).unwrap_or(UniversalFormat::Binary);
    let data = fs::read(path)?;
    let mime = format.mime_type();
    Ok(format!("data:{};base64,{}", mime, BASE64.encode(&data)))
}

/// Read file as text (with size limit)
pub fn read_as_text(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(String, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    
    let truncated = meta.len() > max_bytes as u64;
    
    if truncated {
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; max_bytes];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);
        let text = String::from_utf8_lossy(&buffer).to_string();
        Ok((text, true))
    } else {
        let text = fs::read_to_string(path)?;
        Ok((text, false))
    }
}

/// Read file bytes (with size limit)
pub fn read_bytes(path: impl AsRef<Path>, max_bytes: usize) -> DocumentResult<(Vec<u8>, bool)> {
    let path = path.as_ref();
    let meta = fs::metadata(path)?;
    
    let truncated = meta.len() > max_bytes as u64;
    
    if truncated {
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0u8; max_bytes];
        let n = file.read(&mut buffer)?;
        buffer.truncate(n);
        Ok((buffer, true))
    } else {
        let data = fs::read(path)?;
        Ok((data, false))
    }
}
