// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 (AccessData Logical Image) Parser
//!
//! ## Section Brief
//! Public module for AD1 container format (FTK Forensic Toolkit):
//!
//! ### Submodules
//! - `types` - Type definitions and constants
//! - `parser` - Low-level parsing and session management
//! - `operations` - Public API functions
//! - `utils` - Utility functions
//!
//! ### Re-exported Types
//! - `Ad1Info`, `SegmentHeaderInfo`, `LogicalHeaderInfo`
//! - `TreeEntry`, `VerifyEntry`
//!
//! ### Re-exported Functions
//! - `info()`, `info_fast()` - Container information
//! - `verify()`, `extract()` - Verification and extraction
//! - `get_tree()`, `get_children()` - Tree navigation
//! - `read_entry_data()` - Data reading
//!
//! ---
//!
//! This module provides parsing and verification for AccessData's AD1 logical
//! evidence container format, commonly used in FTK (Forensic Toolkit).
//!
//! ## AD1 Format Structure
//!
//! AD1 files are **segmented logical containers** with zlib-compressed content:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │ Segment Header (64 bytes)                                    │
//! │  - Signature: "ADSEGMENTEDFILE" (15 bytes)                  │
//! │  - Segment Index (u32)                                       │
//! │  - Segment Number (u32) - total segment count               │
//! │  - Fragments Size (u32)                                      │
//! │  - Header Size (u32)                                         │
//! ├──────────────────────────────────────────────────────────────┤
//! │ Logical Header (within 512 bytes after segment header)       │
//! │  - Signature "AD\0\0" (4 bytes)                             │
//! │  - Image Version (u32)                                       │
//! │  - Zlib Chunk Size (u32)                                     │
//! │  - Logical Metadata Address (u64)                            │
//! │  - First Item Address (u64)                                  │
//! │  - Data Source Name                                          │
//! ├──────────────────────────────────────────────────────────────┤
//! │ Item Chain (linked list structure)                           │
//! │  - Each item: next_addr, child_addr, metadata_addr           │
//! │  - Item type: 0x05 = folder, others = files                 │
//! │  - Zlib-compressed data at zlib_metadata_addr                │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Multi-Segment Support
//!
//! AD1 containers can span multiple files (.ad1, .ad2, .ad3, etc.):
//! - First segment contains all headers and metadata structure
//! - Subsequent segments contain additional compressed data blocks
//! - Segment number in header indicates total segment count
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Get container info with file tree
//! let info = ad1::info("/path/to/evidence.ad1", true)?;
//!
//! // Fast info (headers only, no tree parsing)
//! let info_fast = ad1::info_fast("/path/to/evidence.ad1")?;
//!
//! // Verify file hashes
//! let results = ad1::verify("/path/to/evidence.ad1", "sha1")?;
//!
//! // Extract to output directory
//! ad1::extract("/path/to/evidence.ad1", "/output/dir")?;
//! ```

mod types;
mod parser;
mod operations;
mod operations_v2;
mod reader_v2;
mod hash_v2;
mod extract_v2;
mod info_v2;
mod utils;
pub mod vfs;

#[cfg(test)]
mod test_v2_comprehensive;

#[cfg(test)]
mod enumerate_all;

// Re-export public types
#[allow(unused_imports)]
pub use types::{
    Ad1Info, SegmentHeaderInfo, LogicalHeaderInfo, 
    TreeEntry, VerifyEntry, VerifyStatus,
    Ad1Stats, SearchResult, ChunkVerifyResult,
    ItemMetadata,
};

// Re-export public functions
#[allow(unused_imports)]
pub use operations::{
    // Container info
    info, info_fast, is_ad1, get_stats, get_segment_paths,
    // Verification & extraction
    verify, verify_with_progress, verify_chunks, verify_against_log,
    extract, extract_with_progress,
    hash_segments, hash_segments_with_progress, hash_single_segment,
    // Tree navigation
    get_tree, get_children, get_children_at_addr, get_children_at_addr_lazy, get_entry_info,
    // Data reading
    read_entry_data, read_entry_data_by_addr, read_entry_chunk,
    // Search functions
    find_by_name, find_by_extension, find_by_hash,
    // Export functions
    export_tree_json, export_tree_csv,
    export_metadata_json, export_metadata_csv,
};

// Re-export V2 operations (improved from libad1)
#[allow(unused_imports)]
pub use operations_v2::{
    get_root_children as get_root_children_v2,
    get_children_at_addr as get_children_at_addr_v2,
    read_file_data as read_file_data_v2,
    get_item_info as get_item_info_v2,
    verify_item_hash as verify_item_hash_v2,
    get_item_metadata as get_item_metadata_v2,
    get_items_metadata as get_items_metadata_v2,
    get_container_status as get_container_status_v2,
    ContainerStatus,
};

#[allow(unused_imports)]
pub use reader_v2::{SessionV2, ItemHeader, MetadataEntry};

// Re-export V2 hash verification
#[allow(unused_imports)]
pub use hash_v2::{
    HashResult, HashType,
    md5_hash, sha1_hash,
    check_md5, check_sha1,
    verify_all_items, verify_item_by_addr,
    ItemVerifyResult,
};

// Re-export V2 extraction
#[allow(unused_imports)]
pub use extract_v2::{
    ExtractOptions, ExtractionResult,
    extract_all as extract_all_v2,
    extract_item_by_addr as extract_item_by_addr_v2,
};

// Re-export V2 info
#[allow(unused_imports)]
pub use info_v2::{
    Ad1InfoV2, TreeItem,
    get_container_info as get_container_info_v2,
    tree_to_string, format_container_info,
    format_segment_header, format_logical_header, format_item_header,
};

// Re-export VFS
pub use vfs::Ad1Vfs;
