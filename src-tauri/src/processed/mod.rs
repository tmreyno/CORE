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

pub mod autopsy;
pub mod axiom;
pub mod cellebrite;
pub mod commands;
pub mod detection;
pub mod types;

pub use commands::*;
pub use detection::*;
pub use types::*;

use crate::containers::ContainerError;
use std::fs;
use std::io::Read;
use std::path::Path;

pub(crate) fn read_utf8_file_with_limit(
    path: &Path,
    max_bytes: u64,
    label: &str,
) -> Result<String, ContainerError> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > max_bytes {
        return Err(ContainerError::InvalidFormat(format!(
            "{} exceeds {} byte limit: {}",
            label,
            max_bytes,
            path.display()
        )));
    }

    let file = fs::File::open(path)?;
    let mut bytes = Vec::with_capacity(metadata.len().min(max_bytes) as usize);
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;

    if bytes.len() as u64 > max_bytes {
        return Err(ContainerError::InvalidFormat(format!(
            "{} exceeds {} byte limit while reading: {}",
            label,
            max_bytes,
            path.display()
        )));
    }

    String::from_utf8(bytes).map_err(ContainerError::from)
}
