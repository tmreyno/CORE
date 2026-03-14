// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Type definitions for the pure-Rust L01 (Logical Evidence File) writer.
//!
//! L01 is the EWF v1 logical format used by EnCase for logical acquisitions.
//! It uses the LVF signature and stores file-level data (not disk sectors)
//! with an ltree section describing the file hierarchy.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─── Signatures ─────────────────────────────────────────────────────────────

/// LVF signature for L01 files (EWF v1 logical)
pub const LVF_SIGNATURE: &[u8; 8] = b"LVF\x09\x0d\x0a\xff\x00";

/// File header size: 8-byte signature + 1-byte fields_start + 2-byte segment_number + 2-byte fields_end = 13 bytes
pub const FILE_HEADER_SIZE: usize = 13;

// ─── Section Constants ──────────────────────────────────────────────────────

/// Section header size (EWF v1): 76 bytes
pub const SECTION_HEADER_SIZE: usize = 76;

/// Section type field length (null-padded ASCII string)
pub const SECTION_TYPE_LEN: usize = 16;

/// Known section type strings for L01 files
pub const SECTION_TYPE_HEADER: &[u8; 16] = b"header\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_HEADER2: &[u8; 16] = b"header2\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_VOLUME: &[u8; 16] = b"volume\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_SECTORS: &[u8; 16] = b"sectors\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_TABLE: &[u8; 16] = b"table\0\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_TABLE2: &[u8; 16] = b"table2\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_LTYPES: &[u8; 16] = b"ltypes\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_LTREE: &[u8; 16] = b"ltree\0\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_DATA: &[u8; 16] = b"data\0\0\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_HASH: &[u8; 16] = b"hash\0\0\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_DIGEST: &[u8; 16] = b"digest\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_DONE: &[u8; 16] = b"done\0\0\0\0\0\0\0\0\0\0\0\0";
pub const SECTION_TYPE_NEXT: &[u8; 16] = b"next\0\0\0\0\0\0\0\0\0\0\0\0";

// ─── Volume Constants ───────────────────────────────────────────────────────

/// Volume section data size (after section header)
pub const VOLUME_DATA_SIZE: usize = 94;

/// Media type for logical evidence (L01)
pub const MEDIA_TYPE_LOGICAL: u8 = 0x0e;

/// Default bytes per sector for logical images (block size)
pub const DEFAULT_BLOCK_SIZE: u32 = 512;

/// Default sectors per chunk (64 sectors × 512 = 32KB chunks)
pub const DEFAULT_SECTORS_PER_CHUNK: u32 = 64;

/// Default chunk size (32KB)
pub const DEFAULT_CHUNK_SIZE: u32 = DEFAULT_BLOCK_SIZE * DEFAULT_SECTORS_PER_CHUNK;

// ─── Compression ────────────────────────────────────────────────────────────

/// Compression level in volume section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompressionLevel {
    /// No compression (store)
    None = 0,
    /// Default/fast compression
    #[default]
    Fast = 1,
    /// Best compression
    Best = 2,
}

// ─── Hash Algorithm ─────────────────────────────────────────────────────────

/// Hash algorithm for overall image integrity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum L01HashAlgorithm {
    /// MD5 (default for L01 compatibility)
    #[default]
    Md5,
    /// SHA-1
    Sha1,
}

// ─── Case Info ──────────────────────────────────────────────────────────────

/// Case metadata written into the header section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L01CaseInfo {
    /// Case number
    pub case_number: String,
    /// Evidence number
    pub evidence_number: String,
    /// Description
    pub description: String,
    /// Examiner name
    pub examiner: String,
    /// Notes
    pub notes: String,
}

// ─── Writer Configuration ───────────────────────────────────────────────────

/// Configuration for L01 file creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L01WriterConfig {
    /// Output file path (e.g., "/path/to/output.L01")
    pub output_path: PathBuf,
    /// Case metadata
    pub case_info: L01CaseInfo,
    /// Compression level
    pub compression_level: CompressionLevel,
    /// Maximum segment file size in bytes (0 = no splitting)
    /// Default: 2GB (2_147_483_648)
    pub segment_size: u64,
    /// Hash algorithm for image integrity
    pub hash_algorithm: L01HashAlgorithm,
    /// Block size for chunks (default: 512)
    pub block_size: u32,
    /// Sectors per chunk (default: 64, yielding 32KB chunks)
    pub sectors_per_chunk: u32,
}

impl Default for L01WriterConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            case_info: L01CaseInfo::default(),
            compression_level: CompressionLevel::Fast,
            segment_size: 2_147_483_648, // 2 GB
            hash_algorithm: L01HashAlgorithm::Md5,
            block_size: DEFAULT_BLOCK_SIZE,
            sectors_per_chunk: DEFAULT_SECTORS_PER_CHUNK,
        }
    }
}

// ─── LEF File Entry ─────────────────────────────────────────────────────────

/// Record type for file entries (maps to `cid` field in ltree)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LefRecordType {
    /// Regular file
    #[default]
    File = 0,
    /// Directory/folder
    Directory = 1,
}

/// A single file entry in the LEF (Logical Evidence File).
///
/// Each entry corresponds to one file or directory in the logical acquisition.
/// Fields map to the ltree `entry` category tab-separated values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LefFileEntry {
    /// Unique identifier within the image (auto-assigned)
    pub identifier: u64,
    /// Entry type: 0=file, 1=directory
    pub record_type: LefRecordType,
    /// Entry flags
    pub flags: u32,
    /// GUID for this entry
    pub guid: String,
    /// File/directory name (UTF-8)
    pub name: String,
    /// Short name (8.3 format, optional)
    pub short_name: Option<String>,
    /// Logical file size in bytes
    pub size: u64,
    /// Offset into the data section where this file's chunks begin
    pub data_offset: u64,
    /// Size of file data stored (may differ from `size` if sparse)
    pub data_size: u64,
    /// Logical offset (for carved/partial files)
    pub logical_offset: u64,
    /// Physical offset (original disk offset if known)
    pub physical_offset: u64,
    /// Duplicate data offset (points to another entry's data if duplicate)
    pub duplicate_data_offset: Option<u64>,

    // ── Timestamps (POSIX seconds, 0 = not set) ──
    /// File creation time
    pub creation_time: i64,
    /// Last modification time
    pub modification_time: i64,
    /// Last access time
    pub access_time: i64,
    /// Entry metadata modification time
    pub entry_modification_time: i64,
    /// Deletion time (0 = not deleted)
    pub deletion_time: i64,

    // ── Hashes ──
    /// MD5 hash of file content (hex string)
    pub md5_hash: Option<String>,
    /// SHA-1 hash of file content (hex string)
    pub sha1_hash: Option<String>,

    // ── Hierarchy ──
    /// Parent entry identifier (0 = root)
    pub parent_identifier: u64,
    /// Whether this entry has children (is a parent/directory)
    pub is_parent: bool,

    // ── Source tracking ──
    /// Source identifier (links to `srce` category)
    pub source_identifier: u64,
    /// Subject identifier (links to `sub` category)
    pub subject_identifier: u64,
    /// Permission group index (links to `perm` category)
    pub permission_group_index: i32,

    // ── Extended attributes ──
    /// Extended attributes (raw string, optional)
    pub extended_attributes: Option<String>,

    /// Original source path on disk (NOT written to L01, used during acquisition)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<PathBuf>,
}

impl LefFileEntry {
    /// Create a new file entry with the given name and size
    pub fn new_file(identifier: u64, name: String, size: u64) -> Self {
        Self {
            identifier,
            record_type: LefRecordType::File,
            flags: 0,
            guid: uuid::Uuid::new_v4().to_string(),
            name,
            short_name: None,
            size,
            data_offset: 0,
            data_size: 0,
            logical_offset: 0,
            physical_offset: 0,
            duplicate_data_offset: None,
            creation_time: 0,
            modification_time: 0,
            access_time: 0,
            entry_modification_time: 0,
            deletion_time: 0,
            md5_hash: None,
            sha1_hash: None,
            parent_identifier: 0,
            is_parent: false,
            source_identifier: 0,
            subject_identifier: 0,
            permission_group_index: -1,
            extended_attributes: None,
            source_path: None,
        }
    }

    /// Create a new directory entry
    pub fn new_directory(identifier: u64, name: String) -> Self {
        Self {
            identifier,
            record_type: LefRecordType::Directory,
            flags: 0,
            guid: uuid::Uuid::new_v4().to_string(),
            name,
            short_name: None,
            size: 0,
            data_offset: 0,
            data_size: 0,
            logical_offset: 0,
            physical_offset: 0,
            duplicate_data_offset: None,
            creation_time: 0,
            modification_time: 0,
            access_time: 0,
            entry_modification_time: 0,
            deletion_time: 0,
            md5_hash: None,
            sha1_hash: None,
            parent_identifier: 0,
            is_parent: true,
            source_identifier: 0,
            subject_identifier: 0,
            permission_group_index: -1,
            extended_attributes: None,
            source_path: None,
        }
    }

    /// Set timestamps from filesystem metadata
    pub fn with_timestamps(mut self, created: i64, modified: i64, accessed: i64) -> Self {
        self.creation_time = created;
        self.modification_time = modified;
        self.access_time = accessed;
        self.entry_modification_time = modified;
        self
    }

    /// Set the source path for acquisition
    pub fn with_source_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }

    /// Set parent entry
    pub fn with_parent(mut self, parent_id: u64) -> Self {
        self.parent_identifier = parent_id;
        self
    }

    /// Set source identifier
    pub fn with_source(mut self, source_id: u64) -> Self {
        self.source_identifier = source_id;
        self
    }

    /// Binary extents string for the ltree (offset + size pairs)
    pub fn binary_extents(&self) -> String {
        if self.record_type == LefRecordType::Directory || self.data_size == 0 {
            String::new()
        } else {
            format!("{} {}", self.data_offset, self.data_size)
        }
    }
}

// ─── LEF Source ─────────────────────────────────────────────────────────────

/// An acquisition source in the LEF.
///
/// Maps to the `srce` category in the ltree section.
/// Describes where the data was acquired from.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LefSource {
    /// Source identifier (auto-assigned)
    pub identifier: u64,
    /// Source name/label
    pub name: String,
    /// Evidence number
    pub evidence_number: String,
    /// Location/path of original evidence
    pub location: String,
    /// Device GUID
    pub device_guid: String,
    /// Drive type (e.g., "Fixed", "Removable")
    pub drive_type: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Model
    pub model: String,
    /// Serial number
    pub serial_number: String,
}

impl LefSource {
    /// Create a new source with the given name
    pub fn new(identifier: u64, name: String) -> Self {
        Self {
            identifier,
            name,
            ..Default::default()
        }
    }

    /// Create a source from a file path
    pub fn from_path(identifier: u64, path: &std::path::Path) -> Self {
        Self {
            identifier,
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            location: path.to_string_lossy().to_string(),
            ..Default::default()
        }
    }
}

// ─── Permission Group ───────────────────────────────────────────────────────

/// A permission entry within a permission group.
///
/// Maps to individual permission rows in the `perm` category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LefPermission {
    /// Permission name/label
    pub name: String,
    /// Security Identifier (SID) string
    pub sid: String,
    /// Permissions bitmask
    pub permissions_bitmask: u32,
}

/// A permission group containing one or more permission entries.
///
/// Maps to the `perm` category in the ltree section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LefPermissionGroup {
    /// Group identifier
    pub identifier: u64,
    /// Group name
    pub name: String,
    /// Whether this is a parent group node
    pub is_parent: bool,
    /// Permission entries in this group
    pub permissions: Vec<LefPermission>,
}

// ─── Subject ────────────────────────────────────────────────────────────────

/// A subject in the LEF.
///
/// Maps to the `sub` category in the ltree section.
/// Subjects are typically user accounts or profiles associated with files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LefSubject {
    /// Subject identifier
    pub identifier: u64,
    /// Subject name
    pub name: String,
}

// ─── Chunk Table ────────────────────────────────────────────────────────────

/// Describes a compressed chunk stored in the sectors section
#[derive(Debug, Clone)]
pub struct ChunkDescriptor {
    /// Offset of this chunk within the sectors section data
    pub offset: u64,
    /// Compressed size of this chunk in bytes
    pub compressed_size: u32,
    /// Whether this chunk is compressed (vs stored raw)
    pub is_compressed: bool,
}

/// Table of chunk offsets for the table section
#[derive(Debug, Clone, Default)]
pub struct ChunkTable {
    /// Base offset (start of sectors section data)
    pub base_offset: u64,
    /// Individual chunk descriptors
    pub chunks: Vec<ChunkDescriptor>,
}

impl ChunkTable {
    pub fn new(base_offset: u64) -> Self {
        Self {
            base_offset,
            chunks: Vec::new(),
        }
    }

    /// Add a chunk descriptor
    pub fn add_chunk(&mut self, offset: u64, compressed_size: u32, is_compressed: bool) {
        self.chunks.push(ChunkDescriptor {
            offset,
            compressed_size,
            is_compressed,
        });
    }

    /// Total number of chunks
    pub fn chunk_count(&self) -> u32 {
        self.chunks.len() as u32
    }
}

// ─── Write Progress ─────────────────────────────────────────────────────────

/// Progress event emitted during L01 creation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L01WriteProgress {
    /// Output file path
    pub path: String,
    /// Current file being processed
    pub current_file: String,
    /// Files processed so far
    pub files_processed: usize,
    /// Total files to process
    pub total_files: usize,
    /// Bytes written so far
    pub bytes_written: u64,
    /// Total bytes to write
    pub total_bytes: u64,
    /// Progress percentage (0.0 - 100.0)
    pub percent: f64,
    /// Current phase of the write operation
    pub phase: L01WritePhase,
}

/// Phases of the L01 write operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum L01WritePhase {
    /// Preparing metadata and file list
    Preparing,
    /// Writing file data (sectors section)
    WritingData,
    /// Building chunk tables
    BuildingTables,
    /// Writing ltree section
    WritingLtree,
    /// Computing image hash
    ComputingHash,
    /// Finalizing (done section)
    Finalizing,
}

// ─── Adler-32 Checksum ──────────────────────────────────────────────────────

/// Compute Adler-32 checksum over the given data.
///
/// Used for EWF v1 section header checksums and ltree header integrity.
/// Implementation per RFC 1950 §8.2.
pub fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;

    for &byte in data {
        a = (a + byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
}

// ─── L01 Write Result ───────────────────────────────────────────────────────

/// Result of a successful L01 write operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L01WriteResult {
    /// Output file path(s) created
    pub output_paths: Vec<String>,
    /// Total files written into the L01
    pub total_files: usize,
    /// Total directories written
    pub total_directories: usize,
    /// Total bytes of file data written
    pub total_data_bytes: u64,
    /// Total compressed bytes in sectors section
    pub total_compressed_bytes: u64,
    /// Compression ratio (compressed / original)
    pub compression_ratio: f64,
    /// Image MD5 hash (if computed)
    pub md5_hash: Option<String>,
    /// Image SHA-1 hash (if computed)
    pub sha1_hash: Option<String>,
    /// Number of segment files created
    pub segment_count: u32,
    /// Number of chunks written
    pub chunk_count: u32,
}

// ─── Error Types ────────────────────────────────────────────────────────────

/// Errors that can occur during L01 writing
#[derive(Debug, thiserror::Error)]
pub enum L01WriteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No files added to the writer")]
    NoFiles,

    #[error("Output path not set")]
    NoOutputPath,

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Failed to read source file '{path}': {reason}")]
    SourceReadError { path: String, reason: String },

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Entry identifier {0} already exists")]
    DuplicateIdentifier(u64),

    #[error("Parent entry {0} not found")]
    ParentNotFound(u64),

    #[error("Write cancelled")]
    Cancelled,

    #[error("Segment size too small: {0} bytes (minimum 1MB)")]
    SegmentTooSmall(u64),

    #[error("Too many segments: data requires {0} segments but maximum is {1}")]
    TooManySegments(u16, u16),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<L01WriteError> for String {
    fn from(err: L01WriteError) -> String {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32_empty() {
        assert_eq!(adler32(b""), 1);
    }

    #[test]
    fn test_adler32_hello() {
        // Known Adler-32 for "Hello World" = 0x180B041D
        let result = adler32(b"Hello World");
        assert_eq!(result, 0x180B041D);
    }

    #[test]
    fn test_adler32_zeros() {
        let data = vec![0u8; 100];
        let result = adler32(&data);
        // a = 1 (no additions), b = 100
        assert_eq!(result, (100 << 16) | 1);
    }

    #[test]
    fn test_lvf_signature() {
        assert_eq!(LVF_SIGNATURE.len(), 8);
        assert_eq!(LVF_SIGNATURE[0], b'L');
        assert_eq!(LVF_SIGNATURE[1], b'V');
        assert_eq!(LVF_SIGNATURE[2], b'F');
    }

    #[test]
    fn test_section_type_constants() {
        assert_eq!(SECTION_TYPE_HEADER.len(), SECTION_TYPE_LEN);
        assert_eq!(SECTION_TYPE_VOLUME.len(), SECTION_TYPE_LEN);
        assert_eq!(SECTION_TYPE_SECTORS.len(), SECTION_TYPE_LEN);
        assert_eq!(SECTION_TYPE_TABLE.len(), SECTION_TYPE_LEN);
        assert_eq!(SECTION_TYPE_LTREE.len(), SECTION_TYPE_LEN);
        assert_eq!(SECTION_TYPE_DONE.len(), SECTION_TYPE_LEN);
    }

    #[test]
    fn test_lef_file_entry_new_file() {
        let entry = LefFileEntry::new_file(1, "test.txt".into(), 1024);
        assert_eq!(entry.identifier, 1);
        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.record_type, LefRecordType::File);
        assert!(!entry.is_parent);
        assert!(!entry.guid.is_empty());
    }

    #[test]
    fn test_lef_file_entry_new_directory() {
        let entry = LefFileEntry::new_directory(2, "Documents".into());
        assert_eq!(entry.identifier, 2);
        assert_eq!(entry.name, "Documents");
        assert_eq!(entry.size, 0);
        assert_eq!(entry.record_type, LefRecordType::Directory);
        assert!(entry.is_parent);
    }

    #[test]
    fn test_lef_file_entry_with_timestamps() {
        let entry =
            LefFileEntry::new_file(1, "test.txt".into(), 100).with_timestamps(1000, 2000, 3000);
        assert_eq!(entry.creation_time, 1000);
        assert_eq!(entry.modification_time, 2000);
        assert_eq!(entry.access_time, 3000);
        assert_eq!(entry.entry_modification_time, 2000);
    }

    #[test]
    fn test_lef_file_entry_binary_extents() {
        let mut entry = LefFileEntry::new_file(1, "test.txt".into(), 100);
        entry.data_offset = 4096;
        entry.data_size = 100;
        assert_eq!(entry.binary_extents(), "4096 100");

        let dir = LefFileEntry::new_directory(2, "dir".into());
        assert_eq!(dir.binary_extents(), "");
    }

    #[test]
    fn test_lef_source_from_path() {
        let path = std::path::Path::new("/evidence/disk.E01");
        let source = LefSource::from_path(1, path);
        assert_eq!(source.identifier, 1);
        assert_eq!(source.name, "disk.E01");
        assert_eq!(source.location, "/evidence/disk.E01");
    }

    #[test]
    fn test_chunk_table() {
        let mut table = ChunkTable::new(1000);
        table.add_chunk(0, 500, true);
        table.add_chunk(500, 32768, false);
        assert_eq!(table.chunk_count(), 2);
        assert_eq!(table.base_offset, 1000);
        assert!(table.chunks[0].is_compressed);
        assert!(!table.chunks[1].is_compressed);
    }

    #[test]
    fn test_default_config() {
        let config = L01WriterConfig::default();
        assert_eq!(config.compression_level, CompressionLevel::Fast);
        assert_eq!(config.segment_size, 2_147_483_648);
        assert_eq!(config.hash_algorithm, L01HashAlgorithm::Md5);
        assert_eq!(config.block_size, DEFAULT_BLOCK_SIZE);
        assert_eq!(config.sectors_per_chunk, DEFAULT_SECTORS_PER_CHUNK);
    }

    #[test]
    fn test_l01_write_error_display() {
        let err = L01WriteError::NoFiles;
        assert_eq!(err.to_string(), "No files added to the writer");

        let err = L01WriteError::FileNotFound("/path/to/file".into());
        assert!(err.to_string().contains("/path/to/file"));

        let err: String = L01WriteError::Cancelled.into();
        assert_eq!(err, "Write cancelled");
    }
}
