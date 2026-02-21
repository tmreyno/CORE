//! Advanced archive features: multi-volume, raw LZMA, and enhanced error reporting
//!
//! This module provides advanced functionality for:
//! - Split/multi-volume archives for easier transfer and storage
//! - Raw LZMA/LZMA2 compression for .lzma and .xz files
//! - Detailed error reporting with context and suggestions

use crate::error::{Error, Result};
use crate::ffi;
use crate::CompressionLevel;
use std::ffi::{CString, CStr};
use std::path::Path;
use std::os::raw::c_char;

/// Detailed error information with context and actionable suggestions
#[derive(Debug, Clone)]
pub struct DetailedError {
    /// Error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// File being processed when error occurred
    pub file_context: String,
    /// Position in file/archive (-1 if not applicable)
    pub position: i64,
    /// Actionable suggestion to fix the error
    pub suggestion: String,
}

impl DetailedError {
    /// Get the last error from the C library with full context
    pub fn get_last() -> Result<Self> {
        let mut error_info = ffi::SevenZipErrorInfo {
            code: ffi::SevenZipErrorCode::SEVENZIP_OK,
            message: [0; 512],
            file_context: [0; 256],
            position: -1,
            suggestion: [0; 256],
        };
        
        unsafe {
            let result = ffi::sevenzip_get_last_error(&mut error_info);
            if result != ffi::SevenZipErrorCode::SEVENZIP_OK {
                return Err(Error::from_code(result));
            }
            
            let message = CStr::from_ptr(error_info.message.as_ptr())
                .to_string_lossy()
                .to_string();
            
            let file_context = CStr::from_ptr(error_info.file_context.as_ptr())
                .to_string_lossy()
                .to_string();
            
            let suggestion = CStr::from_ptr(error_info.suggestion.as_ptr())
                .to_string_lossy()
                .to_string();
            
            Ok(DetailedError {
                code: error_info.code as i32,
                message,
                file_context,
                position: error_info.position,
                suggestion,
            })
        }
    }
    
    /// Clear the last error in the C library
    pub fn clear() {
        unsafe {
            ffi::sevenzip_clear_last_error();
        }
    }
}

/// Get a human-readable error message for an error code
pub fn get_error_string(code: i32) -> String {
    let error_code = match code {
        0 => ffi::SevenZipErrorCode::SEVENZIP_OK,
        1 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_OPEN_FILE,
        2 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_INVALID_ARCHIVE,
        3 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_MEMORY,
        4 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_EXTRACT,
        5 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_COMPRESS,
        6 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_INVALID_PARAM,
        7 => ffi::SevenZipErrorCode::SEVENZIP_ERROR_NOT_IMPLEMENTED,
        _ => ffi::SevenZipErrorCode::SEVENZIP_ERROR_UNKNOWN,
    };
    
    unsafe {
        let c_str = ffi::sevenzip_get_error_string(error_code);
        CStr::from_ptr(c_str).to_string_lossy().to_string()
    }
}

/// Get the library version string
pub fn get_version() -> String {
    unsafe {
        let c_str = ffi::sevenzip_get_version();
        CStr::from_ptr(c_str).to_string_lossy().to_string()
    }
}

// ============================================================================
// Multi-Volume (Split) Archives
// ============================================================================

/// Create a multi-volume (split) 7z archive
///
/// This creates an archive split into multiple files of the specified size,
/// useful for transfer limits, DVD/Blu-ray burning, or cloud storage limits.
///
/// # Arguments
///
/// * `archive_path` - Base path for the archive (e.g., "backup.7z")
///                    Creates: backup.7z.001, backup.7z.002, etc.
/// * `input_paths` - Files and directories to compress
/// * `level` - Compression level
/// * `volume_size` - Size of each volume in bytes
///                   Common sizes:
///                   - 1_474_560 (1.44MB floppy)
///                   - 734_003_200 (700MB CD)
///                   - 4_700_372_992 (4.7GB DVD)
///                   - 4_294_967_296 (4GB FAT32 limit)
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
/// use seven_zip::CompressionLevel;
///
/// // Create 4GB volumes
/// advanced::create_split_archive(
///     "large_backup.7z",
///     &["data/"],
///     CompressionLevel::Normal,
///     4_294_967_296, // 4GB
///     None,
/// )?;
/// // Creates: large_backup.7z.001, large_backup.7z.002, ...
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn create_split_archive(
    archive_path: impl AsRef<Path>,
    input_paths: &[impl AsRef<Path>],
    level: CompressionLevel,
    volume_size: u64,
    password: Option<&str>,
) -> Result<()> {
    let archive_path = archive_path.as_ref().to_str()
        .ok_or_else(|| Error::Io("Invalid path encoding".to_string()))?;
    let c_archive = CString::new(archive_path)?;
    
    // Convert input paths to C strings
    let c_paths: Result<Vec<CString>> = input_paths
        .iter()
        .map(|p| {
            let path_str = p.as_ref().to_str()
                .ok_or_else(|| Error::Io("Invalid path encoding".to_string()))?;
            CString::new(path_str).map_err(|e| Error::Io(e.to_string()))
        })
        .collect();
    let c_paths = c_paths?;
    
    // Create null-terminated array of pointers
    let mut c_path_ptrs: Vec<*const c_char> = c_paths
        .iter()
        .map(|s| s.as_ptr())
        .collect();
    c_path_ptrs.push(std::ptr::null());
    
    // Setup compression options
    let c_password = password
        .map(|p| CString::new(p))
        .transpose()?;
    
    let c_options = ffi::SevenZipCompressOptions {
        num_threads: 0, // auto
        dict_size: 0,   // auto
        solid: 1,       // solid archive
        password: c_password.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
    };
    
    unsafe {
        let result = ffi::sevenzip_create_multivolume_7z(
            c_archive.as_ptr(),
            c_path_ptrs.as_ptr(),
            level.into(),
            volume_size,
            &c_options,
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

/// Extract a split/multi-volume archive
///
/// Automatically handles archives split across multiple files.
///
/// # Arguments
///
/// * `archive_path` - Path to any volume (e.g., "backup.7z.001" or "backup.7z")
/// * `output_dir` - Directory to extract files to
/// * `password` - Optional password for encrypted archives
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
///
/// // Extract split archive (automatically finds all volumes)
/// advanced::extract_split_archive(
///     "backup.7z.001",
///     "restored/",
///     None,
/// )?;
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn extract_split_archive(
    archive_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    password: Option<&str>,
) -> Result<()> {
    let archive_path = archive_path.as_ref().to_str()
        .ok_or(Error::Io("Invalid path encoding".to_string()))?;
    let output_dir = output_dir.as_ref().to_str()
        .ok_or(Error::Io("Invalid path encoding".to_string()))?;
    
    let c_archive = CString::new(archive_path)?;
    let c_output = CString::new(output_dir)?;
    let c_password = password
        .map(|p| CString::new(p))
        .transpose()?;
    
    unsafe {
        let result = ffi::sevenzip_extract_split_archive(
            c_archive.as_ptr(),
            c_output.as_ptr(),
            c_password.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

// ============================================================================
// Raw LZMA/LZMA2 Compression
// ============================================================================

/// Compress a file to raw LZMA format (.lzma)
///
/// Creates a standalone LZMA compressed file (not a .7z archive).
/// Compatible with `lzma` command-line tools.
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
/// use seven_zip::CompressionLevel;
///
/// advanced::compress_lzma(
///     "large_file.bin",
///     "large_file.bin.lzma",
///     CompressionLevel::Maximum,
/// )?;
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn compress_lzma(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    level: CompressionLevel,
) -> Result<()> {
    let input = input_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    let output = output_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    
    let c_input = CString::new(input)?;
    let c_output = CString::new(output)?;
    
    unsafe {
        let result = ffi::sevenzip_compress_lzma(
            c_input.as_ptr(),
            c_output.as_ptr(),
            level.into(),
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

/// Decompress a raw LZMA file (.lzma)
///
/// Decompresses a standalone LZMA file.
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
///
/// advanced::decompress_lzma(
///     "file.lzma",
///     "file.bin",
/// )?;
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn decompress_lzma(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let input = input_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    let output = output_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    
    let c_input = CString::new(input)?;
    let c_output = CString::new(output)?;
    
    unsafe {
        let result = ffi::sevenzip_decompress_lzma(
            c_input.as_ptr(),
            c_output.as_ptr(),
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

/// Compress a file to LZMA2 format (.xz)
///
/// Creates a standalone LZMA2/XZ compressed file.
/// Compatible with `xz` command-line tools.
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
/// use seven_zip::CompressionLevel;
///
/// advanced::compress_lzma2(
///     "data.bin",
///     "data.bin.xz",
///     CompressionLevel::Maximum,
/// )?;
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn compress_lzma2(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    level: CompressionLevel,
) -> Result<()> {
    let input = input_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    let output = output_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    
    let c_input = CString::new(input)?;
    let c_output = CString::new(output)?;
    
    unsafe {
        let result = ffi::sevenzip_compress_lzma2(
            c_input.as_ptr(),
            c_output.as_ptr(),
            level.into(),
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

/// Decompress a LZMA2/XZ file (.xz)
///
/// Decompresses a standalone LZMA2 or XZ file.
///
/// # Example
///
/// ```no_run
/// use seven_zip::advanced;
///
/// advanced::decompress_lzma2(
///     "file.xz",
///     "file.bin",
/// )?;
/// # Ok::<(), seven_zip::Error>(())
/// ```
pub fn decompress_lzma2(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let input = input_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    let output = output_path.as_ref().to_str().ok_or(Error::Io("Invalid path encoding".to_string()))?;
    
    let c_input = CString::new(input)?;
    let c_output = CString::new(output)?;
    
    unsafe {
        let result = ffi::sevenzip_decompress_lzma2(
            c_input.as_ptr(),
            c_output.as_ptr(),
            None,
            std::ptr::null_mut(),
        );
        
        if result != ffi::SevenZipErrorCode::SEVENZIP_OK { return Err(Error::from_code(result)); }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }
    
    #[test]
    fn test_get_error_string() {
        let msg = get_error_string(0);
        assert!(!msg.is_empty());
        
        let msg = get_error_string(5);
        assert!(!msg.is_empty());
    }
}
