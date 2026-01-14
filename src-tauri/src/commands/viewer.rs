// =============================================================================
// CORE-FFX - Viewer Commands
// =============================================================================

//! File viewer commands for hex/text viewing.

use crate::viewer;

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
