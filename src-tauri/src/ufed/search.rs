// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED search operations
//!
//! Provides file search within UFED containers by name pattern
//! or file extension.

use tracing::{debug, instrument};

use crate::containers::ContainerError;
use super::tree::{get_tree, UfedTreeEntry};

/// Search result from UFED container
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedSearchResult {
    /// The matching entry
    pub entry: UfedTreeEntry,
    /// How the match was found (name, extension, etc.)
    pub match_type: String,
    /// Depth in the tree
    pub depth: u32,
}

/// Search for files by name pattern in a UFED container
#[instrument]
pub fn search_by_name(
    path: &str,
    pattern: &str,
) -> Result<Vec<UfedSearchResult>, ContainerError> {
    debug!(path = %path, pattern = %pattern, "Searching UFED by name");

    let tree = get_tree(path)?;
    let pattern_lower = pattern.to_lowercase();

    let results: Vec<UfedSearchResult> = tree
        .into_iter()
        .filter(|entry| entry.name.to_lowercase().contains(&pattern_lower))
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            UfedSearchResult {
                entry,
                match_type: "name".to_string(),
                depth,
            }
        })
        .collect();

    Ok(results)
}

/// Search for files by extension in a UFED container
#[instrument]
pub fn search_by_extension(
    path: &str,
    extension: &str,
) -> Result<Vec<UfedSearchResult>, ContainerError> {
    debug!(path = %path, extension = %extension, "Searching UFED by extension");

    let tree = get_tree(path)?;
    let ext_lower = extension
        .to_lowercase()
        .trim_start_matches('.')
        .to_string();

    let results: Vec<UfedSearchResult> = tree
        .into_iter()
        .filter(|entry| {
            if entry.is_dir {
                return false;
            }
            if let Some(file_ext) = entry.name.rsplit('.').next() {
                file_ext.to_lowercase() == ext_lower
            } else {
                false
            }
        })
        .map(|entry| {
            let depth = entry.path.matches('/').count() as u32;
            UfedSearchResult {
                entry,
                match_type: "extension".to_string(),
                depth,
            }
        })
        .collect();

    Ok(results)
}
