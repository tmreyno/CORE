// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Schema initialization and migration logic for the project database.
//!
//! Split into two sub-files for maintainability:
//! - `schema_tables.rs` — `init_schema()`: creates all tables, indexes, FTS5 (~720 lines of SQL)
//! - `schema_migrations.rs` — `check_migrations()`: version-gated ALTER/CREATE migrations (v1→v10)
