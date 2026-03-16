// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Re-export shim: all functionality now lives in the `ffx-common` crate.
// This module re-exports everything so that existing `crate::common::X`
// import paths continue to work without mass refactoring.
//

// Re-export all modules so crate::common::hash, crate::common::vfs, etc. still resolve
pub use ffx_common::audit;
pub use ffx_common::binary;
pub use ffx_common::container_detect;
pub use ffx_common::datetime;
pub use ffx_common::filesystem;
pub use ffx_common::hash;
pub use ffx_common::hash_cache;
pub use ffx_common::health;
pub use ffx_common::hex;
pub use ffx_common::io_adaptive;
pub use ffx_common::io_pool;
pub use ffx_common::lazy_loading;
pub use ffx_common::magic;
pub use ffx_common::metrics;
pub use ffx_common::path_security;
pub use ffx_common::progress;
pub use ffx_common::retry;
pub use ffx_common::segment_hash;
pub use ffx_common::segments;
pub use ffx_common::vfs;

// Re-export all top-level items (convenience imports, constants)
pub use ffx_common::*;
