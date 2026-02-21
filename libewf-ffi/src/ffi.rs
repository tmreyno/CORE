// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Raw FFI bindings to libewf C library
//
// These map directly to the libewf C API. Use the safe wrappers in writer.rs
// and reader.rs instead of calling these directly.
//
// Reference: /opt/homebrew/Cellar/libewf/20251220/include/libewf.h
// API version: 20251220 (built from source — libyal/libewf HEAD)

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_void};

// =============================================================================
// Opaque types
// =============================================================================

/// Opaque handle type (typedef intptr_t in C)
pub type libewf_handle_t = isize;

/// Opaque error type (typedef intptr_t in C)
pub type libewf_error_t = isize;

/// Opaque file entry type (typedef intptr_t in C)
pub type libewf_file_entry_t = isize;

/// Opaque data chunk type (typedef intptr_t in C) — modern libewf only
pub type libewf_data_chunk_t = isize;

/// Opaque access control entry type (typedef intptr_t in C) — modern libewf only
pub type libewf_access_control_entry_t = isize;

/// Opaque attribute type (typedef intptr_t in C) — modern libewf only
pub type libewf_attribute_t = isize;

/// Opaque source type (typedef intptr_t in C) — modern libewf only
pub type libewf_source_t = isize;

/// Opaque subject type (typedef intptr_t in C) — modern libewf only
pub type libewf_subject_t = isize;

// =============================================================================
// Size types matching libewf/types.h
// =============================================================================

pub type size32_t = u32;
pub type size64_t = u64;
pub type ssize64_t = i64;
pub type off64_t = i64;

// =============================================================================
// Constants from libewf/definitions.h
// =============================================================================

/// Access flags
pub const LIBEWF_ACCESS_FLAG_READ: c_int = 0x01;
pub const LIBEWF_ACCESS_FLAG_WRITE: c_int = 0x02;
pub const LIBEWF_ACCESS_FLAG_RESUME: c_int = 0x10;

pub const LIBEWF_OPEN_READ: c_int = LIBEWF_ACCESS_FLAG_READ;
pub const LIBEWF_OPEN_WRITE: c_int = LIBEWF_ACCESS_FLAG_WRITE;
pub const LIBEWF_OPEN_READ_WRITE: c_int = LIBEWF_ACCESS_FLAG_READ | LIBEWF_ACCESS_FLAG_WRITE;

/// File formats
pub const LIBEWF_FORMAT_UNKNOWN: u8 = 0x00;
pub const LIBEWF_FORMAT_ENCASE1: u8 = 0x01;
pub const LIBEWF_FORMAT_ENCASE2: u8 = 0x02;
pub const LIBEWF_FORMAT_ENCASE3: u8 = 0x03;
pub const LIBEWF_FORMAT_ENCASE4: u8 = 0x04;
pub const LIBEWF_FORMAT_ENCASE5: u8 = 0x05;
pub const LIBEWF_FORMAT_ENCASE6: u8 = 0x06;
pub const LIBEWF_FORMAT_ENCASE7: u8 = 0x07;
pub const LIBEWF_FORMAT_SMART: u8 = 0x0e;
pub const LIBEWF_FORMAT_FTK_IMAGER: u8 = 0x0f;
pub const LIBEWF_FORMAT_LOGICAL_ENCASE5: u8 = 0x10; // L01
pub const LIBEWF_FORMAT_LOGICAL_ENCASE6: u8 = 0x11; // L01 v6
pub const LIBEWF_FORMAT_LOGICAL_ENCASE7: u8 = 0x12; // L01 v7
pub const LIBEWF_FORMAT_LINEN5: u8 = 0x25;
pub const LIBEWF_FORMAT_LINEN6: u8 = 0x26;
pub const LIBEWF_FORMAT_LINEN7: u8 = 0x27;
pub const LIBEWF_FORMAT_EWF: u8 = 0x70;
pub const LIBEWF_FORMAT_EWFX: u8 = 0x71;

/// V2 format constants (modern libewf only)
pub const LIBEWF_FORMAT_V2_ENCASE7: u8 = 0x37;
pub const LIBEWF_FORMAT_V2_LOGICAL_ENCASE7: u8 = 0x47;

/// Compression levels
pub const LIBEWF_COMPRESSION_DEFAULT: i8 = -1;
pub const LIBEWF_COMPRESSION_NONE: i8 = 0;
pub const LIBEWF_COMPRESSION_FAST: i8 = 1;
pub const LIBEWF_COMPRESSION_BEST: i8 = 2;

/// Compression methods (modern libewf only — used with set_compression_method)
pub const LIBEWF_COMPRESSION_METHOD_NONE: u16 = 0;
pub const LIBEWF_COMPRESSION_METHOD_DEFLATE: u16 = 1;
pub const LIBEWF_COMPRESSION_METHOD_BZIP2: u16 = 2;

/// Compression flags
pub const LIBEWF_COMPRESS_FLAG_USE_EMPTY_BLOCK_COMPRESSION: u8 = 0x01;
pub const LIBEWF_COMPRESS_FLAG_USE_PATTERN_FILL_COMPRESSION: u8 = 0x10;

/// Media types
pub const LIBEWF_MEDIA_TYPE_REMOVABLE: u8 = 0x00;
pub const LIBEWF_MEDIA_TYPE_FIXED: u8 = 0x01;
pub const LIBEWF_MEDIA_TYPE_OPTICAL: u8 = 0x03;
pub const LIBEWF_MEDIA_TYPE_SINGLE_FILES: u8 = 0x0e;
pub const LIBEWF_MEDIA_TYPE_MEMORY: u8 = 0x10;

/// Media flags
pub const LIBEWF_MEDIA_FLAG_PHYSICAL: u8 = 0x02;
pub const LIBEWF_MEDIA_FLAG_FASTBLOC: u8 = 0x04;
pub const LIBEWF_MEDIA_FLAG_TABLEAU: u8 = 0x08;

/// Default segment file size (1500 MB)
pub const LIBEWF_DEFAULT_SEGMENT_FILE_SIZE: u64 = 1500 * 1024 * 1024;

// =============================================================================
// FFI function declarations
// =============================================================================

extern "C" {
    // =========================================================================
    // Version / info
    // =========================================================================

    /// Returns the library version string (e.g. "20140816")
    pub fn libewf_get_version() -> *const c_char;

    /// Returns the access flags for reading
    pub fn libewf_get_access_flags_read() -> c_int;

    /// Returns the access flags for reading and writing
    pub fn libewf_get_access_flags_read_write() -> c_int;

    /// Returns the access flags for writing
    pub fn libewf_get_access_flags_write() -> c_int;

    // =========================================================================
    // Error handling
    // =========================================================================

    /// Frees an error
    pub fn libewf_error_free(error: *mut *mut libewf_error_t);

    /// Prints error to string buffer
    /// Returns number of printed chars or -1 on error
    pub fn libewf_error_sprint(
        error: *mut libewf_error_t,
        string: *mut c_char,
        size: usize,
    ) -> c_int;

    /// Prints error backtrace to string
    pub fn libewf_error_backtrace_sprint(
        error: *mut libewf_error_t,
        string: *mut c_char,
        size: usize,
    ) -> c_int;

    // =========================================================================
    // Handle lifecycle
    // =========================================================================

    /// Initialize the handle (handle must point to NULL)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_initialize(
        handle: *mut *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Frees the handle including elements
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_free(
        handle: *mut *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Signals the handle to abort its current activity
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_signal_abort(
        handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Open / Close
    // =========================================================================

    /// Globs segment files according to the EWF naming schema
    /// Returns 1 if successful or -1 on error
    pub fn libewf_glob(
        filename: *const c_char,
        filename_length: usize,
        format: u8,
        filenames: *mut *mut *mut c_char,
        number_of_filenames: *mut c_int,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Frees globbed filenames
    /// Returns 1 if successful or -1 on error
    pub fn libewf_glob_free(
        filenames: *mut *mut c_char,
        number_of_filenames: c_int,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Opens a set of EWF file(s)
    /// For writing: filenames should contain the BASE filename (extensions auto-added)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_open(
        handle: *mut libewf_handle_t,
        filenames: *const *mut c_char,
        number_of_filenames: c_int,
        access_flags: c_int,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Closes the EWF handle
    /// Returns 0 if successful or -1 on error
    pub fn libewf_handle_close(
        handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Read operations
    // =========================================================================

    /// Reads (media) data at the current offset into a buffer
    /// Returns the number of bytes read or -1 on error
    pub fn libewf_handle_read_buffer(
        handle: *mut libewf_handle_t,
        buffer: *mut c_void,
        buffer_size: usize,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    /// Reads (media) data at a specific offset
    /// Returns the number of bytes read or -1 on error
    pub fn libewf_handle_read_buffer_at_offset(
        handle: *mut libewf_handle_t,
        buffer: *mut c_void,
        buffer_size: usize,
        offset: off64_t,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    // =========================================================================
    // Write operations
    // =========================================================================

    /// Writes (media) data at the current offset
    /// Will initialize write if necessary
    /// Returns the number of bytes written, 0 when no more, or -1 on error
    pub fn libewf_handle_write_buffer(
        handle: *mut libewf_handle_t,
        buffer: *const c_void,
        buffer_size: usize,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    /// Writes (media) data at a specific offset
    /// Returns the number of bytes written, 0 when no more, or -1 on error
    pub fn libewf_handle_write_buffer_at_offset(
        handle: *mut libewf_handle_t,
        buffer: *const c_void,
        buffer_size: usize,
        offset: off64_t,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    /// Finalizes the write by correcting EWF metadata in segment files
    /// This function is REQUIRED after writing
    /// Returns the number of bytes written or -1 on error
    pub fn libewf_handle_write_finalize(
        handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    // =========================================================================
    // Seek
    // =========================================================================

    /// Seeks a certain offset of the (media) data
    /// Returns the offset if successful or -1 on error
    pub fn libewf_handle_seek_offset(
        handle: *mut libewf_handle_t,
        offset: off64_t,
        whence: c_int,
        error: *mut *mut libewf_error_t,
    ) -> off64_t;

    // =========================================================================
    // Segment file settings
    // =========================================================================

    /// Sets the segment filename (base name, extensions auto-added)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_segment_filename(
        handle: *mut libewf_handle_t,
        filename: *const c_char,
        filename_length: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the maximum segment size
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_maximum_segment_size(
        handle: *mut libewf_handle_t,
        maximum_segment_size: *mut size64_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the maximum segment file size
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_maximum_segment_size(
        handle: *mut libewf_handle_t,
        maximum_segment_size: size64_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Media metadata
    // =========================================================================

    /// Gets the number of sectors per chunk
    pub fn libewf_handle_get_sectors_per_chunk(
        handle: *mut libewf_handle_t,
        sectors_per_chunk: *mut u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the number of sectors per chunk
    pub fn libewf_handle_set_sectors_per_chunk(
        handle: *mut libewf_handle_t,
        sectors_per_chunk: u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the number of bytes per sector
    pub fn libewf_handle_get_bytes_per_sector(
        handle: *mut libewf_handle_t,
        bytes_per_sector: *mut u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the number of bytes per sector
    pub fn libewf_handle_set_bytes_per_sector(
        handle: *mut libewf_handle_t,
        bytes_per_sector: u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the media size
    pub fn libewf_handle_get_media_size(
        handle: *mut libewf_handle_t,
        media_size: *mut size64_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the media size (stored as sectors * bytes_per_sector)
    pub fn libewf_handle_set_media_size(
        handle: *mut libewf_handle_t,
        media_size: size64_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the media type
    pub fn libewf_handle_get_media_type(
        handle: *mut libewf_handle_t,
        media_type: *mut u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the media type (e.g. LIBEWF_MEDIA_TYPE_FIXED, SINGLE_FILES)
    pub fn libewf_handle_set_media_type(
        handle: *mut libewf_handle_t,
        media_type: u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the media flags
    pub fn libewf_handle_get_media_flags(
        handle: *mut libewf_handle_t,
        media_flags: *mut u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the media flags
    pub fn libewf_handle_set_media_flags(
        handle: *mut libewf_handle_t,
        media_flags: u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the format type
    pub fn libewf_handle_get_format(
        handle: *mut libewf_handle_t,
        format: *mut u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the output format (e.g. LIBEWF_FORMAT_ENCASE5, LOGICAL_ENCASE5)
    pub fn libewf_handle_set_format(
        handle: *mut libewf_handle_t,
        format: u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Compression settings
    // =========================================================================

    /// Gets compression values
    pub fn libewf_handle_get_compression_values(
        handle: *mut libewf_handle_t,
        compression_level: *mut i8,
        compression_flags: *mut u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets compression values
    pub fn libewf_handle_set_compression_values(
        handle: *mut libewf_handle_t,
        compression_level: i8,
        compression_flags: u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Compression method (modern libewf 20251220+)
    // =========================================================================

    /// Gets the compression method (NONE=0, DEFLATE=1, BZIP2=2)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_compression_method(
        handle: *mut libewf_handle_t,
        compression_method: *mut u16,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the compression method (NONE=0, DEFLATE=1, BZIP2=2)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_compression_method(
        handle: *mut libewf_handle_t,
        compression_method: u16,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Data chunk operations (modern libewf 20251220+)
    // =========================================================================

    /// Retrieves a (media) data chunk
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_data_chunk(
        handle: *mut libewf_handle_t,
        data_chunk: *mut *mut libewf_data_chunk_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Reads a (media) data chunk at the current offset
    /// Returns the number of bytes in the data chunk, 0 when no more, or -1 on error
    pub fn libewf_handle_read_data_chunk(
        handle: *mut libewf_handle_t,
        data_chunk: *mut libewf_data_chunk_t,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    /// Writes a (media) data chunk at the current offset
    /// Returns the number of bytes written, 0 when no more, or -1 on error
    pub fn libewf_handle_write_data_chunk(
        handle: *mut libewf_handle_t,
        data_chunk: *mut libewf_data_chunk_t,
        error: *mut *mut libewf_error_t,
    ) -> isize;

    /// Frees a data chunk
    /// Returns 1 if successful or -1 on error
    pub fn libewf_data_chunk_free(
        data_chunk: *mut *mut libewf_data_chunk_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Offset / position
    // =========================================================================

    /// Retrieves the current offset of the (media) data
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_offset(
        handle: *mut libewf_handle_t,
        offset: *mut off64_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Segment file info (modern libewf 20251220+)
    // =========================================================================

    /// Determines if the segment files are corrupted
    /// Returns 1 if corrupted, 0 if not, or -1 on error
    pub fn libewf_handle_segment_files_corrupted(
        handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Determines if the segment files are encrypted
    /// Returns 1 if encrypted, 0 if not, or -1 on error
    pub fn libewf_handle_segment_files_encrypted(
        handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Retrieves the segment file version (major, minor)
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_segment_file_version(
        handle: *mut libewf_handle_t,
        major_version: *mut u8,
        minor_version: *mut u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the number of chunks written
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_get_number_of_chunks_written(
        handle: *mut libewf_handle_t,
        number_of_chunks: *mut u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the read zero chunk on error flag
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_read_zero_chunk_on_error(
        handle: *mut libewf_handle_t,
        zero_on_error: u8,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Handle copy operations (modern libewf 20251220+)
    // =========================================================================

    /// Copies the media values from the source to the destination handle
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_copy_media_values(
        destination_handle: *mut libewf_handle_t,
        source_handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Copies the header values from the source to the destination handle
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_copy_header_values(
        destination_handle: *mut libewf_handle_t,
        source_handle: *mut libewf_handle_t,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Header values (UTF-8)
    // =========================================================================

    /// Gets the number of header values
    pub fn libewf_handle_get_number_of_header_values(
        handle: *mut libewf_handle_t,
        number_of_values: *mut u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the size of the header value identifier at a specific index
    /// The identifier size includes the end of string character
    pub fn libewf_handle_get_header_value_identifier_size(
        handle: *mut libewf_handle_t,
        index: u32,
        identifier_size: *mut usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the header value identifier at a specific index
    /// The identifier size should include the end of string character
    pub fn libewf_handle_get_header_value_identifier(
        handle: *mut libewf_handle_t,
        index: u32,
        identifier: *mut u8,
        identifier_size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the size of a UTF-8 header value by identifier
    pub fn libewf_handle_get_utf8_header_value_size(
        handle: *mut libewf_handle_t,
        identifier: *const u8,
        identifier_length: usize,
        utf8_string_size: *mut usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets a UTF-8 header value by identifier
    pub fn libewf_handle_get_utf8_header_value(
        handle: *mut libewf_handle_t,
        identifier: *const u8,
        identifier_length: usize,
        utf8_string: *mut u8,
        utf8_string_size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets a UTF-8 header value by identifier
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_utf8_header_value(
        handle: *mut libewf_handle_t,
        identifier: *const u8,
        identifier_length: usize,
        utf8_string: *const u8,
        utf8_string_length: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    // =========================================================================
    // Hash values
    // =========================================================================

    /// Gets the number of hash values
    pub fn libewf_handle_get_number_of_hash_values(
        handle: *mut libewf_handle_t,
        number_of_values: *mut u32,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the MD5 hash (16 bytes)
    pub fn libewf_handle_get_md5_hash(
        handle: *mut libewf_handle_t,
        md5_hash: *mut u8,
        size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the MD5 hash (16 bytes)
    pub fn libewf_handle_set_md5_hash(
        handle: *mut libewf_handle_t,
        md5_hash: *const u8,
        size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Gets the SHA1 hash (20 bytes)
    pub fn libewf_handle_get_sha1_hash(
        handle: *mut libewf_handle_t,
        sha1_hash: *mut u8,
        size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets the SHA1 hash (20 bytes)
    pub fn libewf_handle_set_sha1_hash(
        handle: *mut libewf_handle_t,
        sha1_hash: *const u8,
        size: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;

    /// Sets a UTF-8 hash value by identifier
    /// Returns 1 if successful or -1 on error
    pub fn libewf_handle_set_utf8_hash_value(
        handle: *mut libewf_handle_t,
        identifier: *const u8,
        identifier_length: usize,
        utf8_string: *const u8,
        utf8_string_length: usize,
        error: *mut *mut libewf_error_t,
    ) -> c_int;
}
