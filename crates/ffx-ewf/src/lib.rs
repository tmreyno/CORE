// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF (Expert Witness Format) - E01/L01/Ex01/Lx01 forensic image parser
//!
//! Parsing, verification, and extraction for EnCase Expert Witness Format.

mod cache;
mod handle;
pub mod l01_reader;
mod operations;
pub mod parser;
mod types;
pub mod vfs;

// Re-export public types
pub use types::{
    ChunkVerifyResult, EwfInfo, EwfSearchResult, EwfStats, HeaderInfo, StoredImageHash,
    VerifyResult, VolumeSection,
};

// Re-export parser types for hex viewer
pub use parser::{
    is_ewf_file, is_l01_file, parse_ewf_file, EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry,
    EwfHashInfo, EwfSectionHeader, EwfVariant, EwfVolumeInfo,
};

// Re-export the handle for advanced usage
pub use handle::EwfHandle;

// Re-export VFS
pub use vfs::EwfVfs;

// Re-export L01 reader
pub use l01_reader::{parse_l01_file_tree, L01Entry, L01FileTree};

// Re-export public functions
pub use operations::{
    export_metadata_csv, export_metadata_json, extract, extract_with_progress, get_segment_paths,
    get_stats, hash_single_segment, info, info_fast, is_e01, is_ewf, verify, verify_chunks,
    verify_with_progress,
};
