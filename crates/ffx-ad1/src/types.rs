// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Container Types
//!
//! ## Section Brief
//! Type definitions for AD1 (AccessData Logical Image) container format:
//!
//! ### Error Types
//! - `Ad1Error` - Structured error type for AD1 operations
//!
//! ### Public API Types (Serializable)
//! - `SegmentHeaderInfo` - Segment header metadata
//! - `LogicalHeaderInfo` - Logical header metadata
//! - `VolumeInfo` - Volume information
//! - `CompanionLogInfo` - Companion log file metadata
//! - `TreeEntry` - File/folder entry in container tree
//! - `VerifyEntry` - Hash verification result
//! - `Ad1Info` - Complete container information
//!
//! ### Internal Types
//! - `SegmentHeader` - Raw segment header (parsing)
//! - `LogicalHeader` - Raw logical header (parsing)
//! - `Item` - Parsed file/folder item
//! - `Metadata` - Item metadata entry
//!
//! ### Constants
//! - Magic signatures (`AD1_SIGNATURE`, `AD1_FOLDER_SIGNATURE`)
//! - Metadata type IDs (`CREATED`, `MODIFIED`, `MD5_HASH`, etc.)
//! - Size constants (`SEGMENT_BLOCK_SIZE`, `AD1_LOGICAL_MARGIN`)

use serde::Serialize;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Structured error type for AD1 container operations
#[derive(Debug, Clone, Error)]
pub enum Ad1Error {
    /// File or entry not found
    #[error("File not found: {0}")]
    NotFound(String),
    /// Invalid AD1 format or corrupted file
    #[error("Invalid AD1 format: {0}")]
    InvalidFormat(String),
    /// Missing segment file
    #[error("Missing segment {index} at {path}")]
    SegmentMissing { path: String, index: u32 },
    /// Zlib decompression error
    #[error("Decompression error: {0}")]
    DecompressionError(String),
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
    /// Invalid hash algorithm
    #[error("Invalid hash algorithm: {0}")]
    InvalidAlgorithm(String),
    /// Entry path not found in container
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
    /// Offset out of range
    #[error("Offset {offset} out of range (max: {max})")]
    OutOfRange { offset: u64, max: u64 },
    /// Encrypted AD1 file (requires decryption)
    #[error("Encrypted AD1: {0}")]
    EncryptedFile(String),
}

impl From<std::io::Error> for Ad1Error {
    fn from(e: std::io::Error) -> Self {
        Ad1Error::IoError(e.to_string())
    }
}

/// Convert Ad1Error to String for backwards compatibility
impl From<Ad1Error> for String {
    fn from(e: Ad1Error) -> Self {
        e.to_string()
    }
}

// =============================================================================
// Public API Types (Serializable)
// =============================================================================

/// Segment header information (public view)
#[derive(Debug, Clone, Serialize)]
pub struct SegmentHeaderInfo {
    pub signature: String,
    pub segment_index: u32,
    pub segment_number: u32,
    pub fragments_size: u32,
    pub header_size: u32,
}

/// Logical header information (public view)
#[derive(Debug, Clone, Serialize)]
pub struct LogicalHeaderInfo {
    pub signature: String,
    pub image_version: u32,
    pub zlib_chunk_size: u32,
    pub logical_metadata_addr: u64,
    pub first_item_addr: u64,
    pub data_source_name_length: u32,
    pub ad_signature: String,
    pub data_source_name_addr: u64,
    pub attrguid_footer_addr: u64,
    pub locsguid_footer_addr: u64,
    pub data_source_name: String,
}

/// Volume information from AD1 header
#[derive(Debug, Clone, Default, Serialize)]
pub struct VolumeInfo {
    pub volume_label: Option<String>,
    pub filesystem: Option<String>,
    pub os_info: Option<String>,
    pub block_size: Option<u32>,
    pub volume_serial: Option<String>,
}

impl VolumeInfo {
    /// Create new empty VolumeInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set volume label
    #[inline]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.volume_label = Some(label.into());
        self
    }

    /// Set filesystem type
    #[inline]
    pub fn with_filesystem(mut self, fs: impl Into<String>) -> Self {
        self.filesystem = Some(fs.into());
        self
    }

    /// Set OS info
    #[inline]
    pub fn with_os_info(mut self, os: impl Into<String>) -> Self {
        self.os_info = Some(os.into());
        self
    }

    /// Set block size
    #[inline]
    pub fn with_block_size(mut self, size: u32) -> Self {
        self.block_size = Some(size);
        self
    }

    /// Set volume serial
    #[inline]
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.volume_serial = Some(serial.into());
        self
    }

    /// Check if any volume info is present
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.volume_label.is_none()
            && self.filesystem.is_none()
            && self.os_info.is_none()
            && self.block_size.is_none()
            && self.volume_serial.is_none()
    }
}

/// Companion log file metadata (.ad1.txt, .log, etc.)
/// Contains case metadata, acquisition details, and hash information
#[derive(Debug, Clone, Default, Serialize)]
pub struct CompanionLogInfo {
    /// Case number or identifier
    pub case_number: Option<String>,
    /// Evidence number or item number
    pub evidence_number: Option<String>,
    /// Examiner or analyst name
    pub examiner: Option<String>,
    /// Free-form notes or description
    pub notes: Option<String>,
    /// MD5 hash of the container or source
    pub md5_hash: Option<String>,
    /// SHA1 hash of the container or source
    pub sha1_hash: Option<String>,
    /// SHA256 hash (if available)
    pub sha256_hash: Option<String>,
    /// Date/time of acquisition
    pub acquisition_date: Option<String>,
    /// Source device or media description
    pub source_device: Option<String>,
    /// Source path or location
    pub source_path: Option<String>,
    /// Acquisition tool name and version
    pub acquisition_tool: Option<String>,
    /// Total items or files processed
    pub total_items: Option<u64>,
    /// Total size of acquired data
    pub total_size: Option<u64>,
    /// Acquisition method (logical, physical, etc.)
    pub acquisition_method: Option<String>,
    /// Organization or agency
    pub organization: Option<String>,
}

impl CompanionLogInfo {
    /// Create new empty CompanionLogInfo
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case number
    #[inline]
    pub fn with_case_number(mut self, case_number: impl Into<String>) -> Self {
        self.case_number = Some(case_number.into());
        self
    }

    /// Set evidence number
    #[inline]
    pub fn with_evidence_number(mut self, evidence_number: impl Into<String>) -> Self {
        self.evidence_number = Some(evidence_number.into());
        self
    }

    /// Set examiner
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner = Some(examiner.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set MD5 hash
    #[inline]
    pub fn with_md5(mut self, hash: impl Into<String>) -> Self {
        self.md5_hash = Some(hash.into());
        self
    }

    /// Set SHA1 hash
    #[inline]
    pub fn with_sha1(mut self, hash: impl Into<String>) -> Self {
        self.sha1_hash = Some(hash.into());
        self
    }

    /// Set SHA256 hash
    #[inline]
    pub fn with_sha256(mut self, hash: impl Into<String>) -> Self {
        self.sha256_hash = Some(hash.into());
        self
    }

    /// Set acquisition date
    #[inline]
    pub fn with_acquisition_date(mut self, date: impl Into<String>) -> Self {
        self.acquisition_date = Some(date.into());
        self
    }

    /// Set source device
    #[inline]
    pub fn with_source_device(mut self, device: impl Into<String>) -> Self {
        self.source_device = Some(device.into());
        self
    }

    /// Set acquisition tool
    #[inline]
    pub fn with_acquisition_tool(mut self, tool: impl Into<String>) -> Self {
        self.acquisition_tool = Some(tool.into());
        self
    }

    /// Set acquisition totals
    #[inline]
    pub fn with_totals(mut self, items: u64, size: u64) -> Self {
        self.total_items = Some(items);
        self.total_size = Some(size);
        self
    }

    /// Check if any case info is present
    #[inline]
    pub fn has_case_info(&self) -> bool {
        self.case_number.is_some() || self.evidence_number.is_some() || self.examiner.is_some()
    }

    /// Check if any hash info is present
    #[inline]
    pub fn has_hashes(&self) -> bool {
        self.md5_hash.is_some() || self.sha1_hash.is_some() || self.sha256_hash.is_some()
    }
}

/// File/folder entry in the AD1 tree
///
/// NOTE: This is AD1-specific and differs from `containers::traits::TreeEntryInfo`:
/// - Has separate `md5_hash`, `sha1_hash` fields (AD1 stores multiple hashes)
/// - Has `item_type` field (AD1-specific type identifier)
/// - Uses `is_dir` instead of `is_directory` (legacy API compatibility)
/// - Has `attributes` for Windows file attributes
///
/// The shared `TreeEntryInfo` is used for the unified trait-based API,
/// while this type is used for the AD1-specific direct API.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TreeEntry {
    /// Full path within the container
    pub path: String,
    /// Item name only (extracted from path)
    pub name: String,
    /// Whether this entry is a directory
    pub is_dir: bool,
    /// Decompressed size in bytes
    pub size: u64,
    /// Item type (0x05 = folder, 0x00 = file)
    pub item_type: u32,
    /// Address of first child item (for lazy loading directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_child_addr: Option<u64>,
    /// Address of compressed data (for reading file content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_addr: Option<u64>,
    /// Address of the item header in the container (hex location)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_addr: Option<u64>,
    /// Size of compressed data in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<u64>,
    /// Address where compressed data ends
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_end_addr: Option<u64>,
    /// Address of first metadata entry for this item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_addr: Option<u64>,
    /// MD5 hash if stored in metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_hash: Option<String>,
    /// SHA1 hash if stored in metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1_hash: Option<String>,
    /// Created timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Accessed timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<String>,
    /// Modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// File attributes flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    /// Number of child entries (for lazy loading indicator)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_count: Option<usize>,
}

/// On-demand metadata for a single item
/// Used for lazy loading of metadata after initial tree display
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemMetadata {
    /// Item address this metadata belongs to
    pub item_addr: u64,
    /// MD5 hash if stored in metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_hash: Option<String>,
    /// SHA1 hash if stored in metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1_hash: Option<String>,
    /// Created timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Accessed timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<String>,
    /// Modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// File attributes flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
}

/// Verification status for hash comparison results
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerifyStatus {
    /// Hash matches stored value
    Ok,
    /// Hash doesn't match stored value (verification failed)
    Nok,
    /// No stored hash to compare, only computed hash available
    Computed,
    /// Verification was skipped (e.g., directory, zero-size file)
    Skipped,
}

impl VerifyStatus {
    /// Returns true if verification passed (either matched or computed-only)
    pub fn is_ok(&self) -> bool {
        matches!(self, VerifyStatus::Ok | VerifyStatus::Computed)
    }

    /// Returns true if verification failed (mismatch)
    pub fn is_error(&self) -> bool {
        matches!(self, VerifyStatus::Nok)
    }
}

impl std::fmt::Display for VerifyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyStatus::Ok => write!(f, "ok"),
            VerifyStatus::Nok => write!(f, "nok"),
            VerifyStatus::Computed => write!(f, "computed"),
            VerifyStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Verification result entry for AD1 file-level hash verification
///
/// NOTE: This is AD1-specific and differs from `containers::VerifyEntry`:
/// - Contains hash comparison details (algorithm, computed, stored)
/// - Used for verifying individual files within AD1 containers
/// - Includes file size for progress tracking
///
/// The `containers::VerifyEntry` is used for chunk-level verification
/// (E01 CRC checks, etc.) and has different fields.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyEntry {
    pub path: String,
    pub status: VerifyStatus,
    /// The hash algorithm used (e.g., "md5", "sha1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// The computed hash value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed: Option<String>,
    /// The stored hash value from AD1 metadata (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stored: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Complete AD1 container information
#[derive(Debug, Clone, Serialize)]
pub struct Ad1Info {
    pub segment: SegmentHeaderInfo,
    pub logical: LogicalHeaderInfo,
    pub item_count: u64,
    pub tree: Option<Vec<TreeEntry>>,
    pub segment_files: Option<Vec<String>>,
    /// Size of each segment file in bytes
    pub segment_sizes: Option<Vec<u64>>,
    /// Total size of all segment files combined
    pub total_size: Option<u64>,
    /// Missing segment files (incomplete container)
    pub missing_segments: Option<Vec<String>>,
    /// Detailed segment information with offset ranges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_summary: Option<SegmentSummary>,
    pub volume: Option<VolumeInfo>,
    pub companion_log: Option<CompanionLogInfo>,
}

// =============================================================================
// Internal Types (used by parser and operations)
// =============================================================================

// =============================================================================
// Internal Types (Parsing)
// =============================================================================

/// Internal segment header structure
#[derive(Debug, Clone)]
pub(crate) struct SegmentHeader {
    pub signature: [u8; 16],
    pub segment_index: u32,
    pub segment_number: u32,
    pub fragments_size: u32,
    pub header_size: u32,
}

/// Internal logical header structure  
#[derive(Debug, Clone)]
pub(crate) struct LogicalHeader {
    pub signature: [u8; 16],
    pub image_version: u32,
    pub zlib_chunk_size: u32,
    pub logical_metadata_addr: u64,
    pub first_item_addr: u64,
    pub data_source_name_length: u32,
    pub ad_signature: [u8; 4],
    pub data_source_name_addr: u64,
    pub attrguid_footer_addr: u64,
    pub locsguid_footer_addr: u64,
    pub data_source_name: String,
}

/// Item metadata entry
#[derive(Debug, Clone)]
pub(crate) struct Metadata {
    pub next_metadata_addr: u64,
    pub category: u32,
    pub key: u32,
    pub data: Vec<u8>,
}

/// Item in the AD1 tree (file or folder)
#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub id: u64,
    pub name: String,
    pub item_type: u32,
    pub decompressed_size: u64,
    pub zlib_metadata_addr: u64,
    pub metadata: Vec<Metadata>,
    pub children: Vec<Item>,
}

// =============================================================================
// Segment Information Types
// =============================================================================

/// Detailed information about a single AD1 segment file
#[derive(Debug, Clone, Serialize)]
pub struct SegmentFileInfo {
    /// Segment number (1-based: .ad1, .ad2, etc.)
    pub number: u32,
    /// Full path to the segment file
    pub path: String,
    /// File name only
    pub filename: String,
    /// Segment file size in bytes
    pub size: u64,
    /// Whether the segment file exists
    pub exists: bool,
    /// Expected data size (excluding header margin)
    pub data_size: u64,
    /// Offset range this segment covers (start)
    pub offset_start: u64,
    /// Offset range this segment covers (end)
    pub offset_end: u64,
}

/// Summary of all segment files for an AD1 container
#[derive(Debug, Clone, Default, Serialize)]
pub struct SegmentSummary {
    /// Total number of expected segments
    pub expected_count: u32,
    /// Number of segments found
    pub found_count: u32,
    /// Number of segments missing
    pub missing_count: u32,
    /// Total size of all segment files
    pub total_size: u64,
    /// Total data size (excluding headers)
    pub total_data_size: u64,
    /// List of segment file information
    pub segments: Vec<SegmentFileInfo>,
    /// Whether the container is complete (all segments present)
    pub is_complete: bool,
}

impl SegmentSummary {
    /// Create a new SegmentSummary
    #[inline]
    pub fn new(expected_count: u32) -> Self {
        Self {
            expected_count,
            ..Default::default()
        }
    }

    /// Add a segment
    #[inline]
    pub fn add_segment(mut self, segment: SegmentFileInfo) -> Self {
        self.total_size += segment.size;
        self.total_data_size += segment.data_size;
        self.segments.push(segment);
        self.found_count = self.segments.len() as u32;
        self.missing_count = self.expected_count.saturating_sub(self.found_count);
        self.is_complete = self.found_count >= self.expected_count;
        self
    }

    /// Mark as complete
    #[inline]
    pub fn complete(mut self) -> Self {
        self.is_complete = true;
        self.missing_count = 0;
        self
    }
}

// =============================================================================
// Statistics Types
// =============================================================================

/// Container statistics summary
#[derive(Debug, Clone, Default, Serialize)]
pub struct Ad1Stats {
    /// Total number of items (files + folders)
    pub total_items: u64,
    /// Total number of files
    pub total_files: u64,
    /// Total number of folders
    pub total_folders: u64,
    /// Total uncompressed size of all files
    pub total_size: u64,
    /// Total compressed size (segment files)
    pub compressed_size: u64,
    /// Compression ratio (compressed/uncompressed)
    pub compression_ratio: f64,
    /// Maximum nesting depth
    pub max_depth: u32,
    /// Number of files with MD5 hashes
    pub files_with_md5: u64,
    /// Number of files with SHA1 hashes
    pub files_with_sha1: u64,
    /// Largest file size
    pub largest_file_size: u64,
    /// Largest file path
    pub largest_file_path: Option<String>,
}

impl Ad1Stats {
    /// Create a new Ad1Stats
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a file to the stats
    #[inline]
    pub fn add_file(&mut self, size: u64, has_md5: bool, has_sha1: bool, path: Option<&str>) {
        self.total_files += 1;
        self.total_items += 1;
        self.total_size += size;
        if has_md5 {
            self.files_with_md5 += 1;
        }
        if has_sha1 {
            self.files_with_sha1 += 1;
        }
        if size > self.largest_file_size {
            self.largest_file_size = size;
            self.largest_file_path = path.map(|s| s.to_string());
        }
    }

    /// Add a folder to the stats
    #[inline]
    pub fn add_folder(&mut self) {
        self.total_folders += 1;
        self.total_items += 1;
    }

    /// Update depth if greater than current max
    #[inline]
    pub fn update_depth(&mut self, depth: u32) {
        if depth > self.max_depth {
            self.max_depth = depth;
        }
    }

    /// Set compressed size and calculate ratio
    #[inline]
    pub fn with_compressed_size(mut self, size: u64) -> Self {
        self.compressed_size = size;
        if self.total_size > 0 {
            self.compression_ratio = size as f64 / self.total_size as f64;
        }
        self
    }
}

/// Search result entry
#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchResult {
    /// The matching entry
    pub entry: TreeEntry,
    /// Match reason (name, extension, hash)
    pub match_type: String,
    /// Depth in tree
    pub depth: u32,
}

impl SearchResult {
    /// Create a new search result
    #[inline]
    pub fn new(entry: TreeEntry, match_type: impl Into<String>, depth: u32) -> Self {
        Self {
            entry,
            match_type: match_type.into(),
            depth,
        }
    }

    /// Create a name match result
    #[inline]
    pub fn name_match(entry: TreeEntry, depth: u32) -> Self {
        Self::new(entry, "name", depth)
    }

    /// Create an extension match result
    #[inline]
    pub fn extension_match(entry: TreeEntry, depth: u32) -> Self {
        Self::new(entry, "extension", depth)
    }

    /// Create a hash match result
    #[inline]
    pub fn hash_match(entry: TreeEntry, depth: u32) -> Self {
        Self::new(entry, "hash", depth)
    }
}

/// Chunk verification result (for parity with EWF)
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChunkVerifyResult {
    /// Chunk/item index
    pub index: u64,
    /// Verification status
    pub status: String,
    /// Optional message or hash value
    pub message: Option<String>,
    /// Path to the verified item
    pub path: Option<String>,
}

impl ChunkVerifyResult {
    /// Create a successful verification result
    #[inline]
    pub fn ok(index: u64) -> Self {
        Self {
            index,
            status: "ok".to_string(),
            ..Default::default()
        }
    }

    /// Create a failed verification result
    #[inline]
    pub fn failed(index: u64, message: impl Into<String>) -> Self {
        Self {
            index,
            status: "failed".to_string(),
            message: Some(message.into()),
            path: None,
        }
    }

    /// Create a skipped verification result
    #[inline]
    pub fn skipped(index: u64, reason: impl Into<String>) -> Self {
        Self {
            index,
            status: "skipped".to_string(),
            message: Some(reason.into()),
            path: None,
        }
    }

    /// Set path
    #[inline]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

// =============================================================================
// Constants
// =============================================================================

pub(crate) const AD1_SIGNATURE: &[u8; 15] = b"ADSEGMENTEDFILE";
pub(crate) const AD1_LOGICAL_MARGIN: u64 = 512;
pub(crate) const AD1_FOLDER_SIGNATURE: u32 = 0x05;
pub(crate) const CACHE_SIZE: usize = 100;
pub(crate) const SEGMENT_BLOCK_SIZE: u64 = 65_536;

// Metadata categories
pub(crate) const HASH_INFO: u32 = 0x01;
#[allow(dead_code)]
pub(crate) const FILE_INFO: u32 = 0x02;
pub(crate) const ATTRIBUTES: u32 = 0x03;
#[allow(dead_code)]
pub(crate) const PERMISSIONS: u32 = 0x04;
pub(crate) const TIMESTAMP: u32 = 0x05;

// Hash keys
pub(crate) const MD5_HASH: u32 = 0x5001;
pub(crate) const SHA1_HASH: u32 = 0x5002;
#[allow(dead_code)]
pub(crate) const SHA256_HASH: u32 = 0x5003;

// Timestamp keys
pub(crate) const CREATED: u32 = 0x06;
pub(crate) const ACCESS: u32 = 0x07;
pub(crate) const MODIFIED: u32 = 0x08;
#[allow(dead_code)]
pub(crate) const METADATA_CHANGED: u32 = 0x09;

// File info keys
#[allow(dead_code)]
pub(crate) const FILE_SIZE: u32 = 0x01;
#[allow(dead_code)]
pub(crate) const ORIGINAL_PATH: u32 = 0x02;
#[allow(dead_code)]
pub(crate) const FILE_EXTENSION: u32 = 0x03;

// Attribute keys
pub(crate) const READONLY: u32 = 0x01;
pub(crate) const HIDDEN: u32 = 0x02;
pub(crate) const SYSTEM: u32 = 0x03;
pub(crate) const ARCHIVE: u32 = 0x04;
pub(crate) const ENCRYPTED: u32 = 0x05;
pub(crate) const COMPRESSED: u32 = 0x06;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad1_signature_constant() {
        assert_eq!(AD1_SIGNATURE.len(), 15);
        assert_eq!(AD1_SIGNATURE, b"ADSEGMENTEDFILE");
    }

    #[test]
    fn test_ad1_logical_margin() {
        assert_eq!(AD1_LOGICAL_MARGIN, 512);
    }

    #[test]
    fn test_ad1_folder_signature() {
        assert_eq!(AD1_FOLDER_SIGNATURE, 0x05);
    }

    #[test]
    fn test_segment_block_size() {
        assert_eq!(SEGMENT_BLOCK_SIZE, 65536);
    }

    #[test]
    fn test_cache_size() {
        assert_eq!(CACHE_SIZE, 100);
    }

    #[test]
    fn test_metadata_category_constants() {
        assert_eq!(HASH_INFO, 0x01);
        assert_eq!(ATTRIBUTES, 0x03);
        assert_eq!(TIMESTAMP, 0x05);
    }

    #[test]
    fn test_hash_key_constants() {
        assert_eq!(MD5_HASH, 0x5001);
        assert_eq!(SHA1_HASH, 0x5002);
    }

    #[test]
    fn test_timestamp_key_constants() {
        assert_eq!(CREATED, 0x06);
        assert_eq!(ACCESS, 0x07);
        assert_eq!(MODIFIED, 0x08);
    }

    #[test]
    fn test_attribute_key_constants() {
        assert_eq!(READONLY, 0x01);
        assert_eq!(HIDDEN, 0x02);
        assert_eq!(SYSTEM, 0x03);
        assert_eq!(ARCHIVE, 0x04);
        assert_eq!(ENCRYPTED, 0x05);
        assert_eq!(COMPRESSED, 0x06);
    }

    #[test]
    fn test_volume_info_default() {
        let volume = VolumeInfo::default();
        assert!(volume.volume_label.is_none());
        assert!(volume.filesystem.is_none());
        assert!(volume.os_info.is_none());
        assert!(volume.block_size.is_none());
        assert!(volume.volume_serial.is_none());
    }

    #[test]
    fn test_volume_info_with_values() {
        let volume = VolumeInfo {
            volume_label: Some("EVIDENCE".to_string()),
            filesystem: Some("NTFS".to_string()),
            os_info: Some("Windows 10".to_string()),
            block_size: Some(4096),
            volume_serial: Some("1234-ABCD".to_string()),
        };
        assert_eq!(volume.volume_label, Some("EVIDENCE".to_string()));
        assert_eq!(volume.filesystem, Some("NTFS".to_string()));
    }

    #[test]
    fn test_companion_log_info_default() {
        let log = CompanionLogInfo::default();
        assert!(log.case_number.is_none());
        assert!(log.evidence_number.is_none());
        assert!(log.examiner.is_none());
        assert!(log.md5_hash.is_none());
        assert!(log.sha1_hash.is_none());
    }

    #[test]
    fn test_companion_log_info_with_values() {
        let log = CompanionLogInfo {
            case_number: Some("CASE-2024-001".to_string()),
            evidence_number: Some("EV-001".to_string()),
            examiner: Some("John Doe".to_string()),
            notes: Some("Forensic acquisition".to_string()),
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()),
            sha256_hash: None,
            acquisition_date: Some("2024-01-01T10:00:00Z".to_string()),
            source_device: Some("USB Drive".to_string()),
            source_path: Some("E:\\".to_string()),
            acquisition_tool: Some("FTK Imager 4.7".to_string()),
            total_items: Some(15000),
            total_size: Some(1073741824),
            acquisition_method: Some("Logical".to_string()),
            organization: Some("Forensic Lab".to_string()),
        };
        assert_eq!(log.case_number, Some("CASE-2024-001".to_string()));
        assert_eq!(log.md5_hash.as_ref().unwrap().len(), 32);
    }

    #[test]
    fn test_tree_entry_file() {
        let entry = TreeEntry {
            path: "/Documents/report.pdf".to_string(),
            name: "report.pdf".to_string(),
            is_dir: false,
            size: 1024000,
            item_type: 0x00,
            first_child_addr: None,
            data_addr: Some(12345),
            item_addr: Some(12345),
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: Some("abc123".to_string()),
            sha1_hash: None,
            created: Some("2024-01-01T10:00:00Z".to_string()),
            accessed: None,
            modified: Some("2024-01-02T14:30:00Z".to_string()),
            attributes: None,
            child_count: None,
        };
        assert!(!entry.is_dir);
        assert_eq!(entry.item_type, 0x00);
        assert!(entry.md5_hash.is_some());
    }

    #[test]
    fn test_tree_entry_folder() {
        let entry = TreeEntry {
            path: "/Documents".to_string(),
            name: "Documents".to_string(),
            is_dir: true,
            size: 0,
            item_type: AD1_FOLDER_SIGNATURE,
            first_child_addr: Some(12345),
            data_addr: None,
            item_addr: Some(12345),
            compressed_size: None,
            data_end_addr: None,
            metadata_addr: None,
            md5_hash: None,
            sha1_hash: None,
            created: None,
            accessed: None,
            modified: None,
            attributes: Some(vec!["DIRECTORY".to_string()]),
            child_count: Some(5),
        };
        assert!(entry.is_dir);
        assert_eq!(entry.item_type, 0x05);
        assert!(entry.attributes.is_some());
    }

    #[test]
    fn test_verify_entry_match() {
        let entry = VerifyEntry {
            path: "/test.txt".to_string(),
            status: VerifyStatus::Ok,
            algorithm: Some("md5".to_string()),
            computed: Some("abc123".to_string()),
            stored: Some("abc123".to_string()),
            size: Some(1024),
        };
        assert_eq!(entry.status, VerifyStatus::Ok);
        assert!(entry.status.is_ok());
        assert!(!entry.status.is_error());
        assert_eq!(entry.computed, entry.stored);
    }

    #[test]
    fn test_verify_entry_mismatch() {
        let entry = VerifyEntry {
            path: "/test.txt".to_string(),
            status: VerifyStatus::Nok,
            algorithm: Some("md5".to_string()),
            computed: Some("abc123".to_string()),
            stored: Some("def456".to_string()),
            size: Some(1024),
        };
        assert_eq!(entry.status, VerifyStatus::Nok);
        assert!(!entry.status.is_ok());
        assert!(entry.status.is_error());
        assert_ne!(entry.computed, entry.stored);
    }

    #[test]
    fn test_verify_status_display() {
        assert_eq!(VerifyStatus::Ok.to_string(), "ok");
        assert_eq!(VerifyStatus::Nok.to_string(), "nok");
        assert_eq!(VerifyStatus::Computed.to_string(), "computed");
        assert_eq!(VerifyStatus::Skipped.to_string(), "skipped");
    }

    #[test]
    fn test_segment_file_info() {
        let segment = SegmentFileInfo {
            number: 1,
            path: "/evidence/disk.ad1".to_string(),
            filename: "disk.ad1".to_string(),
            size: 2147483648, // 2 GB
            exists: true,
            data_size: 2147483136,
            offset_start: 0,
            offset_end: 2147483647,
        };
        assert_eq!(segment.number, 1);
        assert!(segment.exists);
        assert!(segment.data_size < segment.size); // Data size excludes header
    }

    #[test]
    fn test_segment_summary_complete() {
        let summary = SegmentSummary {
            expected_count: 3,
            found_count: 3,
            missing_count: 0,
            total_size: 6442450944, // 6 GB
            total_data_size: 6442448896,
            segments: vec![],
            is_complete: true,
        };
        assert!(summary.is_complete);
        assert_eq!(summary.missing_count, 0);
        assert_eq!(summary.expected_count, summary.found_count);
    }

    #[test]
    fn test_segment_summary_incomplete() {
        let summary = SegmentSummary {
            expected_count: 5,
            found_count: 3,
            missing_count: 2,
            total_size: 3221225472,
            total_data_size: 3221223424,
            segments: vec![],
            is_complete: false,
        };
        assert!(!summary.is_complete);
        assert_eq!(summary.missing_count, 2);
    }
}
