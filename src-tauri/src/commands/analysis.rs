// =============================================================================
// CORE-FFX - Data Analysis Commands
// =============================================================================

//! Data viewing, hex dump, and entropy analysis commands.

use crate::common;
use crate::containers;

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

/// Get hex dump of file contents
///
/// Reads bytes from file and returns formatted hex dump string.
#[tauri::command]
pub fn hex_dump(
    path: String,
    offset: u64,
    length: usize,
    #[allow(non_snake_case)]
    showAscii: Option<bool>,
    #[allow(non_snake_case)]
    bytesPerLine: Option<usize>,
) -> Result<common::hex::HexDumpResult, String> {
    let data = read_file_bytes(path, offset, length)?;
    
    let options = common::hex::HexDumpOptions {
        show_ascii: showAscii.unwrap_or(true),
        bytes_per_line: bytesPerLine.unwrap_or(16),
        show_offset: true,
        uppercase: true,
        group_size: 1,
        start_offset: offset,
    };
    
    Ok(common::hex::create_hex_dump(&data, &options))
}

/// Detect file type from magic signature
///
/// Reads file header and identifies type based on magic bytes.
/// Returns None if file type cannot be determined.
#[tauri::command]
pub fn detect_file_type(path: String) -> Result<Option<common::magic::FileType>, String> {
    // Read first 64 bytes for magic detection
    let header = read_file_bytes(path, 0, 64)?;
    Ok(common::magic::detect_file_type(&header))
}

/// Analyze entropy of a file or portion of a file
///
/// Returns entropy statistics useful for detecting encryption.
#[tauri::command]
pub fn analyze_file_entropy(
    path: String,
    offset: Option<u64>,
    length: Option<usize>,
) -> Result<common::entropy::EntropyResult, String> {
    let start = offset.unwrap_or(0);
    // Default to 1MB sample for entropy analysis
    let len = length.unwrap_or(1024 * 1024);
    
    let data = read_file_bytes(path, start, len)?;
    
    Ok(common::entropy::EntropyResult::new(&data)
        .with_offset(start))
}

/// Analyze entropy across file blocks
///
/// Useful for finding encrypted regions in disk images.
#[tauri::command]
pub fn analyze_entropy_blocks(
    path: String,
    #[allow(non_snake_case)]
    blockSize: Option<usize>,
    #[allow(non_snake_case)]
    maxBlocks: Option<usize>,
) -> Result<common::entropy::BlockEntropyAnalysis, String> {
    use std::fs::File;
    use std::io::Read;
    
    let block_size = blockSize.unwrap_or(4096);
    let max_blocks = maxBlocks.unwrap_or(1000);
    
    let mut file = File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let file_size = file.metadata()
        .map_err(|e| format!("Failed to get file size: {}", e))?
        .len();
    
    // Limit data read to prevent memory issues
    let max_bytes = block_size * max_blocks;
    let read_len = (file_size as usize).min(max_bytes);
    
    let mut data = vec![0u8; read_len];
    file.read_exact(&mut data)
        .map_err(|e| format!("Failed to read: {}", e))?;
    
    Ok(common::entropy::analyze_blocks(&data, block_size))
}

/// Compare two hash values
///
/// Returns detailed comparison result including case-sensitivity.
#[tauri::command]
pub fn compare_hashes(
    hash1: String,
    hash2: String,
) -> common::hash::HashMatchResult {
    common::hash::compare_hashes(&hash1, &hash2)
}

/// Verify a file's hash against expected value
#[tauri::command]
pub fn verify_file_hash(
    path: String,
    expected: String,
    algorithm: String,
) -> Result<common::hash::HashVerificationResult, String> {
    let algo: common::hash::HashAlgorithm = algorithm.parse().map_err(|e: containers::ContainerError| e.to_string())?;
    common::hash::verify_file_hash(std::path::Path::new(&path), &expected, algo)
        .map_err(|e| e.to_string())
}
