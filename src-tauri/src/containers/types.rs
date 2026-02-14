// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Type definitions for forensic containers
//!
//! This module contains all the shared types used across the container subsystem.

use serde::{Serialize, Deserialize};

use crate::ad1;
use crate::archive;
use crate::ewf;
use crate::raw;
use crate::ufed;

/// Stored hash from container metadata or companion log files
#[derive(Debug, Serialize, Clone, Default)]
pub struct StoredHash {
    pub algorithm: String,
    pub hash: String,
    /// None if not verified, Some(true) if verified, Some(false) if mismatch
    pub verified: Option<bool>,
    /// When hash was created/verified (ISO 8601 or human-readable)
    pub timestamp: Option<String>,
    /// Where hash came from: "container", "companion", "computed"
    pub source: Option<String>,
    /// Byte offset in file where the raw hash bytes are located
    pub offset: Option<u64>,
    /// Size in bytes of the hash (MD5=16, SHA1=20, SHA256=32)
    pub size: Option<u64>,
}

impl StoredHash {
    /// Create new StoredHash with algorithm and hash value
    #[inline]
    pub fn new(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            hash: hash.into(),
            source: Some("container".to_string()),
            ..Default::default()
        }
    }

    /// Create MD5 hash
    #[inline]
    pub fn md5(hash: impl Into<String>) -> Self {
        Self::new("MD5", hash).with_size(16)
    }

    /// Create SHA1 hash
    #[inline]
    pub fn sha1(hash: impl Into<String>) -> Self {
        Self::new("SHA1", hash).with_size(20)
    }

    /// Create SHA256 hash
    #[inline]
    pub fn sha256(hash: impl Into<String>) -> Self {
        Self::new("SHA256", hash).with_size(32)
    }

    /// Create from companion file
    #[inline]
    pub fn from_companion(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            hash: hash.into(),
            source: Some("companion".to_string()),
            ..Default::default()
        }
    }

    /// Create from computed value
    #[inline]
    pub fn from_computed(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            hash: hash.into(),
            source: Some("computed".to_string()),
            ..Default::default()
        }
    }

    /// Set verification status
    #[inline]
    pub fn verified(mut self, is_verified: bool) -> Self {
        self.verified = Some(is_verified);
        self
    }

    /// Set timestamp
    #[inline]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set source
    #[inline]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set byte offset for hex viewer navigation
    #[inline]
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set size in bytes
    #[inline]
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set offset and size together
    #[inline]
    pub fn at_location(mut self, offset: u64, size: u64) -> Self {
        self.offset = Some(offset);
        self.size = Some(size);
        self
    }
}

/// Per-segment hash information from companion log files
#[derive(Serialize, Deserialize, Clone)]
pub struct SegmentHash {
    /// e.g., "SCHARDT.001"
    pub segment_name: String,
    /// e.g., 1
    pub segment_number: u32,
    /// e.g., "MD5"
    pub algorithm: String,
    /// The hash value
    pub hash: String,
    /// Starting byte/sector offset
    pub offset_from: Option<u64>,
    /// Ending byte/sector offset
    pub offset_to: Option<u64>,
    /// Segment size
    pub size: Option<u64>,
    /// Verification status
    pub verified: Option<bool>,
}

/// Information parsed from companion log files (e.g., FTK logs, Guymager logs)
#[derive(Serialize, Clone)]
pub struct CompanionLogInfo {
    pub log_path: String,
    pub created_by: Option<String>,
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub unique_description: Option<String>,
    pub examiner: Option<String>,
    pub notes: Option<String>,
    pub acquisition_started: Option<String>,
    pub acquisition_finished: Option<String>,
    pub verification_started: Option<String>,
    pub verification_finished: Option<String>,
    pub stored_hashes: Vec<StoredHash>,
    pub segment_list: Vec<String>,
    /// Per-segment hashes
    pub segment_hashes: Vec<SegmentHash>,
}

/// Unified container information structure
/// Holds format-specific info in the appropriate field
#[derive(Serialize)]
pub struct ContainerInfo {
    pub container: String,
    pub ad1: Option<ad1::Ad1Info>,
    /// EWF physical image (E01/Ex01)
    pub e01: Option<ewf::EwfInfo>,
    /// EWF logical evidence (L01/Lx01) - same format as E01
    pub l01: Option<ewf::EwfInfo>,
    pub raw: Option<raw::RawInfo>,
    pub archive: Option<archive::ArchiveInfo>,
    pub ufed: Option<ufed::UfedInfo>,
    pub note: Option<String>,
    pub companion_log: Option<CompanionLogInfo>,
}

/// Represents a discovered forensic container file during directory scanning
#[derive(Clone, Serialize)]
pub struct DiscoveredFile {
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub size: u64,
    pub segment_count: Option<u32>,
    pub segment_files: Option<Vec<String>>,
    pub segment_sizes: Option<Vec<u64>>,
    pub total_segment_size: Option<u64>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Result entry from container-level chunk verification (E01, Raw, etc.)
///
/// NOTE: This is different from `ad1::VerifyEntry` which is used for AD1
/// file-level verification with hash comparison details. This struct is used
/// for chunk-based verification results (e.g., E01 chunk CRC checks).
///
/// - `path`: Optional file path being verified
/// - `chunk_index`: Which chunk failed (for chunk-based formats)
/// - `status`: "ok", "error", "warning"
/// - `message`: Human-readable error/warning message
#[derive(Serialize)]
pub struct VerifyEntry {
    pub path: Option<String>,
    pub chunk_index: Option<usize>,
    pub status: String,
    pub message: Option<String>,
}

/// Internal enum for container type detection
#[derive(Clone, Copy, Debug)]
pub(crate) enum ContainerKind {
    Ad1,
    E01,
    L01,
    Raw,
    Archive,
    Ufed,
}

// =============================================================================
// SEARCH TYPES
// =============================================================================

/// Search query for finding files within containers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search by filename pattern (supports wildcards: *, ?)
    pub name_pattern: Option<String>,
    /// Search by file extension (without dot)
    pub extension: Option<String>,
    /// Search by exact hash value
    pub hash: Option<String>,
    /// Hash algorithm for hash search (md5, sha1, sha256)
    pub hash_algorithm: Option<String>,
    /// Minimum file size in bytes
    pub min_size: Option<u64>,
    /// Maximum file size in bytes
    pub max_size: Option<u64>,
    /// Search only in directories (not files)
    #[serde(default)]
    pub directories_only: bool,
    /// Search only files (not directories)
    #[serde(default)]
    pub files_only: bool,
    /// Maximum number of results to return
    pub max_results: Option<usize>,
}

impl SearchQuery {
    /// Create a new empty search query
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Search by filename pattern
    pub fn with_name(mut self, pattern: &str) -> Self {
        self.name_pattern = Some(pattern.to_string());
        self
    }
    
    /// Search by file extension
    pub fn with_extension(mut self, ext: &str) -> Self {
        self.extension = Some(ext.to_string());
        self
    }
    
    /// Search by hash value
    pub fn with_hash(mut self, hash: &str, algorithm: &str) -> Self {
        self.hash = Some(hash.to_string());
        self.hash_algorithm = Some(algorithm.to_string());
        self
    }
    
    /// Limit results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }
}

/// Unified search result that works across all container types
#[derive(Debug, Clone, Serialize)]
pub struct ContainerSearchResult {
    /// Container path
    pub container_path: String,
    /// Container type (AD1, E01, Archive, etc.)
    pub container_type: String,
    /// Path within the container
    pub entry_path: String,
    /// Entry name
    pub name: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File size in bytes (0 for directories)
    pub size: u64,
    /// How this result matched the query
    pub match_type: SearchMatchType,
    /// Depth in the file tree
    pub depth: u32,
    /// MD5 hash if available
    pub md5: Option<String>,
    /// SHA1 hash if available
    pub sha1: Option<String>,
    /// Created timestamp
    pub created: Option<String>,
    /// Modified timestamp
    pub modified: Option<String>,
}

/// How a search result matched the query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchMatchType {
    /// Matched by filename pattern
    Name,
    /// Matched by file extension
    Extension,
    /// Matched by hash value
    Hash,
    /// Matched by size criteria
    Size,
    /// Matched multiple criteria
    Multiple,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_hash_creation() {
        let hash = StoredHash {
            algorithm: "SHA-256".to_string(),
            hash: "abc123def456".to_string(),
            verified: Some(true),
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            source: Some("container".to_string()),
            offset: Some(0),
            size: Some(32),
        };
        
        assert_eq!(hash.algorithm, "SHA-256");
        assert_eq!(hash.hash, "abc123def456");
        assert!(hash.verified.unwrap());
    }

    #[test]
    fn test_segment_hash_creation() {
        let seg_hash = SegmentHash {
            segment_name: "image.001".to_string(),
            segment_number: 1,
            algorithm: "MD5".to_string(),
            hash: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            offset_from: Some(0),
            offset_to: Some(1048576),
            size: Some(1048576),
            verified: None,
        };
        
        assert_eq!(seg_hash.segment_name, "image.001");
        assert_eq!(seg_hash.segment_number, 1);
        assert_eq!(seg_hash.algorithm, "MD5");
    }

    #[test]
    fn test_companion_log_info_creation() {
        let log = CompanionLogInfo {
            log_path: "/test/image.txt".to_string(),
            created_by: Some("FTK Imager".to_string()),
            case_number: Some("2024-001".to_string()),
            evidence_number: Some("E001".to_string()),
            unique_description: Some("Hard Drive".to_string()),
            examiner: Some("John Doe".to_string()),
            notes: Some("Test acquisition".to_string()),
            acquisition_started: Some("2024-01-01 10:00:00".to_string()),
            acquisition_finished: Some("2024-01-01 12:00:00".to_string()),
            verification_started: Some("2024-01-01 12:00:00".to_string()),
            verification_finished: Some("2024-01-01 14:00:00".to_string()),
            stored_hashes: vec![],
            segment_list: vec!["image.001".to_string(), "image.002".to_string()],
            segment_hashes: vec![],
        };
        
        assert_eq!(log.created_by, Some("FTK Imager".to_string()));
        assert_eq!(log.case_number, Some("2024-001".to_string()));
        assert_eq!(log.segment_list.len(), 2);
    }

    #[test]
    fn test_discovered_file_creation() {
        let file = DiscoveredFile {
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            size: 640 * 1024 * 1024,
            segment_count: Some(2),
            segment_files: Some(vec!["disk.E01".to_string(), "disk.E02".to_string()]),
            segment_sizes: Some(vec![640 * 1024 * 1024, 300 * 1024 * 1024]),
            total_segment_size: Some(940 * 1024 * 1024),
            created: Some("2024-01-01".to_string()),
            modified: Some("2024-01-02".to_string()),
        };
        
        assert_eq!(file.filename, "disk.E01");
        assert_eq!(file.segment_count, Some(2));
    }

    #[test]
    fn test_verify_entry_creation() {
        let entry = VerifyEntry {
            path: Some("/evidence/disk.E01".to_string()),
            chunk_index: Some(42),
            status: "error".to_string(),
            message: Some("CRC mismatch".to_string()),
        };
        
        assert_eq!(entry.status, "error");
        assert_eq!(entry.chunk_index, Some(42));
    }

    #[test]
    fn test_container_kind_debug() {
        let kind = ContainerKind::E01;
        let debug_str = format!("{:?}", kind);
        assert_eq!(debug_str, "E01");
    }

    #[test]
    fn test_container_kind_variants() {
        // Ensure all variants exist and can be matched
        let kinds = [
            ContainerKind::Ad1,
            ContainerKind::E01,
            ContainerKind::L01,
            ContainerKind::Raw,
            ContainerKind::Archive,
            ContainerKind::Ufed,
        ];
        
        for kind in kinds {
            match kind {
                ContainerKind::Ad1 => assert_eq!(format!("{:?}", kind), "Ad1"),
                ContainerKind::E01 => assert_eq!(format!("{:?}", kind), "E01"),
                ContainerKind::L01 => assert_eq!(format!("{:?}", kind), "L01"),
                ContainerKind::Raw => assert_eq!(format!("{:?}", kind), "Raw"),
                ContainerKind::Archive => assert_eq!(format!("{:?}", kind), "Archive"),
                ContainerKind::Ufed => assert_eq!(format!("{:?}", kind), "Ufed"),
            }
        }
    }
}
