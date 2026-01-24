// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for memory-mapped hex viewer

use tauri::State;
use std::sync::{Arc, Mutex};
use crate::viewer::MmapHexViewer;

// Global mmap viewer state
pub struct MmapViewerState(pub Arc<Mutex<Option<MmapHexViewer>>>);

// Initialize mmap viewer
#[tauri::command]
pub async fn mmap_hex_init(state: State<'_, MmapViewerState>) -> Result<(), String> {
    let mut viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    *viewer_opt = Some(MmapHexViewer::new());
    Ok(())
}

// Get file size
#[tauri::command]
pub async fn mmap_hex_get_file_size(
    path: String,
    state: State<'_, MmapViewerState>,
) -> Result<u64, String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    viewer.get_file_size(&path)
}

// Get a single page
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HexPageResponse {
    pub page_index: usize,
    pub offset: u64,
    pub data: Vec<u8>,
    pub size: usize,
}

#[tauri::command]
pub async fn mmap_hex_get_page(
    path: String,
    page_index: usize,
    state: State<'_, MmapViewerState>,
) -> Result<HexPageResponse, String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    let page = viewer.get_page(&path, page_index)?;
    
    Ok(HexPageResponse {
        page_index: page.page_index,
        offset: page.offset,
        data: page.data.clone(),
        size: page.size,
    })
}

// Get multiple pages (window)
#[tauri::command]
pub async fn mmap_hex_get_pages_window(
    path: String,
    center_page: usize,
    visible_pages: usize,
    state: State<'_, MmapViewerState>,
) -> Result<Vec<HexPageResponse>, String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    let pages = viewer.get_pages_window(&path, center_page, visible_pages)?;
    
    Ok(pages
        .iter()
        .map(|page| HexPageResponse {
            page_index: page.page_index,
            offset: page.offset,
            data: page.data.clone(),
            size: page.size,
        })
        .collect())
}

// Close file (remove from cache)
#[tauri::command]
pub async fn mmap_hex_close_file(
    path: String,
    state: State<'_, MmapViewerState>,
) -> Result<(), String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    viewer.close_file(&path)
}

// Get cache statistics
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MmapCacheStats {
    pub cached_files: usize,
    pub cached_pages: usize,
    pub total_cache_bytes: usize,
}

#[tauri::command]
pub async fn mmap_hex_get_cache_stats(
    state: State<'_, MmapViewerState>,
) -> Result<MmapCacheStats, String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    let (files, pages, bytes) = viewer.get_cache_stats()?;
    
    Ok(MmapCacheStats {
        cached_files: files,
        cached_pages: pages,
        total_cache_bytes: bytes,
    })
}

// Clear all caches
#[tauri::command]
pub async fn mmap_hex_clear_caches(
    state: State<'_, MmapViewerState>,
) -> Result<(), String> {
    let viewer_opt = state.0.lock().map_err(|e| e.to_string())?;
    let viewer = viewer_opt
        .as_ref()
        .ok_or("Mmap viewer not initialized")?;
    
    viewer.clear_caches()
}
