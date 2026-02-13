// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Common Utilities for Forensic Container Parsers
//!
//! This module provides shared functionality used across all container format
//! parsers (AD1, E01, RAW, UFED, Archives). These utilities are designed with
//! forensic requirements in mind: read-only operations, audit logging, and
//! secure path handling.
//!
//! # Submodules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`hash`] | Cryptographic hashing (MD5, SHA-*, BLAKE3, XXH3) |
//! | [`binary`] | Little-endian binary reading utilities |
//! | [`segments`] | Multi-segment file discovery (.E01/.E02, .ad1/.ad2) |
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
//! The module provides tuned constants for high-throughput I/O:
//!
//! - `BUFFER_SIZE`: 16MB buffer optimized for modern NVMe SSDs
//! - `MMAP_THRESHOLD`: 64MB threshold for memory-mapped I/O
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::common::{
//!     hash::{compute_hash, HashAlgorithm},
//!     segments::discover_e01_segments,
//!     path_security::safe_join,
//! };
//!
//! // Compute SHA-256 hash
//! let hash = compute_hash(data, HashAlgorithm::Sha256);
//!
//! // Discover all E01 segments
//! let segments = discover_e01_segments("/path/to/image.E01")?;
//!
//! // Safely join paths (prevents traversal attacks)
//! let safe_path = safe_join("/output", "user/../../../etc/passwd")?;
//! ```

pub mod hash;
pub mod hash_cache;
pub mod binary;
pub mod segments;
pub mod io_pool;
pub mod io_adaptive;
pub mod hex;
pub mod magic;
pub mod path_security;
pub mod audit;
pub mod vfs;
pub mod progress;
pub mod filesystem;
pub mod lazy_loading;
pub mod datetime;
pub mod container_detect;
pub mod segment_hash;
pub mod retry;
pub mod metrics;
pub mod tracing_setup;
pub mod health;

// Re-exports for convenience
pub use hash::{HashAlgorithm, StreamingHasher, compute_hash, hash_file_with_progress};
pub use segment_hash::{hash_segment, hash_segment_with_progress, hash_segments_combined};
pub use hash::{compare_hashes, HashMatchResult, HashVerificationResult, verify_hash};
pub use hash_cache::{HashCache, HashCacheKey, HashCacheEntry, HashCacheStats, GLOBAL_HASH_CACHE, get_cached_hash, cache_hash, get_or_compute_hash};
pub use binary::{read_u8, read_u16_le, read_u32_le, read_u64_le, read_u32_be};
pub use io_adaptive::{AdaptiveBuffer, Operation as IoOperation, AdaptiveStats};
pub use segments::{
    discover_numbered_segments, discover_e01_segments, get_segment_basename, is_numbered_segment,
    is_ad1_segment, is_first_ad1_segment, extract_ad1_segment_number, build_ad1_segment_path,
    discover_ad1_segments, is_segmented_file,
};
pub use io_pool::{FileIoPool, DEFAULT_MAX_OPEN_FILES};
pub use hex::{format_hex_inline, format_hex_string, format_size, format_size_compact, escape_csv, csv_row, csv_header};
pub use magic::{detect_file_type, FileType, FileCategory, is_image, is_archive, is_executable};
pub use path_security::{safe_join, sanitize_filename, is_safe_path, contains_traversal_pattern};
pub use audit::{log_evidence_access, log_hash_verification, log_container_opened, log_report_generation, log_security_event};
pub use vfs::{VirtualFileSystem, VfsError, FileAttr, DirEntry, MountHandle, normalize_path, join_path};
pub use progress::{Progress, ProgressCallback, ProgressTracker, SharedProgressTracker, shared_tracker};
pub use lazy_loading::{LazyLoadConfig, LazyTreeEntry, LazyLoadResult, ContainerSummary, LazyLoadable};
pub use datetime::{now_rfc3339, now_local_display, format_duration, format_display, parse_rfc3339};
pub use container_detect::{ContainerType, detect_container_type, is_forensic_container, is_container, is_segmented_container};
pub use retry::{RetryConfig, retry_async, retry_sync, retry_if_async};

// =============================================================================
// Buffer Size Constants
// =============================================================================

/// Default I/O buffer size (16MB).
///
/// This size is optimized for high throughput on modern NVMe SSDs and HDDs.
/// Larger buffers reduce syscall overhead and enable better sequential
/// read performance for forensic image verification.
pub const BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Copy buffer size for file transfers (8MB).
///
/// Slightly smaller than BUFFER_SIZE to leave room for OS caching and
/// provide good balance between memory usage and throughput.
pub const COPY_BUFFER_SIZE: usize = 8 * 1024 * 1024;

/// Hash calculation buffer size (16MB).
///
/// Matches BUFFER_SIZE for optimal hashing throughput. Large buffers
/// help SIMD-accelerated hash algorithms (BLAKE3, XXH3) perform better.
pub const HASH_BUFFER_SIZE: usize = 16 * 1024 * 1024;

/// Small buffer size for metadata reading (64KB).
///
/// Used for reading headers, chunk tables, and other small structures.
/// Sized to fit common filesystem block sizes and L2 cache.
pub const SMALL_BUFFER_SIZE: usize = 64 * 1024;

/// Streaming threshold (16MB).
///
/// Files smaller than this are read entirely into memory;
/// files larger use streaming/buffered I/O.
pub const STREAMING_THRESHOLD: u64 = 16 * 1024 * 1024;

/// Threshold for memory-mapped I/O (64MB).
///
/// Files larger than this threshold may use memory-mapped I/O for
/// improved random access performance. Memory mapping is particularly
/// beneficial for container formats that require seeking (E01 chunk tables).
pub const MMAP_THRESHOLD: u64 = 64 * 1024 * 1024;

/// Progress update chunk size for hashing (64MB).
///
/// Controls how often progress callbacks are invoked during long operations.
/// Balance between responsiveness and overhead.
pub const PROGRESS_CHUNK_SIZE: u64 = 64 * 1024 * 1024;

