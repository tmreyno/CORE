// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Type definitions for EWF (Expert Witness Format) parsing.
//!
//! This module defines the data structures used throughout the EWF parser
//! for representing container metadata, chunk locations, and verification
//! results.
//!
//! # Public Types
//!
//! | Type | Description |
//! |------|-------------|
//! | [`EwfInfo`] | Complete container metadata |
//! | [`EwfStats`] | Statistical information about the container |
//! | [`HeaderInfo`] | Case/evidence metadata from EWF header section |
//! | [`VolumeSection`] | Volume geometry and compression settings |
//! | [`VerifyResult`] | Per-chunk verification result |
//! | [`ChunkVerifyResult`] | Detailed chunk verification with status |
//! | [`EwfSearchResult`] | Search result for L01 logical containers |
//!
//! # Internal Types
//!
//! The following types are `pub(crate)` and used internally:
//! - `SectionDescriptor`: Raw section header from file
//! - `SegmentFile`: Metadata for a single segment file
//! - `SegmentSection`: Section within a segment
//! - `TableSection`: Chunk offset table data
//! - `ChunkLocation`: Maps logical chunks to physical storage

use serde::Serialize;

// Re-export StoredHash from containers for backward compatibility
pub use crate::containers::StoredHash as StoredImageHash;

// =============================================================================
// Core Constants - EWF Signatures
// =============================================================================

/// EVF v1 signature (E01 - physical disk image).
///
/// The 8-byte signature at offset 0 of all E01 segment files:
/// `EVF\x09\x0d\x0a\xff\x00`
pub(crate) const EWF_SIGNATURE: &[u8; 8] = b"EVF\x09\x0d\x0a\xff\x00";

/// EVF v2 signature (Ex01 - physical disk image).
///
/// The 8-byte signature for EnCase v7+ EWF2 format:
/// `EVF2\x0d\x0a\x81\x00`
pub(crate) const EWF2_SIGNATURE: &[u8; 8] = b"EVF2\x0d\x0a\x81\x00";

/// LVF v1 signature (L01 - logical evidence file).
///
/// The 8-byte signature for logical evidence containers:
/// `LVF\x09\x0d\x0a\xff\x00`
pub(crate) const LVF_SIGNATURE: &[u8; 8] = b"LVF\x09\x0d\x0a\xff\x00";

/// LVF v2 signature (Lx01 - logical evidence file).
///
/// The 8-byte signature for EnCase v7+ logical evidence:
/// `LVF2\x0d\x0a\x81\x00`
pub(crate) const LVF2_SIGNATURE: &[u8; 8] = b"LVF2\x0d\x0a\x81\x00";

// =============================================================================
// Core Constants - Sizes
// =============================================================================

/// Standard sector size in bytes (512 bytes).
#[allow(dead_code)]
pub(crate) const SECTOR_SIZE: u64 = 512;

/// Maximum number of segment files to keep open simultaneously.
///
/// Matches libewf's approach to respecting system file descriptor limits.
pub(crate) const MAX_OPEN_FILES: usize = 16;

// =============================================================================
// Section Descriptors - EWF Format Structures
// =============================================================================

/// Raw section descriptor from EWF file.
///
/// Each section in an EWF file starts with this 76-byte header that
/// identifies the section type and provides navigation to the next section.
#[derive(Clone, Debug)]
pub(crate) struct SectionDescriptor {
    /// Section type identifier (null-padded ASCII, e.g., "header", "volume")
    pub section_type: [u8; 16],
    /// Absolute file offset of the next section
    pub next_offset: u64,
    /// Size of this section's data (excluding header)
    pub size: u64,
}

/// Volume section data containing disk geometry and compression info.
///
/// The volume section appears once in the first segment and defines
/// the logical structure of the imaged media.
#[derive(Clone, Debug)]
pub struct VolumeSection {
    /// Total number of chunks in the image
    pub chunk_count: u32,
    /// Number of sectors per chunk (typically 64)
    pub sectors_per_chunk: u32,
    /// Bytes per sector (typically 512)
    pub bytes_per_sector: u32,
    /// Total number of sectors in the image
    pub sector_count: u64,
    /// Compression level (0=none, 1=fast, 2=best)
    pub compression_level: u8,
}

// =============================================================================
// Segment File - Represents one physical E01/E02 file
// =============================================================================

/// Metadata for a single segment file.
///
/// EWF images can span multiple segment files (.E01, .E02, etc.).
/// This struct tracks the sections and chunk data within each segment.
pub(crate) struct SegmentFile {
    /// Index in the file pool (for handle management)
    pub file_index: usize,
    /// Segment number (1 for E01, 2 for E02, etc.)
    #[allow(dead_code)]
    pub segment_number: u16,
    /// Size of this segment file in bytes
    pub file_size: u64,
    /// Sections found in this segment
    pub sections: Vec<SegmentSection>,
}

/// Section within a segment file.
#[derive(Clone)]
pub(crate) struct SegmentSection {
    /// Section type name (e.g., "sectors", "table", "hash")
    #[allow(dead_code)]
    pub section_type: String,
    /// Offset within the segment file
    #[allow(dead_code)]
    pub offset_in_segment: u64,
    /// Size of section data
    #[allow(dead_code)]
    pub size: u64,
    /// For 'sectors' sections: where compressed chunk data starts
    pub data_offset: Option<u64>,
    /// For 'table' sections: parsed chunk offset table
    pub table_data: Option<TableSection>,
}

/// Chunk offset table from a 'table' section.
///
/// The table section contains offsets to locate each compressed chunk
/// within the sectors section.
#[derive(Clone)]
pub(crate) struct TableSection {
    /// Number of chunks described by this table
    #[allow(dead_code)]
    pub chunk_count: u32,
    /// Base offset for relative chunk offsets (EnCase 6+)
    pub base_offset: u64,
    /// Chunk offsets (may be relative or absolute depending on version)
    pub offsets: Vec<u64>,
}

/// Case/evidence metadata from the EWF header section.
///
/// Contains forensic chain-of-custody information embedded in the
/// container by the acquisition tool.
#[derive(Clone, Debug, Default)]
pub struct HeaderInfo {
    /// Case number or identifier
    pub case_number: Option<String>,
    /// Evidence item number
    pub evidence_number: Option<String>,
    /// Description of the evidence
    pub description: Option<String>,
    /// Name of the examiner who created the image
    pub examiner_name: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Date/time when acquisition started
    pub acquiry_date: Option<String>,
    /// System date/time during acquisition
    pub system_date: Option<String>,
    /// Operating system of acquisition machine
    pub acquiry_os: Option<String>,
    /// Version of acquisition software
    pub acquiry_sw_version: Option<String>,
}

impl HeaderInfo {
    /// Create new empty HeaderInfo
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

    /// Set description
    #[inline]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set examiner name
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner_name = Some(examiner.into());
        self
    }

    /// Set notes
    #[inline]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set acquisition date
    #[inline]
    pub fn with_acquiry_date(mut self, date: impl Into<String>) -> Self {
        self.acquiry_date = Some(date.into());
        self
    }

    /// Set system date
    #[inline]
    pub fn with_system_date(mut self, date: impl Into<String>) -> Self {
        self.system_date = Some(date.into());
        self
    }

    /// Set acquisition OS
    #[inline]
    pub fn with_acquiry_os(mut self, os: impl Into<String>) -> Self {
        self.acquiry_os = Some(os.into());
        self
    }

    /// Set acquisition software version
    #[inline]
    pub fn with_acquiry_sw_version(mut self, version: impl Into<String>) -> Self {
        self.acquiry_sw_version = Some(version.into());
        self
    }

    /// Check if any case info is present
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.case_number.is_none()
            && self.evidence_number.is_none()
            && self.description.is_none()
            && self.examiner_name.is_none()
            && self.notes.is_none()
    }
}

// =============================================================================
// Chunk Location - Maps chunks to their storage location
// =============================================================================

/// Maps a logical chunk index to its physical storage location.
///
/// Used internally to locate compressed chunk data across multiple
/// segment files and handle different EWF version offset formats.
#[derive(Clone)]
pub(crate) struct ChunkLocation {
    /// Index of the segment file containing this chunk
    pub segment_index: usize,
    /// Index of the 'sectors' section within the segment
    #[allow(dead_code)]
    pub section_index: usize,
    /// Position of this chunk within the table
    #[allow(dead_code)]
    pub chunk_in_table: usize,
    /// Offset value from the table (interpretation depends on version)
    pub offset: u64,
    /// Table base offset for EnCase 6+ (0 for older versions)
    pub base_offset: u64,
    /// Global file offset where sectors section data begins
    pub sectors_base: u64,
    /// True if this chunk was discovered via inline delta format scanning
    pub is_delta_chunk: bool,
}

// =============================================================================
// Public API Types
// =============================================================================

/// Complete container information for EWF format files.
///
/// This is the primary return type from `ewf::info()` and contains
/// all parsed metadata from the container including case info,
/// geometry, and stored hashes.
///
/// # Supported Formats
///
/// - **E01**: Physical disk images (EWF v1)
/// - **L01**: Logical evidence files (EWF v1)
/// - **Ex01**: Physical disk images (EWF v2)
/// - **Lx01**: Logical evidence files (EWF v2)
#[derive(Debug, Serialize, Default)]
pub struct EwfInfo {
    /// Format version string (e.g., "EWF-E01", "EWF2-Ex01")
    pub format_version: String,
    /// Number of segment files
    pub segment_count: u32,
    /// Total number of data chunks
    pub chunk_count: u32,
    /// Total number of sectors
    pub sector_count: u64,
    /// Bytes per sector (typically 512)
    pub bytes_per_sector: u32,
    /// Sectors per chunk (typically 64)
    pub sectors_per_chunk: u32,
    /// Total uncompressed data size in bytes
    pub total_size: u64,
    /// Compression method description
    pub compression: String,
    /// Case number from header
    pub case_number: Option<String>,
    /// Evidence description
    pub description: Option<String>,
    /// Examiner name
    pub examiner_name: Option<String>,
    /// Evidence number
    pub evidence_number: Option<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Acquisition date/time
    pub acquiry_date: Option<String>,
    /// System date/time during acquisition
    pub system_date: Option<String>,
    /// Drive model (if available)
    pub model: Option<String>,
    /// Drive serial number (if available)
    pub serial_number: Option<String>,
    /// Hashes stored in the container (MD5, SHA1)
    pub stored_hashes: Vec<StoredImageHash>,
    /// Paths to all segment files
    pub segment_files: Option<Vec<String>>,
    /// Offset of header section (for hex viewer navigation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_section_offset: Option<u64>,
    /// Offset of volume section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_section_offset: Option<u64>,
    /// Offset of hash section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_section_offset: Option<u64>,
    /// Offset of digest section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest_section_offset: Option<u64>,
}

impl EwfInfo {
    /// Create new EwfInfo with format version
    #[inline]
    pub fn new(format_version: impl Into<String>) -> Self {
        Self {
            format_version: format_version.into(),
            bytes_per_sector: 512, // Default sector size
            sectors_per_chunk: 64, // Default chunk size
            ..Default::default()
        }
    }

    /// Set segment count
    #[inline]
    pub fn with_segments(mut self, count: u32) -> Self {
        self.segment_count = count;
        self
    }

    /// Set chunk count and calculate sector count
    #[inline]
    pub fn with_chunks(mut self, chunk_count: u32, sectors_per_chunk: u32) -> Self {
        self.chunk_count = chunk_count;
        self.sectors_per_chunk = sectors_per_chunk;
        self.sector_count = chunk_count as u64 * sectors_per_chunk as u64;
        self
    }

    /// Set total size
    #[inline]
    pub fn with_size(mut self, total_size: u64) -> Self {
        self.total_size = total_size;
        self
    }

    /// Set compression method
    #[inline]
    pub fn with_compression(mut self, compression: impl Into<String>) -> Self {
        self.compression = compression.into();
        self
    }

    /// Apply header info (case metadata)
    #[inline]
    pub fn with_header_info(mut self, info: &HeaderInfo) -> Self {
        self.case_number = info.case_number.clone();
        self.evidence_number = info.evidence_number.clone();
        self.examiner_name = info.examiner_name.clone();
        self.description = info.description.clone();
        self.notes = info.notes.clone();
        self.acquiry_date = info.acquiry_date.clone();
        self.system_date = info.system_date.clone();
        self
    }

    /// Set device info
    #[inline]
    pub fn with_device(mut self, model: Option<String>, serial: Option<String>) -> Self {
        self.model = model;
        self.serial_number = serial;
        self
    }

    /// Add a stored hash
    #[inline]
    pub fn add_hash(mut self, hash: StoredImageHash) -> Self {
        self.stored_hashes.push(hash);
        self
    }

    /// Set segment file paths
    #[inline]
    pub fn with_segment_files(mut self, files: Vec<String>) -> Self {
        self.segment_files = Some(files);
        self
    }

    /// Set section offsets for hex viewer navigation
    #[inline]
    pub fn with_section_offsets(
        mut self,
        header: Option<u64>,
        volume: Option<u64>,
        hash: Option<u64>,
        digest: Option<u64>,
    ) -> Self {
        self.header_section_offset = header;
        self.volume_section_offset = volume;
        self.hash_section_offset = hash;
        self.digest_section_offset = digest;
        self
    }
}

/// Per-chunk verification result.
///
/// Used during chunk-level integrity verification to track the
/// status of individual compressed chunks.
#[derive(Serialize)]
pub struct VerifyResult {
    /// Index of the chunk being verified
    pub chunk_index: usize,
    /// Verification status ("ok", "error", "skipped")
    pub status: String,
    /// Error message if verification failed
    pub message: Option<String>,
}

/// Statistical information about an EWF container.
///
/// Provides summary metrics useful for understanding container
/// characteristics and compression efficiency.
#[derive(Debug, Clone, Serialize, Default)]
pub struct EwfStats {
    /// Total number of chunks
    pub total_chunks: u64,
    /// Total number of segment files
    pub total_segments: u32,
    /// Total uncompressed data size in bytes
    pub total_size: u64,
    /// Total compressed size on disk
    pub compressed_size: u64,
    /// Compression ratio (uncompressed / compressed)
    pub compression_ratio: f64,
    /// Bytes per sector
    pub bytes_per_sector: u32,
    /// Sectors per chunk
    pub sectors_per_chunk: u32,
    /// Total sectors
    pub sector_count: u64,
    /// Number of stored hashes
    pub stored_hash_count: usize,
    /// Whether MD5 hash is stored
    pub has_md5: bool,
    /// Whether SHA1 hash is stored
    pub has_sha1: bool,
    /// EWF format variant (E01, L01, Ex01, Lx01)
    pub format_variant: String,
}

impl EwfStats {
    /// Create a new EwfStats
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total chunks
    #[inline]
    pub fn with_chunks(mut self, count: u64) -> Self {
        self.total_chunks = count;
        self
    }

    /// Set segment count
    #[inline]
    pub fn with_segments(mut self, count: u32) -> Self {
        self.total_segments = count;
        self
    }

    /// Set sizes and calculate compression ratio
    #[inline]
    pub fn with_sizes(mut self, total: u64, compressed: u64) -> Self {
        self.total_size = total;
        self.compressed_size = compressed;
        if compressed > 0 {
            self.compression_ratio = total as f64 / compressed as f64;
        }
        self
    }

    /// Set sector information
    #[inline]
    pub fn with_sectors(
        mut self,
        bytes_per_sector: u32,
        sectors_per_chunk: u32,
        sector_count: u64,
    ) -> Self {
        self.bytes_per_sector = bytes_per_sector;
        self.sectors_per_chunk = sectors_per_chunk;
        self.sector_count = sector_count;
        self
    }

    /// Set hash information
    #[inline]
    pub fn with_hashes(mut self, count: usize, has_md5: bool, has_sha1: bool) -> Self {
        self.stored_hash_count = count;
        self.has_md5 = has_md5;
        self.has_sha1 = has_sha1;
        self
    }

    /// Set format variant
    #[inline]
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.format_variant = variant.into();
        self
    }
}

/// Search result for finding files within L01 logical containers.
///
/// L01 files contain a file tree, and this type represents matches
/// from search operations on that tree.
#[derive(Debug, Clone, Serialize)]
pub struct EwfSearchResult {
    /// Full path of the matching entry
    pub path: String,
    /// Entry filename
    pub name: String,
    /// Entry size in bytes
    pub size: u64,
    /// How the entry was matched ("name", "extension", "hash")
    pub match_type: String,
}

/// Detailed chunk verification result.
///
/// Provides more detailed information than `VerifyResult` for
/// comprehensive integrity reporting.
#[derive(Debug, Clone, Serialize)]
pub struct ChunkVerifyResult {
    /// Chunk index (0-based)
    pub index: u64,
    /// Verification status ("ok", "error", "skipped")
    pub status: String,
    /// Detailed error message if verification failed
    pub message: Option<String>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewf_signature_constants() {
        assert_eq!(EWF_SIGNATURE.len(), 8);
        assert_eq!(EWF2_SIGNATURE.len(), 8);
        assert_eq!(LVF_SIGNATURE.len(), 8);
        assert_eq!(LVF2_SIGNATURE.len(), 8);

        // EVF v1 starts with "EVF"
        assert_eq!(&EWF_SIGNATURE[0..3], b"EVF");
        // EVF v2 starts with "EVF2"
        assert_eq!(&EWF2_SIGNATURE[0..4], b"EVF2");
        // LVF v1 starts with "LVF"
        assert_eq!(&LVF_SIGNATURE[0..3], b"LVF");
        // LVF v2 starts with "LVF2"
        assert_eq!(&LVF2_SIGNATURE[0..4], b"LVF2");
    }

    #[test]
    fn test_sector_size_constant() {
        assert_eq!(SECTOR_SIZE, 512);
    }

    #[test]
    fn test_max_open_files_constant() {
        assert_eq!(MAX_OPEN_FILES, 16);
    }

    #[test]
    fn test_section_descriptor() {
        let mut section_type = [0u8; 16];
        section_type[..6].copy_from_slice(b"header");

        let descriptor = SectionDescriptor {
            section_type,
            next_offset: 1024,
            size: 512,
        };

        assert_eq!(descriptor.next_offset, 1024);
        assert_eq!(descriptor.size, 512);
        assert_eq!(&descriptor.section_type[..6], b"header");
    }

    #[test]
    fn test_volume_section() {
        let volume = VolumeSection {
            chunk_count: 1000,
            sectors_per_chunk: 64,
            bytes_per_sector: 512,
            sector_count: 64000,
            compression_level: 1,
        };

        assert_eq!(volume.chunk_count, 1000);
        assert_eq!(volume.sectors_per_chunk, 64);
        assert_eq!(volume.bytes_per_sector, 512);
        assert_eq!(volume.sector_count, 64000);
        assert_eq!(volume.compression_level, 1);

        // Verify chunk size calculation
        let chunk_size = volume.sectors_per_chunk * volume.bytes_per_sector;
        assert_eq!(chunk_size, 32768); // 64 * 512 = 32KB
    }

    #[test]
    fn test_header_info_default() {
        let header = HeaderInfo::default();
        assert!(header.case_number.is_none());
        assert!(header.evidence_number.is_none());
        assert!(header.description.is_none());
        assert!(header.examiner_name.is_none());
        assert!(header.notes.is_none());
        assert!(header.acquiry_date.is_none());
        assert!(header.system_date.is_none());
        assert!(header.acquiry_os.is_none());
        assert!(header.acquiry_sw_version.is_none());
    }

    #[test]
    fn test_header_info_with_values() {
        let header = HeaderInfo {
            case_number: Some("CASE-001".to_string()),
            evidence_number: Some("EV-001".to_string()),
            description: Some("Hard Drive Image".to_string()),
            examiner_name: Some("John Doe".to_string()),
            notes: Some("Acquired from suspect laptop".to_string()),
            acquiry_date: Some("2024-01-01".to_string()),
            system_date: Some("2024-01-01".to_string()),
            acquiry_os: Some("Windows 11".to_string()),
            acquiry_sw_version: Some("FTK Imager 4.7".to_string()),
        };

        assert_eq!(header.case_number, Some("CASE-001".to_string()));
        assert_eq!(header.examiner_name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_chunk_location() {
        let location = ChunkLocation {
            segment_index: 0,
            section_index: 1,
            chunk_in_table: 42,
            offset: 65536,
            base_offset: 1024,
            sectors_base: 2048,
            is_delta_chunk: false,
        };

        assert_eq!(location.segment_index, 0);
        assert_eq!(location.section_index, 1);
        assert_eq!(location.chunk_in_table, 42);
        assert_eq!(location.offset, 65536);
        assert_eq!(location.base_offset, 1024);
        assert_eq!(location.sectors_base, 2048);
        assert!(!location.is_delta_chunk);
    }

    #[test]
    fn test_chunk_location_delta_chunk() {
        let location = ChunkLocation {
            segment_index: 2,
            section_index: 0,
            chunk_in_table: 100,
            offset: 131072,
            base_offset: 0,
            sectors_base: 4096,
            is_delta_chunk: true,
        };

        assert!(location.is_delta_chunk);
        assert_eq!(location.segment_index, 2);
    }

    #[test]
    fn test_table_section() {
        let table = TableSection {
            chunk_count: 3,
            base_offset: 1024,
            offsets: vec![0, 32768, 65536],
        };

        assert_eq!(table.chunk_count, 3);
        assert_eq!(table.base_offset, 1024);
        assert_eq!(table.offsets.len(), 3);
        assert_eq!(table.offsets[0], 0);
        assert_eq!(table.offsets[2], 65536);
    }

    #[test]
    fn test_segment_section() {
        let section = SegmentSection {
            section_type: "sectors".to_string(),
            offset_in_segment: 4096,
            size: 1048576,
            data_offset: Some(4096),
            table_data: None,
        };

        assert_eq!(section.section_type, "sectors");
        assert_eq!(section.offset_in_segment, 4096);
        assert!(section.data_offset.is_some());
        assert!(section.table_data.is_none());
    }

    #[test]
    fn test_segment_section_with_table() {
        let table = TableSection {
            chunk_count: 10,
            base_offset: 0,
            offsets: vec![0; 10],
        };

        let section = SegmentSection {
            section_type: "table".to_string(),
            offset_in_segment: 8192,
            size: 256,
            data_offset: None,
            table_data: Some(table),
        };

        assert_eq!(section.section_type, "table");
        assert!(section.table_data.is_some());
        assert_eq!(section.table_data.as_ref().unwrap().chunk_count, 10);
    }

    #[test]
    fn test_ewf_info_creation() {
        let info = EwfInfo {
            format_version: "EVF 1.0".to_string(),
            segment_count: 5,
            chunk_count: 1000,
            sector_count: 64000,
            bytes_per_sector: 512,
            sectors_per_chunk: 64,
            total_size: 32768000,
            compression: "zlib".to_string(),
            case_number: Some("CASE-001".to_string()),
            description: Some("Test image".to_string()),
            examiner_name: None,
            evidence_number: None,
            notes: None,
            acquiry_date: None,
            system_date: None,
            model: None,
            serial_number: None,
            stored_hashes: vec![],
            segment_files: Some(vec!["test.E01".to_string(), "test.E02".to_string()]),
            header_section_offset: Some(13),
            volume_section_offset: Some(1024),
            hash_section_offset: None,
            digest_section_offset: None,
        };

        assert_eq!(info.format_version, "EVF 1.0");
        assert_eq!(info.segment_count, 5);
        assert_eq!(info.compression, "zlib");
        assert!(info.segment_files.is_some());
        assert_eq!(info.segment_files.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_verify_result() {
        let result = VerifyResult {
            chunk_index: 42,
            status: "verified".to_string(),
            message: None,
        };

        assert_eq!(result.chunk_index, 42);
        assert_eq!(result.status, "verified");
        assert!(result.message.is_none());
    }

    #[test]
    fn test_verify_result_with_error() {
        let result = VerifyResult {
            chunk_index: 100,
            status: "error".to_string(),
            message: Some("CRC mismatch".to_string()),
        };

        assert_eq!(result.status, "error");
        assert!(result.message.is_some());
        assert_eq!(result.message.unwrap(), "CRC mismatch");
    }

    #[test]
    fn test_segment_file() {
        let segment = SegmentFile {
            file_index: 0,
            segment_number: 1,
            file_size: 1073741824, // 1 GB
            sections: vec![],
        };

        assert_eq!(segment.file_index, 0);
        assert_eq!(segment.segment_number, 1);
        assert_eq!(segment.file_size, 1073741824);
        assert!(segment.sections.is_empty());
    }
}
