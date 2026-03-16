// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 re-export shim — delegates to the `ffx_ad1` crate.
//!
//! Wraps `info()`, `info_fast()`, and `parse_companion_log()` to inject the
//! real `find_companion_log` from `containers::companion`.

#[allow(unused_imports)]
pub use ffx_ad1::{
    // Types
    Ad1Info, Ad1InfoV2, Ad1Stats, Ad1Vfs, ChunkVerifyResult, CompanionLogInfo,
    ContainerStatus, ItemMetadata, LogicalHeaderInfo, SearchResult, SegmentHeaderInfo,
    SharedCompanionInfo, SharedStoredHash, TreeEntry, TreeItem, VerifyEntry, VerifyStatus,
    // Functions — excluding info/info_fast which we wrap below
    export_metadata_csv, export_metadata_json, export_tree_csv, export_tree_json,
    extract, extract_with_progress,
    find_by_extension, find_by_hash, find_by_name,
    get_children, get_children_at_addr, get_children_at_addr_lazy,
    get_children_at_addr_v2, get_container_info_v2, get_container_status_v2,
    get_entry_info, get_item_info_v2, get_item_metadata_v2, get_items_metadata_v2,
    get_root_children_v2, get_segment_paths, get_stats, get_tree,
    hash_segments, hash_segments_with_progress, hash_single_segment,
    is_ad1, read_entry_chunk, read_entry_data, read_entry_data_by_addr,
    read_file_data_v2, verify, verify_against_log, verify_chunks,
    verify_item_hash_v2, verify_with_progress,
    parse_companion_log_with_finder,
};

// Re-export the vfs module for direct submodule access
pub use ffx_ad1::vfs;

// Re-export submodules consumers use directly
pub use ffx_ad1::operations;
pub use ffx_ad1::types;
pub use ffx_ad1::utils;

use crate::containers::companion::find_companion_log as find_shared_companion_log;
use ffx_errors::ContainerError;

/// Wrapper around `ffx_ad1::info` that injects the real companion log finder.
pub fn info(path: &str, include_tree: bool) -> Result<Ad1Info, ContainerError> {
    let mut result = ffx_ad1::info(path, include_tree)?;
    result.companion_log = parse_companion_log(path);
    Ok(result)
}

/// Wrapper around `ffx_ad1::info_fast` that injects the real companion log finder.
pub fn info_fast(path: &str) -> Result<Ad1Info, ContainerError> {
    let mut result = ffx_ad1::info_fast(path)?;
    result.companion_log = parse_companion_log(path);
    Ok(result)
}

/// Parse companion log by delegating to the shared `find_companion_log` finder.
pub fn parse_companion_log(path: &str) -> Option<CompanionLogInfo> {
    parse_companion_log_with_finder(path, Some(|p: &str| {
        let shared = find_shared_companion_log(p)?;
        Some(SharedCompanionInfo {
            case_number: shared.case_number,
            evidence_number: shared.evidence_number,
            examiner: shared.examiner,
            notes: shared.notes,
            acquisition_started: shared.acquisition_started,
            unique_description: shared.unique_description,
            created_by: shared.created_by,
            stored_hashes: shared
                .stored_hashes
                .into_iter()
                .map(|h| SharedStoredHash {
                    algorithm: h.algorithm,
                    hash: h.hash,
                })
                .collect(),
        })
    }))
}
