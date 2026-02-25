// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Safe Rust wrapper for libewf write operations
//
// Provides EwfWriter for creating E01 forensic containers with
// proper metadata, hashing, and chain-of-custody information.
//
// NOTE: Logical formats (L01/Lx01) are NOT supported for writing.
// libewf's `libewf_handle_set_format()` explicitly rejects logical format
// constants (they are commented out with a TODO in the C source).
// Reading existing L01 files is handled by the separate EwfReader/EwfDetectedFormat.

use crate::error::{self, Error, Result};
use crate::ffi;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;

// =============================================================================
// Public types
// =============================================================================

/// EWF output format (physical image formats only)
///
/// Logical formats (L01/Lx01) are not supported for writing by libewf.
/// Use [`crate::reader::EwfDetectedFormat`] for reading existing L01 files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfFormat {
    /// EnCase 5 format (.E01) — most compatible
    Encase5,
    /// EnCase 6 format (.E01) — supports larger images
    Encase6,
    /// EnCase 7 format (.Ex01)
    Encase7,
    /// EnCase 7 V2 format (.Ex01) — modern libewf only
    V2Encase7,
    /// FTK Imager format
    FtkImager,
    /// EWFX (libewf extended format)
    Ewfx,
}

impl EwfFormat {
    /// Convert to libewf format constant
    pub fn to_libewf_format(self) -> u8 {
        match self {
            EwfFormat::Encase5 => ffi::LIBEWF_FORMAT_ENCASE5,
            EwfFormat::Encase6 => ffi::LIBEWF_FORMAT_ENCASE6,
            EwfFormat::Encase7 => ffi::LIBEWF_FORMAT_ENCASE7,
            EwfFormat::V2Encase7 => ffi::LIBEWF_FORMAT_V2_ENCASE7,
            EwfFormat::FtkImager => ffi::LIBEWF_FORMAT_FTK_IMAGER,
            EwfFormat::Ewfx => ffi::LIBEWF_FORMAT_EWFX,
        }
    }

    /// Get the file extension produced by this format
    ///
    /// Note: `Encase7` (format 0x07) uses EWF1 segment type → `.E01` extension.
    /// Only `V2Encase7` (format 0x37) uses EWF2 segment type → `.Ex01` extension.
    pub fn extension(self) -> &'static str {
        match self {
            EwfFormat::Encase5
            | EwfFormat::Encase6
            | EwfFormat::Encase7
            | EwfFormat::FtkImager
            | EwfFormat::Ewfx => ".E01",
            EwfFormat::V2Encase7 => ".Ex01",
        }
    }

    /// Whether this is a V2 format (modern libewf only)
    pub fn is_v2(self) -> bool {
        matches!(self, EwfFormat::V2Encase7)
    }
}

/// Compression level for EWF output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfCompression {
    /// No compression (fastest, largest files)
    None,
    /// Fast compression (good speed, decent compression)
    Fast,
    /// Best compression (slowest, smallest files)
    Best,
}

impl EwfCompression {
    fn to_libewf_level(self) -> i8 {
        match self {
            EwfCompression::None => ffi::LIBEWF_COMPRESSION_NONE,
            EwfCompression::Fast => ffi::LIBEWF_COMPRESSION_FAST,
            EwfCompression::Best => ffi::LIBEWF_COMPRESSION_BEST,
        }
    }
}

/// Compression method for EWF output (modern libewf 20251220+)
///
/// This selects the compression algorithm. Use in combination with
/// `EwfCompression` which sets the compression level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfCompressionMethod {
    /// No compression method
    None,
    /// Deflate (zlib) — default, most compatible
    Deflate,
    /// BZIP2 — better compression ratio, requires modern libewf
    Bzip2,
}

impl EwfCompressionMethod {
    fn to_libewf_method(self) -> u16 {
        match self {
            EwfCompressionMethod::None => ffi::LIBEWF_COMPRESSION_METHOD_NONE,
            EwfCompressionMethod::Deflate => ffi::LIBEWF_COMPRESSION_METHOD_DEFLATE,
            EwfCompressionMethod::Bzip2 => ffi::LIBEWF_COMPRESSION_METHOD_BZIP2,
        }
    }
}

/// Case metadata for EWF header values
#[derive(Debug, Clone, Default)]
pub struct EwfCaseInfo {
    /// Case number
    pub case_number: Option<String>,
    /// Evidence number/ID
    pub evidence_number: Option<String>,
    /// Examiner name
    pub examiner_name: Option<String>,
    /// Description of the evidence
    pub description: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Acquisition software version
    pub acquiry_software_version: Option<String>,
    /// Model of the source device
    pub model: Option<String>,
    /// Serial number of the source device
    pub serial_number: Option<String>,
}

/// Configuration for creating an EWF container
#[derive(Debug, Clone)]
pub struct EwfWriterConfig {
    /// Output format
    pub format: EwfFormat,
    /// Compression level
    pub compression: EwfCompression,
    /// Compression method (Deflate or Bzip2) — modern libewf only
    pub compression_method: EwfCompressionMethod,
    /// Maximum segment file size in bytes (default: 1500 MB)
    pub segment_size: u64,
    /// Sectors per chunk (default: 64)
    pub sectors_per_chunk: u32,
    /// Bytes per sector (default: 512)
    pub bytes_per_sector: u32,
    /// Case metadata
    pub case_info: EwfCaseInfo,
    /// Total media size to write (must be set for physical images)
    pub media_size: Option<u64>,
}

impl Default for EwfWriterConfig {
    fn default() -> Self {
        Self {
            format: EwfFormat::Encase5,
            compression: EwfCompression::Fast,
            compression_method: EwfCompressionMethod::Deflate,
            segment_size: ffi::LIBEWF_DEFAULT_SEGMENT_FILE_SIZE,
            sectors_per_chunk: 64,
            bytes_per_sector: 512,
            case_info: EwfCaseInfo::default(),
            media_size: None,
        }
    }
}

// =============================================================================
// EwfWriter
// =============================================================================

/// Safe wrapper for creating EWF/E01 forensic containers
///
/// # Example
/// ```no_run
/// use libewf_ffi::{EwfWriter, EwfWriterConfig, EwfFormat, EwfCompression, EwfCaseInfo};
///
/// let config = EwfWriterConfig {
///     format: EwfFormat::Encase5,
///     compression: EwfCompression::Fast,
///     case_info: EwfCaseInfo {
///         case_number: Some("2024-001".to_string()),
///         examiner_name: Some("J. Doe".to_string()),
///         ..Default::default()
///     },
///     media_size: Some(1024 * 1024), // 1 MB
///     ..Default::default()
/// };
///
/// let mut writer = EwfWriter::create("/tmp/evidence", config).unwrap();
/// writer.write(&[0u8; 512]).unwrap();
/// writer.finalize().unwrap();
/// ```
pub struct EwfWriter {
    handle: *mut ffi::libewf_handle_t,
    bytes_written: u64,
    finalized: bool,
}

// libewf handle is not Send/Sync safe — single-threaded use only
// We don't impl Send/Sync.

impl EwfWriter {
    /// Create a new EWF container at the given path
    ///
    /// The path should be the base filename WITHOUT extension.
    /// libewf will automatically add .E01/.Ex01 extensions.
    ///
    /// # Arguments
    /// * `output_path` - Base path for output (e.g., "/tmp/evidence" → creates "/tmp/evidence.E01")
    /// * `config` - Writer configuration
    pub fn create<P: AsRef<Path>>(output_path: P, config: EwfWriterConfig) -> Result<Self> {
        let path = output_path.as_ref();

        // Validate path
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(Error::InvalidParam(format!(
                    "Parent directory does not exist: {}",
                    parent.display()
                )));
            }
        }

        unsafe {
            // Initialize handle
            let mut handle: *mut ffi::libewf_handle_t = ptr::null_mut();
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();

            let rc = ffi::libewf_handle_initialize(&mut handle, &mut error);
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!("Failed to initialize handle: {}", msg)));
            }

            // Open for writing
            let path_str = path
                .to_str()
                .ok_or_else(|| Error::InvalidParam("Path is not valid UTF-8".to_string()))?;
            let c_path = CString::new(path_str)
                .map_err(|_| Error::InvalidParam("Path contains null bytes".to_string()))?;
            let mut filename_ptr = c_path.as_ptr() as *mut c_char;

            let rc = ffi::libewf_handle_open(
                handle,
                &mut filename_ptr,
                1, // number_of_filenames
                ffi::LIBEWF_OPEN_WRITE,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                ffi::libewf_handle_free(&mut handle, ptr::null_mut());
                return Err(Error::Libewf(format!("Failed to open for writing: {}", msg)));
            }

            // Configure the writer
            let writer = EwfWriter {
                handle,
                bytes_written: 0,
                finalized: false,
            };

            writer.apply_config(&config)?;

            Ok(writer)
        }
    }

    /// Apply configuration to the open handle
    unsafe fn apply_config(&self, config: &EwfWriterConfig) -> Result<()> {
        let mut error: *mut ffi::libewf_error_t = ptr::null_mut();

        // Validate: BZIP2 compression requires V2 format
        if config.compression_method == EwfCompressionMethod::Bzip2 && !config.format.is_v2() {
            return Err(Error::InvalidParam(
                "BZIP2 compression method requires a V2 format (V2Encase7)"
                    .to_string(),
            ));
        }

        // Set format
        let rc = ffi::libewf_handle_set_format(
            self.handle,
            config.format.to_libewf_format(),
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!("Failed to set format: {}", msg)));
        }

        // Set compression
        let rc = ffi::libewf_handle_set_compression_values(
            self.handle,
            config.compression.to_libewf_level(),
            0, // no empty block compression flag
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!("Failed to set compression: {}", msg)));
        }

        // Set compression method (modern libewf — Deflate or Bzip2)
        let rc = ffi::libewf_handle_set_compression_method(
            self.handle,
            config.compression_method.to_libewf_method(),
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set compression method: {}",
                msg
            )));
        }

        // Set segment size
        let rc = ffi::libewf_handle_set_maximum_segment_size(
            self.handle,
            config.segment_size,
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set segment size: {}",
                msg
            )));
        }

        // Set sectors per chunk
        let rc = ffi::libewf_handle_set_sectors_per_chunk(
            self.handle,
            config.sectors_per_chunk,
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set sectors per chunk: {}",
                msg
            )));
        }

        // Set bytes per sector
        let rc = ffi::libewf_handle_set_bytes_per_sector(
            self.handle,
            config.bytes_per_sector,
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set bytes per sector: {}",
                msg
            )));
        }

        // Set media type (always fixed disk — logical formats not supported for writing)
        let media_type = ffi::LIBEWF_MEDIA_TYPE_FIXED;
        let rc = ffi::libewf_handle_set_media_type(self.handle, media_type, &mut error);
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set media type: {}",
                msg
            )));
        }

        // Set media size if provided
        if let Some(size) = config.media_size {
            let rc = ffi::libewf_handle_set_media_size(self.handle, size, &mut error);
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to set media size: {}",
                    msg
                )));
            }
        }

        // Set header values
        self.set_header_values(&config.case_info)?;

        Ok(())
    }

    /// Set header values from case info
    unsafe fn set_header_values(&self, info: &EwfCaseInfo) -> Result<()> {
        if let Some(ref val) = info.case_number {
            self.set_header_value("case_number", val)?;
        }
        if let Some(ref val) = info.evidence_number {
            self.set_header_value("evidence_number", val)?;
        }
        if let Some(ref val) = info.examiner_name {
            self.set_header_value("examiner_name", val)?;
        }
        if let Some(ref val) = info.description {
            self.set_header_value("description", val)?;
        }
        if let Some(ref val) = info.notes {
            self.set_header_value("notes", val)?;
        }
        if let Some(ref val) = info.acquiry_software_version {
            self.set_header_value("acquiry_software_version", val)?;
        }
        if let Some(ref val) = info.model {
            self.set_header_value("model", val)?;
        }
        if let Some(ref val) = info.serial_number {
            self.set_header_value("serial_number", val)?;
        }

        // Always set the software version
        self.set_header_value("acquiry_software_version", "CORE-FFX")?;

        // Set acquisition OS
        #[cfg(target_os = "macos")]
        self.set_header_value("acquiry_operating_system", "macOS")?;
        #[cfg(target_os = "linux")]
        self.set_header_value("acquiry_operating_system", "Linux")?;
        #[cfg(target_os = "windows")]
        self.set_header_value("acquiry_operating_system", "Windows")?;

        Ok(())
    }

    /// Set a single header value
    unsafe fn set_header_value(&self, identifier: &str, value: &str) -> Result<()> {
        let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
        // Use CString to ensure null-terminated strings — libewf may read
        // beyond identifier_length for internal comparisons
        let c_identifier = CString::new(identifier).map_err(|_| {
            Error::InvalidParam(format!("Header identifier contains null: {}", identifier))
        })?;
        let c_value = CString::new(value).map_err(|_| {
            Error::InvalidParam(format!("Header value contains null: {}", value))
        })?;
        let rc = ffi::libewf_handle_set_utf8_header_value(
            self.handle,
            c_identifier.as_ptr() as *const u8,
            identifier.len(),
            c_value.as_ptr() as *const u8,
            value.len(),
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set header '{}': {}",
                identifier, msg
            )));
        }
        Ok(())
    }

    /// Write data to the EWF container
    ///
    /// Writes the buffer contents at the current offset. Data is automatically
    /// compressed and chunked according to the configuration.
    ///
    /// Returns the number of bytes written.
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        if self.finalized {
            return Err(Error::InvalidParam(
                "Cannot write to finalized container".to_string(),
            ));
        }

        if data.is_empty() {
            return Ok(0);
        }

        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let bytes_written = ffi::libewf_handle_write_buffer(
                self.handle,
                data.as_ptr() as *const std::os::raw::c_void,
                data.len(),
                &mut error,
            );

            if bytes_written < 0 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!("Write failed: {}", msg)));
            }

            self.bytes_written += bytes_written as u64;
            Ok(bytes_written as usize)
        }
    }

    /// Write all data, looping until the entire buffer is consumed
    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0;
        while offset < data.len() {
            let written = self.write(&data[offset..])?;
            if written == 0 {
                return Err(Error::Libewf(
                    "Write returned 0 bytes before all data was written".to_string(),
                ));
            }
            offset += written;
        }
        Ok(())
    }

    /// Set the stored MD5 hash value (hex string, 32 hex chars)
    ///
    /// Stores the hash using both the binary API (`set_md5_hash`) and the
    /// UTF-8 hash value API (`set_utf8_hash_value("MD5", ...)`) so it can
    /// be retrieved by either mechanism on read-back.
    pub fn set_md5_hash(&self, md5_hex: &str) -> Result<()> {
        if md5_hex.len() != 32 {
            return Err(Error::InvalidParam(format!(
                "MD5 hash must be 32 hex characters, got {}",
                md5_hex.len()
            )));
        }
        // Store via binary API (raw 16 bytes)
        let bytes = hex_decode(md5_hex).map_err(|e| {
            Error::InvalidParam(format!("Invalid MD5 hex string: {}", e))
        })?;
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_set_md5_hash(
                self.handle,
                bytes.as_ptr(),
                16,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to set MD5 hash (binary): {}",
                    msg
                )));
            }
            // Also store via UTF-8 hash value API for completeness
            self.set_hash_value("MD5", md5_hex)?;
        }
        Ok(())
    }

    /// Set the stored SHA1 hash value (hex string, 40 hex chars)
    ///
    /// Stores the hash using both the binary API (`set_sha1_hash`) and the
    /// UTF-8 hash value API (`set_utf8_hash_value("SHA1", ...)`) so it can
    /// be retrieved by either mechanism on read-back.
    pub fn set_sha1_hash(&self, sha1_hex: &str) -> Result<()> {
        if sha1_hex.len() != 40 {
            return Err(Error::InvalidParam(format!(
                "SHA1 hash must be 40 hex characters, got {}",
                sha1_hex.len()
            )));
        }
        // Store via binary API (raw 20 bytes)
        let bytes = hex_decode(sha1_hex).map_err(|e| {
            Error::InvalidParam(format!("Invalid SHA1 hex string: {}", e))
        })?;
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_set_sha1_hash(
                self.handle,
                bytes.as_ptr(),
                20,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to set SHA1 hash (binary): {}",
                    msg
                )));
            }
            // Also store via UTF-8 hash value API for completeness
            self.set_hash_value("SHA1", sha1_hex)?;
        }
        Ok(())
    }

    /// Set a hash value by identifier
    unsafe fn set_hash_value(&self, identifier: &str, value: &str) -> Result<()> {
        let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
        // Use CString to ensure null-terminated strings
        let c_identifier = CString::new(identifier).map_err(|_| {
            Error::InvalidParam(format!("Hash identifier contains null: {}", identifier))
        })?;
        let c_value = CString::new(value).map_err(|_| {
            Error::InvalidParam(format!("Hash value contains null: {}", value))
        })?;
        let rc = ffi::libewf_handle_set_utf8_hash_value(
            self.handle,
            c_identifier.as_ptr() as *const u8,
            identifier.len(),
            c_value.as_ptr() as *const u8,
            value.len(),
            &mut error,
        );
        if rc != 1 {
            let msg = error::extract_error(error);
            return Err(Error::Libewf(format!(
                "Failed to set hash '{}': {}",
                identifier, msg
            )));
        }
        Ok(())
    }

    /// Get the total number of bytes written so far
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Finalize the container (REQUIRED after all writes)
    ///
    /// This corrects the EWF metadata in the segment files and must be called
    /// before the writer is dropped. Failing to call this will result in a
    /// corrupted container.
    pub fn finalize(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }

        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_write_finalize(self.handle, &mut error);
            if rc < 0 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!("Finalize failed: {}", msg)));
            }
            self.finalized = true;
        }

        Ok(())
    }

    /// Check if the container has been finalized
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }
}

impl Drop for EwfWriter {
    fn drop(&mut self) {
        unsafe {
            // Try to finalize if not already done
            if !self.finalized && !self.handle.is_null() {
                let _ = ffi::libewf_handle_write_finalize(
                    self.handle,
                    ptr::null_mut(),
                );
            }

            // Close and free
            if !self.handle.is_null() {
                ffi::libewf_handle_close(self.handle, ptr::null_mut());
                ffi::libewf_handle_free(&mut self.handle, ptr::null_mut());
            }
        }
    }
}

// =============================================================================
// Utility functions
// =============================================================================

/// Get the libewf library version string
pub fn libewf_version() -> String {
    unsafe {
        let version_ptr = ffi::libewf_get_version();
        if version_ptr.is_null() {
            return "unknown".to_string();
        }
        let c_str = std::ffi::CStr::from_ptr(version_ptr);
        c_str.to_string_lossy().to_string()
    }
}

/// Decode a hex string into bytes
fn hex_decode(hex: &str) -> std::result::Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
}
