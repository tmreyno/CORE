// =============================================================================
// ffx-containers — Unified Container Abstraction Layer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified Container Abstraction Layer for CORE-FFX
//!
//! This crate provides the orchestration layer that dispatches operations
//! (info, verify, extract, search, scan) across all supported forensic
//! container formats (AD1, E01, RAW, UFED, Archive).
//!
//! # Architecture
//!
//! Format-specific crates are re-exported as crate-level aliases so that
//! internal modules can use `crate::ad1::*`, `crate::ewf::*`, etc. without
//! modification from the original source layout.

// Re-export format crates as crate-level aliases for internal use.
// This allows source files to use `crate::ad1::*` etc. unchanged.
pub use ffx_ad1 as ad1;
pub use ffx_archive as archive;
pub use ffx_common as common;
pub use ffx_ewf as ewf;
pub use ffx_formats as formats;
pub use ffx_raw as raw;
pub use ffx_ufed as ufed;

// Crate modules
pub mod companion;
pub mod case_documents;
#[allow(deprecated)]
mod impls;
mod operations;
mod scanning;
mod segments;
mod traits;
mod types;

// Re-exports
pub use case_documents::*;
#[allow(deprecated)]
pub use impls::*;
pub use operations::*;
pub use scanning::*;
pub use traits::*;
pub use types::*;
