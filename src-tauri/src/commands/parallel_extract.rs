// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Parallel file extraction with progress streaming and integrity verification
//!
//! Features:
//! - Tokio-based async parallel extraction
//! - Real-time progress streaming via Tauri events
//! - Automatic hash verification during extraction
//! - Rate limiting to prevent disk saturation
//! - Graceful cancellation support

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, Mutex};
use tokio::io::AsyncWriteExt;
use tauri::{Emitter, Runtime};
use tracing::{error, info, instrument};

use crate::containers::unified::ContainerType;

// Extraction job status
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExtractionStatus {
    Queued,
    Extracting,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

// Individual file extraction job
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionJob {
    pub id: String,
    pub source_path: String,
    pub entry_path: String,
    pub destination_path: String,
    pub size_bytes: u64,
    pub status: ExtractionStatus,
    pub bytes_extracted: u64,
    pub percent_complete: f64,
    pub hash_algorithm: Option<String>,
    pub computed_hash: Option<String>,
    pub expected_hash: Option<String>,
    pub error_message: Option<String>,
}

// Batch extraction progress
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchExtractionProgress {
    pub batch_id: String,
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub cancelled_files: usize,
    pub total_bytes: u64,
    pub extracted_bytes: u64,
    pub percent_complete: f64,
    pub throughput_mbps: f64,
    pub estimated_seconds_remaining: Option<f64>,
    pub active_jobs: Vec<ExtractionJob>,
}

/// Configuration for batch extraction operations
#[derive(Clone, Debug)]
pub struct BatchExtractionConfig {
    pub batch_id: String,
    pub container_path: String,
    pub container_type: ContainerType,
    /// Entries: (entry_path, dest_path, size, expected_hash)
    pub entries: Vec<(String, String, u64, Option<String>)>,
    pub destination_base: String,
    pub hash_algorithm: Option<String>,
    pub max_concurrent: usize,
}

// Extraction manager state
pub struct ParallelExtractor {
    jobs: Arc<RwLock<Vec<ExtractionJob>>>,
    active_batches: Arc<RwLock<Vec<String>>>,
    cancelled_batches: Arc<Mutex<Vec<String>>>,
}

impl Default for ParallelExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelExtractor {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(Vec::new())),
            active_batches: Arc::new(RwLock::new(Vec::new())),
            cancelled_batches: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Extract multiple files in parallel
    #[instrument(skip(self, config, window), fields(batch_id = %config.batch_id))]
    pub async fn extract_batch<R: Runtime>(
        &self,
        config: BatchExtractionConfig,
        window: tauri::Window<R>,
    ) -> Result<(), String> {
        info!(
            batch_id = %config.batch_id,
            num_files = config.entries.len(),
            "Starting parallel extraction"
        );

        // Register batch
        {
            let mut batches = self.active_batches.write().await;
            batches.push(config.batch_id.clone());
        }

        // Create jobs
        let jobs: Vec<ExtractionJob> = config.entries
            .into_iter()
            .enumerate()
            .map(|(idx, (entry_path, dest_path, size, expected_hash))| ExtractionJob {
                id: format!("{}-{}", config.batch_id, idx),
                source_path: config.container_path.clone(),
                entry_path,
                destination_path: dest_path,
                size_bytes: size,
                status: ExtractionStatus::Queued,
                bytes_extracted: 0,
                percent_complete: 0.0,
                hash_algorithm: config.hash_algorithm.clone(),
                computed_hash: None,
                expected_hash,
                error_message: None,
            })
            .collect();

        let total_files = jobs.len();
        let total_bytes: u64 = jobs.iter().map(|j| j.size_bytes).sum();
        let batch_id = config.batch_id.clone();
        let container_path = config.container_path.clone();
        let container_type = config.container_type;
        let destination_base = config.destination_base.clone();
        let _hash_algorithm = config.hash_algorithm; // Used in job creation, not needed here
        let max_concurrent = config.max_concurrent;

        // Store jobs
        {
            let mut job_list = self.jobs.write().await;
            job_list.extend(jobs.clone());
        }

        // Emit initial progress
        let _ = window.emit(
            "batch-extraction-progress",
            BatchExtractionProgress {
                batch_id: batch_id.clone(),
                total_files,
                completed_files: 0,
                failed_files: 0,
                cancelled_files: 0,
                total_bytes,
                extracted_bytes: 0,
                percent_complete: 0.0,
                throughput_mbps: 0.0,
                estimated_seconds_remaining: None,
                active_jobs: vec![],
            },
        );

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let start_time = std::time::Instant::now();

        // Spawn extraction tasks
        let mut handles = Vec::new();
        for job in jobs {
            let sem = Arc::clone(&semaphore);
            let job_id = job.id.clone();
            let batch_id_clone = batch_id.clone();
            let container_path_clone = container_path.clone();
            let container_type_clone = container_type;
            let destination_base_clone = destination_base.clone();
            let window_clone = window.clone();
            let jobs_clone = Arc::clone(&self.jobs);
            let cancelled_clone = Arc::clone(&self.cancelled_batches);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // Check if batch cancelled
                {
                    let cancelled = cancelled_clone.lock().await;
                    if cancelled.contains(&batch_id_clone) {
                        Self::update_job_status(
                            &jobs_clone,
                            &job_id,
                            ExtractionStatus::Cancelled,
                            None,
                        )
                        .await;
                        return;
                    }
                }

                // Extract file
                match Self::extract_single_file(
                    &jobs_clone,
                    &job_id,
                    &container_path_clone,
                    container_type_clone,
                    &destination_base_clone,
                    &window_clone,
                )
                .await
                {
                    Ok(_) => {
                        Self::update_job_status(
                            &jobs_clone,
                            &job_id,
                            ExtractionStatus::Completed,
                            None,
                        )
                        .await;
                    }
                    Err(e) => {
                        error!(job_id, error = %e, "Extraction failed");
                        Self::update_job_status(
                            &jobs_clone,
                            &job_id,
                            ExtractionStatus::Failed,
                            Some(e),
                        )
                        .await;
                    }
                }
            });

            handles.push(handle);
        }

        // Progress monitoring task
        let jobs_monitor = Arc::clone(&self.jobs);
        let batch_id_monitor = batch_id.clone();
        let window_monitor = window.clone();
        let cancelled_monitor = Arc::clone(&self.cancelled_batches);

        tokio::spawn(async move {
            let mut last_extracted_bytes = 0u64;
            let mut last_update = start_time;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Check if cancelled
                {
                    let cancelled = cancelled_monitor.lock().await;
                    if cancelled.contains(&batch_id_monitor) {
                        break;
                    }
                }

                // Calculate progress
                let jobs = jobs_monitor.read().await;
                let batch_jobs: Vec<_> = jobs
                    .iter()
                    .filter(|j| j.id.starts_with(&batch_id_monitor))
                    .cloned()
                    .collect();

                if batch_jobs.is_empty() {
                    break;
                }

                let completed = batch_jobs
                    .iter()
                    .filter(|j| matches!(j.status, ExtractionStatus::Completed))
                    .count();
                let failed = batch_jobs
                    .iter()
                    .filter(|j| matches!(j.status, ExtractionStatus::Failed))
                    .count();
                let cancelled_count = batch_jobs
                    .iter()
                    .filter(|j| matches!(j.status, ExtractionStatus::Cancelled))
                    .count();

                let extracted_bytes: u64 = batch_jobs.iter().map(|j| j.bytes_extracted).sum();
                let active_jobs: Vec<_> = batch_jobs
                    .iter()
                    .filter(|j| {
                        matches!(
                            j.status,
                            ExtractionStatus::Extracting | ExtractionStatus::Verifying
                        )
                    })
                    .cloned()
                    .collect();

                // Calculate throughput
                let now = std::time::Instant::now();
                let elapsed = (now - last_update).as_secs_f64();
                let bytes_delta = extracted_bytes.saturating_sub(last_extracted_bytes);
                let throughput_mbps = if elapsed > 0.0 {
                    (bytes_delta as f64 / elapsed) / (1024.0 * 1024.0)
                } else {
                    0.0
                };

                // Calculate ETA
                let remaining_bytes = total_bytes.saturating_sub(extracted_bytes);
                let eta = if throughput_mbps > 0.0 {
                    Some(remaining_bytes as f64 / (throughput_mbps * 1024.0 * 1024.0))
                } else {
                    None
                };

                // Emit progress
                let _ = window_monitor.emit(
                    "batch-extraction-progress",
                    BatchExtractionProgress {
                        batch_id: batch_id_monitor.clone(),
                        total_files,
                        completed_files: completed,
                        failed_files: failed,
                        cancelled_files: cancelled_count,
                        total_bytes,
                        extracted_bytes,
                        percent_complete: if total_bytes > 0 {
                            (extracted_bytes as f64 / total_bytes as f64) * 100.0
                        } else {
                            0.0
                        },
                        throughput_mbps,
                        estimated_seconds_remaining: eta,
                        active_jobs,
                    },
                );

                last_extracted_bytes = extracted_bytes;
                last_update = now;

                // Break if all done
                if completed + failed + cancelled_count >= total_files {
                    break;
                }
            }
        });

        // Wait for all tasks
        for handle in handles {
            let _ = handle.await;
        }

        // Cleanup
        {
            let mut batches = self.active_batches.write().await;
            batches.retain(|b| b != &batch_id);
        }

        info!(batch_id, "Parallel extraction completed");
        Ok(())
    }

    /// Extract a single file
    async fn extract_single_file<R: Runtime>(
        jobs: &Arc<RwLock<Vec<ExtractionJob>>>,
        job_id: &str,
        container_path: &str,
        container_type: ContainerType,
        destination_base: &str,
        _window: &tauri::Window<R>,
    ) -> Result<(), String> {
        // Get job details
        let (entry_path, destination_path, hash_algorithm, expected_hash) = {
            let jobs_read = jobs.read().await;
            let job = jobs_read
                .iter()
                .find(|j| j.id == job_id)
                .ok_or("Job not found")?;
            (
                job.entry_path.clone(),
                job.destination_path.clone(),
                job.hash_algorithm.clone(),
                job.expected_hash.clone(),
            )
        };

        // Update status to extracting
        Self::update_job_status(jobs, job_id, ExtractionStatus::Extracting, None).await;

        // Create destination directory
        let dest_path = PathBuf::from(destination_base).join(&destination_path);
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Open container and read file
        let data = Self::read_from_container(container_path, &entry_path, container_type).await?;

        // Write to destination
        let mut file = tokio::fs::File::create(&dest_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        file.write_all(&data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        file.sync_all()
            .await
            .map_err(|e| format!("Failed to sync file: {}", e))?;

        // Update bytes extracted
        {
            let mut jobs_write = jobs.write().await;
            if let Some(job) = jobs_write.iter_mut().find(|j| j.id == job_id) {
                job.bytes_extracted = data.len() as u64;
                job.percent_complete = 100.0;
            }
        }

        // Verify hash if requested
        if let Some(algorithm_str) = hash_algorithm {
            Self::update_job_status(jobs, job_id, ExtractionStatus::Verifying, None).await;

            // Simplified hash verification using file read
            let file_data = tokio::fs::read(&dest_path)
                .await
                .map_err(|e| format!("Failed to read file for hashing: {}", e))?;
            
            // Compute hash using the algorithm string
            let computed = match algorithm_str.to_uppercase().as_str() {
                "SHA-1" | "SHA1" => {
                    use sha1::Digest;
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(&file_data);
                    format!("{:x}", hasher.finalize())
                }
                "SHA-256" | "SHA256" => {
                    use sha2::Digest;
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(&file_data);
                    format!("{:x}", hasher.finalize())
                }
                "MD5" => {
                    use md5::Digest;
                    let mut hasher = md5::Md5::new();
                    hasher.update(&file_data);
                    format!("{:x}", hasher.finalize())
                }
                _ => return Err(format!("Unsupported hash algorithm: {}", algorithm_str)),
            };

            // Store computed hash
            {
                let mut jobs_write = jobs.write().await;
                if let Some(job) = jobs_write.iter_mut().find(|j| j.id == job_id) {
                    job.computed_hash = Some(computed.clone());
                }
            }

            // Verify against expected hash
            if let Some(expected) = expected_hash {
                if computed.to_lowercase() != expected.to_lowercase() {
                    return Err(format!(
                        "Hash mismatch: expected {}, got {}",
                        expected, computed
                    ));
                }
            }
        }

        Ok(())
    }

    /// Read file data from container
    async fn read_from_container(
        container_path: &str,
        entry_path: &str,
        container_type: ContainerType,
    ) -> Result<Vec<u8>, String> {
        // Use the appropriate container reader based on type
        tokio::task::spawn_blocking({
            let path = container_path.to_string();
            let entry = entry_path.to_string();
            move || {
                match container_type {
                    ContainerType::Ad1 => {
                        // AD1 has dedicated entry reading support
                        crate::ad1::read_entry_data(&path, &entry)
                            .map_err(|e| format!("AD1 read failed: {}", e))
                    }
                    ContainerType::Zip | ContainerType::SevenZip => {
                        // Use libarchive backend for archive formats
                        let handler = crate::archive::LibarchiveHandler::new(&path);
                        handler.read_entry(&entry)
                            .map_err(|e| format!("Archive entry read failed: {}", e))
                    }
                    ContainerType::Ewf => {
                        // EWF contains disk images — entry_path points to
                        // a file on the mounted/extracted filesystem
                        std::fs::read(&entry)
                            .map_err(|e| format!("EWF source read failed: {}", e))
                    }
                    ContainerType::Ufed => {
                        // UFED entries are accessible via the filesystem path
                        std::fs::read(&entry)
                            .map_err(|e| format!("UFED source read failed: {}", e))
                    }
                    _ => {
                        // Unknown type — try AD1 first, then fall back to direct read
                        if let Ok(data) = crate::ad1::read_entry_data(&path, &entry) {
                            return Ok(data);
                        }
                        std::fs::read(&entry)
                            .map_err(|e| format!("File read failed: {}", e))
                    }
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    /// Update job status
    async fn update_job_status(
        jobs: &Arc<RwLock<Vec<ExtractionJob>>>,
        job_id: &str,
        status: ExtractionStatus,
        error: Option<String>,
    ) {
        let mut jobs_write = jobs.write().await;
        if let Some(job) = jobs_write.iter_mut().find(|j| j.id == job_id) {
            job.status = status;
            if let Some(err) = error {
                job.error_message = Some(err);
            }
        }
    }

    /// Cancel batch extraction
    pub async fn cancel_batch(&self, batch_id: String) -> Result<(), String> {
        let mut cancelled = self.cancelled_batches.lock().await;
        if !cancelled.contains(&batch_id) {
            cancelled.push(batch_id.clone());
        }
        info!(batch_id, "Batch extraction cancelled");
        Ok(())
    }

    /// Get active batches
    pub async fn get_active_batches(&self) -> Result<Vec<String>, String> {
        let batches = self.active_batches.read().await;
        Ok(batches.clone())
    }
}

// Tauri commands
use tauri::State;

pub struct ParallelExtractorState(pub Arc<tokio::sync::Mutex<Option<ParallelExtractor>>>);

#[tauri::command]
pub async fn parallel_extract_init(
    state: State<'_, ParallelExtractorState>,
) -> Result<(), String> {
    let mut extractor_opt = state.0.lock().await;
    *extractor_opt = Some(ParallelExtractor::new());
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionEntry {
    pub entry_path: String,
    pub destination_path: String,
    pub size_bytes: u64,
    pub expected_hash: Option<String>,
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri command with injected window/state
pub async fn parallel_extract_batch(
    batch_id: String,
    container_path: String,
    container_type: String,
    entries: Vec<ExtractionEntry>,
    destination_base: String,
    hash_algorithm: Option<String>,
    max_concurrent: Option<usize>,
    window: tauri::Window,
    state: State<'_, ParallelExtractorState>,
) -> Result<(), String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Parallel extractor not initialized")?;

    // Parse container type - default to Ad1 if unknown
    let container_type_enum = match container_type.to_lowercase().as_str() {
        "ad1" => ContainerType::Ad1,
        "e01" | "ex01" | "ewf" | "l01" | "lx01" => ContainerType::Ewf,
        "ufed" | "ufd" => ContainerType::Ufed,
        "zip" => ContainerType::Zip,
        "7z" => ContainerType::SevenZip,
        _ => ContainerType::Ad1, // Default
    };

    let entries_tuple: Vec<_> = entries
        .into_iter()
        .map(|e| {
            (
                e.entry_path,
                e.destination_path,
                e.size_bytes,
                e.expected_hash,
            )
        })
        .collect();

    let config = BatchExtractionConfig {
        batch_id,
        container_path,
        container_type: container_type_enum,
        entries: entries_tuple,
        destination_base,
        hash_algorithm,
        max_concurrent: max_concurrent.unwrap_or(4),
    };

    extractor
        .extract_batch(config, window)
        .await
}

#[tauri::command]
pub async fn parallel_extract_cancel(
    batch_id: String,
    state: State<'_, ParallelExtractorState>,
) -> Result<(), String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Parallel extractor not initialized")?;

    extractor.cancel_batch(batch_id).await
}

#[tauri::command]
pub async fn parallel_extract_get_active(
    state: State<'_, ParallelExtractorState>,
) -> Result<Vec<String>, String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Parallel extractor not initialized")?;

    extractor.get_active_batches().await
}
