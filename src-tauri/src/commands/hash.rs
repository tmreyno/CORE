// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Parallel batch hashing operations for multiple files.

use std::sync::Arc;
use tauri::Emitter;
use tracing::{debug, info, instrument};

use crate::ad1;
use crate::common::HashQueue;
use crate::containers;
use crate::ewf;
use crate::raw;

// Batch hashing result for a single file
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchHashResult {
    pub path: String,
    pub algorithm: String,
    pub hash: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub throughput_mbs: Option<f64>,
}

// Progress update for batch hashing - includes hash result when completed
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub path: String,
    pub status: String,  // "started", "progress", "completed", "error"
    pub percent: f64,
    pub files_completed: usize,
    pub files_total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // Decompression progress (for E01/compressed containers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_processed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunks_total: Option<usize>,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFileInput {
    pub path: String,
    pub container_type: String,
}

/// Hash multiple files in parallel with smart scheduling
/// 
/// Optimizations:
/// Simple sequential batch hash - hash files one by one with progress updates
#[tauri::command]
#[instrument(skip(files, app), fields(num_files = files.len(), algorithm = %algorithm))]
pub async fn batch_hash(
    files: Vec<BatchFileInput>,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<Vec<BatchHashResult>, String> {
    let num_files = files.len();
    info!("Starting parallel batch hash");
    if num_files == 0 {
        return Ok(Vec::new());
    }
    
    // Determine parallelism based on available CPU cores
    // Use all available cores for maximum throughput
    let num_cpus = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4);
    // Allow processing up to num_cpus files concurrently (or fewer for small batches)
    let max_concurrent = num_cpus.min(num_files);
    debug!(max_concurrent, num_cpus, "Parallel file limit set based on CPU cores");
    
    // Use a semaphore to limit concurrent file processing
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    
    // Spawn all file processing tasks
    let mut handles = Vec::with_capacity(num_files);
    
    for (idx, file) in files.into_iter().enumerate() {
        let path = file.path.clone();
        let container_type = file.container_type.to_lowercase();
        let algo = algorithm.clone();
        let app_clone = app.clone();
        let sem = semaphore.clone();
        
        // Emit progress: queued
        let _ = app.emit("batch-progress", BatchProgress {
            path: path.clone(),
            status: "queued".to_string(),
            percent: 0.0,
            files_completed: 0,
            files_total: num_files,
            hash: None,
            algorithm: None,
            error: None,
            chunks_processed: None,
            chunks_total: None,
        });
        
        let handle = tauri::async_runtime::spawn(async move {
            // Acquire semaphore permit (limits concurrent files)
            let _permit = sem.acquire_owned().await.map_err(|e| format!("Semaphore error: {}", e))?;
            
            debug!(idx = idx + 1, total = num_files, path = %path, "File started");
            
            // Emit progress: started
            let _ = app_clone.emit("batch-progress", BatchProgress {
                path: path.clone(),
                status: "started".to_string(),
                percent: 0.0,
                files_completed: idx,
                files_total: num_files,
                hash: None,
                algorithm: None,
                error: None,
                chunks_processed: None,
                chunks_total: None,
            });
            
            let path_for_hash = path.clone();
            let algo_for_hash = algo.clone();
            let container_for_hash = container_type.clone();
            let app_for_hash = app_clone.clone();
            
            // Run blocking hash in spawn_blocking
            let hash_result = tauri::async_runtime::spawn_blocking(move || {
                let start_time = std::time::Instant::now();
                let file_size = std::fs::metadata(&path_for_hash).map(|m| m.len()).unwrap_or(0);
                debug!(idx = idx + 1, size_mb = file_size / 1024 / 1024, "Processing file");
                
                // Progress counters
                let progress_current = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                let progress_total = Arc::new(std::sync::atomic::AtomicUsize::new(1)); // Start with 1 to avoid div by zero
                let done_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                
                // Progress reporter thread
                let progress_current_clone = progress_current.clone();
                let progress_total_clone = progress_total.clone();
                let done_flag_clone = done_flag.clone();
                let app_for_timer = app_for_hash.clone();
                let path_for_timer = path_for_hash.clone();
                let progress_thread = std::thread::spawn(move || {
                    let mut last_percent = 0u32;
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if done_flag_clone.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        let current = progress_current_clone.load(std::sync::atomic::Ordering::Relaxed);
                        let total = progress_total_clone.load(std::sync::atomic::Ordering::Relaxed);
                        if total > 1 {
                            let percent = ((current as f64 / total as f64) * 100.0) as u32;
                            // Only emit if percent increased (prevents bouncing)
                            if percent > last_percent {
                                let _ = app_for_timer.emit("batch-progress", BatchProgress {
                                    path: path_for_timer.clone(),
                                    status: "progress".to_string(),
                                    percent: percent as f64,
                                    files_completed: idx,
                                    files_total: num_files,
                                    hash: None,
                                    algorithm: None,
                                    error: None,
                                    chunks_processed: Some(current),
                                    chunks_total: Some(total),
                                });
                                last_percent = percent;
                            }
                        }
                    }
                });
                
                // Hash based on container type
                let result: Result<String, String> = if container_for_hash.contains("e01") || container_for_hash.contains("encase") || container_for_hash.contains("ex01") {
                    ewf::verify_with_progress(&path_for_hash, &algo_for_hash, |current: usize, total: usize| {
                        progress_total.store(total, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("raw") || container_for_hash.contains("dd") {
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ufed") || container_for_hash.contains("zip") || container_for_hash.contains("archive") || container_for_hash.contains("tar") || container_for_hash.contains("7z") {
                    // UFED containers, archives (ZIP, TAR, 7z) - hash the file directly
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ad1") {
                    // AD1 containers - hash the segment files (image-level hash)
                    ad1::hash_segments_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                } else if container_for_hash.contains("l01") {
                    // L01 containers - verify and return hash from message
                    containers::verify(&path_for_hash, &algo_for_hash)
                        .map(|entries| {
                            entries.first()
                                .and_then(|e| e.message.clone())
                                .unwrap_or_else(|| "Verified".to_string())
                        })
                } else {
                    // Unknown - try raw verification
                    raw::verify_with_progress(&path_for_hash, &algo_for_hash, |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    }).map_err(|e| e.to_string())
                };
                
                // Stop progress thread
                done_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                let _ = progress_thread.join();
                
                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let throughput_mbs = if duration_ms > 0 && file_size > 0 {
                    Some((file_size as f64 / (1024.0 * 1024.0)) / (duration_ms as f64 / 1000.0))
                } else {
                    None
                };
                
                (result, duration_ms, throughput_mbs)
            }).await.map_err(|e| format!("Task error: {}", e))?;
            
            let (result, duration_ms, throughput_mbs) = hash_result;
            
            // Build result
            let batch_result = match result {
                Ok(hash) => {
                    debug!(idx = idx + 1, hash_prefix = %&hash[..8.min(hash.len())], "File completed");
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: path.clone(),
                        status: "completed".to_string(),
                        percent: 100.0,
                        files_completed: idx + 1,
                        files_total: num_files,
                        hash: Some(hash.clone()),
                        algorithm: Some(algo.to_uppercase()),
                        error: None,
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path,
                        algorithm: algo.to_uppercase(),
                        hash: Some(hash),
                        error: None,
                        duration_ms: Some(duration_ms),
                        throughput_mbs,
                    }
                }
                Err(e) => {
                    debug!(idx = idx + 1, error = %e, "File error");
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: path.clone(),
                        status: "error".to_string(),
                        percent: 0.0,
                        files_completed: idx + 1,
                        files_total: num_files,
                        hash: None,
                        algorithm: None,
                        error: Some(e.clone()),
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path,
                        algorithm: algo.to_uppercase(),
                        hash: None,
                        error: Some(e),
                        duration_ms: Some(duration_ms),
                        throughput_mbs: None,
                    }
                }
            };
            
            Ok::<BatchHashResult, String>(batch_result)
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(num_files);
    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => results.push(result),
            Ok(Err(e)) => {
                debug!(error = %e, "Task error");
                // Error already emitted in the task
            }
            Err(e) => {
                debug!(error = %e, "Join error");
            }
        }
    }
    
    info!(num_files, results = results.len(), "Batch hash complete");
    Ok(results)
}

/// Smart batch hash using priority queue for optimal scheduling
/// 
/// Improvements over basic batch_hash:
/// - Priority scheduling: small files first for quick wins
/// - Better progress tracking with queue stats
/// - Adaptive concurrency based on workload type
/// - ETA prediction based on throughput
#[tauri::command]
#[instrument(skip(files, app), fields(num_files = files.len(), algorithm = %algorithm))]
pub async fn batch_hash_smart(
    files: Vec<BatchFileInput>,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<Vec<BatchHashResult>, String> {
    let num_files = files.len();
    info!("Starting smart batch hash");
    if num_files == 0 {
        return Ok(Vec::new());
    }
    
    // Create smart queue
    let queue = Arc::new(HashQueue::new());
    
    // Submit all jobs to queue
    for file in &files {
        queue.submit(
            file.path.clone(),
            file.container_type.clone(),
            algorithm.clone(),
        ).map_err(|e| format!("Failed to submit job: {}", e))?;
    }
    
    info!(queue_depth = queue.depth(), "All jobs queued");
    
    // Spawn worker tasks
    let mut handles = Vec::new();
    let results = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(num_files)));
    
    loop {
        // Check if we can start another worker
        if !queue.can_start_worker() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            continue;
        }
        
        // Get next job
        let job = match queue.next_job() {
            Some(j) => j,
            None => {
                // No more jobs
                if queue.active_worker_count() == 0 {
                    break; // All done
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        };
        
        let queue_clone = queue.clone();
        let app_clone = app.clone();
        let results_clone = results.clone();
        let job_path = job.path.clone();
        let job_algorithm = job.algorithm.clone();
        let job_size = job.file_size;
        
        queue.worker_started();
        
        let handle = tauri::async_runtime::spawn(async move {
            let job_start = std::time::Instant::now();
            
            debug!(job_id = job.job_id, path = %job.path, "Processing job");
            
            // Emit started
            let stats = queue_clone.get_stats();
            let _ = app_clone.emit("batch-progress", BatchProgress {
                path: job_path.clone(),
                status: "started".to_string(),
                percent: 0.0,
                files_completed: stats.jobs_completed,
                files_total: num_files,
                hash: None,
                algorithm: None,
                error: None,
                chunks_processed: None,
                chunks_total: None,
            });
            
            // Clone values for the blocking task
            let path_for_hash = job.path.clone();
            let algorithm_for_hash = job.algorithm.clone();
            let container_for_hash = job.container_type.to_lowercase();
            
            // Hash the file (blocking operation) - simplified without internal progress
            let hash_result = tauri::async_runtime::spawn_blocking(move || {
                let start_time = std::time::Instant::now();
                
                // Simple hash without progress callbacks
                let result: Result<String, String> = if container_for_hash.contains("e01") || container_for_hash.contains("encase") || container_for_hash.contains("ex01") {
                    ewf::verify_with_progress(&path_for_hash, &algorithm_for_hash, |_, _| {}).map_err(|e| e.to_string())
                } else if container_for_hash.contains("raw") || container_for_hash.contains("dd") {
                    raw::verify_with_progress(&path_for_hash, &algorithm_for_hash, |_, _| {}).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ufed") || container_for_hash.contains("zip") || container_for_hash.contains("archive") || container_for_hash.contains("tar") || container_for_hash.contains("7z") {
                    raw::verify_with_progress(&path_for_hash, &algorithm_for_hash, |_, _| {}).map_err(|e| e.to_string())
                } else if container_for_hash.contains("ad1") {
                    ad1::hash_segments_with_progress(&path_for_hash, &algorithm_for_hash, |_, _| {}).map_err(|e| e.to_string())
                } else if container_for_hash.contains("l01") {
                    containers::verify(&path_for_hash, &algorithm_for_hash)
                        .map(|entries| {
                            entries.first()
                                .and_then(|e| e.message.clone())
                                .unwrap_or_else(|| "Verified".to_string())
                        })
                } else {
                    raw::verify_with_progress(&path_for_hash, &algorithm_for_hash, |_, _| {}).map_err(|e| e.to_string())
                };
                
                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let throughput_mbs = if duration_ms > 0 && job_size > 0 {
                    Some((job_size as f64 / (1024.0 * 1024.0)) / (duration_ms as f64 / 1000.0))
                } else {
                    None
                };
                
                (result, duration_ms, throughput_mbs)
            }).await.map_err(|e| format!("Task error: {}", e))?;
            
            let (result, duration_ms, throughput_mbs) = hash_result;
            let job_duration = job_start.elapsed();
            
            // Update queue stats
            match &result {
                Ok(_) => queue_clone.job_completed(&job, job_duration),
                Err(e) => queue_clone.job_failed(&job, e),
            }
            
            // Build result
            let batch_result = match result {
                Ok(hash) => {
                    let stats = queue_clone.get_stats();
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: job_path.clone(),
                        status: "completed".to_string(),
                        percent: 100.0,
                        files_completed: stats.jobs_completed,
                        files_total: num_files,
                        hash: Some(hash.clone()),
                        algorithm: Some(job_algorithm.to_uppercase()),
                        error: None,
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path: job_path.clone(),
                        algorithm: job_algorithm.to_uppercase(),
                        hash: Some(hash),
                        error: None,
                        duration_ms: Some(duration_ms),
                        throughput_mbs,
                    }
                }
                Err(e) => {
                    let stats = queue_clone.get_stats();
                    let _ = app_clone.emit("batch-progress", BatchProgress {
                        path: job_path.clone(),
                        status: "error".to_string(),
                        percent: 0.0,
                        files_completed: stats.jobs_completed,
                        files_total: num_files,
                        hash: None,
                        algorithm: None,
                        error: Some(e.clone()),
                        chunks_processed: None,
                        chunks_total: None,
                    });
                    BatchHashResult {
                        path: job_path,
                        algorithm: job_algorithm.to_uppercase(),
                        hash: None,
                        error: Some(e),
                        duration_ms: Some(duration_ms),
                        throughput_mbs: None,
                    }
                }
            };
            
            // Store result
            let mut results = results_clone.lock().await;
            results.push(batch_result);
            
            queue_clone.worker_finished();
            Ok::<(), String>(())
        });
        
        handles.push(handle);
    }
    
    // Wait for all workers to finish
    for handle in handles {
        let _ = handle.await;
    }
    
    let final_results = results.lock().await.clone();
    let final_stats = queue.get_stats();
    
    info!(
        num_files,
        completed = final_stats.jobs_completed,
        failed = final_stats.jobs_failed,
        avg_throughput_mbs = final_stats.avg_throughput_mbs,
        "Smart batch hash complete"
    );
    
    Ok(final_results)
}

// Queue monitoring and control commands
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatsResponse {
    pub total_items: usize,
    pub completed_items: usize,
    pub active_items: usize,
    pub throughput_mbps: f64,
    pub estimated_seconds_remaining: Option<f64>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemResponse {
    pub file_path: String,
    pub priority: u8,
    pub size_bytes: u64,
    pub status: String,
    pub progress: f64,
    pub error_message: Option<String>,
}

/// Get current queue statistics
#[tauri::command]
pub async fn hash_queue_get_stats() -> Result<QueueStatsResponse, String> {
    // For now, return empty stats - this would integrate with a global queue state
    Ok(QueueStatsResponse {
        total_items: 0,
        completed_items: 0,
        active_items: 0,
        throughput_mbps: 0.0,
        estimated_seconds_remaining: None,
    })
}

/// Get all queue items
#[tauri::command]
pub async fn hash_queue_get_items() -> Result<Vec<QueueItemResponse>, String> {
    // For now, return empty - this would integrate with a global queue state
    Ok(Vec::new())
}

/// Pause queue processing
#[tauri::command]
pub async fn hash_queue_pause() -> Result<(), String> {
    // Placeholder - would integrate with global queue state
    Ok(())
}

/// Resume queue processing
#[tauri::command]
pub async fn hash_queue_resume() -> Result<(), String> {
    // Placeholder - would integrate with global queue state
    Ok(())
}

/// Clear completed items from queue
#[tauri::command]
pub async fn hash_queue_clear_completed() -> Result<(), String> {
    // Placeholder - would integrate with global queue state
    Ok(())
}
