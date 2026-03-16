// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Re-export shim — implementation moved to the `ffx-raw` crate.

// Re-export the vfs submodule so `crate::raw::vfs::*` paths keep working
pub mod vfs {
    pub use ffx_raw::vfs::*;
}

// Re-export all public items from ffx-raw
pub use ffx_raw::*;
