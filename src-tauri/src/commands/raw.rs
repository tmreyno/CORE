// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Raw disk image operations (.dd, .raw, .img, .001).

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::thread;
use tauri::Emitter;

use crate::containers;
use crate::raw;

use super::{VerifyProgress, SegmentVerifyProgress, SegmentHashResult};

// RAW Commands - Raw disk image implementation (.dd, .raw, .img, .001)
#[tauri::command]
pub async fn raw_info(
    #[allow(non_snake_case)]
    inputPath: String,
) -> Result<raw::RawInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        raw::info(&inputPath).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
pub async fn raw_verify(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_for_closure = inputPath.clone();
    tauri::async_runtime::spawn_blocking(move || {
        raw::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
            let _ = app.emit("verify-progress", VerifyProgress {
                path: path_for_closure.clone(),
                current: current as usize,
                total: total as usize,
                percent,
            });
        }).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Verify individual segments of a raw image, comparing against stored hashes
#[tauri::command]
pub async fn raw_verify_segments(
    #[allow(non_snake_case)]
    inputPath: String,
    algorithm: String,
    #[allow(non_snake_case)]
    expectedHashes: Vec<containers::SegmentHash>,  // Optional: stored hashes from companion log
    app: tauri::AppHandle,
) -> Result<Vec<SegmentHashResult>, String> {
    use std::sync::Mutex;
    use std::time::Instant;
    
    // Get all segment paths
    let segment_paths = raw::get_segment_paths(&inputPath).map_err(|e| e.to_string())?;
    let num_segments = segment_paths.len();
    
    if num_segments == 0 {
        return Err("No segments found".to_string());
    }
    
    // Build expected hash lookup (by segment name, case-insensitive)
    let expected_map: std::collections::HashMap<String, String> = expectedHashes
        .iter()
        .map(|h| (h.segment_name.to_lowercase(), h.hash.clone()))
        .collect();
    
    // Use rayon for parallel processing
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
                let hash_result = raw::hash_single_segment(&seg_path_str, &algorithm, |current, total| {
                    let percent = (current as f64 / total as f64) * 100.0;
                    let _ = app.emit("segment-verify-progress", SegmentVerifyProgress {
                        segment_name: segment_name.clone(),
                        segment_number,
                        percent,
                        segments_completed: segments_completed.load(Ordering::Relaxed),
                        segments_total: num_segments,
                    });
                }).map_err(|e| e.to_string());
                
                let duration = start_time.elapsed().as_secs_f64();
                let completed = segments_completed.fetch_add(1, Ordering::SeqCst) + 1;
                
                // Get file size
                let size = std::fs::metadata(&seg_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                match hash_result {
                    Ok(computed_hash) => {
                        // Look up expected hash
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
                        
                        // Use if-let to handle potential mutex poisoning gracefully
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
                            tracing::error!("Mutex poisoned while storing hash result for segment {}", segment_number);
                        }
                    }
                    Err(e) => {
                        // Return error result for this segment
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
                            tracing::error!("Mutex poisoned while storing error result for segment {}", segment_number);
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
    
    // Sort by segment number
    final_results.sort_by_key(|r| r.segment_number);
    
    Ok(final_results)
}
