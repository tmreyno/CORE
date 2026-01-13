// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Processed Database Support
//!
//! This module handles detection and parsing of processed forensic databases
//! from tools like Magnet AXIOM, Cellebrite Physical Analyzer, X-Ways, etc.
//!
//! These are DISTINCT from raw evidence containers (E01, AD1, L01) - they contain
//! parsed/processed results from forensic examinations.

pub mod types;
pub mod detection;
pub mod axiom;
pub mod commands;

pub use types::*;
pub use detection::*;
pub use commands::*;
