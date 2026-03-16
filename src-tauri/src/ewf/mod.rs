// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF module re-export shim.
//!
//! Core EWF logic lives in the `ffx-ewf` crate.
//! This module re-exports everything and adds viewer-coupled parser functions.

// Local parser module (contains viewer-coupled metadata function)
pub mod parser;

// Re-export VFS as a submodule (callers use `ewf::vfs::EwfVfs`)
pub mod vfs {
    pub use ffx_ewf::vfs::*;
}

// Re-export L01 reader as a submodule
pub mod l01_reader {
    pub use ffx_ewf::l01_reader::*;
}

// Re-export all public types from the crate root
pub use ffx_ewf::{
    ChunkVerifyResult, EwfInfo, EwfSearchResult, EwfStats, HeaderInfo,
    StoredImageHash, VerifyResult, VolumeSection,
};

// Re-export parser types (also available via parser::)
pub use parser::{
    ewf_detailed_info_to_metadata, is_ewf_file, is_l01_file, parse_ewf_file,
    EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry, EwfHashInfo, EwfSectionHeader,
    EwfVariant, EwfVolumeInfo,
};

// Re-export handle
pub use ffx_ewf::EwfHandle;

// Re-export VFS type at module level
pub use ffx_ewf::EwfVfs;

// Re-export L01 reader types at module level
pub use ffx_ewf::{parse_l01_file_tree, L01Entry, L01FileTree};

// Re-export public functions from operations
pub use ffx_ewf::{
    export_metadata_csv, export_metadata_json, extract, extract_with_progress,
    get_segment_paths, get_stats, hash_single_segment, info, info_fast, is_e01,
    is_ewf, verify, verify_chunks, verify_with_progress,
};
