// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File Transfer Commands
//!
//! Tauri commands for file copy/transfer operations with progress tracking.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

use crate::common::transfer::{
    self, execute_transfer, scan_transfer_items_limited, TransferConfig,
    TransferState,
};
use crate::common::hex::format_size_compact;

// =============================================================================
// Global State for Active Transfers
// =============================================================================

/// Active transfer operations that can be cancelled
static ACTIVE_TRANSFERS: Lazy<Mutex<HashMap<String, Arc<TransferState>>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// =============================================================================
// Types
// =============================================================================

/// Request to start a file transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    /// Source paths to copy
    pub sources: Vec<String>,
    /// Destination directory
    pub destination: String,
    /// Whether to verify files after copying (hash comparison)
    pub verify: Option<bool>,
    /// Hash algorithm for verification ("md5", "sha256", "xxh3", etc.)
    pub hash_algorithm: Option<String>,
    /// Whether to preserve file timestamps
    pub preserve_timestamps: Option<bool>,
    /// Whether to preserve permissions
    pub preserve_permissions: Option<bool>,
    /// Whether to overwrite existing files
    pub overwrite: Option<bool>,
    /// Whether to copy directories recursively
    pub recursive: Option<bool>,
    /// Whether to flatten directory structure
    pub flatten: Option<bool>,
    /// Whether sources contain forensic containers that should be treated as logical units
    pub container_aware: Option<bool>,
    /// Number of parallel transfer threads (1-8, default: 4)
    pub parallel_threads: Option<u8>,
}

/// Summary of files to be transferred (for preview)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferPreview {
    /// Total number of files (may be limited for large directories)
    pub total_files: u64,
    /// Total number of directories
    pub total_directories: u64,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Human-readable size
    pub total_size_formatted: String,
    /// List of files with their relative paths and sizes
    pub files: Vec<TransferFileInfo>,
    /// Whether the preview was limited (more files exist)
    #[serde(default)]
    pub is_limited: bool,
}

/// Information about a file in the transfer preview
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferFileInfo {
    /// Source path
    pub source: String,
    /// Relative path (for destination)
    pub relative_path: String,
    /// Size in bytes
    pub size: u64,
    /// Human-readable size
    pub size_formatted: String,
}

// =============================================================================
// Commands
// =============================================================================

/// Maximum number of files to include in preview (for performance)
const MAX_PREVIEW_FILES: usize = 1000;

/// Preview a transfer operation without executing it
/// 
/// Returns a summary of files that would be transferred.
/// Note: File list is limited to MAX_PREVIEW_FILES for performance.
/// When limited, `is_limited` will be true to indicate more files exist.
#[tauri::command]
pub fn transfer_preview(
    sources: Vec<String>,
    recursive: Option<bool>,
) -> Result<TransferPreview, String> {
    info!("transfer_preview: {} sources, recursive={:?}", sources.len(), recursive);
    
    let recursive = recursive.unwrap_or(true);
    
    // Use limited scan for faster preview - only scan up to MAX_PREVIEW_FILES + 1
    // to detect if there are more files
    let items = scan_transfer_items_limited(&sources, recursive, None, Some(MAX_PREVIEW_FILES + 1))
        .map_err(|e| format!("Failed to scan sources: {}", e))?;
    
    let is_limited = items.iter().filter(|i| !i.is_directory).count() > MAX_PREVIEW_FILES;
    
    let mut total_files: u64 = 0;
    let mut total_directories: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut files = Vec::with_capacity(MAX_PREVIEW_FILES.min(items.len()));
    
    for item in items {
        if item.is_directory {
            total_directories += 1;
        } else {
            total_files += 1;
            total_bytes += item.size;
            // Only include up to MAX_PREVIEW_FILES in the detailed list
            if files.len() < MAX_PREVIEW_FILES {
                files.push(TransferFileInfo {
                    source: item.source.to_string_lossy().to_string(),
                    relative_path: item.relative_path.to_string_lossy().to_string(),
                    size: item.size,
                    size_formatted: format_size_compact(item.size),
                });
            }
        }
    }
    
    Ok(TransferPreview {
        total_files,
        total_directories,
        total_bytes,
        total_size_formatted: format_size_compact(total_bytes),
        files,
        is_limited,
    })
}

/// Start a file transfer operation
/// 
/// Emits "transfer-progress" events as the transfer progresses.
/// Returns the operation ID which can be used to cancel the transfer.
#[tauri::command]
pub async fn transfer_start(
    app: AppHandle,
    request: TransferRequest,
) -> Result<String, String> {
    info!(
        "transfer_start: {} sources -> {}, container_aware={:?}, parallel_threads={:?}", 
        request.sources.len(), 
        request.destination,
        request.container_aware,
        request.parallel_threads
    );
    
    // Create transfer configuration
    // Clamp parallel_threads to 1-8 range, default to 4
    let threads = request.parallel_threads.unwrap_or(4).clamp(1, 8);
    
    let config = TransferConfig {
        sources: request.sources,
        destination: request.destination,
        verify_after_copy: request.verify.unwrap_or(false),
        hash_algorithm: request.hash_algorithm.or(Some("sha256".to_string())),
        preserve_timestamps: request.preserve_timestamps.unwrap_or(true),
        preserve_permissions: request.preserve_permissions.unwrap_or(true),
        overwrite_existing: request.overwrite.unwrap_or(false),
        recursive: request.recursive.unwrap_or(true),
        flatten: request.flatten.unwrap_or(false),
        container_aware: request.container_aware.unwrap_or(false),
        parallel_threads: threads,
    };
    
    // Create shared state for cancellation
    let state = Arc::new(TransferState::new());
    let operation_id = uuid::Uuid::new_v4().to_string();
    
    // Register the transfer
    {
        let mut transfers = ACTIVE_TRANSFERS.lock()
            .map_err(|e| format!("Failed to acquire transfer lock: {}", e))?;
        transfers.insert(operation_id.clone(), Arc::clone(&state));
    }
    
    let op_id = operation_id.clone();
    let app_handle = app.clone();
    
    // Run transfer in background thread
    thread::spawn(move || {
        let result = execute_transfer(&config, Arc::clone(&state), op_id.clone(), |progress| {
            // Emit progress event
            if let Err(e) = app_handle.emit("transfer-progress", &progress) {
                error!("Failed to emit transfer progress: {}", e);
            }
        });
        
        // Emit completion event
        if let Err(e) = app_handle.emit("transfer-complete", &result) {
            error!("Failed to emit transfer complete: {}", e);
        }
        
        // Remove from active transfers
        if let Ok(mut transfers) = ACTIVE_TRANSFERS.lock() {
            transfers.remove(&op_id);
        }
        
        info!(
            "Transfer {} complete: {} files, {} bytes",
            op_id, result.successful_files, result.bytes_transferred
        );
    });
    
    Ok(operation_id)
}

/// Cancel an active transfer operation
#[tauri::command]
pub fn transfer_cancel(operation_id: String) -> Result<bool, String> {
    info!("transfer_cancel: {}", operation_id);
    
    let transfers = ACTIVE_TRANSFERS.lock()
        .map_err(|e| format!("Failed to acquire transfer lock: {}", e))?;
    if let Some(state) = transfers.get(&operation_id) {
        state.cancel();
        Ok(true)
    } else {
        Ok(false) // Transfer not found or already complete
    }
}

/// Get list of active transfer operations
#[tauri::command]
pub fn transfer_list_active() -> Result<Vec<String>, String> {
    let transfers = ACTIVE_TRANSFERS.lock()
        .map_err(|e| format!("Failed to acquire transfer lock: {}", e))?;
    Ok(transfers.keys().cloned().collect())
}

/// Copy a single file with optional verification
/// 
/// Simpler API for single file copies without progress events.
#[tauri::command]
pub async fn transfer_copy_file(
    source: String,
    destination: String,
    verify: Option<bool>,
    hash_algorithm: Option<String>,
) -> Result<transfer::FileTransferResult, String> {
    info!("transfer_copy_file: {} -> {}", source, destination);
    
    let src_path = std::path::PathBuf::from(&source);
    let dst_path = std::path::PathBuf::from(&destination);
    
    if !src_path.exists() {
        return Err(format!("Source file does not exist: {}", source));
    }
    
    if !src_path.is_file() {
        return Err(format!("Source is not a file: {}", source));
    }
    
    // Create parent directory if needed
    if let Some(parent) = dst_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }
    
    let file_size = src_path.metadata()
        .map_err(|e| format!("Failed to read source metadata: {}", e))?
        .len();
    
    // Copy the file
    let copy_result = transfer::copy_file_with_progress(
        &src_path,
        &dst_path,
        true, // preserve timestamps
        true, // preserve permissions
        None, // no cancellation state
        |_, _| {}, // no progress callback for single file
    );
    
    match copy_result {
        Ok(_) => {
            // Verify if requested - use sha256 as default for forensic integrity
            let algo = hash_algorithm.as_deref().unwrap_or("sha256");
            let (source_hash, dest_hash, verified) = if verify.unwrap_or(false) {
                let src_hash = transfer::compute_file_hash(&src_path, algo).ok();
                let dst_hash = transfer::compute_file_hash(&dst_path, algo).ok();
                let is_verified = match (&src_hash, &dst_hash) {
                    (Some(s), Some(d)) => Some(s == d),
                    _ => None,
                };
                (src_hash, dst_hash, is_verified)
            } else {
                (None, None, None)
            };
            
            if verified == Some(false) {
                return Err("Verification failed - hash mismatch".to_string());
            }
            
            Ok(transfer::FileTransferResult {
                source,
                destination,
                size: file_size,
                success: true,
                error: None,
                source_hash,
                destination_hash: dest_hash,
                verified,
            })
        }
        Err(e) => Err(format!("Copy failed: {}", e)),
    }
}

/// Copy entire directory with optional verification
#[tauri::command]
pub async fn transfer_copy_directory(
    app: AppHandle,
    source: String,
    destination: String,
    verify: Option<bool>,
    recursive: Option<bool>,
) -> Result<String, String> {
    info!("transfer_copy_directory: {} -> {}", source, destination);
    
    let src_path = std::path::PathBuf::from(&source);
    
    if !src_path.exists() {
        return Err(format!("Source does not exist: {}", source));
    }
    
    if !src_path.is_dir() {
        return Err(format!("Source is not a directory: {}", source));
    }
    
    // Use the main transfer API
    let request = TransferRequest {
        sources: vec![source],
        destination,
        verify,
        hash_algorithm: Some("xxh3".to_string()),
        preserve_timestamps: Some(true),
        preserve_permissions: Some(true),
        overwrite: Some(false),
        recursive,
        flatten: Some(false),
        container_aware: Some(false),
        parallel_threads: None, // Use default
    };
    
    transfer_start(app, request).await
}

/// Get the size of files that would be transferred
#[tauri::command]
pub fn transfer_calculate_size(
    sources: Vec<String>,
    recursive: Option<bool>,
) -> Result<u64, String> {
    let recursive = recursive.unwrap_or(true);
    
    transfer::calculate_transfer_size(&sources, recursive)
        .map_err(|e| format!("Failed to calculate size: {}", e))
}
