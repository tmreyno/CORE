// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive container support — thin shim re-exporting the `ffx-archive` crate.
//!
//! All archive parsing, detection, extraction, verification, and VFS logic lives
//! in the `ffx-archive` workspace crate. This module re-exports the full public
//! API and provides the UFED detection bridge so `info()` can detect UFED
//! containers (UFDR/UFDX/UFD) inside ZIP archives.

pub use ffx_archive::*;

/// Register the UFED-in-ZIP detector so that `archive::info()` can detect
/// UFED containers inside ZIP archives without a direct dependency on the
/// `ufed` module (which would be circular since `ufed` depends on `archive`
/// via the `ArchiveOps` trait).
///
/// Must be called once during app startup (from `common_setup()` in `lib.rs`).
pub fn init_archive_ufed_bridge() {
    ffx_archive::register_ufed_detector(Box::new(|path: &str| {
        crate::ufed::detect_in_zip(path).map_err(|e| e.to_string())
    }));
}
