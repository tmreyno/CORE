// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Expert Witness Format (E01/L01/Ex01/Lx01) operations.

use tauri::Emitter;

use crate::ewf;

use super::VerifyProgress;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct E01MediaInfo {
    pub media_size: u64,
    pub chunk_size: u32,
    pub sector_size: u32,
    pub sector_count: u64,
    pub chunk_count: u64,
}

// EWF Commands - Expert Witness Format implementation (E01/L01/Ex01/Lx01)
#[tauri::command]
pub async fn e01_v3_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<ewf::EwfInfo, String> {
    // Run on blocking thread pool to prevent UI freeze during file parsing
    tauri::async_runtime::spawn_blocking(move || {
        ewf::info(&inputPath)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn e01_v3_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_for_closure = inputPath.clone();
    // Run on blocking thread pool to prevent UI freeze
    tauri::async_runtime::spawn_blocking(move || {
        ewf::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
            let _ = app.emit("verify-progress", VerifyProgress {
                path: path_for_closure.clone(),
                current,
                total,
                percent,
            });
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

/// Read bytes from E01 image at arbitrary offset (for filesystem browsing)
#[tauri::command]
pub async fn e01_read_at(
    #[allow(non_snake_case)]
    inputPath: String,
    offset: u64,
    length: usize,
) -> Result<Vec<u8>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut handle = ewf::EwfHandle::open(&inputPath)?;
        handle.read_at(offset, length)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

/// Get media info for E01 (size, sector size, chunk size)
#[tauri::command]
pub async fn e01_media_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<E01MediaInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let handle = ewf::EwfHandle::open(&inputPath)?;
        Ok(E01MediaInfo {
            media_size: handle.get_media_size(),
            chunk_size: handle.get_chunk_size(),
            sector_size: handle.get_volume_info().bytes_per_sector,
            sector_count: handle.get_volume_info().sector_count,
            chunk_count: handle.get_chunk_count() as u64,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}
