// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive operations module - tree listing, extraction, and tools.
//!
//! Organized into logical submodules:
//! - `metadata`: Quick metadata and full tree listing
//! - `extraction`: Entry extraction and chunk reading
//! - `nested`: Nested container support (archives within archives)
//! - `tools`: Archive tools (test, repair, validate, extract split)

pub mod extraction;
pub mod metadata;
pub mod nested;
pub mod tools;

// Re-export all commands for lib.rs registration
pub use extraction::{archive_extract_entry, archive_read_entry_chunk};
pub use metadata::{archive_get_metadata, archive_get_tree};
pub use nested::{
    nested_archive_read_entry_chunk, nested_container_clear_cache, nested_container_get_info,
    nested_container_get_tree,
};
pub use tools::{
    clear_last_archive_error, decrypt_data_native, encrypt_data_native, extract_split_7z_archive,
    get_last_archive_error, repair_7z_archive, test_7z_archive, validate_7z_archive,
};

// Re-export types
pub use metadata::{ArchiveQuickMetadata, ArchiveTreeEntry};
pub use nested::{NestedContainerEntry, NestedContainerInfo};
pub use tools::ArchiveValidationResult;
