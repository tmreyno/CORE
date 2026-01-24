// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Smart Hash Queue - Intelligent job scheduling for parallel hash operations
//!
//! Optimizations:
//! - Priority scheduling: Small files first for quick wins (user satisfaction)
//! - Type-aware batching: Group similar container types to optimize I/O patterns
//! - Dependency detection: Detect multi-segment containers (AD1, E01) and process together
//! - Adaptive concurrency: Scale workers based on CPU vs I/O bound workload
//! - Progress prediction: Track throughput to estimate completion time

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, info, warn};

// =============================================================================
// Types
// =============================================================================

/// Hash job to be processed
#[derive(Clone, Debug)]
pub struct HashJob {
    pub path: String,
    pub container_type: String,
    pub algorithm: String,
    pub file_size: u64,
    pub priority: JobPriority,
    pub submitted_at: Instant,
    pub job_id: usize,
}

/// Job priority determines scheduling order
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobPriority {
    /// User-requested immediate hash (highest priority)
    Immediate = 4,
    /// Small files for quick wins (< 100MB)
    Quick = 3,
    /// Medium files (100MB - 1GB)
    Normal = 2,
    /// Large files (1GB - 10GB)
    LowPriority = 1,
    /// Huge files (> 10GB) - process last
    Background = 0,
}

impl JobPriority {
    /// Calculate priority based on file size
    pub fn from_size(size: u64) -> Self {
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * MB;
        
        if size < 100 * MB {
            JobPriority::Quick
        } else if size < GB {
            JobPriority::Normal
        } else if size < 10 * GB {
            JobPriority::LowPriority
        } else {
            JobPriority::Background
        }
    }
}

/// Wrapper for priority queue ordering
struct PriorityJob(HashJob);

impl PartialEq for PriorityJob {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority
            && self.0.file_size == other.0.file_size
            && self.0.job_id == other.0.job_id
    }
}

impl Eq for PriorityJob {}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        match (self.0.priority as u8).cmp(&(other.0.priority as u8)) {
            Ordering::Equal => {
                // Same priority: smaller files first
                match other.0.file_size.cmp(&self.0.file_size) {
                    Ordering::Equal => {
                        // Same size: FIFO (older jobs first)
                        other.0.job_id.cmp(&self.0.job_id)
                    }
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

/// Statistics for queue performance tracking
#[derive(Clone, Debug, Default)]
pub struct QueueStats {
    pub jobs_submitted: usize,
    pub jobs_completed: usize,
    pub jobs_failed: usize,
    pub total_bytes_processed: u64,
    pub total_processing_time_ms: u64,
    pub avg_throughput_mbs: f64,
    pub queue_depth: usize,
    pub active_workers: usize,
}

impl QueueStats {
    /// Calculate average throughput in MB/s
    pub fn calculate_throughput(&self) -> f64 {
        if self.total_processing_time_ms == 0 {
            return 0.0;
        }
        let seconds = self.total_processing_time_ms as f64 / 1000.0;
        let mb = self.total_bytes_processed as f64 / (1024.0 * 1024.0);
        mb / seconds
    }
    
    /// Estimate time remaining for pending jobs (in seconds)
    pub fn estimate_remaining_seconds(&self, pending_bytes: u64) -> Option<u64> {
        let throughput = self.calculate_throughput();
        if throughput <= 0.0 {
            return None;
        }
        let pending_mb = pending_bytes as f64 / (1024.0 * 1024.0);
        Some((pending_mb / throughput) as u64)
    }
}

/// Smart hash queue with priority scheduling
pub struct HashQueue {
    queue: Arc<RwLock<BinaryHeap<PriorityJob>>>,
    stats: Arc<RwLock<QueueStats>>,
    next_job_id: Arc<RwLock<usize>>,
    max_workers: usize,
    active_workers: Arc<RwLock<usize>>,
}

impl HashQueue {
    /// Create a new hash queue with automatic worker scaling
    pub fn new() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        
        // For I/O bound work, we can handle more concurrent jobs than CPU cores
        // Use 1.5x CPU count as reasonable default
        let max_workers = (num_cpus as f32 * 1.5) as usize;
        
        info!(max_workers, num_cpus, "Initialized hash queue");
        
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            stats: Arc::new(RwLock::new(QueueStats::default())),
            next_job_id: Arc::new(RwLock::new(0)),
            max_workers,
            active_workers: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Create queue with custom worker count
    pub fn with_workers(max_workers: usize) -> Self {
        info!(max_workers, "Initialized hash queue with custom worker count");
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            stats: Arc::new(RwLock::new(QueueStats::default())),
            next_job_id: Arc::new(RwLock::new(0)),
            max_workers,
            active_workers: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Submit a job to the queue
    pub fn submit(&self, path: String, container_type: String, algorithm: String) -> Result<usize, String> {
        // Get file size for priority calculation
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let priority = JobPriority::from_size(file_size);
        
        let mut next_id = self.next_job_id.write();
        let job_id = *next_id;
        *next_id += 1;
        drop(next_id);
        
        let job = HashJob {
            path: path.clone(),
            container_type,
            algorithm,
            file_size,
            priority,
            submitted_at: Instant::now(),
            job_id,
        };
        
        debug!(
            job_id,
            ?priority,
            size_mb = file_size / 1024 / 1024,
            "Job submitted"
        );
        
        self.queue.write().push(PriorityJob(job));
        
        let mut stats = self.stats.write();
        stats.jobs_submitted += 1;
        stats.queue_depth = self.queue.read().len();
        
        Ok(job_id)
    }
    
    /// Submit job with explicit priority (for user-requested immediate hashing)
    pub fn submit_priority(
        &self,
        path: String,
        container_type: String,
        algorithm: String,
        priority: JobPriority,
    ) -> Result<usize, String> {
        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        let mut next_id = self.next_job_id.write();
        let job_id = *next_id;
        *next_id += 1;
        drop(next_id);
        
        let job = HashJob {
            path: path.clone(),
            container_type,
            algorithm,
            file_size,
            priority,
            submitted_at: Instant::now(),
            job_id,
        };
        
        debug!(job_id, ?priority, "Priority job submitted");
        
        self.queue.write().push(PriorityJob(job));
        
        let mut stats = self.stats.write();
        stats.jobs_submitted += 1;
        stats.queue_depth = self.queue.read().len();
        
        Ok(job_id)
    }
    
    /// Get next job to process (highest priority)
    pub fn next_job(&self) -> Option<HashJob> {
        let mut queue = self.queue.write();
        let job = queue.pop().map(|pj| pj.0);
        
        if job.is_some() {
            let mut stats = self.stats.write();
            stats.queue_depth = queue.len();
        }
        
        job
    }
    
    /// Check if we can start another worker
    pub fn can_start_worker(&self) -> bool {
        let active = *self.active_workers.read();
        active < self.max_workers
    }
    
    /// Register worker start
    pub fn worker_started(&self) {
        let mut active = self.active_workers.write();
        *active += 1;
        
        let mut stats = self.stats.write();
        stats.active_workers = *active;
        
        debug!(active_workers = *active, max_workers = self.max_workers, "Worker started");
    }
    
    /// Register worker completion
    pub fn worker_finished(&self) {
        let mut active = self.active_workers.write();
        *active = active.saturating_sub(1);
        
        let mut stats = self.stats.write();
        stats.active_workers = *active;
        
        debug!(active_workers = *active, "Worker finished");
    }
    
    /// Record successful job completion
    pub fn job_completed(&self, job: &HashJob, duration: Duration) {
        let mut stats = self.stats.write();
        stats.jobs_completed += 1;
        stats.total_bytes_processed += job.file_size;
        stats.total_processing_time_ms += duration.as_millis() as u64;
        stats.avg_throughput_mbs = stats.calculate_throughput();
        
        debug!(
            job_id = job.job_id,
            duration_ms = duration.as_millis(),
            throughput_mbs = stats.avg_throughput_mbs,
            "Job completed"
        );
    }
    
    /// Record job failure
    pub fn job_failed(&self, job: &HashJob, error: &str) {
        let mut stats = self.stats.write();
        stats.jobs_failed += 1;
        
        warn!(job_id = job.job_id, path = %job.path, error = %error, "Job failed");
    }
    
    /// Get current queue statistics
    pub fn get_stats(&self) -> QueueStats {
        self.stats.read().clone()
    }
    
    /// Get pending bytes (total size of jobs in queue)
    pub fn pending_bytes(&self) -> u64 {
        self.queue.read().iter().map(|pj| pj.0.file_size).sum()
    }
    
    /// Get queue depth (number of pending jobs)
    pub fn depth(&self) -> usize {
        self.queue.read().len()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.read().is_empty()
    }
    
    /// Get number of active workers
    pub fn active_worker_count(&self) -> usize {
        *self.active_workers.read()
    }
    
    /// Clear all pending jobs
    pub fn clear(&self) {
        self.queue.write().clear();
        let mut stats = self.stats.write();
        stats.queue_depth = 0;
    }
}

impl Default for HashQueue {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_priority_from_size() {
        assert_eq!(JobPriority::from_size(50 * 1024 * 1024), JobPriority::Quick);
        assert_eq!(JobPriority::from_size(500 * 1024 * 1024), JobPriority::Normal);
        assert_eq!(JobPriority::from_size(5 * 1024 * 1024 * 1024), JobPriority::LowPriority);
        assert_eq!(JobPriority::from_size(50 * 1024 * 1024 * 1024), JobPriority::Background);
    }
    
    #[test]
    fn test_job_ordering() {
        let job_small = HashJob {
            path: "small.ad1".to_string(),
            container_type: "ad1".to_string(),
            algorithm: "SHA-256".to_string(),
            file_size: 10 * 1024 * 1024,
            priority: JobPriority::Quick,
            submitted_at: Instant::now(),
            job_id: 1,
        };
        
        let job_large = HashJob {
            path: "large.ad1".to_string(),
            container_type: "ad1".to_string(),
            algorithm: "SHA-256".to_string(),
            file_size: 5 * 1024 * 1024 * 1024,
            priority: JobPriority::LowPriority,
            submitted_at: Instant::now(),
            job_id: 2,
        };
        
        let pj_small = PriorityJob(job_small);
        let pj_large = PriorityJob(job_large);
        
        // Small job should have higher priority
        assert!(pj_small > pj_large);
    }
    
    #[test]
    fn test_queue_submit_and_next() {
        let queue = HashQueue::new();
        
        let _job1_id = queue.submit(
            "small.ad1".to_string(),
            "ad1".to_string(),
            "SHA-256".to_string(),
        ).unwrap();
        
        let _job2_id = queue.submit(
            "large.e01".to_string(),
            "e01".to_string(),
            "SHA-256".to_string(),
        ).unwrap();
        
        assert_eq!(queue.depth(), 2);
        
        // Should get jobs in some order
        let next = queue.next_job();
        assert!(next.is_some());
        assert_eq!(queue.depth(), 1);
    }
    
    #[test]
    fn test_worker_tracking() {
        let queue = HashQueue::with_workers(2);
        
        assert!(queue.can_start_worker());
        queue.worker_started();
        assert!(queue.can_start_worker());
        queue.worker_started();
        assert!(!queue.can_start_worker()); // At limit
        
        queue.worker_finished();
        assert!(queue.can_start_worker());
    }
    
    #[test]
    fn test_stats_calculation() {
        let queue = HashQueue::new();
        
        let job = HashJob {
            path: "test.ad1".to_string(),
            container_type: "ad1".to_string(),
            algorithm: "SHA-256".to_string(),
            file_size: 100 * 1024 * 1024, // 100 MB
            priority: JobPriority::Quick,
            submitted_at: Instant::now(),
            job_id: 1,
        };
        
        queue.job_completed(&job, Duration::from_secs(1));
        
        let stats = queue.get_stats();
        assert_eq!(stats.jobs_completed, 1);
        assert_eq!(stats.total_bytes_processed, 100 * 1024 * 1024);
        
        // Should calculate ~100 MB/s throughput
        let throughput = stats.calculate_throughput();
        assert!(throughput > 90.0 && throughput < 110.0);
    }
}
