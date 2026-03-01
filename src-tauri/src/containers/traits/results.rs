// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Result types and data structures for evidence container operations.

use serde::Serialize;

use crate::formats::FormatCategory;

// =============================================================================
// LIFECYCLE STAGES
// =============================================================================

/// Evidence container lifecycle stages
///
/// Represents the stages a container goes through during analysis:
///
/// ```text
/// Discovered → Detected → Opened → Verified → Extracted
///     ↓           ↓          ↓         ↓          ↓
/// scan_dir   detect()   info()   verify()   extract()
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, strum::Display, strum::EnumString, strum::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum LifecycleStage {
    /// Container file discovered during directory scan
    Discovered,
    /// Format detected (magic bytes verified)
    Detected,
    /// Container opened and metadata parsed
    Opened,
    /// Integrity verified (hashes checked)
    Verified,
    /// Contents extracted to output
    Extracted,
    /// Error state
    Error,
}

// =============================================================================
// CONTAINER METADATA
// =============================================================================

/// Basic information about a container format
#[derive(Debug, Clone, Serialize)]
pub struct FormatInfo {
    /// Unique identifier (e.g., "e01", "ad1")
    pub id: &'static str,
    /// Display name (e.g., "Expert Witness Format")
    pub name: &'static str,
    /// File extensions (lowercase, no dot)
    pub extensions: &'static [&'static str],
    /// Format category
    pub category: FormatCategory,
    /// Whether format supports segmentation
    pub supports_segments: bool,
    /// Whether format stores hashes internally
    pub stores_hashes: bool,
    /// Whether format contains file/folder tree
    pub has_file_tree: bool,
}

/// Segment information for multi-segment containers
#[derive(Debug, Clone, Serialize)]
pub struct SegmentInfo {
    /// Total number of segments
    pub count: u32,
    /// Paths to all segment files
    pub files: Vec<String>,
    /// Size of each segment in bytes
    pub sizes: Vec<u64>,
    /// Total size of all segments
    pub total_size: u64,
    /// Missing segment files (gaps)
    pub missing: Vec<String>,
}

/// Hash verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct HashResult {
    /// Hash algorithm used
    pub algorithm: String,
    /// Computed hash value
    pub computed: String,
    /// Expected hash value (if stored)
    pub expected: Option<String>,
    /// Whether hashes match
    pub verified: Option<bool>,
    /// Time taken to compute (seconds)
    pub duration_secs: f64,
}

impl HashResult {
    /// Create a new HashResult
    #[inline]
    pub fn new(algorithm: impl Into<String>, computed: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            computed: computed.into(),
            ..Default::default()
        }
    }

    /// Set expected hash value
    #[inline]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    /// Set verification result
    #[inline]
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = Some(verified);
        self
    }

    /// Set duration
    #[inline]
    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Mark as verified (hashes match)
    #[inline]
    pub fn verified(algorithm: impl Into<String>, hash: impl Into<String>, duration: f64) -> Self {
        let hash_str = hash.into();
        Self {
            algorithm: algorithm.into(),
            computed: hash_str.clone(),
            expected: Some(hash_str),
            verified: Some(true),
            duration_secs: duration,
        }
    }

    /// Mark as mismatch
    #[inline]
    pub fn mismatch(
        algorithm: impl Into<String>,
        computed: impl Into<String>,
        expected: impl Into<String>,
        duration: f64,
    ) -> Self {
        Self {
            algorithm: algorithm.into(),
            computed: computed.into(),
            expected: Some(expected.into()),
            verified: Some(false),
            duration_secs: duration,
        }
    }
}

/// Container verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct VerifyResult {
    /// Overall verification status
    pub status: VerifyStatus,
    /// Hash verification results
    pub hashes: Vec<HashResult>,
    /// Per-chunk/block verification (if applicable)
    pub chunks: Vec<ChunkVerifyResult>,
    /// Verification messages/warnings
    pub messages: Vec<String>,
}

impl VerifyResult {
    /// Create a new VerifyResult with a status
    #[inline]
    pub fn new(status: VerifyStatus) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }

    /// Create a verified result
    #[inline]
    pub fn verified() -> Self {
        Self::new(VerifyStatus::Verified)
    }

    /// Create a computed (no comparison) result
    #[inline]
    pub fn computed() -> Self {
        Self::new(VerifyStatus::Computed)
    }

    /// Create a mismatch result
    #[inline]
    pub fn mismatched() -> Self {
        Self::new(VerifyStatus::Mismatch)
    }

    /// Add a hash result
    #[inline]
    pub fn with_hash(mut self, hash: HashResult) -> Self {
        self.hashes.push(hash);
        self
    }

    /// Add a chunk verification result
    #[inline]
    pub fn with_chunk(mut self, chunk: ChunkVerifyResult) -> Self {
        self.chunks.push(chunk);
        self
    }

    /// Add a message
    #[inline]
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum VerifyStatus {
    /// All hashes verified successfully
    Verified,
    /// Computed hash (no stored hash to compare)
    #[default]
    Computed,
    /// Hash mismatch detected
    Mismatch,
    /// Verification not supported for this format
    NotSupported,
    /// Verification failed due to error
    Error,
}

/// Per-chunk verification result
#[derive(Debug, Clone, Default, Serialize)]
pub struct ChunkVerifyResult {
    /// Chunk index
    pub index: usize,
    /// Status ("ok", "mismatch", "error")
    pub status: String,
    /// Error message if any
    pub message: Option<String>,
}

impl ChunkVerifyResult {
    /// Create a successful chunk result
    #[inline]
    pub fn ok(index: usize) -> Self {
        Self {
            index,
            status: "ok".to_string(),
            message: None,
        }
    }

    /// Create a mismatch chunk result
    #[inline]
    pub fn mismatch(index: usize) -> Self {
        Self {
            index,
            status: "mismatch".to_string(),
            message: None,
        }
    }

    /// Create an error chunk result
    #[inline]
    pub fn error(index: usize, msg: impl Into<String>) -> Self {
        Self {
            index,
            status: "error".to_string(),
            message: Some(msg.into()),
        }
    }

    /// Add a message
    #[inline]
    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }
}

// =============================================================================
// CONTAINER METADATA STRUCTS
// =============================================================================

/// Generic container metadata (returned by info())
#[derive(Debug, Clone, Serialize, Default)]
pub struct ContainerMetadata {
    /// Format identifier
    pub format: String,
    /// Format version (if applicable)
    pub version: Option<String>,
    /// Total logical size of data
    pub total_size: u64,
    /// Segment information
    pub segments: Option<SegmentInfo>,
    /// Stored hashes
    pub stored_hashes: Vec<StoredHashInfo>,
    /// Case/evidence metadata
    pub case_info: Option<CaseMetadata>,
    /// Additional format-specific data (serialized as JSON)
    pub format_specific: Option<serde_json::Value>,
}

impl ContainerMetadata {
    /// Create new ContainerMetadata with format
    #[inline]
    pub fn new(format: impl Into<String>) -> Self {
        Self {
            format: format.into(),
            ..Default::default()
        }
    }

    /// Set format version
    #[inline]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set total size
    #[inline]
    pub fn with_size(mut self, size: u64) -> Self {
        self.total_size = size;
        self
    }

    /// Set segment info
    #[inline]
    pub fn with_segments(mut self, segments: SegmentInfo) -> Self {
        self.segments = Some(segments);
        self
    }

    /// Add a stored hash
    #[inline]
    pub fn add_hash(mut self, hash: StoredHashInfo) -> Self {
        self.stored_hashes.push(hash);
        self
    }

    /// Set case metadata
    #[inline]
    pub fn with_case_info(mut self, case_info: CaseMetadata) -> Self {
        self.case_info = Some(case_info);
        self
    }

    /// Set format-specific data
    #[inline]
    pub fn with_format_data(mut self, data: serde_json::Value) -> Self {
        self.format_specific = Some(data);
        self
    }
}

/// Stored hash information
#[derive(Debug, Clone, Serialize, Default)]
pub struct StoredHashInfo {
    /// Algorithm (e.g., "MD5", "SHA1")
    pub algorithm: String,
    /// Hash value (hex string)
    pub hash: String,
    /// Hash source ("container", "companion", "computed")
    pub source: String,
    /// Verification status
    pub verified: Option<bool>,
}

impl StoredHashInfo {
    /// Create new StoredHashInfo
    #[inline]
    pub fn new(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            hash: hash.into(),
            source: "container".to_string(),
            verified: None,
        }
    }

    /// Set source type
    #[inline]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Create from companion file
    #[inline]
    pub fn from_companion(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(algorithm, hash).with_source("companion")
    }

    /// Create from computed value
    #[inline]
    pub fn from_computed(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Self::new(algorithm, hash).with_source("computed")
    }

    /// Mark as verified
    #[inline]
    pub fn verified(mut self, is_verified: bool) -> Self {
        self.verified = Some(is_verified);
        self
    }
}

/// Case metadata from container
#[derive(Debug, Clone, Serialize, Default)]
pub struct CaseMetadata {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquisition_date: Option<String>,
}

impl CaseMetadata {
    /// Create empty CaseMetadata
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

    /// Set examiner name
    #[inline]
    pub fn with_examiner(mut self, examiner: impl Into<String>) -> Self {
        self.examiner_name = Some(examiner.into());
        self
    }

    /// Set description
    #[inline]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
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
    pub fn with_acquisition_date(mut self, date: impl Into<String>) -> Self {
        self.acquisition_date = Some(date.into());
        self
    }

    /// Check if any case info is present
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.case_number.is_none()
            && self.evidence_number.is_none()
            && self.examiner_name.is_none()
            && self.description.is_none()
            && self.notes.is_none()
            && self.acquisition_date.is_none()
    }
}

/// Metadata for a single segment
#[derive(Debug, Clone, Serialize)]
pub struct SegmentMetadata {
    pub index: u32,
    pub path: String,
    pub size: u64,
    pub hash: Option<String>,
}

/// Tree entry information
#[derive(Debug, Clone, Serialize, Default)]
pub struct TreeEntryInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub hash: Option<String>,
}

impl TreeEntryInfo {
    /// Create new TreeEntryInfo for a file
    #[inline]
    pub fn file(path: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_directory: false,
            size,
            ..Default::default()
        }
    }

    /// Create new TreeEntryInfo for a directory
    #[inline]
    pub fn directory(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            is_directory: true,
            size: 0,
            ..Default::default()
        }
    }

    /// Set created timestamp
    #[inline]
    pub fn with_created(mut self, created: impl Into<String>) -> Self {
        self.created = Some(created.into());
        self
    }

    /// Set modified timestamp
    #[inline]
    pub fn with_modified(mut self, modified: impl Into<String>) -> Self {
        self.modified = Some(modified.into());
        self
    }

    /// Set accessed timestamp
    #[inline]
    pub fn with_accessed(mut self, accessed: impl Into<String>) -> Self {
        self.accessed = Some(accessed.into());
        self
    }

    /// Set all timestamps at once
    #[inline]
    pub fn with_timestamps(
        mut self,
        created: Option<String>,
        modified: Option<String>,
        accessed: Option<String>,
    ) -> Self {
        self.created = created;
        self.modified = modified;
        self.accessed = accessed;
        self
    }

    /// Set hash value
    #[inline]
    pub fn with_hash(mut self, hash: impl Into<String>) -> Self {
        self.hash = Some(hash.into());
        self
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // LifecycleStage
    // =========================================================================

    #[test]
    fn test_lifecycle_stage_display() {
        assert_eq!(LifecycleStage::Discovered.to_string(), "discovered");
        assert_eq!(LifecycleStage::Detected.to_string(), "detected");
        assert_eq!(LifecycleStage::Opened.to_string(), "opened");
        assert_eq!(LifecycleStage::Verified.to_string(), "verified");
        assert_eq!(LifecycleStage::Extracted.to_string(), "extracted");
        assert_eq!(LifecycleStage::Error.to_string(), "error");
    }

    #[test]
    fn test_lifecycle_stage_from_string() {
        use std::str::FromStr;
        assert_eq!(
            LifecycleStage::from_str("discovered").unwrap(),
            LifecycleStage::Discovered
        );
        assert_eq!(
            LifecycleStage::from_str("verified").unwrap(),
            LifecycleStage::Verified
        );
        assert_eq!(
            LifecycleStage::from_str("error").unwrap(),
            LifecycleStage::Error
        );
    }

    #[test]
    fn test_lifecycle_stage_as_ref() {
        assert_eq!(LifecycleStage::Discovered.as_ref(), "discovered");
        assert_eq!(LifecycleStage::Extracted.as_ref(), "extracted");
    }

    #[test]
    fn test_lifecycle_stage_equality() {
        assert_eq!(LifecycleStage::Verified, LifecycleStage::Verified);
        assert_ne!(LifecycleStage::Discovered, LifecycleStage::Detected);
    }

    #[test]
    fn test_lifecycle_stage_serialization() {
        let json = serde_json::to_string(&LifecycleStage::Opened).unwrap();
        assert!(json.contains("Opened"));
    }

    // =========================================================================
    // HashResult
    // =========================================================================

    #[test]
    fn test_hash_result_new() {
        let hr = HashResult::new("SHA-256", "abcdef1234567890");
        assert_eq!(hr.algorithm, "SHA-256");
        assert_eq!(hr.computed, "abcdef1234567890");
        assert!(hr.expected.is_none());
        assert!(hr.verified.is_none());
        assert_eq!(hr.duration_secs, 0.0);
    }

    #[test]
    fn test_hash_result_builder_chain() {
        let hr = HashResult::new("MD5", "abc")
            .with_expected("abc")
            .with_verified(true)
            .with_duration(1.5);

        assert_eq!(hr.expected.unwrap(), "abc");
        assert_eq!(hr.verified, Some(true));
        assert_eq!(hr.duration_secs, 1.5);
    }

    #[test]
    fn test_hash_result_verified_constructor() {
        let hr = HashResult::verified("SHA-1", "deadbeef", 2.0);
        assert_eq!(hr.algorithm, "SHA-1");
        assert_eq!(hr.computed, "deadbeef");
        assert_eq!(hr.expected.unwrap(), "deadbeef");
        assert_eq!(hr.verified, Some(true));
        assert_eq!(hr.duration_secs, 2.0);
    }

    #[test]
    fn test_hash_result_mismatch_constructor() {
        let hr = HashResult::mismatch("MD5", "aaa", "bbb", 0.5);
        assert_eq!(hr.computed, "aaa");
        assert_eq!(hr.expected.unwrap(), "bbb");
        assert_eq!(hr.verified, Some(false));
    }

    // =========================================================================
    // VerifyResult
    // =========================================================================

    #[test]
    fn test_verify_result_constructors() {
        assert_eq!(VerifyResult::verified().status, VerifyStatus::Verified);
        assert_eq!(VerifyResult::computed().status, VerifyStatus::Computed);
        assert_eq!(VerifyResult::mismatched().status, VerifyStatus::Mismatch);
    }

    #[test]
    fn test_verify_result_default_status() {
        let vr = VerifyResult::default();
        assert_eq!(vr.status, VerifyStatus::Computed); // Computed is #[default]
    }

    #[test]
    fn test_verify_result_builder() {
        let vr = VerifyResult::verified()
            .with_hash(HashResult::new("SHA-256", "abc"))
            .with_chunk(ChunkVerifyResult::ok(0))
            .with_message("All chunks verified");

        assert_eq!(vr.hashes.len(), 1);
        assert_eq!(vr.chunks.len(), 1);
        assert_eq!(vr.messages.len(), 1);
        assert_eq!(vr.messages[0], "All chunks verified");
    }

    #[test]
    fn test_verify_status_equality() {
        assert_eq!(VerifyStatus::Verified, VerifyStatus::Verified);
        assert_ne!(VerifyStatus::Verified, VerifyStatus::Mismatch);
        assert_ne!(VerifyStatus::NotSupported, VerifyStatus::Error);
    }

    // =========================================================================
    // ChunkVerifyResult
    // =========================================================================

    #[test]
    fn test_chunk_verify_result_ok() {
        let chunk = ChunkVerifyResult::ok(0);
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.status, "ok");
        assert!(chunk.message.is_none());
    }

    #[test]
    fn test_chunk_verify_result_mismatch() {
        let chunk = ChunkVerifyResult::mismatch(5);
        assert_eq!(chunk.index, 5);
        assert_eq!(chunk.status, "mismatch");
    }

    #[test]
    fn test_chunk_verify_result_error() {
        let chunk = ChunkVerifyResult::error(10, "CRC failed");
        assert_eq!(chunk.index, 10);
        assert_eq!(chunk.status, "error");
        assert_eq!(chunk.message.unwrap(), "CRC failed");
    }

    #[test]
    fn test_chunk_verify_result_with_message() {
        let chunk = ChunkVerifyResult::ok(0).with_message("sector 0");
        assert_eq!(chunk.message.unwrap(), "sector 0");
    }

    // =========================================================================
    // ContainerMetadata
    // =========================================================================

    #[test]
    fn test_container_metadata_new() {
        let meta = ContainerMetadata::new("e01");
        assert_eq!(meta.format, "e01");
        assert_eq!(meta.total_size, 0);
        assert!(meta.version.is_none());
        assert!(meta.segments.is_none());
    }

    #[test]
    fn test_container_metadata_builder_chain() {
        let meta = ContainerMetadata::new("ad1")
            .with_version("3.0")
            .with_size(1024 * 1024)
            .with_case_info(CaseMetadata::new().with_case_number("CASE-1"))
            .with_format_data(serde_json::json!({"compression": "zlib"}));

        assert_eq!(meta.version.unwrap(), "3.0");
        assert_eq!(meta.total_size, 1024 * 1024);
        assert!(meta.case_info.is_some());
        assert!(meta.format_specific.is_some());
    }

    #[test]
    fn test_container_metadata_add_hash() {
        let meta = ContainerMetadata::new("e01")
            .add_hash(StoredHashInfo::new("MD5", "abc123"))
            .add_hash(StoredHashInfo::new("SHA-1", "def456"));

        assert_eq!(meta.stored_hashes.len(), 2);
    }

    // =========================================================================
    // StoredHashInfo
    // =========================================================================

    #[test]
    fn test_stored_hash_info_new() {
        let h = StoredHashInfo::new("MD5", "abcdef");
        assert_eq!(h.algorithm, "MD5");
        assert_eq!(h.hash, "abcdef");
        assert_eq!(h.source, "container");
        assert!(h.verified.is_none());
    }

    #[test]
    fn test_stored_hash_info_from_companion() {
        let h = StoredHashInfo::from_companion("SHA-256", "deadbeef");
        assert_eq!(h.source, "companion");
    }

    #[test]
    fn test_stored_hash_info_from_computed() {
        let h = StoredHashInfo::from_computed("BLAKE3", "cafebabe");
        assert_eq!(h.source, "computed");
    }

    #[test]
    fn test_stored_hash_info_verified() {
        let h = StoredHashInfo::new("MD5", "abc").verified(true);
        assert_eq!(h.verified, Some(true));

        let h2 = StoredHashInfo::new("MD5", "abc").verified(false);
        assert_eq!(h2.verified, Some(false));
    }

    // =========================================================================
    // CaseMetadata
    // =========================================================================

    #[test]
    fn test_case_metadata_new_is_empty() {
        let meta = CaseMetadata::new();
        assert!(meta.is_empty());
    }

    #[test]
    fn test_case_metadata_builder_chain() {
        let meta = CaseMetadata::new()
            .with_case_number("C-100")
            .with_evidence_number("E-001")
            .with_examiner("Dr. Jones")
            .with_description("Test case")
            .with_notes("Important notes")
            .with_acquisition_date("2026-01-15");

        assert_eq!(meta.case_number.as_deref(), Some("C-100"));
        assert_eq!(meta.evidence_number.as_deref(), Some("E-001"));
        assert_eq!(meta.examiner_name.as_deref(), Some("Dr. Jones"));
        assert_eq!(meta.description.as_deref(), Some("Test case"));
        assert_eq!(meta.notes.as_deref(), Some("Important notes"));
        assert_eq!(meta.acquisition_date.as_deref(), Some("2026-01-15"));
        assert!(!meta.is_empty());
    }

    #[test]
    fn test_case_metadata_is_empty_with_one_field() {
        let meta = CaseMetadata::new().with_case_number("X");
        assert!(!meta.is_empty());
    }

    // =========================================================================
    // TreeEntryInfo
    // =========================================================================

    #[test]
    fn test_tree_entry_info_file() {
        let entry = TreeEntryInfo::file("/docs/readme.txt", "readme.txt", 1024);
        assert_eq!(entry.path, "/docs/readme.txt");
        assert_eq!(entry.name, "readme.txt");
        assert!(!entry.is_directory);
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_tree_entry_info_directory() {
        let entry = TreeEntryInfo::directory("/docs", "docs");
        assert_eq!(entry.path, "/docs");
        assert!(entry.is_directory);
        assert_eq!(entry.size, 0);
    }

    #[test]
    fn test_tree_entry_info_with_timestamps() {
        let entry = TreeEntryInfo::file("/a.txt", "a.txt", 10)
            .with_created("2026-01-01T00:00:00Z")
            .with_modified("2026-01-02T00:00:00Z")
            .with_accessed("2026-01-03T00:00:00Z");

        assert_eq!(entry.created.unwrap(), "2026-01-01T00:00:00Z");
        assert_eq!(entry.modified.unwrap(), "2026-01-02T00:00:00Z");
        assert_eq!(entry.accessed.unwrap(), "2026-01-03T00:00:00Z");
    }

    #[test]
    fn test_tree_entry_info_with_timestamps_batch() {
        let entry = TreeEntryInfo::file("/b.txt", "b.txt", 20).with_timestamps(
            Some("2026-01-01".into()),
            Some("2026-01-02".into()),
            None,
        );

        assert!(entry.created.is_some());
        assert!(entry.modified.is_some());
        assert!(entry.accessed.is_none());
    }

    #[test]
    fn test_tree_entry_info_with_hash() {
        let entry = TreeEntryInfo::file("/c.dat", "c.dat", 100).with_hash("abcdef1234567890");
        assert_eq!(entry.hash.unwrap(), "abcdef1234567890");
    }

    // =========================================================================
    // FormatInfo
    // =========================================================================

    #[test]
    fn test_format_info_fields() {
        let info = FormatInfo {
            id: "e01",
            name: "Expert Witness Format",
            extensions: &["e01", "ex01"],
            category: FormatCategory::ForensicContainer,
            supports_segments: true,
            stores_hashes: true,
            has_file_tree: false,
        };
        assert_eq!(info.id, "e01");
        assert!(info.supports_segments);
        assert!(info.stores_hashes);
        assert!(!info.has_file_tree);
    }

    // =========================================================================
    // SegmentInfo
    // =========================================================================

    #[test]
    fn test_segment_info() {
        let seg = SegmentInfo {
            count: 3,
            files: vec!["a.E01".into(), "a.E02".into(), "a.E03".into()],
            sizes: vec![100, 200, 150],
            total_size: 450,
            missing: vec![],
        };
        assert_eq!(seg.count, 3);
        assert_eq!(seg.total_size, 450);
        assert!(seg.missing.is_empty());
    }

    // =========================================================================
    // SegmentMetadata
    // =========================================================================

    #[test]
    fn test_segment_metadata() {
        let sm = SegmentMetadata {
            index: 0,
            path: "/data/image.E01".into(),
            size: 1_000_000,
            hash: Some("abc123".into()),
        };
        assert_eq!(sm.index, 0);
        assert_eq!(sm.size, 1_000_000);
        assert!(sm.hash.is_some());
    }
}
