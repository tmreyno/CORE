// =============================================================================
// ffx-common — Common Utilities for Forensic Container Parsers
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Common Utilities for Forensic Container Parsers
//!
//! This crate provides shared functionality used across all container format
//! parsers (AD1, E01, RAW, UFED, Archives). These utilities are designed with
//! forensic requirements in mind: read-only operations, audit logging, and
//! secure path handling.
//!
//! # Submodules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`hash`] | Cryptographic hashing (MD5, SHA-*, BLAKE3, XXH3) |
//! | [`artifacts`] | Normalized artifact extraction from evidence sources |
//! | [`binary`] | Little-endian binary reading utilities |
//! | [`segments`] | Multi-segment file discovery (.E01/.E02, .ad1/.ad2) |
//! | [`evidence_source`] | Read-only byte sources for evidence data |
//! | [`source_analysis`] | Source-aware byte statistics and signature analysis |
//! | [`io_pool`] | File handle pooling for segment management |
//! | [`hex`] | Hex dump formatting for viewers |
//! | [`magic`] | File type detection by magic bytes |
//! | [`path_security`] | Path traversal protection |
//! | [`audit`] | Forensic audit logging |
//! | [`vfs`] | Virtual filesystem abstraction |
//! | [`progress`] | Progress tracking for long operations |
//! | [`filesystem`] | Cross-platform filesystem utilities |
//!
//! # Performance Tuning
//!
//! The crate provides tuned constants for high-throughput I/O:
//!
//! - `BUFFER_SIZE`: 16MB buffer optimized for modern NVMe SSDs
//! - `MMAP_THRESHOLD`: 64MB threshold for memory-mapped I/O

pub mod artifacts;
pub mod audit;
pub mod binary;
pub mod container_detect;
pub mod datetime;
pub mod evidence_source;
pub mod filesystem;
pub mod hash;
pub mod hash_cache;
pub mod health;
pub mod hex;
pub mod io_adaptive;
pub mod io_pool;
pub mod lazy_loading;
pub mod magic;
pub mod metrics;
pub mod path_security;
pub mod progress;
pub mod retry;
pub mod segment_hash;
pub mod segments;
pub mod source_analysis;
pub mod vfs;

// Re-exports for convenience
pub use artifacts::{extract_normalized_artifact, ArtifactExtractionOptions, NormalizedArtifact};
pub use audit::{
    log_container_opened, log_evidence_access, log_hash_verification, log_report_generation,
    log_security_event,
};
pub use binary::{read_u16_le, read_u32_be, read_u32_le, read_u64_le, read_u8};
pub use container_detect::{
    detect_container_type, is_container, is_forensic_container, is_segmented_container,
    ContainerType,
};
pub use datetime::{
    format_display, format_duration, now_local_display, now_rfc3339, parse_rfc3339,
};
pub use evidence_source::{
    bounded_read_size, read_all_with_limit, read_range_fully, EvidenceByteSource,
    EvidenceSourceError, EvidenceSourceReader, EvidenceSourceRef, EvidenceSourceResult,
    LocalFileByteSource, VfsEntryByteSource,
};
pub use hash::{compare_hashes, verify_hash, HashMatchResult, HashVerificationResult};
pub use hash::{
    compute_hash, hash_byte_source, hash_byte_source_with_progress, hash_file_with_progress,
    HashAlgorithm, StreamingHasher,
};
pub use hash_cache::{
    cache_hash, get_cached_hash, get_or_compute_hash, HashCache, HashCacheEntry, HashCacheKey,
    HashCacheStats, GLOBAL_HASH_CACHE,
};
pub use hex::{
    csv_header, csv_row, escape_csv, format_hex_inline, format_hex_string, format_size,
    format_size_compact,
};
pub use io_adaptive::{AdaptiveBuffer, AdaptiveStats, Operation as IoOperation};
pub use io_pool::{FileIoPool, DEFAULT_MAX_OPEN_FILES};
pub use lazy_loading::{
    ContainerSummary, LazyLoadConfig, LazyLoadResult, LazyLoadable, LazyTreeEntry,
};
pub use magic::{detect_file_type, is_archive, is_executable, is_image, FileCategory, FileType};
pub use path_security::{contains_traversal_pattern, is_safe_path, safe_join, sanitize_filename};
pub use progress::{
    shared_tracker, Progress, ProgressCallback, ProgressTracker, SharedProgressTracker,
};
pub use retry::{retry_async, retry_if_async, retry_sync, RetryConfig};
pub use segment_hash::{hash_segment, hash_segment_with_progress, hash_segments_combined};
pub use segments::{
    build_ad1_segment_path, discover_ad1_segments, discover_e01_segments,
    discover_numbered_segments, extract_ad1_segment_number, get_segment_basename, is_ad1_segment,
    is_first_ad1_segment, is_numbered_segment, is_segmented_file,
};
pub use source_analysis::{
    analyze_byte_source, EntropyWindow, SourceAnalysis, SourceAnalysisOptions, SourceIndicator,
    SourceSignature,
};
pub use vfs::{
    join_path, normalize_path, DirEntry, FileAttr, MountHandle, VfsError, VirtualFileSystem,
};

// =============================================================================
// Buffer Size Constants
// =============================================================================

/// Default I/O buffer size (16MB).
pub const BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Copy buffer size for file transfers (8MB).
pub const COPY_BUFFER_SIZE: usize = 8 * 1024 * 1024;

/// Hash calculation buffer size (16MB).
pub const HASH_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Small buffer size for metadata reading (64KB).
pub const SMALL_BUFFER_SIZE: usize = 64 * 1024;

/// Streaming threshold (16MB).
pub const STREAMING_THRESHOLD: u64 = 16 * 1024 * 1024;

/// Threshold for memory-mapped I/O (64MB).
pub const MMAP_THRESHOLD: u64 = 64 * 1024 * 1024;

/// Progress update chunk size for hashing (64MB).
pub const PROGRESS_CHUNK_SIZE: u64 = 64 * 1024 * 1024;
