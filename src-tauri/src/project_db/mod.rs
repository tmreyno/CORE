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
mod schema_migrations;
mod schema_tables;

// --- Domain operation modules (add impl methods to ProjectDatabase) ---
mod activity;
mod bookmarks;
mod collections;
mod evidence;
mod forensic;
mod forms;
mod fts;
mod migration;
mod processed;
mod search;
mod ui_state;
mod utilities;
mod workflow;

#[cfg(test)]
mod tests;
