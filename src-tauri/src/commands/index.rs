// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for index cache operations

use crate::common::{IndexCache, IndexEntry, IndexSummary, CacheStats, IndexWorker, IndexWorkerInfo};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Global index cache instance
pub struct IndexCacheState(pub Arc<Mutex<Option<IndexCache>>>);

/// Global index worker instance
pub struct IndexWorkerState(pub Arc<Mutex<Option<IndexWorker>>>);

/// Initialize index cache with database path
#[tauri::command]
pub async fn index_cache_init(
    db_path: String,
    cache_state: State<'_, IndexCacheState>,
    worker_state: State<'_, IndexWorkerState>,
) -> Result<(), String> {
    let cache = IndexCache::new(&db_path)?;
    let cache_arc = Arc::new(cache.clone());
    
    // Initialize cache
    let mut cache_lock = cache_state.0.lock().unwrap();
    *cache_lock = Some(cache);
    
    // Initialize worker with cache
    let worker = IndexWorker::new(cache_arc);
    let mut worker_lock = worker_state.0.lock().unwrap();
    *worker_lock = Some(worker);
    
    Ok(())
}

/// Check if container has a valid cached index
#[tauri::command]
pub async fn index_cache_has_index(
    container_path: String,
    state: State<'_, IndexCacheState>,
) -> Result<bool, String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.has_index(&container_path)
}

/// Get index summary without loading all entries
#[tauri::command]
pub async fn index_cache_get_summary(
    container_path: String,
    state: State<'_, IndexCacheState>,
) -> Result<Option<IndexSummary>, String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.get_summary(&container_path)
}

/// Store index entries (batch operation)
#[tauri::command]
pub async fn index_cache_store(
    container_path: String,
    entries: Vec<IndexEntry>,
    is_complete: bool,
    state: State<'_, IndexCacheState>,
) -> Result<(), String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.store_index(&container_path, &entries, is_complete)
}

/// Load index entries from cache
#[tauri::command]
pub async fn index_cache_load(
    container_path: String,
    state: State<'_, IndexCacheState>,
) -> Result<Vec<IndexEntry>, String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.load_index(&container_path)
}

/// Invalidate cache for a container
#[tauri::command]
pub async fn index_cache_invalidate(
    container_path: String,
    state: State<'_, IndexCacheState>,
) -> Result<(), String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.invalidate(&container_path)
}

/// Get cache statistics
#[tauri::command]
pub async fn index_cache_stats(
    state: State<'_, IndexCacheState>,
) -> Result<CacheStats, String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.get_stats()
}

/// Clear entire cache
#[tauri::command]
pub async fn index_cache_clear(
    state: State<'_, IndexCacheState>,
) -> Result<(), String> {
    let cache_lock = state.0.lock().unwrap();
    let cache = cache_lock.as_ref().ok_or("Index cache not initialized")?;
    cache.clear_all()
}

// =============================================================================
// Background Index Worker Commands
// =============================================================================

/// Start background indexing for a container
#[tauri::command]
pub async fn index_worker_start(
    container_path: String,
    container_type: String,
    window: tauri::Window,
    state: State<'_, IndexWorkerState>,
) -> Result<(), String> {
    let worker_lock = state.0.lock().unwrap();
    let worker = worker_lock.as_ref().ok_or("Index worker not initialized")?;
    worker.start_indexing(container_path, container_type, window)
}

/// Cancel an active indexing operation
#[tauri::command]
pub async fn index_worker_cancel(
    container_path: String,
    state: State<'_, IndexWorkerState>,
) -> Result<(), String> {
    let worker_lock = state.0.lock().unwrap();
    let worker = worker_lock.as_ref().ok_or("Index worker not initialized")?;
    worker.cancel_indexing(&container_path)
}

/// Get info about active workers
#[tauri::command]
pub async fn index_worker_get_active(
    state: State<'_, IndexWorkerState>,
) -> Result<Vec<IndexWorkerInfo>, String> {
    let worker_lock = state.0.lock().unwrap();
    let worker = worker_lock.as_ref().ok_or("Index worker not initialized")?;
    Ok(worker.get_active_workers())
}

/// Check if container is being indexed
#[tauri::command]
pub async fn index_worker_is_indexing(
    container_path: String,
    state: State<'_, IndexWorkerState>,
) -> Result<bool, String> {
    let worker_lock = state.0.lock().unwrap();
    let worker = worker_lock.as_ref().ok_or("Index worker not initialized")?;
    Ok(worker.is_indexing(&container_path))
}
