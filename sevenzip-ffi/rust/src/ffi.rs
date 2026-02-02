//! Complete FFI bindings to the 7z C library
//!
//! This module contains comprehensive low-level FFI declarations matching the full C API,
//! including encryption support. These are not meant to be used directly - use the safe
//! wrappers in the parent modules instead.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_void};

/// Error codes returned by the C API
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SevenZipErrorCode {
    SEVENZIP_OK = 0,
    SEVENZIP_ERROR_OPEN_FILE = 1,
    SEVENZIP_ERROR_INVALID_ARCHIVE = 2,
    SEVENZIP_ERROR_MEMORY = 3,
    SEVENZIP_ERROR_EXTRACT = 4,
    SEVENZIP_ERROR_COMPRESS = 5,
    SEVENZIP_ERROR_INVALID_PARAM = 6,
    SEVENZIP_ERROR_NOT_IMPLEMENTED = 7,
    SEVENZIP_ERROR_UNKNOWN = 99,
}

/// Archive entry information from C API
#[repr(C)]
#[derive(Debug)]
pub struct SevenZipEntry {
    pub name: *mut c_char,
    pub size: u64,
    pub packed_size: u64,
    pub modified_time: u64,
    pub attributes: u32,
    pub is_directory: c_int,
}

/// Archive list result from C API
#[repr(C)]
#[derive(Debug)]
pub struct SevenZipList {
    pub entries: *mut SevenZipEntry,
    pub count: usize,
}

/// Progress callback function type
pub type SevenZipProgressCallback =
    Option<unsafe extern "C" fn(completed: u64, total: u64, user_data: *mut c_void)>;

/// Byte-level progress callback for streaming operations
pub type SevenZipBytesProgressCallback = Option<
    unsafe extern "C" fn(
        bytes_processed: u64,
        bytes_total: u64,
        current_file_bytes: u64,
        current_file_total: u64,
        current_file_name: *const c_char,
        user_data: *mut c_void,
    ),
>;

/// Compression levels
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SevenZipCompressionLevel {
    SEVENZIP_LEVEL_STORE = 0,
    SEVENZIP_LEVEL_FASTEST = 1,
    SEVENZIP_LEVEL_FAST = 3,
    SEVENZIP_LEVEL_NORMAL = 5,
    SEVENZIP_LEVEL_MAXIMUM = 7,
    SEVENZIP_LEVEL_ULTRA = 9,
}

/// Advanced compression options
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SevenZipCompressOptions {
    pub num_threads: c_int,
    pub dict_size: u64,
    pub solid: c_int,
    pub password: *const c_char,
}

/// Streaming compression options for large files and split archives
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SevenZipStreamOptions {
    pub num_threads: c_int,
    pub dict_size: u64,
    pub solid: c_int,
    pub password: *const c_char,
    pub split_size: u64,
    pub chunk_size: u64,
    pub temp_dir: *const c_char,
    pub delete_temp_on_error: c_int,
}

/// AES encryption constants
pub const AES_KEY_SIZE: usize = 32;
pub const AES_BLOCK_SIZE: usize = 16;
pub const AES_NUM_IVMRK_WORDS: usize = (1 + 1 + 15) * 4;

#[link(name = "7z_ffi", kind = "static")]
extern "C" {
    // ============================================================================
    // Core Library Functions
    // ============================================================================
    
    /// Initialize the 7z library
    pub fn sevenzip_init() -> SevenZipErrorCode;
    
    /// Cleanup the 7z library
    pub fn sevenzip_cleanup();

    // ============================================================================
    // Archive Extraction Functions
    // ============================================================================
    
    /// Extract a 7z archive
    pub fn sevenzip_extract(
        archive_path: *const c_char,
        output_dir: *const c_char,
        password: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Extract specific files from a 7z archive
    pub fn sevenzip_extract_files(
        archive_path: *const c_char,
        output_dir: *const c_char,
        files: *const *const c_char,
        password: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Extract a multi-file archive created with sevenzip_create_archive()
    pub fn sevenzip_extract_archive(
        archive_path: *const c_char,
        output_dir: *const c_char,
        password: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Archive Creation Functions
    // ============================================================================
    
    /// Create a 7z archive
    pub fn sevenzip_compress(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        password: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Create a multi-file archive with LZMA2 compression
    pub fn sevenzip_create_archive(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        password: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Create a standard .7z archive (compatible with 7-Zip)
    pub fn sevenzip_create_7z(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        options: *const SevenZipCompressOptions,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Streaming Compression (Large Files & Split Archives)
    // ============================================================================
    
    /// Initialize streaming options with defaults
    pub fn sevenzip_stream_options_init(options: *mut SevenZipStreamOptions);
    
    /// Create a 7z archive with streaming compression (handles large files and splits)
    pub fn sevenzip_create_7z_streaming(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        options: *const SevenZipStreamOptions,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;
    
    /// Create a 7z archive using TRUE streaming (processes in chunks, ~250MB RAM max)
    /// This is the recommended function for large archives (10GB+) to avoid OOM crashes
    pub fn sevenzip_create_7z_true_streaming(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        options: *const SevenZipStreamOptions,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;
    
    /// Extract a 7z archive with streaming decompression and byte-level progress
    pub fn sevenzip_extract_streaming(
        archive_path: *const c_char,
        output_dir: *const c_char,
        password: *const c_char,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Compress files with streaming support and split archives
    pub fn sevenzip_compress_stream(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        options: *const SevenZipStreamOptions,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Resume interrupted compression from checkpoint
    pub fn sevenzip_compress_resume(
        archive_path: *const c_char,
        checkpoint_path: *const c_char,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Archive Inspection Functions
    // ============================================================================
    
    /// List contents of a 7z archive
    pub fn sevenzip_list(
        archive_path: *const c_char,
        password: *const c_char,
        list: *mut *mut SevenZipList,
    ) -> SevenZipErrorCode;

    /// Free memory allocated by sevenzip_list
    pub fn sevenzip_free_list(list: *mut SevenZipList);

    /// Test archive integrity without extracting
    pub fn sevenzip_test_archive(
        archive_path: *const c_char,
        password: *const c_char,
        progress_callback: SevenZipBytesProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Single File Compression/Decompression
    // ============================================================================
    
    /// Compress a single file to LZMA2 format
    pub fn sevenzip_compress_file(
        input_path: *const c_char,
        output_path: *const c_char,
        level: SevenZipCompressionLevel,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    /// Decompress a single LZMA2 file
    pub fn sevenzip_decompress_file(
        input_path: *const c_char,
        output_path: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Encryption Functions (AES-256-CBC)
    // ============================================================================
    
    /// Initialize encryption context with password
    pub fn sevenzip_init_encryption(
        password: *const c_char,
        key: *mut u8,
        iv: *mut u8,
        aes_context: *mut u32,
    ) -> SevenZipErrorCode;

    /// Initialize decryption context with password and salt
    pub fn sevenzip_init_decryption(
        password: *const c_char,
        salt: *const u8,
        salt_len: usize,
        key: *mut u8,
        aes_context: *mut u32,
    ) -> SevenZipErrorCode;

    /// Encrypt data using AES-256-CBC with PKCS#7 padding
    pub fn sevenzip_encrypt_data(
        aes_context: *mut u32,
        iv: *const u8,
        plaintext: *const u8,
        plaintext_len: usize,
        ciphertext: *mut u8,
        ciphertext_len: *mut usize,
    ) -> SevenZipErrorCode;

    /// Decrypt data using AES-256-CBC and verify PKCS#7 padding
    pub fn sevenzip_decrypt_data(
        aes_context: *mut u32,
        iv: *const u8,
        ciphertext: *const u8,
        ciphertext_len: usize,
        plaintext: *mut u8,
        plaintext_len: *mut usize,
    ) -> SevenZipErrorCode;

    /// Verify password correctness by decrypting test block
    pub fn sevenzip_verify_password(
        password: *const c_char,
        encrypted_test_block: *const u8,
        test_block_len: usize,
        salt: *const u8,
        salt_len: usize,
        iv: *const u8,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // LZMA/LZMA2 Raw Decompression
    // ============================================================================
    
    /// Decompress a standalone LZMA file (.lzma)
    pub fn sevenzip_decompress_lzma(
        lzma_path: *const c_char,
        output_path: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;
    
    /// Decompress a standalone LZMA2 file (.xz)
    pub fn sevenzip_decompress_lzma2(
        lzma2_path: *const c_char,
        output_path: *const c_char,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;

    // ============================================================================
    // Multi-Volume (Split) Archives
    // ============================================================================
    
    /// Create a multi-volume 7z archive (splits into multiple files)
    pub fn sevenzip_create_multivolume_7z_complete(
        archive_path: *const c_char,
        input_paths: *const *const c_char,
        level: SevenZipCompressionLevel,
        volume_size: u64,
        options: *const SevenZipCompressOptions,
        progress_callback: SevenZipProgressCallback,
        user_data: *mut c_void,
    ) -> SevenZipErrorCode;
    
    // Note: Split archive extraction uses sevenzip_extract_streaming which
    // automatically handles multi-volume archives

    // ============================================================================
    // Enhanced Error Reporting
    // ============================================================================
    
    /// Get detailed information about the last error
    pub fn sevenzip_get_last_error(error_info: *mut SevenZipErrorInfo) -> SevenZipErrorCode;
    
    /// Clear the last error information
    pub fn sevenzip_clear_last_error();
    
    /// Get human-readable error message for error code
    pub fn sevenzip_get_error_string(code: SevenZipErrorCode) -> *const c_char;
    
    /// Get library version string
    pub fn sevenzip_get_version() -> *const c_char;
}

/// Detailed error information structure
#[repr(C)]
#[derive(Debug)]
pub struct SevenZipErrorInfo {
    pub code: SevenZipErrorCode,
    pub message: [c_char; 512],
    pub file_context: [c_char; 256],
    pub position: i64,
    pub suggestion: [c_char; 256],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(SevenZipErrorCode::SEVENZIP_OK as i32, 0);
        assert_eq!(SevenZipErrorCode::SEVENZIP_ERROR_OPEN_FILE as i32, 1);
        assert_eq!(SevenZipErrorCode::SEVENZIP_ERROR_UNKNOWN as i32, 99);
    }

    #[test]
    fn test_compression_level_values() {
        assert_eq!(SevenZipCompressionLevel::SEVENZIP_LEVEL_STORE as i32, 0);
        assert_eq!(SevenZipCompressionLevel::SEVENZIP_LEVEL_NORMAL as i32, 5);
        assert_eq!(SevenZipCompressionLevel::SEVENZIP_LEVEL_ULTRA as i32, 9);
    }

    #[test]
    fn test_aes_constants() {
        assert_eq!(AES_KEY_SIZE, 32);
        assert_eq!(AES_BLOCK_SIZE, 16);
        assert!(AES_NUM_IVMRK_WORDS > 0);
    }
}
