// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Expert Witness Format (E01/L01/Ex01/Lx01) operations.

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::thread;
use tauri::Emitter;

use crate::containers;
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

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentVerifyProgress {
    pub segment_name: String,
    pub segment_number: u32,
    pub percent: f64,
    pub segments_completed: usize,
    pub segments_total: usize,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentHashResult {
    pub segment_name: String,
    pub segment_number: u32,
    pub segment_path: String,
    pub algorithm: String,
    pub computed_hash: String,
    pub expected_hash: Option<String>,
    pub verified: Option<bool>,  // None = no expected, true = match, false = mismatch
    pub size: u64,
    pub duration_secs: f64,
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

/// Verify individual E01 segment files by hashing each .E01, .E02, etc. file
#[tauri::command]
pub async fn e01_verify_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    #[allow(non_snake_case)]
    expectedHashes: Vec<containers::SegmentHash>,
    app: tauri::AppHandle,
) -> Result<Vec<SegmentHashResult>, String> {
    use std::sync::Mutex;
    use std::time::Instant;
    
    // Get all segment paths
    let segment_paths = ewf::get_segment_paths(&inputPath)?;
    let num_segments = segment_paths.len();
    
    if num_segments == 0 {
        return Err("No E01 segments found".to_string());
    }
    
    // Build expected hash lookup (by segment name, case-insensitive)
    let expected_map: std::collections::HashMap<String, String> = expectedHashes
        .iter()
        .map(|h| (h.segment_name.to_lowercase(), h.hash.clone()))
        .collect();
    
    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let parallelism = num_cpus.min(num_segments);
    
    let segments_completed = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<Vec<SegmentHashResult>>> = Arc::new(Mutex::new(Vec::with_capacity(num_segments)));
    let app = Arc::new(app);
    let algorithm = Arc::new(algorithm);
    let expected_map = Arc::new(expected_map);
    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build()
        .map_err(|e| format!("Failed to create thread pool: {}", e))?;
    
    pool.scope(|s| {
        for (idx, seg_path) in segment_paths.into_iter().enumerate() {
            let segments_completed = Arc::clone(&segments_completed);
            let results = Arc::clone(&results);
            let app = Arc::clone(&app);
            let algorithm = Arc::clone(&algorithm);
            let expected_map = Arc::clone(&expected_map);
            let segment_number = (idx + 1) as u32;
            
            s.spawn(move |_| {
                let segment_name: String = seg_path.file_name()
                    .map(|f: &std::ffi::OsStr| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("segment_{}", segment_number));
                let seg_path_str = seg_path.to_string_lossy().to_string();
                
                // Emit start event
                let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                    segment_name: segment_name.clone(),
                    segment_number,
                    percent: 0.0,
                    segments_completed: segments_completed.load(Ordering::Relaxed),
                    segments_total: num_segments,
                });
                
                let start_time = Instant::now();
                
                // Hash the segment
                let hash_result = ewf::hash_single_segment(&seg_path_str, &algorithm, |current, total| {
                    let percent = (current as f64 / total as f64) * 100.0;
                    let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                        segment_name: segment_name.clone(),
                        segment_number,
                        percent,
                        segments_completed: segments_completed.load(Ordering::Relaxed),
                        segments_total: num_segments,
                    });
                });
                
                let duration = start_time.elapsed().as_secs_f64();
                let completed = segments_completed.fetch_add(1, Ordering::Relaxed) + 1;
                let size = std::fs::metadata(&seg_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                match hash_result {
                    Ok(computed_hash) => {
                        let expected_hash = expected_map.get(&segment_name.to_lowercase()).cloned();
                        let verified = expected_hash.as_ref().map(|expected| {
                            computed_hash.to_lowercase() == expected.to_lowercase()
                        });
                        
                        let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                            segment_name: segment_name.clone(),
                            segment_number,
                            percent: 100.0,
                            segments_completed: completed,
                            segments_total: num_segments,
                        });
                        
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash,
                                expected_hash,
                                verified,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing E01 hash result for segment {}", segment_number);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut guard) = results.lock() {
                            guard.push(SegmentHashResult {
                                segment_name,
                                segment_number,
                                segment_path: seg_path_str,
                                algorithm: algorithm.to_uppercase(),
                                computed_hash: format!("ERROR: {}", e),
                                expected_hash: None,
                                verified: None,
                                size,
                                duration_secs: duration,
                            });
                        } else {
                            tracing::error!("Mutex poisoned while storing E01 error result for segment {}", segment_number);
                        }
                    }
                }
            });
        }
    });
    
    let mut final_results = Arc::try_unwrap(results)
        .map_err(|_| "Failed to unwrap results")?
        .into_inner()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    final_results.sort_by_key(|r| r.segment_number);
    
    Ok(final_results)
}
