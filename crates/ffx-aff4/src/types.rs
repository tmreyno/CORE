// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Core types for AFF4 containers.
//!
//! Covers configuration, progress tracking, results, and shared domain types
//! used by both the reader and writer.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ─── AFF4 Version ────────────────────────────────────────────────────────────

/// AFF4 container version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aff4Version {
    /// AFF4-Standard v1.0 — physical disk imaging.
    Standard,
    /// AFF4-L v1.1 — logical file collection.
    Logical,
}

impl Aff4Version {
    pub fn major(self) -> u32 {
        1
    }

    pub fn minor(self) -> u32 {
        match self {
            Self::Standard => 0,
            Self::Logical => 1,
        }
    }
}

// ─── Compression ─────────────────────────────────────────────────────────────

/// Compression algorithm for AFF4 bevies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Aff4Compression {
    /// No compression — chunks stored verbatim.
    Stored,
    /// Deflate (RFC 1951) — most compatible.
    #[default]
    Deflate,
    /// LZ4 — fast compression/decompression.
    Lz4,
    /// Snappy — Google's fast compressor.
    Snappy,
}

impl Aff4Compression {
    /// RDF URI for the compression method.
    /// `Stored` has no URI (absence of `compressionMethod` means stored).
    pub fn rdf_uri(&self) -> Option<&'static str> {
        match self {
            Self::Stored => None,
            Self::Deflate => Some("https://tools.ietf.org/html/rfc1951"),
            Self::Lz4 => Some("https://code.google.com/p/lz4/"),
            Self::Snappy => Some("http://code.google.com/p/snappy/"),
        }
    }

    /// Parse from an RDF URI string.
    pub fn from_rdf_uri(uri: &str) -> Option<Self> {
        match uri {
            "https://tools.ietf.org/html/rfc1951" => Some(Self::Deflate),
            "https://code.google.com/p/lz4/" | "http://code.google.com/p/lz4/" => Some(Self::Lz4),
            "http://code.google.com/p/snappy/" | "https://code.google.com/p/snappy/" => {
                Some(Self::Snappy)
            }
            _ => None,
        }
    }
}

// ─── Hash Algorithm ──────────────────────────────────────────────────────────

/// Hash algorithm for AFF4 integrity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aff4HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake2b,
}

impl Aff4HashAlgorithm {
    /// RDF URI for this hash algorithm.
    pub fn rdf_uri(&self) -> &'static str {
        match self {
            Self::Md5 => "http://aff4.org/Schema#MD5",
            Self::Sha1 => "http://aff4.org/Schema#SHA1",
            Self::Sha256 => "http://aff4.org/Schema#SHA256",
            Self::Sha512 => "http://aff4.org/Schema#SHA512",
            Self::Blake2b => "http://aff4.org/Schema#Blake2b",
        }
    }

    /// Parse from an RDF URI. Returns None for unknown URIs.
    pub fn from_rdf_uri(uri: &str) -> Option<Self> {
        match uri {
            "http://aff4.org/Schema#MD5" => Some(Self::Md5),
            "http://aff4.org/Schema#SHA1" => Some(Self::Sha1),
            "http://aff4.org/Schema#SHA256" => Some(Self::Sha256),
            "http://aff4.org/Schema#SHA512" => Some(Self::Sha512),
            "http://aff4.org/Schema#Blake2b" => Some(Self::Blake2b),
            _ => None,
        }
    }

    /// File extension for block hash files.
    pub fn block_hash_extension(&self) -> &'static str {
        match self {
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
            Self::Blake2b => "blake2b",
        }
    }

    /// Digest size in bytes.
    pub fn digest_size(&self) -> usize {
        match self {
            Self::Md5 => 16,
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha512 => 64,
            Self::Blake2b => 64,
        }
    }
}

// ─── RDF Types ───────────────────────────────────────────────────────────────

/// AFF4 RDF type URIs.
pub mod rdf_types {
    pub const DISK_IMAGE: &str = "http://aff4.org/Schema#DiskImage";
    pub const CONTIGUOUS_IMAGE: &str = "http://aff4.org/Schema#ContiguousImage";
    pub const IMAGE: &str = "http://aff4.org/Schema#Image";
    pub const MAP: &str = "http://aff4.org/Schema#Map";
    pub const IMAGE_STREAM: &str = "http://aff4.org/Schema#ImageStream";
    pub const FILE_IMAGE: &str = "http://aff4.org/Schema#FileImage";
    pub const FOLDER: &str = "http://aff4.org/Schema#Folder";
    pub const ZIP_VOLUME: &str = "http://aff4.org/Schema#ZipVolume";
    pub const ZIP_SEGMENT: &str = "http://aff4.org/Schema#ZipSegment";
}

/// AFF4 RDF predicate URIs.
pub mod rdf_predicates {
    pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    pub const CHUNK_SIZE: &str = "http://aff4.org/Schema#chunkSize";
    pub const CHUNKS_PER_SEGMENT: &str = "http://aff4.org/Schema#chunksInSegment";
    pub const COMPRESSION_METHOD: &str = "http://aff4.org/Schema#compressionMethod";
    pub const SIZE: &str = "http://aff4.org/Schema#size";
    pub const STORED_HASH: &str = "http://aff4.org/Schema#hash";
    pub const DEPENDENT_STREAM: &str = "http://aff4.org/Schema#dependentStream";
    pub const TARGET: &str = "http://aff4.org/Schema#target";
    pub const MAP_POINT_HASH: &str = "http://aff4.org/Schema#mapPointHash";
    pub const MAP_IDX_HASH: &str = "http://aff4.org/Schema#mapIdxHash";
    pub const MAP_PATH_HASH: &str = "http://aff4.org/Schema#mapPathHash";
    pub const BLOCK_MAP_HASH: &str = "http://aff4.org/Schema#blockMapHash";
    pub const INTERFACE: &str = "http://aff4.org/Schema#interface";
    pub const ORIGINAL_FILENAME: &str = "http://aff4.org/Schema#originalFileName";
    pub const LAST_WRITTEN: &str = "http://aff4.org/Schema#lastWritten";
    pub const LAST_ACCESSED: &str = "http://aff4.org/Schema#lastAccessed";
    pub const RECORD_CHANGED: &str = "http://aff4.org/Schema#recordChanged";
    pub const BIRTH_TIME: &str = "http://aff4.org/Schema#birthTime";
    pub const PATH_NAME: &str = "http://aff4.org/Schema#pathName";
    // Dublin Core metadata
    pub const DC_CASE_NUMBER: &str = "http://purl.org/dc/elements/1.1/caseNumber";
    pub const DC_EVIDENCE_NUMBER: &str = "http://purl.org/dc/elements/1.1/evidenceNumber";
    pub const DC_EXAMINER: &str = "http://purl.org/dc/elements/1.1/examiner";
    pub const DC_DESCRIPTION: &str = "http://purl.org/dc/elements/1.1/description";
}

// ─── Container Constants ─────────────────────────────────────────────────────

/// The first ZIP member in any AFF4 container.
pub const CONTAINER_DESCRIPTION: &str = "container.description";

/// AFF4 version file.
pub const VERSION_TXT: &str = "version.txt";

/// RDF metadata file.
pub const INFORMATION_TURTLE: &str = "information.turtle";

/// Default chunk size in bytes (32 KiB).
pub const DEFAULT_CHUNK_SIZE: u32 = 32_768;

/// Default chunks per bevy segment.
pub const DEFAULT_CHUNKS_PER_SEGMENT: u32 = 1024;

/// Compression savings threshold: `compressed_len < chunk_size - COMPRESSION_THRESHOLD`.
/// If compression does not save at least this many bytes, store the chunk uncompressed.
pub const COMPRESSION_THRESHOLD: u32 = 16;

/// Size of a bevy index entry in bytes: u64 offset + u32 length = 12.
pub const BEVY_INDEX_ENTRY_SIZE: usize = 12;

/// Size of a map entry in bytes: u64 + u64 + u64 + u32 = 28.
pub const MAP_ENTRY_SIZE: usize = 28;

/// AFF4-L small file threshold: files ≤ this size are stored as ZIP segments.
pub const LOGICAL_SMALL_FILE_THRESHOLD: u64 = 1_048_576; // 1 MiB

// ─── Writer Configuration ────────────────────────────────────────────────────

/// Configuration for creating an AFF4 container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4WriterConfig {
    /// Output file path (without extension — `.aff4` appended automatically).
    pub output_path: PathBuf,

    /// AFF4 version to create.
    pub version: Aff4Version,

    /// Compression algorithm.
    pub compression: Aff4Compression,

    /// Chunk size in bytes.
    pub chunk_size: u32,

    /// Number of chunks per bevy segment.
    pub chunks_per_segment: u32,

    /// Hash algorithms to use for linear (whole-stream) hashing.
    pub linear_hashes: Vec<Aff4HashAlgorithm>,

    /// Hash algorithms to use for block (per-chunk) hashing.
    pub block_hashes: Vec<Aff4HashAlgorithm>,

    /// Case metadata.
    pub case_number: String,
    pub evidence_number: String,
    pub examiner: String,
    pub description: String,
    pub notes: String,

    /// Tool identity string for version.txt.
    pub tool_name: String,
}

impl Default for Aff4WriterConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::new(),
            version: Aff4Version::Standard,
            compression: Aff4Compression::Deflate,
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunks_per_segment: DEFAULT_CHUNKS_PER_SEGMENT,
            linear_hashes: vec![Aff4HashAlgorithm::Sha256],
            block_hashes: vec![Aff4HashAlgorithm::Sha256],
            case_number: String::new(),
            evidence_number: String::new(),
            examiner: String::new(),
            description: String::new(),
            notes: String::new(),
            tool_name: "CORE-FFX".to_string(),
        }
    }
}

// ─── Progress ────────────────────────────────────────────────────────────────

/// Write/read operation phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Aff4Phase {
    Preparing,
    WritingData,
    WritingMetadata,
    ComputingHashes,
    Finalizing,
    Reading,
    Verifying,
}

/// Progress report during write/read operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4Progress {
    pub phase: Aff4Phase,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub current_file: String,
    pub files_processed: usize,
    pub total_files: usize,
}

impl Aff4Progress {
    pub fn percent(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_processed as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

// ─── Write Result ────────────────────────────────────────────────────────────

/// Result returned after successfully writing an AFF4 container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4WriteResult {
    /// Path to the output .aff4 file.
    pub output_path: PathBuf,

    /// Container volume URN.
    pub volume_urn: String,

    /// Image URN (for physical) or root URN (for logical).
    pub image_urn: String,

    /// Total bytes of source data written.
    pub total_bytes: u64,

    /// Total bytes in the container (compressed).
    pub container_bytes: u64,

    /// Compression ratio (container_bytes / total_bytes).
    pub compression_ratio: f64,

    /// Number of bevies written.
    pub bevy_count: u32,

    /// Number of files (logical mode).
    pub file_count: usize,

    /// Linear (whole-stream) hash values: algorithm → hex digest.
    pub linear_hashes: HashMap<Aff4HashAlgorithm, String>,

    /// Block map hash values: algorithm → hex digest.
    pub block_map_hashes: HashMap<Aff4HashAlgorithm, String>,
}

// ─── Read Result ─────────────────────────────────────────────────────────────

/// Metadata read from an AFF4 container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4ContainerInfo {
    /// Container volume URN.
    pub volume_urn: String,

    /// AFF4 version.
    pub version: Aff4Version,

    /// Tool that created the container.
    pub tool: String,

    /// Image streams contained.
    pub streams: Vec<Aff4StreamInfo>,

    /// Case metadata from RDF.
    pub case_number: String,
    pub evidence_number: String,
    pub examiner: String,
    pub description: String,
}

/// Information about a single image stream in the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4StreamInfo {
    /// Stream URN.
    pub urn: String,

    /// Stream type (DiskImage, FileImage, etc.).
    pub stream_type: String,

    /// Size in bytes.
    pub size: u64,

    /// Compression algorithm.
    pub compression: Aff4Compression,

    /// Chunk size.
    pub chunk_size: u32,

    /// Chunks per segment.
    pub chunks_per_segment: u32,

    /// Stored hash values: algorithm → hex digest.
    pub hashes: HashMap<Aff4HashAlgorithm, String>,
}

// ─── Logical File Entry ──────────────────────────────────────────────────────

/// A logical file entry for AFF4-L containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4LogicalEntry {
    /// Original file path (relative to collection root).
    pub original_path: String,

    /// Size in bytes.
    pub size: u64,

    /// Whether this is a directory.
    pub is_directory: bool,

    /// Last modified timestamp (nanoseconds since Unix epoch).
    pub last_written: Option<i64>,

    /// Last accessed timestamp.
    pub last_accessed: Option<i64>,

    /// Record changed timestamp.
    pub record_changed: Option<i64>,

    /// Birth/creation timestamp.
    pub birth_time: Option<i64>,

    /// Source path on disk (for writer to read from).
    pub source_path: Option<PathBuf>,

    /// Per-file hashes computed during write.
    pub hashes: HashMap<Aff4HashAlgorithm, String>,
}

impl Aff4LogicalEntry {
    /// Create a file entry from a source path and relative path.
    pub fn from_source(source: PathBuf, relative_path: String) -> Self {
        let metadata = std::fs::metadata(&source).ok();
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let is_directory = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);

        Self {
            original_path: relative_path,
            size,
            is_directory,
            last_written: metadata.as_ref().and_then(|m| {
                m.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
            }),
            last_accessed: metadata.as_ref().and_then(|m| {
                m.accessed()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
            }),
            record_changed: None,
            birth_time: None,
            source_path: Some(source),
            hashes: HashMap::new(),
        }
    }
}

// ─── Verification Result ─────────────────────────────────────────────────────

/// Result of verifying an AFF4 container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4VerifyResult {
    /// Whether all verifications passed.
    pub passed: bool,

    /// Linear hash verification results.
    pub linear_checks: Vec<Aff4HashCheck>,

    /// Block map hash verification results.
    pub block_map_checks: Vec<Aff4HashCheck>,

    /// Number of chunks verified.
    pub chunks_verified: u64,

    /// Number of chunk hash mismatches.
    pub chunk_errors: u64,
}

/// Result of a single hash check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aff4HashCheck {
    pub algorithm: Aff4HashAlgorithm,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}
