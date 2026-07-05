// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for project merge and relocate operations.

use crate::project::merge;

/// Analyze multiple .cffx files and return summaries for the merge wizard.
#[tauri::command]
pub async fn project_merge_analyze(cffx_paths: Vec<String>) -> Vec<merge::ProjectMergeSummary> {
    tauri::async_runtime::spawn_blocking(move || merge::analyze_projects(&cffx_paths))
        .await
        .unwrap_or_default()
}

/// Execute a full project merge pipeline.
///
/// - `cffx_paths`: list of source .cffx file paths to merge
/// - `output_path`: where to save the merged .cffx
/// - `merged_name`: name for the merged project
/// - `new_root`: optional new root directory (for relocation)
/// - `owner_assignments`: optional owner/examiner assignments for each source project
/// - `exclusions`: optional exclusion configuration for selective merging
///   (skip entire categories, individual evidence files, COC items, collections, or form submissions)
#[tauri::command]
pub async fn project_merge_execute(
    cffx_paths: Vec<String>,
    output_path: String,
    merged_name: String,
    new_root: Option<String>,
    owner_assignments: Option<Vec<merge::MergeSourceAssignment>>,
    exclusions: Option<merge::MergeExclusions>,
) -> merge::MergeResult {
    tauri::async_runtime::spawn_blocking(move || {
        merge::execute_merge(
            &cffx_paths,
            &output_path,
            &merged_name,
            new_root.as_deref(),
            owner_assignments.as_deref(),
            exclusions.as_ref(),
        )
    })
    .await
    .unwrap_or_else(|e| merge::MergeResult {
        success: false,
        cffx_path: None,
        ffxdb_path: None,
        error: Some(format!("Task failed: {}", e)),
        stats: None,
        sources: None,
    })
}
