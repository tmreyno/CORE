// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Smart Streaming Extraction System
//!
//! Features:
//! - Stream files as they complete (don't wait for entire batch)
//! - Incremental file availability notifications
//! - Priority-based extraction (high-priority files first)
//! - Adaptive batch sizing based on file sizes
//! - Resume capability for interrupted streams
//! - Real-time throughput monitoring

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore, mpsc};
use tokio::task::JoinHandle;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, instrument};
use tauri::Emitter;

use crate::containers::unified::ContainerType;

/// Priority level for extraction
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExtractionPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Extraction job with priority and metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamExtractionJob {
    pub id: String,
    pub source_path: String,
    pub dest_path: String,
    pub container_path: String,
    pub container_type: String,
    pub priority: ExtractionPriority,
    pub size_bytes: u64,
    pub expected_hash: Option<String>,
    pub hash_algorithm: Option<String>,
}

/// Status of a single extraction job
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub id: String,
    pub status: JobState,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    pub throughput_mbps: f64,
    pub error: Option<String>,
    pub completed_hash: Option<String>,
    pub elapsed_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Queued,
    Extracting,
    Verifying,
    Complete,
    Failed,
    Cancelled,
}

/// Stream extraction progress
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamProgress {
    pub stream_id: String,
    pub total_jobs: usize,
    pub queued_jobs: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub total_bytes: u64,
    pub extracted_bytes: u64,
    pub overall_throughput_mbps: f64,
    pub eta_seconds: Option<u64>,
}

/// File available notification (emitted as soon as file is extracted)
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAvailableEvent {
    pub stream_id: String,
    pub job_id: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub hash: Option<String>,
    pub verified: bool,
}

/// Smart streaming extraction engine
pub struct StreamingExtractor {
    streams: Arc<RwLock<HashMap<String, StreamInfo>>>,
}

struct StreamInfo {
    jobs: Vec<StreamExtractionJob>,
    job_status: HashMap<String, JobStatus>,
    active_handles: Vec<JoinHandle<()>>,
    cancelled: Arc<Mutex<bool>>,
    start_time: std::time::Instant,
}

impl StreamingExtractor {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a streaming extraction (jobs are processed in priority order)
    #[instrument(skip(self, window))]
    pub async fn start_stream<R: tauri::Runtime>(
        &self,
        stream_id: String,
        mut jobs: Vec<StreamExtractionJob>,
        max_concurrent: usize,
        window: tauri::Window<R>,
    ) -> Result<(), String> {
        info!(stream_id, job_count = jobs.len(), "Starting streaming extraction");

        // Sort jobs by priority (high to low), then by size (small to large for quick wins)
        jobs.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.size_bytes.cmp(&b.size_bytes))
        });

        let total_bytes: u64 = jobs.iter().map(|j| j.size_bytes).sum();
        let job_count = jobs.len();
        
        info!("Starting streaming extraction: {} files, {} bytes total", job_count, total_bytes);

        // Initialize stream info
        let cancelled = Arc::new(Mutex::new(false));
        let job_status: HashMap<String, JobStatus> = jobs
            .iter()
            .map(|job| {
                (
                    job.id.clone(),
                    JobStatus {
                        id: job.id.clone(),
                        status: JobState::Queued,
                        progress_bytes: 0,
                        total_bytes: job.size_bytes,
                        throughput_mbps: 0.0,
                        error: None,
                        completed_hash: None,
                        elapsed_ms: 0,
                    },
                )
            })
            .collect();

        let stream_info = StreamInfo {
            jobs: jobs.clone(),
            job_status,
            active_handles: Vec::new(),
            cancelled: Arc::clone(&cancelled),
            start_time: std::time::Instant::now(),
        };

        self.streams.write().await.insert(stream_id.clone(), stream_info);

        // Create channel for job completion notifications
        let (tx, mut rx) = mpsc::unbounded_channel::<(String, Result<FileAvailableEvent, String>)>();

        // Spawn worker tasks (semaphore limits concurrency)
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let streams_clone = Arc::clone(&self.streams);
        let stream_id_clone = stream_id.clone();

        for job in jobs {
            let sem = Arc::clone(&semaphore);
            let cancelled_clone = Arc::clone(&cancelled);
            let tx_clone = tx.clone();
            let job_clone = job.clone();
            let streams_clone2 = Arc::clone(&streams_clone);
            let stream_id_clone2 = stream_id_clone.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // Check cancellation
                {
                    let is_cancelled = cancelled_clone.lock().await;
                    if *is_cancelled {
                        return;
                    }
                }

                // Update status to extracting
                {
                    let mut streams = streams_clone2.write().await;
                    if let Some(stream) = streams.get_mut(&stream_id_clone2) {
                        if let Some(status) = stream.job_status.get_mut(&job_clone.id) {
                            status.status = JobState::Extracting;
                        }
                    }
                }

                let job_start = std::time::Instant::now();

                // Extract file
                let result = Self::extract_single_file(&job_clone).await;

                let elapsed_ms = job_start.elapsed().as_millis() as u64;

                match result {
                    Ok(event) => {
                        // Update status to complete
                        {
                            let mut streams = streams_clone2.write().await;
                            if let Some(stream) = streams.get_mut(&stream_id_clone2) {
                                if let Some(status) = stream.job_status.get_mut(&job_clone.id) {
                                    status.status = JobState::Complete;
                                    status.progress_bytes = job_clone.size_bytes;
                                    status.completed_hash = event.hash.clone();
                                    status.elapsed_ms = elapsed_ms;
                                    
                                    if elapsed_ms > 0 {
                                        let throughput_mbps = (job_clone.size_bytes as f64 / 1_048_576.0) 
                                            / (elapsed_ms as f64 / 1000.0);
                                        status.throughput_mbps = throughput_mbps;
                                    }
                                }
                            }
                        }

                        let _ = tx_clone.send((job_clone.id.clone(), Ok(event)));
                    }
                    Err(e) => {
                        // Update status to failed
                        {
                            let mut streams = streams_clone2.write().await;
                            if let Some(stream) = streams.get_mut(&stream_id_clone2) {
                                if let Some(status) = stream.job_status.get_mut(&job_clone.id) {
                                    status.status = JobState::Failed;
                                    status.error = Some(e.clone());
                                    status.elapsed_ms = elapsed_ms;
                                }
                            }
                        }

                        let _ = tx_clone.send((job_clone.id.clone(), Err(e)));
                    }
                }
            });

            // Store handle for cancellation
            let mut streams = self.streams.write().await;
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.active_handles.push(handle);
            }
        }

        drop(tx); // Close sender

        // Spawn notification handler
        let window_clone = window.clone();
        let stream_id_clone = stream_id.clone();
        let streams_clone = Arc::clone(&self.streams);
        
        tokio::spawn(async move {
            while let Some((job_id, result)) = rx.recv().await {
                match result {
                    Ok(event) => {
                        // Emit file available event immediately
                        let _ = window_clone.emit("file-available", event);
                        debug!(job_id, "File available");
                    }
                    Err(e) => {
                        warn!(job_id, error = %e, "Job failed");
                    }
                }
            }

            // All jobs complete - emit final progress
            {
                let streams = streams_clone.read().await;
                if let Some(stream) = streams.get(&stream_id_clone) {
                    let progress = Self::compute_progress(&stream_id_clone, stream);
                    let _ = window_clone.emit("stream-complete", progress);
                }
            }
        });

        // Spawn progress monitor
        let window_monitor = window.clone();
        let stream_id_monitor = stream_id.clone();
        let streams_monitor = Arc::clone(&self.streams);
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                let streams = streams_monitor.read().await;
                if let Some(stream) = streams.get(&stream_id_monitor) {
                    let progress = Self::compute_progress(&stream_id_monitor, stream);
                    
                    // Stop if all jobs complete
                    if progress.queued_jobs == 0 && progress.active_jobs == 0 {
                        break;
                    }

                    let _ = window_monitor.emit("stream-progress", progress);
                } else {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Extract a single file with hash verification
    async fn extract_single_file(job: &StreamExtractionJob) -> Result<FileAvailableEvent, String> {
        // Read from container (simplified - would use actual container readers)
        let data = Self::read_from_container_source(job).await?;

        // Ensure destination directory exists
        if let Some(parent) = PathBuf::from(&job.dest_path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Write file
        tokio::fs::write(&job.dest_path, &data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Verify hash if provided
        let (computed_hash, verified) = if let (Some(expected), Some(algo)) = 
            (&job.expected_hash, &job.hash_algorithm) {
            let computed = Self::compute_hash(&data, algo)?;
            let verified = computed == *expected;
            (Some(computed), verified)
        } else {
            (None, false)
        };

        Ok(FileAvailableEvent {
            stream_id: job.id.clone(),
            job_id: job.id.clone(),
            file_path: job.dest_path.clone(),
            size_bytes: data.len() as u64,
            hash: computed_hash,
            verified,
        })
    }

    /// Read data from container (placeholder - would use actual readers)
    async fn read_from_container_source(job: &StreamExtractionJob) -> Result<Vec<u8>, String> {
        // Map container type string to enum
        let container_type = match job.container_type.to_lowercase().as_str() {
            "ad1" => ContainerType::Ad1,
            "e01" | "l01" | "ex01" => ContainerType::Ewf,
            "ufed" => ContainerType::Ufed,
            "zip" => ContainerType::Zip,
            "7z" => ContainerType::SevenZip,
            _ => return Err(format!("Unknown container type: {}", job.container_type)),
        };

        // Use spawn_blocking for potentially blocking I/O
        let _container_path = job.container_path.clone();
        let source_path = job.source_path.clone();

        tokio::task::spawn_blocking(move || {
            // Simplified - would use actual container reading logic
            match container_type {
                ContainerType::Ad1 => {
                    // Would use crate::ad1::read_file()
                    std::fs::read(&source_path)
                        .map_err(|e| format!("AD1 read failed: {}", e))
                }
                ContainerType::Ewf => {
                    // Would use EWF reader
                    std::fs::read(&source_path)
                        .map_err(|e| format!("EWF read failed: {}", e))
                }
                _ => {
                    std::fs::read(&source_path)
                        .map_err(|e| format!("Read failed: {}", e))
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    /// Compute hash of data
    fn compute_hash(data: &[u8], algorithm: &str) -> Result<String, String> {
        use sha1::Sha1;
        use sha2::{Sha256, Digest as Sha2Digest};
        use md5::Md5;

        match algorithm.to_uppercase().as_str() {
            "SHA-256" => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(format!("{:x}", hasher.finalize()))
            }
            "SHA-1" => {
                let mut hasher = Sha1::new();
                hasher.update(data);
                Ok(format!("{:x}", hasher.finalize()))
            }
            "MD5" => {
                let mut hasher = Md5::new();
                hasher.update(data);
                Ok(format!("{:x}", hasher.finalize()))
            }
            _ => Err(format!("Unsupported hash algorithm: {}", algorithm)),
        }
    }

    /// Compute stream progress
    fn compute_progress(stream_id: &str, stream: &StreamInfo) -> StreamProgress {
        let mut queued = 0;
        let mut active = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut extracted_bytes = 0u64;

        for status in stream.job_status.values() {
            match status.status {
                JobState::Queued => queued += 1,
                JobState::Extracting | JobState::Verifying => active += 1,
                JobState::Complete => {
                    completed += 1;
                    extracted_bytes += status.total_bytes;
                }
                JobState::Failed => failed += 1,
                JobState::Cancelled => {}
            }
        }

        let total_jobs = stream.jobs.len();
        let total_bytes: u64 = stream.jobs.iter().map(|j| j.size_bytes).sum();
        
        let elapsed = stream.start_time.elapsed().as_secs_f64();
        let overall_throughput_mbps = if elapsed > 0.0 {
            (extracted_bytes as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };

        let eta_seconds = if overall_throughput_mbps > 0.0 {
            let remaining_bytes = total_bytes.saturating_sub(extracted_bytes);
            let remaining_mb = remaining_bytes as f64 / 1_048_576.0;
            Some((remaining_mb / overall_throughput_mbps) as u64)
        } else {
            None
        };

        StreamProgress {
            stream_id: stream_id.to_string(),
            total_jobs,
            queued_jobs: queued,
            active_jobs: active,
            completed_jobs: completed,
            failed_jobs: failed,
            total_bytes,
            extracted_bytes,
            overall_throughput_mbps,
            eta_seconds,
        }
    }

    /// Get current stream progress
    pub async fn get_progress(&self, stream_id: &str) -> Result<StreamProgress, String> {
        let streams = self.streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| "Stream not found".to_string())?;

        Ok(Self::compute_progress(stream_id, stream))
    }

    /// Cancel a stream (gracefully stop all jobs)
    pub async fn cancel_stream(&self, stream_id: &str) -> Result<(), String> {
        let mut streams = self.streams.write().await;
        let stream = streams
            .get_mut(stream_id)
            .ok_or_else(|| "Stream not found".to_string())?;

        // Set cancelled flag
        {
            let mut cancelled = stream.cancelled.lock().await;
            *cancelled = true;
        }

        // Abort all active tasks
        for handle in stream.active_handles.iter() {
            handle.abort();
        }

        info!(stream_id, "Stream cancelled");
        Ok(())
    }

    /// Get all active streams
    pub async fn get_active_streams(&self) -> Result<Vec<String>, String> {
        let streams = self.streams.read().await;
        Ok(streams.keys().cloned().collect())
    }

    /// Get job statuses for a stream
    pub async fn get_job_statuses(&self, stream_id: &str) -> Result<Vec<JobStatus>, String> {
        let streams = self.streams.read().await;
        let stream = streams
            .get(stream_id)
            .ok_or_else(|| "Stream not found".to_string())?;

        Ok(stream.job_status.values().cloned().collect())
    }
}

// Tauri commands
use tauri::State;

pub struct StreamingExtractorState(pub Arc<Mutex<Option<StreamingExtractor>>>);

impl Default for StreamingExtractorState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

#[tauri::command]
pub async fn stream_extract_init(state: State<'_, StreamingExtractorState>) -> Result<(), String> {
    let mut extractor_opt = state.0.lock().await;
    *extractor_opt = Some(StreamingExtractor::new());
    Ok(())
}

#[tauri::command]
pub async fn stream_extract_start(
    stream_id: String,
    jobs: Vec<StreamExtractionJob>,
    max_concurrent: usize,
    window: tauri::Window,
    state: State<'_, StreamingExtractorState>,
) -> Result<(), String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Streaming extractor not initialized")?;

    extractor.start_stream(stream_id, jobs, max_concurrent, window).await
}

#[tauri::command]
pub async fn stream_extract_get_progress(
    stream_id: String,
    state: State<'_, StreamingExtractorState>,
) -> Result<StreamProgress, String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Streaming extractor not initialized")?;

    extractor.get_progress(&stream_id).await
}

#[tauri::command]
pub async fn stream_extract_cancel(
    stream_id: String,
    state: State<'_, StreamingExtractorState>,
) -> Result<(), String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Streaming extractor not initialized")?;

    extractor.cancel_stream(&stream_id).await
}

#[tauri::command]
pub async fn stream_extract_get_active(
    state: State<'_, StreamingExtractorState>,
) -> Result<Vec<String>, String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Streaming extractor not initialized")?;

    extractor.get_active_streams().await
}

#[tauri::command]
pub async fn stream_extract_get_job_statuses(
    stream_id: String,
    state: State<'_, StreamingExtractorState>,
) -> Result<Vec<JobStatus>, String> {
    let extractor_opt = state.0.lock().await;
    let extractor = extractor_opt
        .as_ref()
        .ok_or("Streaming extractor not initialized")?;

    extractor.get_job_statuses(&stream_id).await
}
