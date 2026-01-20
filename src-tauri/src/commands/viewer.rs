// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File viewer commands for hex/text viewing.

use crate::viewer;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;

/// Read a chunk of a file for hex viewing
#[tauri::command]
pub fn viewer_read_chunk(
    path: String, 
    offset: u64, 
    size: Option<usize>
) -> Result<viewer::FileChunk, String> {
    viewer::read_file_chunk(&path, offset, size).map_err(|e| e.to_string())
}

/// Detect file type from magic bytes and extension
#[tauri::command]
pub fn viewer_detect_type(path: String) -> Result<viewer::FileTypeInfo, String> {
    viewer::detect_file_type(&path).map_err(|e| e.to_string())
}

/// Parse file header and extract metadata with regions for hex highlighting
#[tauri::command]
pub fn viewer_parse_header(path: String) -> Result<viewer::ParsedMetadata, String> {
    viewer::parse_file_header(&path).map_err(|e| e.to_string())
}

/// Read file as text for text viewer
#[tauri::command]
pub fn viewer_read_text(path: String, offset: u64, max_chars: usize) -> Result<String, String> {
    viewer::read_file_text(&path, offset, max_chars).map_err(|e| e.to_string())
}

/// Read entire file as base64 for PDF/binary viewing
/// Returns the file content as a base64-encoded string
#[tauri::command]
pub fn viewer_read_binary_base64(path: String) -> Result<String, String> {
    let data = fs::read(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(STANDARD.encode(&data))
}
