// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Data viewing and raw byte reading commands.

/// Read raw bytes from a file at specified offset
/// 
/// Returns up to `length` bytes starting at `offset`.
/// Useful for previewing file contents without full extraction.
#[tauri::command]
pub fn read_file_bytes(
    path: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    
    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    if offset >= file_size {
        return Ok(Vec::new());
    }
    
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Failed to seek: {}", e))?;
    
    let read_len = length.min((file_size - offset) as usize);
    let mut buffer = vec![0u8; read_len];
    
    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    Ok(buffer)
}
