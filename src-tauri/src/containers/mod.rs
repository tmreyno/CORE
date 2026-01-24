// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Container abstraction layer for forensic image formats
//! 
//! This module provides a unified interface for working with various forensic
//! container formats including AD1, E01, L01, Raw, Archive, and UFED.
//!
//! # Evidence Lifecycle
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │  1. Discovery    │  scan_directory() finds container files          │
//! │  2. Detection    │  detect_container() identifies format by magic   │
//! │  3. Parsing      │  info() extracts metadata and file tree          │
//! │  4. Verification │  verify() computes/compares hashes               │
//! │  5. Extraction   │  extract() exports files to output directory     │
//! └──────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Adding New Formats
//!
//! See `traits.rs` for the `EvidenceContainer` trait that new format parsers
//! should implement for full integration.

mod types;
mod traits;
mod operations;
mod scanning;
mod segments;
mod impls;
pub(crate) mod companion;
pub mod case_documents;
pub mod unified;

// Re-export all public types
pub use types::*;

// Re-export traits for extension developers
pub use traits::{
    EvidenceContainer, SegmentedContainer, TreeContainer, HashableContainer,
    MountableContainer, FormatInfo, SegmentInfo, HashResult, VerifyResult, VerifyStatus,
    ContainerMetadata, CaseMetadata, StoredHashInfo, TreeEntryInfo,
    ContainerError, LifecycleStage,
};

// Re-export parser implementations (DEPRECATED - use operations::* functions instead)
#[allow(deprecated)]
pub use impls::{Ad1Parser, EwfParser, RawParser, UfedParser, ArchiveParser, get_parsers, detect_parser};

// Re-export main operations (ACTIVE - preferred API for container operations)
pub use operations::{
    info, info_fast, verify, verify_with_progress, 
    extract, extract_with_progress, search,
    get_stats, get_segment_paths, export_metadata_json, export_metadata_csv,
    get_stored_hashes_only,
    ContainerStats,
};

// Re-export scanning functions
pub use scanning::{scan_directory, scan_directory_recursive, scan_directory_streaming};

// Re-export case document discovery
pub use case_documents::{
    find_case_documents, find_case_document_folders, find_coc_forms,
    CaseDocument, CaseDocumentType, CaseDocumentSearchConfig,
};

// Re-export unified handler API
pub use unified::{
    ContainerType, UnifiedContainerHandler,
    get_handler, get_handler_for_path,
    get_summary, get_entry_count, get_root_children, get_children, get_children_typed,
};
