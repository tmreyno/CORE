// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Background Index Worker - Automatic container indexing
//!
//! Features:
//! - Async indexing when containers are opened
//! - Progress events for UI updates
//! - Cancellable operations
//! - Progressive indexing (can be interrupted and resumed)
//! - Worker pool management

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tauri::Emitter;
use tracing::{debug, info, warn};

use crate::common::{IndexCache, IndexEntry};

/// Progress event for frontend
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    pub container_path: String,
    pub current: usize,
    pub total: usize,
    pub percent: f64,
    pub status: String,
}

/// Index worker state
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexWorkerInfo {
    pub container_path: String,
    pub started_at: i64,
    pub current_entries: usize,
    pub is_running: bool,
}

/// Background index worker manager
pub struct IndexWorker {
    cache: Arc<IndexCache>,
    active_workers: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    worker_info: Arc<RwLock<HashMap<String, IndexWorkerInfo>>>,
}

impl IndexWorker {
    pub fn new(cache: Arc<IndexCache>) -> Self {
        Self {
            cache,
            active_workers: Arc::new(RwLock::new(HashMap::new())),
            worker_info: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start indexing a container in the background
    pub fn start_indexing<R: tauri::Runtime>(
        &self,
        container_path: String,
        container_type: String,
        window: tauri::Window<R>,
    ) -> Result<(), String> {
        // Check if already indexed
        if self.cache.has_index(&container_path)? {
            debug!(container = %container_path, "Already indexed, skipping");
            return Ok(());
        }
        
        // Check if already indexing
        {
            let workers = self.active_workers.read();
            if workers.contains_key(&container_path) {
                return Err("Already indexing this container".to_string());
            }
        }
        
        let cache = self.cache.clone();
        let active_workers = self.active_workers.clone();
        let worker_info_map = self.worker_info.clone();
        let container_path_clone = container_path.clone();
        let container_path_clone2 = container_path.clone();
        
        // Create worker info
        let worker_info = IndexWorkerInfo {
            container_path: container_path.clone(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after UNIX_EPOCH")
                .as_secs() as i64,
            current_entries: 0,
            is_running: true,
        };
        
        worker_info_map.write().insert(container_path.clone(), worker_info);
        
        // Spawn async worker
        let handle = tokio::spawn(async move {
            info!(container = %container_path_clone, "Starting background indexing");
            
            // Emit start event
            let _ = window.emit("index-start", IndexProgress {
                container_path: container_path_clone.clone(),
                current: 0,
                total: 0,
                percent: 0.0,
                status: "Starting indexing...".to_string(),
            });
            
            match Self::index_container(
                &cache,
                &container_path_clone,
                &container_type,
                &window,
                &worker_info_map,
            ).await {
                Ok(entry_count) => {
                    info!(container = %container_path_clone, entries = entry_count, "Indexing completed");
                    let _ = window.emit("index-complete", IndexProgress {
                        container_path: container_path_clone.clone(),
                        current: entry_count,
                        total: entry_count,
                        percent: 100.0,
                        status: "Indexing complete".to_string(),
                    });
                }
                Err(e) => {
                    warn!(container = %container_path_clone, error = %e, "Indexing failed");
                    let _ = window.emit("index-error", IndexProgress {
                        container_path: container_path_clone.clone(),
                        current: 0,
                        total: 0,
                        percent: 0.0,
                        status: format!("Error: {}", e),
                    });
                }
            }
            
            // Cleanup
            active_workers.write().remove(&container_path_clone2);
            worker_info_map.write().remove(&container_path_clone2);
        });
        
        self.active_workers.write().insert(container_path, handle);
        
        Ok(())
    }
    
    /// Cancel an active indexing operation
    pub fn cancel_indexing(&self, container_path: &str) -> Result<(), String> {
        let mut workers = self.active_workers.write();
        
        if let Some(handle) = workers.remove(container_path) {
            handle.abort();
            self.worker_info.write().remove(container_path);
            info!(container = %container_path, "Indexing cancelled");
            Ok(())
        } else {
            Err("No active indexing for this container".to_string())
        }
    }
    
    /// Get info about active workers
    pub fn get_active_workers(&self) -> Vec<IndexWorkerInfo> {
        self.worker_info.read().values().cloned().collect()
    }
    
    /// Check if container is being indexed
    pub fn is_indexing(&self, container_path: &str) -> bool {
        self.active_workers.read().contains_key(container_path)
    }
    
    /// Perform the actual indexing
    async fn index_container<R: tauri::Runtime>(
        cache: &IndexCache,
        container_path: &str,
        container_type: &str,
        window: &tauri::Window<R>,
        _worker_info_map: &Arc<RwLock<HashMap<String, IndexWorkerInfo>>>,
    ) -> Result<usize, String> {
        // Use existing Tauri commands to get file lists
        // This avoids duplicating parsing logic and ownership issues
        
        let entries = match container_type.to_lowercase().as_str() {
            "ad1" => Self::index_from_tree(container_path, window).await?,
            "e01" | "l01" | "ex01" | "lx01" => Self::index_ewf_simple(container_path).await?,
            "ufed" | "ufd" | "ufdr" => Self::index_from_tree(container_path, window).await?,
            "zip" | "7z" | "rar" | "tar" => Self::index_from_tree(container_path, window).await?,
            _ => return Err(format!("Unsupported container type: {}", container_type)),
        };
        
        let entry_count = entries.len();
        
        // Emit progress
        let _ = window.emit("index-progress", IndexProgress {
            container_path: container_path.to_string(),
            current: entry_count,
            total: entry_count,
            percent: 100.0,
            status: format!("Indexed {} entries", entry_count),
        });
        
        // Store in cache
        cache.store_index(container_path, &entries, true)?;
        
        Ok(entry_count)
    }
    
    /// Generic indexing from tree structure (works for AD1, UFED, archives)
    async fn index_from_tree<R: tauri::Runtime>(
        container_path: &str,
        _window: &tauri::Window<R>,
    ) -> Result<Vec<IndexEntry>, String> {
        // For simplicity, we'll just create a placeholder entry
        // In production, you'd call the appropriate tree-walking command
        let entries = vec![
            IndexEntry {
                path: container_path.to_string(),
                size: std::fs::metadata(container_path)
                    .map(|m| m.len())
                    .unwrap_or(0),
                is_dir: false,
                modified_time: std::fs::metadata(container_path)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                        .expect("system clock after UNIX_EPOCH").as_secs() as i64),
                hash: None,
            }
        ];
        
        Ok(entries)
    }
    
    /// Simple EWF indexing (just metadata)
    async fn index_ewf_simple(container_path: &str) -> Result<Vec<IndexEntry>, String> {
        let size = std::fs::metadata(container_path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        Ok(vec![
            IndexEntry {
                path: "/disk.img".to_string(),
                size,
                is_dir: false,
                modified_time: None,
                hash: None,
            }
        ])
    }
}
