// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Data viewing and raw byte reading commands.

const READ_FILE_BYTES_MAX_LENGTH: usize = 16 * 1024 * 1024;

/// Read raw bytes from a file at specified offset
///
/// Returns up to `length` bytes starting at `offset`.
/// Useful for previewing file contents without full extraction.
#[tauri::command]
pub fn read_file_bytes(path: String, offset: u64, length: usize) -> Result<Vec<u8>, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    if length > READ_FILE_BYTES_MAX_LENGTH {
        return Err(format!(
            "Requested byte range is too large: {} bytes > {} bytes",
            length, READ_FILE_BYTES_MAX_LENGTH
        ));
    }

    let mut file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    let file_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();

    if offset >= file_size {
        return Ok(Vec::new());
    }

    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Failed to seek: {}", e))?;

    let Some(read_len) = bounded_file_read_len(file_size, offset, length) else {
        return Ok(Vec::new());
    };
    let mut buffer = vec![0u8; read_len];

    file.read_exact(&mut buffer)
        .map_err(|e| format!("Failed to read: {}", e))?;

    Ok(buffer)
}

fn bounded_file_read_len(file_size: u64, offset: u64, requested_len: usize) -> Option<usize> {
    let remaining = file_size.checked_sub(offset)?;
    if remaining == 0 {
        return None;
    }
    let remaining = usize::try_from(remaining).unwrap_or(usize::MAX);
    Some(requested_len.min(remaining))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_file_read_len_rejects_eof() {
        assert_eq!(bounded_file_read_len(10, 10, 4), None);
    }

    #[test]
    fn test_bounded_file_read_len_clamps_to_remaining() {
        assert_eq!(bounded_file_read_len(10, 8, 5), Some(2));
    }

    #[test]
    fn test_bounded_file_read_len_handles_large_remaining() {
        assert_eq!(bounded_file_read_len(u64::MAX, 0, 8), Some(8));
    }

    #[test]
    fn test_read_file_bytes_rejects_oversized_request() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abcdef").unwrap();

        let err = read_file_bytes(
            path.to_string_lossy().to_string(),
            0,
            READ_FILE_BYTES_MAX_LENGTH + 1,
        )
        .unwrap_err();
        assert!(err.contains("Requested byte range is too large"));
    }

    #[test]
    fn test_read_file_bytes_rejects_oversized_request_even_past_eof() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abcdef").unwrap();

        let err = read_file_bytes(
            path.to_string_lossy().to_string(),
            99,
            READ_FILE_BYTES_MAX_LENGTH + 1,
        )
        .unwrap_err();
        assert!(err.contains("Requested byte range is too large"));
    }

    #[test]
    fn test_read_file_bytes_reads_partial_tail() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("sample.bin");
        std::fs::write(&path, b"abcdef").unwrap();

        let bytes = read_file_bytes(path.to_string_lossy().to_string(), 4, 16).unwrap();
        assert_eq!(bytes, b"ef");
    }
}
