// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Normalized artifact extraction commands.

use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{extract_normalized_artifact, ArtifactExtractionOptions, NormalizedArtifact};

/// Extract a normalized artifact record from a local file or supported
/// container entry without forcing a full-file read.
#[tauri::command]
pub async fn artifact_extract_source(
    source: HashSourceInput,
    options: Option<ArtifactExtractionOptions>,
) -> Result<NormalizedArtifact, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        extract_normalized_artifact(byte_source.as_ref(), options.unwrap_or_default())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Internal artifact extraction error: {e}"))?
}
