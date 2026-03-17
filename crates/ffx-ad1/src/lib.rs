// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # ffx-ad1 - AD1 (AccessData Logical Image) Parser
//!
//! Pure-Rust parser for AccessData's AD1 logical evidence container format,
//! commonly used in FTK (Forensic Toolkit).

mod info_v2;
pub mod operations;
mod operations_v2;
mod parser;
mod reader_v2;
pub mod types;
pub mod utils;
pub mod vfs;

// Re-export public types
pub use types::{
    Ad1Info, Ad1Stats, ChunkVerifyResult, CompanionLogInfo, ItemMetadata, LogicalHeaderInfo,
    SearchResult, SegmentHeaderInfo, TreeEntry, VerifyEntry, VerifyStatus,
};

// Re-export public functions from operations
pub use operations::{
    export_metadata_csv, export_metadata_json, export_tree_csv, export_tree_json, extract,
    extract_with_progress, find_by_extension, find_by_hash, find_by_name, get_children,
    get_children_at_addr, get_children_at_addr_lazy, get_entry_info, get_segment_paths, get_stats,
    get_tree, hash_segments, hash_segments_with_progress, hash_single_segment, info, info_fast,
    is_ad1, read_entry_chunk, read_entry_data, read_entry_data_by_addr, verify, verify_against_log,
    verify_chunks, verify_with_progress,
};

// Re-export V2 operations
pub use operations_v2::{
    get_children_at_addr as get_children_at_addr_v2,
    get_container_status as get_container_status_v2, get_item_info as get_item_info_v2,
    get_item_metadata as get_item_metadata_v2, get_items_metadata as get_items_metadata_v2,
    get_root_children as get_root_children_v2, read_file_data as read_file_data_v2,
    verify_item_hash as verify_item_hash_v2, ContainerStatus,
};

// Re-export V2 info
pub use info_v2::{get_container_info as get_container_info_v2, Ad1InfoV2, TreeItem};

// Re-export VFS
pub use vfs::Ad1Vfs;

// Re-export companion log helper types
pub use utils::parsing::{parse_companion_log_with_finder, SharedCompanionInfo, SharedStoredHash};
