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

pub mod case_documents;
pub(crate) mod companion;
mod impls;
mod operations;
mod scanning;
mod segments;
mod traits;
mod types;

// Re-export all public types
pub use types::*;

// Re-export traits for extension developers
pub use traits::{
    CaseMetadata, ContainerError, ContainerMetadata, EvidenceContainer, FormatInfo, HashResult,
    HashableContainer, LifecycleStage, MountableContainer, SegmentInfo, SegmentedContainer,
    StoredHashInfo, TreeContainer, TreeEntryInfo, VerifyResult, VerifyStatus,
};

// Re-export parser implementations (DEPRECATED - use operations::* functions instead)
#[allow(deprecated)]
pub use impls::{
    detect_parser, get_parsers, Ad1Parser, ArchiveParser, EwfParser, RawParser, UfedParser,
};

// Re-export main operations (ACTIVE - preferred API for container operations)
pub use operations::{
    export_metadata_csv, export_metadata_json, extract, extract_with_progress, get_segment_paths,
    get_stats, get_stored_hashes_only, info, info_fast, search, verify, verify_with_progress,
    ContainerStats,
};

// Re-export scanning functions
pub use scanning::{scan_directory, scan_directory_recursive, scan_directory_streaming};

// Re-export case document discovery
pub use case_documents::{
    find_case_document_folders, find_case_documents, find_coc_forms, CaseDocument,
    CaseDocumentSearchConfig, CaseDocumentType,
};
