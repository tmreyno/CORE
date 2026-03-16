// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # ffx-aff4 — Pure-Rust AFF4 forensic container reader and writer
//!
//! AFF4 (Advanced Forensic Framework 4) is an open forensic container format
//! built on ZIP64. Each `.aff4` file is a ZIP archive containing compressed
//! evidence data in bevies, RDF metadata (Turtle), and integrity hashes.
//!
//! This crate supports:
//! - **AFF4-Standard** (v1.0): Physical disk imaging with ImageStream + Map
//! - **AFF4-L** (v1.1): Logical file collection with per-file hashes
//! - **Reading**: Container browsing, metadata extraction, data verification
//! - **Writing**: Creating AFF4 containers with compression and hashing
//!
//! ## Architecture
//!
//! ```text
//! ┌───────────────────────────────────────────┐
//! │           ZIP64 Container (.aff4)         │
//! │  ┌─────────────────────────────────────┐  │
//! │  │ container.description  (Volume URN) │  │
//! │  │ version.txt            (1.0 / 1.1)  │  │
//! │  │ <urn>/00000000         (Bevy data)  │  │
//! │  │ <urn>/00000000.index   (Bevy index) │  │
//! │  │ <urn>/00000000.sha256  (Block hash) │  │
//! │  │ <map>/map              (Map stream) │  │
//! │  │ <map>/idx              (Map index)  │  │
//! │  │ information.turtle     (RDF meta)   │  │
//! │  └─────────────────────────────────────┘  │
//! └───────────────────────────────────────────┘
//! ```

pub mod bevy;
pub mod compression;
pub mod error;
pub mod hashing;
pub(crate) mod helpers;
pub mod logical;
pub mod map;
pub mod rdf;
pub mod reader;
pub mod types;
pub mod uri;
pub mod writer;

// Re-exports for convenience
pub use error::Aff4Error;
pub use logical::Aff4LogicalWriter;
pub use reader::Aff4Reader;
pub use types::*;
pub use writer::Aff4Writer;
