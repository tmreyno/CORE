// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri Commands Module
//!
//! This module organizes all Tauri commands into logical groups for better
//! maintainability. Each submodule contains related commands.
//!
//! ## Command Groups
//!
//! - `container`: AD1 container operations (V1 and V2)
//! - `lazy_loading`: Unified lazy loading for all container types
//! - `archive`: Archive tree listing and extraction (ZIP, 7z, RAR, TAR)
//! - `archive_create`: Archive creation using sevenzip-ffi (7z format)
//! - `ufed`: UFED container operations
//! - `ewf`: EWF/E01 format operations
//! - `raw`: Raw disk image operations
//! - `vfs`: Virtual filesystem for disk image mounting
//! - `hash`: Batch hashing operations
//! - `system`: System stats and monitoring
//! - `analysis`: Data viewing, hex dump, entropy analysis
//! - `database`: SQLite persistence operations
//! - `project`: Project file handling
//! - `viewer`: File viewer operations
//! - `discovery`: Path and evidence discovery utilities
//! - `export`: File export operations

pub mod analysis;
pub mod archive; // Archive inspection only (no creation)
pub mod archive_create; // Archive creation with sevenzip-ffi
pub mod container;
pub mod database;
pub mod discovery;
pub mod ewf;
pub mod ewf_export; // EWF/E01 export (write) commands using libewf-ffi
pub mod export;
pub mod hash;
pub mod l01_export; // L01 logical evidence export (pure-Rust writer)
pub mod lazy_loading;
pub mod project;
pub mod project_advanced;
pub mod project_db; // Per-project .ffxdb database commands
pub mod project_extended;
pub mod raw;
pub mod system;
pub mod ufed;
pub mod vfs;
pub mod viewer;

// =============================================================================
// Shared Types (used across multiple command modules)
// =============================================================================

/// Progress event for verification operations
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyProgress {
    pub path: String,
    pub current: u64,
    pub total: u64,
    pub percent: f64,
}

// Re-export all commands for easy registration in lib.rs
pub use analysis::*;
pub use archive::*; // Archive inspection commands only
pub use archive_create::*; // Archive creation commands
pub use container::*;
pub use database::*;
pub use discovery::*;
pub use ewf::*;
pub use ewf_export::*;
pub use export::*;
pub use hash::*;
pub use l01_export::*;
pub use lazy_loading::*;
pub use project::*;
pub use project_advanced::*;
pub use project_db::*;
pub use project_extended::*;
pub use raw::*;
pub use system::*;
pub use vfs::*;
pub use viewer::*;
