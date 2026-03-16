// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF parser re-export shim.
//!
//! Core parsing logic lives in the `ffx-ewf` crate.
//! This module re-exports everything from the crate and adds the
//! viewer-coupled `ewf_detailed_info_to_metadata` function which
//! depends on `crate::viewer` types.

mod metadata;

// Re-export types submodule so local metadata.rs can use `super::types::{...}`
pub mod types {
    pub use ffx_ewf::parser::types::*;
}

// Re-export all public items from the crate's parser
pub use ffx_ewf::parser::{
    parse_ewf_file, is_ewf_file, is_l01_file,
    EwfCaseInfo, EwfDetailedInfo, EwfErrorEntry, EwfHashInfo,
    EwfSectionHeader, EwfVariant, EwfVolumeInfo,
};

// Re-export the local viewer function
pub use self::metadata::ewf_detailed_info_to_metadata;
