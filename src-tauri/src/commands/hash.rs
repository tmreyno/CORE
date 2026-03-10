// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Parallel batch hashing operations for multiple files.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tracing::{debug, info, instrument};

use crate::ad1;
use crate::common::hash_cache;
use crate::common::health::QUEUE_METRICS;
use crate::ewf;
use crate::raw;

// =============================================================================
// Global Queue State
// =============================================================================

/// Global pause flag for hash queue operations.
/// When set to true, the `batch_hash_smart` worker loop will wait before
/// starting new jobs, effectively pausing the queue.
static QUEUE_PAUSED: AtomicBool = AtomicBool::new(false);

/// Check if the queue is currently paused
pub fn is_queue_paused() -> bool {
    QUEUE_PAUSED.load(Ordering::Relaxed)
}

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
    pub status: String, // "started", "progress", "completed", "error"
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

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a container type string represents an EWF-based format (E01, Ex01, L01)
fn is_ewf_type(container_type: &str) -> bool {
    container_type.contains("e01")
        || container_type.contains("encase")
        || container_type.contains("ex01")
        || container_type.contains("l01")
}

/// Check if a container type string represents an AD1 format
fn is_ad1_type(container_type: &str) -> bool {
    container_type.contains("ad1")
}

/// Spawn a progress reporter thread that periodically emits batch-progress events.
///
/// Emits an immediate 0% event, then polls every 250ms with 0.5% granularity.
/// Uses a 3-second heartbeat (1-second during startup) to prove the operation is alive.
fn spawn_progress_reporter(
    app: tauri::AppHandle,
    path: String,
    idx: usize,
    num_files: usize,
    progress_current: Arc<std::sync::atomic::AtomicUsize>,
    progress_total: Arc<std::sync::atomic::AtomicUsize>,
    done_flag: Arc<std::sync::atomic::AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_percent_key = 0u32;
        let mut last_emit = std::time::Instant::now();
        let heartbeat_interval = std::time::Duration::from_secs(3);
        let startup_heartbeat = std::time::Duration::from_secs(1);

        // Emit immediate 0% so the UI shows activity right away
        let _ = app.emit(
            "batch-progress",
            BatchProgress {
                path: path.clone(),
                status: "progress".to_string(),
                percent: 0.0,
                files_completed: idx,
                files_total: num_files,
                hash: None,
                algorithm: None,
                error: None,
                chunks_processed: Some(0),
                chunks_total: None,
            },
        );

        loop {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if done_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let current = progress_current.load(std::sync::atomic::Ordering::Relaxed);
            let total = progress_total.load(std::sync::atomic::Ordering::Relaxed);

            if total > 1 {
                let percent_f64 = (current as f64 / total as f64) * 100.0;
                let percent_key = (percent_f64 * 2.0) as u32; // 0.5% steps

                let should_emit =
                    percent_key > last_percent_key || last_emit.elapsed() >= heartbeat_interval;

                if should_emit {
                    let _ = app.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "progress".to_string(),
                            percent: percent_f64.min(100.0),
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: None,
                            chunks_processed: Some(current),
                            chunks_total: Some(total),
                        },
                    );
                    last_percent_key = percent_key;
                    last_emit = std::time::Instant::now();
                }
            } else {
                // Total not yet set — still emit heartbeat so frontend knows we're alive
                // Use shorter interval during startup (file open phase)
                if last_emit.elapsed() >= startup_heartbeat {
                    let _ = app.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "progress".to_string(),
                            percent: 0.0,
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: None,
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    last_emit = std::time::Instant::now();
                }
            }
        }
    })
}

/// Maximum concurrent hash I/O operations.
///
/// Hash verification is **I/O-bound** (reading from disk), not CPU-bound.
/// Using `num_cpus` as the concurrency limit causes severe I/O contention when
/// many large containers are on the same physical drive (especially external
/// USB drives or spinning disks). Each concurrent operation opens its own
/// set of segment file handles and does 16 MB batch reads, which degrades
/// into random I/O when multiple containers compete for the same drive head.
///
/// 3 concurrent provides a good balance:
/// - On HDD/USB: enough pipeline overlap without thrashing
/// - On SSD/NVMe: I/O controller handles concurrent reads well
/// - Memory: 3 × 64 MB sync_channel buffers = ~192 MB (manageable)
/// - File descriptors: 3 × 16 handles = 48 FDs (well under OS limits)
const MAX_IO_CONCURRENCY: usize = 3;

/// Hash multiple files in parallel with I/O-aware scheduling
///
/// Concurrency is capped at [`MAX_IO_CONCURRENCY`] (not CPU count) because
/// hash verification is I/O-bound. Too many concurrent reads from the same
/// drive causes thrashing and apparent hangs on large containers.
#[tauri::command]
#[instrument(skip(files, app), fields(num_files = files.len(), algorithm = %algorithm))]
pub async fn batch_hash(
    files: Vec<BatchFileInput>,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<Vec<BatchHashResult>, String> {
    let cmd_start = std::time::Instant::now();
    debug!("batch_hash command started");

    let num_files = files.len();
    info!("Starting parallel batch hash");
    if num_files == 0 {
        return Ok(Vec::new());
    }

    // I/O-bound concurrency limit — NOT cpu-based.
    // Hash operations spend most time reading from disk, not computing.
    // More than 3 concurrent reads on the same drive causes thrashing.
    let max_concurrent = MAX_IO_CONCURRENCY.min(num_files);
    info!(
        max_concurrent,
        num_files,
        max_io = MAX_IO_CONCURRENCY,
        "Batch hash concurrency (I/O-limited)"
    );
    debug!(
        elapsed_ms = cmd_start.elapsed().as_millis(),
        num_files, "Setup complete, spawning tasks"
    );

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
        let _ = app.emit(
            "batch-progress",
            BatchProgress {
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
            },
        );

        let handle = tauri::async_runtime::spawn(async move {
            // Wait while the queue is paused
            while QUEUE_PAUSED.load(Ordering::Relaxed) {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            // Acquire semaphore permit (limits concurrent files)
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Semaphore error: {}", e);
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path.clone(),
                            status: "error".to_string(),
                            percent: 0.0,
                            files_completed: idx,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: Some(err_msg.clone()),
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    return BatchHashResult {
                        path,
                        algorithm: algo.to_uppercase(),
                        hash: None,
                        error: Some(err_msg),
                        duration_ms: None,
                        throughput_mbs: None,
                    };
                }
            };

            debug!(idx = idx + 1, total = num_files, path = %path, "File started");

            // Emit progress: started
            let _ = app_clone.emit(
                "batch-progress",
                BatchProgress {
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
                },
            );

            let path_for_hash = path.clone();
            let algo_for_hash = algo.clone();
            let container_for_hash = container_type.clone();
            let app_for_hash = app_clone.clone();
            let path_for_error = path.clone();
            let algo_for_error = algo.clone();

            // Run blocking hash in spawn_blocking
            let hash_result = tauri::async_runtime::spawn_blocking(move || {
                let blocking_start = std::time::Instant::now();
                debug!(path = %path_for_hash, "spawn_blocking started");

                let start_time = std::time::Instant::now();
                let file_size = std::fs::metadata(&path_for_hash).map(|m| m.len()).unwrap_or(0);
                debug!(idx = idx + 1, size_mb = file_size / 1024 / 1024, "Processing file");
                debug!(elapsed_ms = blocking_start.elapsed().as_millis(), "File metadata read");

                // Progress counters
                let progress_current = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                let progress_total = Arc::new(std::sync::atomic::AtomicUsize::new(1)); // Start with 1 to avoid div by zero
                let done_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

                // Spawn progress reporter thread
                let progress_thread = spawn_progress_reporter(
                    app_for_hash.clone(),
                    path_for_hash.clone(),
                    idx,
                    num_files,
                    progress_current.clone(),
                    progress_total.clone(),
                    done_flag.clone(),
                );

                debug!(container_type = %container_for_hash, algorithm = %algo_for_hash, "About to start hashing");
                let _hash_start = std::time::Instant::now();

                // Check cache first - this can skip expensive recomputation
                let cached_hash = hash_cache::get_cached_hash(&path_for_hash, &algo_for_hash);

                // Hash based on container type (or use cached result)
                let result: Result<String, String> = if let Some(hash) = cached_hash {
                    debug!(path = %path_for_hash, algorithm = %algo_for_hash, "Cache hit");
                    // Signal 100% progress immediately for cached results
                    progress_total.store(1, std::sync::atomic::Ordering::Relaxed);
                    progress_current.store(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(hash)
                } else {
                    // Shared progress callback — all hash functions use the same pattern
                    let mut progress_cb = |current: u64, total: u64| {
                        progress_total.store(total as usize, std::sync::atomic::Ordering::Relaxed);
                        progress_current.store(current as usize, std::sync::atomic::Ordering::Relaxed);
                    };

                    // Route to the appropriate hash function (3 paths:
                    //   EWF → ewf::verify_with_progress
                    //   AD1 → ad1::hash_segments_with_progress
                    //   Everything else → raw::verify_with_progress)
                    if is_ewf_type(&container_for_hash) {
                        ewf::verify_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                            .map_err(|e| e.to_string())
                    } else if is_ad1_type(&container_for_hash) {
                        ad1::hash_segments_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                            .map_err(|e| e.to_string())
                    } else {
                        // Raw, UFED, archives, unknown — hash file bytes directly
                        raw::verify_with_progress(&path_for_hash, &algo_for_hash, &mut progress_cb)
                            .map_err(|e| e.to_string())
                    }
                };

                // Cache successful hash results for future lookups
                if let Ok(ref hash) = result {
                    hash_cache::cache_hash(&path_for_hash, &algo_for_hash, hash.clone());
                }

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
            }).await;

            // Handle spawn_blocking failure (panics) — always emit error event
            let (result, duration_ms, throughput_mbs) = match hash_result {
                Ok(r) => r,
                Err(e) => {
                    let err_msg = format!("Internal hash error: {}", e);
                    debug!(error = %err_msg, "spawn_blocking failed");
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
                            path: path_for_error.clone(),
                            status: "error".to_string(),
                            percent: 0.0,
                            files_completed: idx + 1,
                            files_total: num_files,
                            hash: None,
                            algorithm: None,
                            error: Some(err_msg.clone()),
                            chunks_processed: None,
                            chunks_total: None,
                        },
                    );
                    return BatchHashResult {
                        path: path_for_error,
                        algorithm: algo_for_error.to_uppercase(),
                        hash: None,
                        error: Some(err_msg),
                        duration_ms: None,
                        throughput_mbs: None,
                    };
                }
            };

            // Build result
            let batch_result = match result {
                Ok(hash) => {
                    debug!(idx = idx + 1, hash_prefix = %&hash[..8.min(hash.len())], "File completed");
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
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
                        },
                    );
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
                    let _ = app_clone.emit(
                        "batch-progress",
                        BatchProgress {
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
                        },
                    );
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

            batch_result
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete and collect results
    let mut results = Vec::with_capacity(num_files);
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(e) => {
                // Extremely rare: the outer spawn itself panicked
                debug!(error = %e, "Outer task join error");
            }
        }
    }

    info!(num_files, results = results.len(), "Batch hash complete");
    Ok(results)
}

/// Pause queue processing
///
/// Sets a global pause flag that prevents batch operations from starting
/// new jobs. Jobs already in progress will continue to completion.
#[tauri::command]
pub async fn hash_queue_pause() -> Result<(), String> {
    QUEUE_PAUSED.store(true, Ordering::Relaxed);
    info!("Hash queue paused");
    Ok(())
}

/// Resume queue processing
///
/// Clears the global pause flag, allowing `batch_hash_smart` to resume
/// dispatching new jobs from the queue.
#[tauri::command]
pub async fn hash_queue_resume() -> Result<(), String> {
    QUEUE_PAUSED.store(false, Ordering::Relaxed);
    info!("Hash queue resumed");
    Ok(())
}

/// Clear completed items from queue
///
/// Resets the global queue metrics counters (completed/failed counts,
/// throughput tracking). Active and pending jobs are unaffected.
#[tauri::command]
pub async fn hash_queue_clear_completed() -> Result<(), String> {
    // Reset the completed/failed counters while preserving active state
    let submitted = QUEUE_METRICS.jobs_submitted.load(Ordering::Relaxed);
    let completed = QUEUE_METRICS.jobs_completed.load(Ordering::Relaxed);
    let failed = QUEUE_METRICS.jobs_failed.load(Ordering::Relaxed);

    // Subtract completed+failed from submitted to keep only pending count accurate
    let pending = submitted.saturating_sub(completed + failed);
    QUEUE_METRICS
        .jobs_submitted
        .store(pending, Ordering::Relaxed);
    QUEUE_METRICS.jobs_completed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.jobs_failed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.bytes_processed.store(0, Ordering::Relaxed);
    QUEUE_METRICS.processing_time_ms.store(0, Ordering::Relaxed);

    info!(
        cleared_completed = completed,
        cleared_failed = failed,
        remaining_pending = pending,
        "Hash queue completed items cleared"
    );
    Ok(())
}
