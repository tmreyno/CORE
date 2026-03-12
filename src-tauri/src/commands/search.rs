// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for the full-text search engine.
//!
//! Provides commands for:
//! - Opening/closing search indexes (linked to project lifecycle)
//! - Indexing individual containers or all evidence
//! - Executing searches with filters
//! - Managing the index (rebuild, stats, delete)

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::Emitter;
use tracing::{info, warn};

use crate::search;
use crate::search::indexer::{IndexPhase, IndexProgress, IndexingState};
use crate::search::query::{SearchOptions, SearchResults};
use crate::search::IndexStats;

// =============================================================================
// Index Lifecycle Commands
// =============================================================================

/// Open or create a search index for the current project.
///
/// Called when a project is opened. The index is stored at `<ffxdb_path>-index/`.
#[tauri::command]
pub async fn search_open_index(
    window: tauri::Window,
    #[allow(non_snake_case)] ffxdbPath: String,
) -> Result<IndexStats, String> {
    let label = window.label().to_string();

    tauri::async_runtime::spawn_blocking(move || {
        let index_dir = search::index_path_from_ffxdb(&ffxdbPath);
        let idx = search::SearchIndex::open_or_create(&index_dir)?;
        let stats = idx.stats();
        search::set_search_index(&label, Arc::new(idx));
        info!(
            "Search index opened for window '{}' at {:?}",
            label, index_dir
        );
        Ok(stats)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Close the search index for the current window.
///
/// Called when a project is closed or the window is destroyed.
#[tauri::command]
pub async fn search_close_index(window: tauri::Window) -> Result<(), String> {
    let label = window.label().to_string();
    search::remove_search_index(&label);
    Ok(())
}

/// Delete the search index entirely and recreate an empty one.
#[tauri::command]
pub async fn search_delete_index(
    window: tauri::Window,
    #[allow(non_snake_case)] ffxdbPath: String,
) -> Result<(), String> {
    let label = window.label().to_string();

    // Remove current index
    search::remove_search_index(&label);

    tauri::async_runtime::spawn_blocking(move || {
        let index_dir = search::index_path_from_ffxdb(&ffxdbPath);
        if index_dir.exists() {
            std::fs::remove_dir_all(&index_dir)
                .map_err(|e| format!("Failed to delete index: {}", e))?;
        }
        info!("Search index deleted at {:?}", index_dir);
        Ok(())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Get statistics about the current search index.
#[tauri::command]
pub async fn search_get_stats(window: tauri::Window) -> Result<IndexStats, String> {
    let label = window.label().to_string();
    let idx = search::get_search_index(&label).ok_or("No search index open for this window")?;
    Ok(idx.stats())
}

// =============================================================================
// Indexing Commands
// =============================================================================

/// Index a single container file into the search index.
///
/// Runs in the background and emits progress events.
#[tauri::command]
pub async fn search_index_container(
    window: tauri::Window,
    #[allow(non_snake_case)] containerPath: String,
    #[allow(non_snake_case)] indexContent: bool,
) -> Result<u64, String> {
    let label = window.label().to_string();
    let idx = search::get_search_index(&label).ok_or("No search index open for this window")?;

    let window_clone = window.clone();
    let container = containerPath.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let state = IndexingState::new();

        // Emit initial progress
        let _ = window_clone.emit(
            "search-index-progress",
            state.progress(&container, IndexPhase::Scanning, ""),
        );

        // Delete existing entries for this container (re-index)
        idx.delete_container(&container)?;
        idx.commit()?;

        let result = search::indexer::index_container(&idx, &container, indexContent, &state);

        // Emit done
        let _ = window_clone.emit(
            "search-index-progress",
            state.progress(&container, IndexPhase::Done, ""),
        );

        result
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Index all containers at once.
///
/// Takes an array of container file paths and indexes them sequentially
/// with progress events for the overall operation.
#[tauri::command]
pub async fn search_index_all(
    window: tauri::Window,
    #[allow(non_snake_case)] containerPaths: Vec<String>,
    #[allow(non_snake_case)] indexContent: bool,
) -> Result<u64, String> {
    let label = window.label().to_string();
    let idx = search::get_search_index(&label).ok_or("No search index open for this window")?;

    let window_clone = window.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let state = IndexingState::new();
        let mut total_indexed = 0u64;

        for (i, container) in containerPaths.iter().enumerate() {
            // Emit progress
            let _ = window_clone.emit(
                "search-index-progress",
                IndexProgress {
                    container_path: container.clone(),
                    phase: IndexPhase::IndexingMetadata,
                    files_indexed: state.files_indexed.load(Ordering::Relaxed),
                    files_total: state.files_total.load(Ordering::Relaxed),
                    content_extracted: state.content_extracted.load(Ordering::Relaxed),
                    current_file: format!("Container {}/{}", i + 1, containerPaths.len()),
                    percent: if !containerPaths.is_empty() {
                        (i as f64 / containerPaths.len() as f64) * 100.0
                    } else {
                        0.0
                    },
                },
            );

            // Delete existing entries for this container
            if let Err(e) = idx.delete_container(container) {
                warn!("Failed to delete container entries: {}", e);
            }

            match search::indexer::index_container(&idx, container, indexContent, &state) {
                Ok(count) => total_indexed += count,
                Err(e) => warn!("Failed to index container {}: {}", container, e),
            }
        }

        // Emit done
        let _ = window_clone.emit(
            "search-index-progress",
            IndexProgress {
                container_path: String::new(),
                phase: IndexPhase::Done,
                files_indexed: state.files_indexed.load(Ordering::Relaxed),
                files_total: state.files_total.load(Ordering::Relaxed),
                content_extracted: state.content_extracted.load(Ordering::Relaxed),
                current_file: String::new(),
                percent: 100.0,
            },
        );

        info!(
            "Indexed {} files across {} containers",
            total_indexed,
            containerPaths.len()
        );
        Ok(total_indexed)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Rebuild the entire search index from scratch.
#[tauri::command]
pub async fn search_rebuild_index(
    window: tauri::Window,
    #[allow(non_snake_case)] containerPaths: Vec<String>,
    #[allow(non_snake_case)] indexContent: bool,
) -> Result<u64, String> {
    let label = window.label().to_string();
    let idx = search::get_search_index(&label).ok_or("No search index open for this window")?;

    let window_clone = window.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let state = IndexingState::new();

        let _ = window_clone.emit(
            "search-index-progress",
            IndexProgress {
                container_path: String::new(),
                phase: IndexPhase::Scanning,
                files_indexed: 0,
                files_total: 0,
                content_extracted: 0,
                current_file: "Rebuilding index...".to_string(),
                percent: 0.0,
            },
        );

        let result = search::indexer::rebuild_index(&idx, &containerPaths, indexContent, &state);

        let _ = window_clone.emit(
            "search-index-progress",
            IndexProgress {
                container_path: String::new(),
                phase: IndexPhase::Done,
                files_indexed: state.files_indexed.load(Ordering::Relaxed),
                files_total: state.files_total.load(Ordering::Relaxed),
                content_extracted: state.content_extracted.load(Ordering::Relaxed),
                current_file: String::new(),
                percent: 100.0,
            },
        );

        result
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

// =============================================================================
// Search Commands
// =============================================================================

/// Execute a search query against the index.
#[tauri::command]
pub async fn search_query(
    window: tauri::Window,
    options: SearchOptions,
) -> Result<SearchResults, String> {
    let label = window.label().to_string();
    let idx = search::get_search_index(&label).ok_or("No search index open for this window")?;

    tauri::async_runtime::spawn_blocking(move || search::query::search(&idx, &options))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
}
