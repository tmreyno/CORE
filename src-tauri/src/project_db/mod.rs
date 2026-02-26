// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Per-project SQLite database (.ffxdb) for forensic project persistence.
//!
//! This module is split into domain-specific submodules for maintainability.
//! Each submodule adds methods to `ProjectDatabase` via separate `impl` blocks.

// --- Type definitions & constants ---
mod types;
pub use types::*;

// --- Core struct & constructor ---
mod database;
pub use database::ProjectDatabase;

// --- Schema initialization & migrations ---
mod schema;

// --- Domain operation modules (add impl methods to ProjectDatabase) ---
mod activity;
mod evidence;
mod bookmarks;
mod ui_state;
mod search;
mod processed;
mod forensic;
mod forms;
mod collections;
mod workflow;
mod fts;
mod utilities;
mod migration;

#[cfg(test)]
mod tests;
