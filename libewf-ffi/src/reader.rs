// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Safe Rust wrapper for libewf read operations
//
// Provides EwfReader for opening and reading E01/Ex01/L01/Lx01 forensic
// containers with metadata extraction, hash retrieval, and random-access
// data reading via libewf's native C library.

use crate::error::{self, Error, Result};
use crate::ffi;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::path::Path;
use std::ptr;

// =============================================================================
// Public types
// =============================================================================

/// Detected EWF format after opening for reading
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfDetectedFormat {
    /// EnCase 1 format
    Encase1,
    /// EnCase 2 format
    Encase2,
    /// EnCase 3 format
    Encase3,
    /// EnCase 4 format
    Encase4,
    /// EnCase 5 format (.E01)
    Encase5,
    /// EnCase 6 format (.E01)
    Encase6,
    /// EnCase 7 format (.E01, EWF1 segment type)
    Encase7,
    /// SMART format
    Smart,
    /// FTK Imager format
    FtkImager,
    /// Logical EnCase 5 (.L01)
    LogicalEncase5,
    /// Logical EnCase 6 (.L01)
    LogicalEncase6,
    /// Logical EnCase 7 (.L01)
    LogicalEncase7,
    /// Linen 5
    Linen5,
    /// Linen 6
    Linen6,
    /// Linen 7
    Linen7,
    /// EWF format
    Ewf,
    /// EWFX format
    Ewfx,
    /// V2 EnCase 7 (.Ex01, EWF2 segment type)
    V2Encase7,
    /// V2 Logical EnCase 7 (.Lx01, EWF2 segment type)
    V2LogicalEncase7,
    /// Unknown/unrecognized format
    Unknown(u8),
}

impl EwfDetectedFormat {
    /// Convert from the libewf format constant
    pub fn from_libewf(format: u8) -> Self {
        match format {
            ffi::LIBEWF_FORMAT_ENCASE1 => Self::Encase1,
            ffi::LIBEWF_FORMAT_ENCASE2 => Self::Encase2,
            ffi::LIBEWF_FORMAT_ENCASE3 => Self::Encase3,
            ffi::LIBEWF_FORMAT_ENCASE4 => Self::Encase4,
            ffi::LIBEWF_FORMAT_ENCASE5 => Self::Encase5,
            ffi::LIBEWF_FORMAT_ENCASE6 => Self::Encase6,
            ffi::LIBEWF_FORMAT_ENCASE7 => Self::Encase7,
            ffi::LIBEWF_FORMAT_SMART => Self::Smart,
            ffi::LIBEWF_FORMAT_FTK_IMAGER => Self::FtkImager,
            ffi::LIBEWF_FORMAT_LOGICAL_ENCASE5 => Self::LogicalEncase5,
            ffi::LIBEWF_FORMAT_LOGICAL_ENCASE6 => Self::LogicalEncase6,
            ffi::LIBEWF_FORMAT_LOGICAL_ENCASE7 => Self::LogicalEncase7,
            ffi::LIBEWF_FORMAT_LINEN5 => Self::Linen5,
            ffi::LIBEWF_FORMAT_LINEN6 => Self::Linen6,
            ffi::LIBEWF_FORMAT_LINEN7 => Self::Linen7,
            ffi::LIBEWF_FORMAT_EWF => Self::Ewf,
            ffi::LIBEWF_FORMAT_EWFX => Self::Ewfx,
            ffi::LIBEWF_FORMAT_V2_ENCASE7 => Self::V2Encase7,
            ffi::LIBEWF_FORMAT_V2_LOGICAL_ENCASE7 => Self::V2LogicalEncase7,
            other => Self::Unknown(other),
        }
    }

    /// Human-readable format name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Encase1 => "EnCase 1",
            Self::Encase2 => "EnCase 2",
            Self::Encase3 => "EnCase 3",
            Self::Encase4 => "EnCase 4",
            Self::Encase5 => "EnCase 5",
            Self::Encase6 => "EnCase 6",
            Self::Encase7 => "EnCase 7",
            Self::Smart => "SMART",
            Self::FtkImager => "FTK Imager",
            Self::LogicalEncase5 => "Logical EnCase 5",
            Self::LogicalEncase6 => "Logical EnCase 6",
            Self::LogicalEncase7 => "Logical EnCase 7",
            Self::Linen5 => "Linen 5",
            Self::Linen6 => "Linen 6",
            Self::Linen7 => "Linen 7",
            Self::Ewf => "EWF",
            Self::Ewfx => "EWFX",
            Self::V2Encase7 => "EnCase 7 V2",
            Self::V2LogicalEncase7 => "Logical EnCase 7 V2",
            Self::Unknown(_) => "Unknown",
        }
    }

    /// Whether this is a logical evidence format (L01/Lx01)
    pub fn is_logical(&self) -> bool {
        matches!(
            self,
            Self::LogicalEncase5
                | Self::LogicalEncase6
                | Self::LogicalEncase7
                | Self::V2LogicalEncase7
        )
    }

    /// Whether this is an EWF2 (V2) format
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::V2Encase7 | Self::V2LogicalEncase7)
    }

    /// File extension associated with this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::V2Encase7 => ".Ex01",
            Self::V2LogicalEncase7 => ".Lx01",
            Self::LogicalEncase5 | Self::LogicalEncase6 | Self::LogicalEncase7 => ".L01",
            _ => ".E01",
        }
    }
}

impl std::fmt::Display for EwfDetectedFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Detected compression method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfDetectedCompressionMethod {
    /// No compression
    None,
    /// Deflate (zlib)
    Deflate,
    /// BZIP2
    Bzip2,
    /// Unknown method
    Unknown(u16),
}

impl EwfDetectedCompressionMethod {
    fn from_libewf(method: u16) -> Self {
        match method {
            ffi::LIBEWF_COMPRESSION_METHOD_NONE => Self::None,
            ffi::LIBEWF_COMPRESSION_METHOD_DEFLATE => Self::Deflate,
            ffi::LIBEWF_COMPRESSION_METHOD_BZIP2 => Self::Bzip2,
            other => Self::Unknown(other),
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Deflate => "Deflate",
            Self::Bzip2 => "BZIP2",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl std::fmt::Display for EwfDetectedCompressionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Complete image metadata extracted from an EWF container
#[derive(Debug, Clone)]
pub struct EwfImageInfo {
    /// Detected format
    pub format: EwfDetectedFormat,
    /// Total media size in bytes
    pub media_size: u64,
    /// Bytes per sector
    pub bytes_per_sector: u32,
    /// Sectors per chunk
    pub sectors_per_chunk: u32,
    /// Compression level (-1=default, 0=none, 1=fast, 2=best)
    pub compression_level: i8,
    /// Compression method (Deflate, BZIP2, etc.)
    pub compression_method: EwfDetectedCompressionMethod,
    /// Media type constant
    pub media_type: u8,
    /// Media flags
    pub media_flags: u8,
    /// Segment file version (major, minor) — None if unavailable
    pub segment_file_version: Option<(u8, u8)>,
    /// Whether segment files are corrupted
    pub is_corrupted: bool,
    /// Whether segment files are encrypted
    pub is_encrypted: bool,
    /// Case/evidence metadata from header values
    pub case_info: EwfReadCaseInfo,
    /// Stored MD5 hash (hex string, if present)
    pub md5_hash: Option<String>,
    /// Stored SHA1 hash (hex string, if present)
    pub sha1_hash: Option<String>,
}

/// Case metadata extracted from EWF header values
#[derive(Debug, Clone, Default)]
pub struct EwfReadCaseInfo {
    /// Case number
    pub case_number: Option<String>,
    /// Evidence number
    pub evidence_number: Option<String>,
    /// Examiner name
    pub examiner_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Acquisition software version
    pub acquiry_software_version: Option<String>,
    /// Acquisition date
    pub acquiry_date: Option<String>,
    /// Acquisition operating system
    pub acquiry_operating_system: Option<String>,
    /// Model of the source device
    pub model: Option<String>,
    /// Serial number of the source device
    pub serial_number: Option<String>,
}

// =============================================================================
// EwfReader
// =============================================================================

/// Safe wrapper for reading EWF/E01/Ex01/L01/Lx01 forensic containers
///
/// Opens an EWF container set (handles multi-segment files automatically)
/// and provides random-access reading of the media data plus metadata
/// extraction.
///
/// # Example
/// ```no_run
/// use libewf_ffi::reader::EwfReader;
///
/// let reader = EwfReader::open("/path/to/image.E01").unwrap();
/// let info = reader.image_info().unwrap();
/// println!("Format: {}", info.format);
/// println!("Media size: {} bytes", info.media_size);
///
/// // Read first 512 bytes (MBR)
/// let mut buf = vec![0u8; 512];
/// let n = reader.read_at(0, &mut buf).unwrap();
/// println!("Read {} bytes", n);
/// ```
pub struct EwfReader {
    handle: *mut ffi::libewf_handle_t,
    /// Cached media size (avoids repeated FFI calls)
    media_size: u64,
}

// libewf handle is not Send/Sync safe — single-threaded use only

impl EwfReader {
    /// Open an EWF container for reading
    ///
    /// The path should point to the first segment file (e.g., `image.E01`).
    /// libewf will automatically discover and open all related segment files
    /// (.E02, .E03, etc.).
    ///
    /// # Arguments
    /// * `path` - Path to the first segment file (.E01, .Ex01, .L01, .Lx01)
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::InvalidParam(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        unsafe {
            // Initialize handle
            let mut handle: *mut ffi::libewf_handle_t = ptr::null_mut();
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();

            let rc = ffi::libewf_handle_initialize(&mut handle, &mut error);
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to initialize handle: {}",
                    msg
                )));
            }

            // Use libewf_glob to discover all segment files
            let path_str = path
                .to_str()
                .ok_or_else(|| Error::InvalidParam("Path is not valid UTF-8".to_string()))?;
            let c_path = CString::new(path_str)
                .map_err(|_| Error::InvalidParam("Path contains null bytes".to_string()))?;

            let mut glob_filenames: *mut *mut c_char = ptr::null_mut();
            let mut glob_count: c_int = 0;

            let rc = ffi::libewf_glob(
                c_path.as_ptr(),
                path_str.len(),
                ffi::LIBEWF_FORMAT_UNKNOWN,
                &mut glob_filenames,
                &mut glob_count,
                &mut error,
            );

            if rc == 1 && glob_count > 0 && !glob_filenames.is_null() {
                // Open using globbed filenames
                let rc = ffi::libewf_handle_open(
                    handle,
                    glob_filenames as *const *mut c_char,
                    glob_count,
                    ffi::LIBEWF_OPEN_READ,
                    &mut error,
                );

                // Free globbed filenames regardless of open result
                ffi::libewf_glob_free(glob_filenames, glob_count, ptr::null_mut());

                if rc != 1 {
                    let msg = error::extract_error(error);
                    ffi::libewf_handle_free(&mut handle, ptr::null_mut());
                    return Err(Error::Libewf(format!(
                        "Failed to open for reading: {}",
                        msg
                    )));
                }
            } else {
                // Glob failed or returned no files — fall back to direct open
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                    error = ptr::null_mut();
                }

                let mut filename_ptr = c_path.as_ptr() as *mut c_char;
                let rc = ffi::libewf_handle_open(
                    handle,
                    &mut filename_ptr,
                    1,
                    ffi::LIBEWF_OPEN_READ,
                    &mut error,
                );
                if rc != 1 {
                    let msg = error::extract_error(error);
                    ffi::libewf_handle_free(&mut handle, ptr::null_mut());
                    return Err(Error::Libewf(format!(
                        "Failed to open for reading: {}",
                        msg
                    )));
                }
            }

            // Cache media size
            let mut media_size: ffi::size64_t = 0;
            let rc =
                ffi::libewf_handle_get_media_size(handle, &mut media_size, &mut error);
            if rc != 1 {
                let msg = error::extract_error(error);
                ffi::libewf_handle_close(handle, ptr::null_mut());
                ffi::libewf_handle_free(&mut handle, ptr::null_mut());
                return Err(Error::Libewf(format!(
                    "Failed to get media size: {}",
                    msg
                )));
            }

            Ok(Self {
                handle,
                media_size,
            })
        }
    }

    /// Get the total media size in bytes
    pub fn media_size(&self) -> u64 {
        self.media_size
    }

    /// Read data at a specific byte offset in the media
    ///
    /// Returns the number of bytes actually read. May be less than
    /// `buffer.len()` if the offset is near the end of the media.
    pub fn read_at(&self, offset: i64, buffer: &mut [u8]) -> Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let bytes_read = ffi::libewf_handle_read_buffer_at_offset(
                self.handle,
                buffer.as_mut_ptr() as *mut std::os::raw::c_void,
                buffer.len(),
                offset,
                &mut error,
            );

            if bytes_read < 0 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Read at offset {} failed: {}",
                    offset, msg
                )));
            }

            Ok(bytes_read as usize)
        }
    }

    /// Read data sequentially from the current position
    ///
    /// Returns the number of bytes actually read.
    pub fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let bytes_read = ffi::libewf_handle_read_buffer(
                self.handle,
                buffer.as_mut_ptr() as *mut std::os::raw::c_void,
                buffer.len(),
                &mut error,
            );

            if bytes_read < 0 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!("Sequential read failed: {}", msg)));
            }

            Ok(bytes_read as usize)
        }
    }

    /// Seek to a specific byte offset in the media
    ///
    /// Returns the new offset after seeking.
    pub fn seek(&self, offset: i64, whence: i32) -> Result<i64> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let new_offset = ffi::libewf_handle_seek_offset(
                self.handle,
                offset,
                whence,
                &mut error,
            );

            if new_offset < 0 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Seek to offset {} failed: {}",
                    offset, msg
                )));
            }

            Ok(new_offset)
        }
    }

    /// Get the current read offset
    pub fn current_offset(&self) -> Result<i64> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut offset: ffi::off64_t = 0;
            let rc = ffi::libewf_handle_get_offset(
                self.handle,
                &mut offset,
                &mut error,
            );

            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to get current offset: {}",
                    msg
                )));
            }

            Ok(offset)
        }
    }

    /// Get complete image metadata
    pub fn image_info(&self) -> Result<EwfImageInfo> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();

            // Format
            let mut format: u8 = 0;
            let rc = ffi::libewf_handle_get_format(
                self.handle,
                &mut format,
                &mut error,
            );
            let detected_format = if rc == 1 {
                EwfDetectedFormat::from_libewf(format)
            } else {
                error::extract_error(error);
                error = ptr::null_mut();
                EwfDetectedFormat::Unknown(0)
            };

            // Bytes per sector
            let mut bytes_per_sector: u32 = 0;
            let rc = ffi::libewf_handle_get_bytes_per_sector(
                self.handle,
                &mut bytes_per_sector,
                &mut error,
            );
            if rc != 1 {
                error::extract_error(error);
                error = ptr::null_mut();
                bytes_per_sector = 512; // fallback
            }

            // Sectors per chunk
            let mut sectors_per_chunk: u32 = 0;
            let rc = ffi::libewf_handle_get_sectors_per_chunk(
                self.handle,
                &mut sectors_per_chunk,
                &mut error,
            );
            if rc != 1 {
                error::extract_error(error);
                error = ptr::null_mut();
                sectors_per_chunk = 64; // fallback
            }

            // Compression values
            let mut compression_level: i8 = 0;
            let mut compression_flags: u8 = 0;
            let rc = ffi::libewf_handle_get_compression_values(
                self.handle,
                &mut compression_level,
                &mut compression_flags,
                &mut error,
            );
            if rc != 1 {
                error::extract_error(error);
                error = ptr::null_mut();
            }

            // Compression method
            let mut compression_method_raw: u16 = 0;
            let rc = ffi::libewf_handle_get_compression_method(
                self.handle,
                &mut compression_method_raw,
                &mut error,
            );
            let compression_method = if rc == 1 {
                EwfDetectedCompressionMethod::from_libewf(compression_method_raw)
            } else {
                error::extract_error(error);
                error = ptr::null_mut();
                EwfDetectedCompressionMethod::Deflate // default fallback
            };

            // Media type
            let mut media_type: u8 = 0;
            let rc = ffi::libewf_handle_get_media_type(
                self.handle,
                &mut media_type,
                &mut error,
            );
            if rc != 1 {
                error::extract_error(error);
                error = ptr::null_mut();
            }

            // Media flags
            let mut media_flags: u8 = 0;
            let rc = ffi::libewf_handle_get_media_flags(
                self.handle,
                &mut media_flags,
                &mut error,
            );
            if rc != 1 {
                error::extract_error(error);
                error = ptr::null_mut();
            }

            // Segment file version
            let mut major_version: u8 = 0;
            let mut minor_version: u8 = 0;
            let rc = ffi::libewf_handle_get_segment_file_version(
                self.handle,
                &mut major_version,
                &mut minor_version,
                &mut error,
            );
            let segment_file_version = if rc == 1 {
                Some((major_version, minor_version))
            } else {
                error::extract_error(error);
                error = ptr::null_mut();
                None
            };

            // Corruption check
            let rc = ffi::libewf_handle_segment_files_corrupted(
                self.handle,
                &mut error,
            );
            let is_corrupted = rc == 1;
            if rc < 0 {
                error::extract_error(error);
                error = ptr::null_mut();
            }

            // Encryption check
            let rc = ffi::libewf_handle_segment_files_encrypted(
                self.handle,
                &mut error,
            );
            let is_encrypted = rc == 1;
            if rc < 0 {
                error::extract_error(error);
                // error consumed by extract_error, no need to reset
            }

            // Case info from header values
            let case_info = self.read_case_info();

            // Hashes
            let md5_hash = self.get_md5_hash();
            let sha1_hash = self.get_sha1_hash();

            Ok(EwfImageInfo {
                format: detected_format,
                media_size: self.media_size,
                bytes_per_sector,
                sectors_per_chunk,
                compression_level,
                compression_method,
                media_type,
                media_flags,
                segment_file_version,
                is_corrupted,
                is_encrypted,
                case_info,
                md5_hash,
                sha1_hash,
            })
        }
    }

    /// Get the detected EWF format
    pub fn format(&self) -> Result<EwfDetectedFormat> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut format: u8 = 0;
            let rc = ffi::libewf_handle_get_format(
                self.handle,
                &mut format,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!("Failed to get format: {}", msg)));
            }
            Ok(EwfDetectedFormat::from_libewf(format))
        }
    }

    /// Get the stored MD5 hash as a hex string (if present in the container)
    pub fn get_md5_hash(&self) -> Option<String> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut hash_bytes = [0u8; 16];
            let rc = ffi::libewf_handle_get_md5_hash(
                self.handle,
                hash_bytes.as_mut_ptr(),
                16,
                &mut error,
            );
            if rc == 1 {
                Some(hex_encode(&hash_bytes))
            } else {
                // Not an error — hash may simply not be stored
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                None
            }
        }
    }

    /// Get the stored SHA1 hash as a hex string (if present in the container)
    pub fn get_sha1_hash(&self) -> Option<String> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut hash_bytes = [0u8; 20];
            let rc = ffi::libewf_handle_get_sha1_hash(
                self.handle,
                hash_bytes.as_mut_ptr(),
                20,
                &mut error,
            );
            if rc == 1 {
                Some(hex_encode(&hash_bytes))
            } else {
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                None
            }
        }
    }

    /// Get a specific header value by identifier
    ///
    /// Common identifiers: "case_number", "evidence_number", "examiner_name",
    /// "description", "notes", "acquiry_date", "acquiry_software_version",
    /// "acquiry_operating_system", "model", "serial_number"
    pub fn get_header_value(&self, identifier: &str) -> Option<String> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();

            // Use CString to ensure null-terminated identifier — libewf
            // may read beyond identifier_length for internal comparisons
            let c_identifier = CString::new(identifier).ok()?;

            // First get the size of the value
            let mut value_size: usize = 0;
            let rc = ffi::libewf_handle_get_utf8_header_value_size(
                self.handle,
                c_identifier.as_ptr() as *const u8,
                identifier.len(),
                &mut value_size,
                &mut error,
            );
            if rc != 1 || value_size == 0 {
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                return None;
            }

            // Allocate buffer and read the value
            let mut buffer = vec![0u8; value_size];
            error = ptr::null_mut();
            let rc = ffi::libewf_handle_get_utf8_header_value(
                self.handle,
                c_identifier.as_ptr() as *const u8,
                identifier.len(),
                buffer.as_mut_ptr(),
                value_size,
                &mut error,
            );
            if rc != 1 {
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                return None;
            }

            // Convert to string (null-terminated)
            let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
            String::from_utf8(buffer[..len].to_vec()).ok()
        }
    }

    /// Get the number of header values stored in the container
    pub fn header_value_count(&self) -> Result<u32> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut count: u32 = 0;
            let rc = ffi::libewf_handle_get_number_of_header_values(
                self.handle,
                &mut count,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to get header value count: {}",
                    msg
                )));
            }
            Ok(count)
        }
    }

    /// Get the number of hash values stored in the container
    pub fn hash_value_count(&self) -> Result<u32> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut count: u32 = 0;
            let rc = ffi::libewf_handle_get_number_of_hash_values(
                self.handle,
                &mut count,
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to get hash value count: {}",
                    msg
                )));
            }
            Ok(count)
        }
    }

    /// List all header value identifiers stored in the container
    ///
    /// Returns a vector of (identifier, value) pairs for all stored header values.
    /// Useful for diagnostics and enumerating available metadata.
    pub fn list_header_values(&self) -> Vec<(String, Option<String>)> {
        let count = match self.header_value_count() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut result = Vec::new();
        for i in 0..count {
            if let Some(identifier) = self.get_header_identifier(i) {
                let value = self.get_header_value(&identifier);
                result.push((identifier, value));
            }
        }
        result
    }

    /// Get the header value identifier at a specific index
    fn get_header_identifier(&self, index: u32) -> Option<String> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let mut id_size: usize = 0;

            let rc = ffi::libewf_handle_get_header_value_identifier_size(
                self.handle,
                index,
                &mut id_size,
                &mut error,
            );
            if rc != 1 || id_size == 0 {
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                return None;
            }

            let mut buffer = vec![0u8; id_size];
            error = ptr::null_mut();
            let rc = ffi::libewf_handle_get_header_value_identifier(
                self.handle,
                index,
                buffer.as_mut_ptr(),
                id_size,
                &mut error,
            );
            if rc != 1 {
                if !error.is_null() {
                    let mut err_ptr = error;
                    ffi::libewf_error_free(&mut err_ptr);
                }
                return None;
            }

            let len = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
            String::from_utf8(buffer[..len].to_vec()).ok()
        }
    }

    /// Read all case metadata from header values
    fn read_case_info(&self) -> EwfReadCaseInfo {
        EwfReadCaseInfo {
            case_number: self.get_header_value("case_number"),
            evidence_number: self.get_header_value("evidence_number"),
            examiner_name: self.get_header_value("examiner_name"),
            description: self.get_header_value("description"),
            notes: self.get_header_value("notes"),
            acquiry_software_version: self.get_header_value("acquiry_software_version"),
            acquiry_date: self.get_header_value("acquiry_date"),
            acquiry_operating_system: self.get_header_value("acquiry_operating_system"),
            model: self.get_header_value("model"),
            serial_number: self.get_header_value("serial_number"),
        }
    }

    /// Check if segment files are corrupted
    pub fn is_corrupted(&self) -> bool {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_segment_files_corrupted(
                self.handle,
                &mut error,
            );
            if rc < 0 && !error.is_null() {
                let mut err_ptr = error;
                ffi::libewf_error_free(&mut err_ptr);
            }
            rc == 1
        }
    }

    /// Check if segment files are encrypted
    pub fn is_encrypted(&self) -> bool {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_segment_files_encrypted(
                self.handle,
                &mut error,
            );
            if rc < 0 && !error.is_null() {
                let mut err_ptr = error;
                ffi::libewf_error_free(&mut err_ptr);
            }
            rc == 1
        }
    }

    /// Set whether to zero out chunks on read errors
    ///
    /// When enabled, any chunk that fails to read/decompress will be
    /// returned as zeroed data instead of raising an error. Useful for
    /// reading damaged images.
    pub fn set_zero_on_error(&self, zero_on_error: bool) -> Result<()> {
        unsafe {
            let mut error: *mut ffi::libewf_error_t = ptr::null_mut();
            let rc = ffi::libewf_handle_set_read_zero_chunk_on_error(
                self.handle,
                if zero_on_error { 1 } else { 0 },
                &mut error,
            );
            if rc != 1 {
                let msg = error::extract_error(error);
                return Err(Error::Libewf(format!(
                    "Failed to set zero-on-error flag: {}",
                    msg
                )));
            }
            Ok(())
        }
    }
}

impl Drop for EwfReader {
    fn drop(&mut self) {
        unsafe {
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

/// Encode bytes as lowercase hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
