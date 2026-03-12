// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for file deduplication analysis.
//!
//! Provides commands for:
//! - Running deduplication scans across all indexed containers
//! - Exporting dedup results as CSV/JSON reports

use std::collections::HashMap;

use tracing::info;

use crate::dedup;
use crate::dedup::types::{DedupOptions, DedupResults};

// =============================================================================
// Dedup Commands
// =============================================================================

/// Run deduplication analysis on all indexed containers.
///
/// Reads file metadata from the Tantivy search index, groups by size and name,
/// and optionally enriches with stored hashes from the project database.
#[tauri::command]
pub async fn dedup_analyze(
    window: tauri::Window,
    options: DedupOptions,
) -> Result<DedupResults, String> {
    let label = window.label().to_string();

    tauri::async_runtime::spawn_blocking(move || {
        dedup::analyze_duplicates(&label, &options)
    })
    .await
    .map_err(|e| format!("Dedup task failed: {}", e))?
}

/// Enrich dedup results with stored hashes from .ffxdb.
///
/// Takes a hash map of (container_path:entry_path → hash_value) and upgrades
/// match types from SizeAndName to ExactHash where hashes confirm identity.
#[tauri::command]
pub async fn dedup_enrich_hashes(
    mut results: DedupResults,
    hash_map: HashMap<String, String>,
) -> Result<DedupResults, String> {
    dedup::enrich_with_hashes(&mut results, &hash_map);
    Ok(results)
}

/// Export dedup results as a CSV string.
#[tauri::command]
pub async fn dedup_export_csv(results: DedupResults) -> Result<String, String> {
    let mut csv = String::new();
    csv.push_str("Group ID,Representative Name,Match Type,File Size,Wasted Bytes,Cross Container,Container Path,Entry Path,Filename,Hash\n");

    for group in &results.groups {
        let match_type = match group.match_type {
            dedup::types::DuplicateMatchType::ExactHash => "Exact Hash",
            dedup::types::DuplicateMatchType::SizeAndName => "Size + Name",
            dedup::types::DuplicateMatchType::SizeOnly => "Size Only",
        };

        for file in &group.files {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",{},{},{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
                group.id.replace('"', "\"\""),
                group.representative_name.replace('"', "\"\""),
                match_type,
                group.file_size,
                group.wasted_bytes,
                group.cross_container,
                file.container_path.replace('"', "\"\""),
                file.entry_path.replace('"', "\"\""),
                file.filename.replace('"', "\"\""),
                file.hash.as_deref().unwrap_or(""),
            ));
        }
    }

    info!(
        "Dedup CSV export: {} groups, {} total lines",
        results.groups.len(),
        csv.lines().count()
    );

    Ok(csv)
}
