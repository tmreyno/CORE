// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Progress Tracking Module
//!
//! ## Section Brief
//! Unified progress reporting for long-running operations across all container types.
//!
//! ### Key Types
//! - `Progress` - Current progress state
//! - `ProgressCallback` - Callback type for progress updates
//! - `ProgressTracker` - Track progress with automatic rate limiting
//!
//! ### Patterns (inspired by libewf)
//! - `process_status_t` pattern: unified status tracking
//! - Rate-limited callbacks to prevent UI flooding
//! - Support for indeterminate progress
//!
//! ### Usage
//! ```rust,ignore
//! use crate::common::progress::{ProgressTracker, Progress};
//!
//! let tracker = ProgressTracker::new(total_bytes, |p| {
//!     println!("{}%", p.percent());
//! });
//!
//! for chunk in chunks {
//!     process(chunk);
//!     tracker.advance(chunk.len() as u64);
//! }
//! ```

use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;

// =============================================================================
// Progress State
// =============================================================================

/// Progress information for a long-running operation
#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    /// Current position (bytes processed, items completed, etc.)
    pub current: u64,
    /// Total amount (0 if indeterminate)
    pub total: u64,
    /// Human-readable message
    pub message: Option<String>,
    /// Operation stage/phase
    pub stage: Option<String>,
    /// Estimated bytes per second (if available)
    pub bytes_per_second: Option<u64>,
    /// Estimated time remaining in seconds (if available)
    pub eta_seconds: Option<u64>,
    /// Whether the operation was cancelled
    pub cancelled: bool,
}

impl Progress {
    /// Create new progress at the start of an operation
    pub fn new(total: u64) -> Self {
        Self {
            current: 0,
            total,
            message: None,
            stage: None,
            bytes_per_second: None,
            eta_seconds: None,
            cancelled: false,
        }
    }

    /// Create indeterminate progress (total unknown)
    pub fn indeterminate() -> Self {
        Self {
            current: 0,
            total: 0,
            message: None,
            stage: None,
            bytes_per_second: None,
            eta_seconds: None,
            cancelled: false,
        }
    }

    /// Get progress percentage (0-100)
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if progress is determinate
    pub fn is_determinate(&self) -> bool {
        self.total > 0
    }

    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        self.total > 0 && self.current >= self.total
    }

    /// Set a message
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Set a stage
    pub fn with_stage(mut self, stage: &str) -> Self {
        self.stage = Some(stage.to_string());
        self
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new(0)
    }
}

// =============================================================================
// Progress Callback
// =============================================================================

/// Type alias for progress callbacks
pub type ProgressCallback = Box<dyn Fn(&Progress) + Send + Sync>;

/// Type alias for cancellation check
pub type CancelCheck = Box<dyn Fn() -> bool + Send + Sync>;

// =============================================================================
// Progress Tracker
// =============================================================================

/// Rate-limited progress tracker
/// 
/// Provides thread-safe progress tracking with automatic rate limiting
/// to prevent flooding the UI with updates.
pub struct ProgressTracker {
    /// Current progress value
    current: AtomicU64,
    /// Total expected value
    total: AtomicU64,
    /// Whether operation was cancelled
    cancelled: AtomicBool,
    /// Callback for progress updates
    callback: Option<ProgressCallback>,
    /// Minimum interval between callbacks
    min_interval: Duration,
    /// Last callback time
    last_callback: Mutex<Instant>,
    /// Start time for rate calculation
    start_time: Instant,
    /// Current message
    message: RwLock<Option<String>>,
    /// Current stage
    stage: RwLock<Option<String>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total: u64, callback: impl Fn(&Progress) + Send + Sync + 'static) -> Self {
        Self {
            current: AtomicU64::new(0),
            total: AtomicU64::new(total),
            cancelled: AtomicBool::new(false),
            callback: Some(Box::new(callback)),
            min_interval: Duration::from_millis(100),
            last_callback: Mutex::new(Instant::now()),
            start_time: Instant::now(),
            message: RwLock::new(None),
            stage: RwLock::new(None),
        }
    }

    /// Create a tracker without callback (for testing)
    pub fn silent(total: u64) -> Self {
        Self {
            current: AtomicU64::new(0),
            total: AtomicU64::new(total),
            cancelled: AtomicBool::new(false),
            callback: None,
            min_interval: Duration::from_millis(100),
            last_callback: Mutex::new(Instant::now()),
            start_time: Instant::now(),
            message: RwLock::new(None),
            stage: RwLock::new(None),
        }
    }

    /// Create a tracker with custom interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    /// Set the current progress value
    pub fn set(&self, current: u64) {
        self.current.store(current, Ordering::SeqCst);
        self.maybe_callback();
    }

    /// Advance progress by delta
    pub fn advance(&self, delta: u64) {
        self.current.fetch_add(delta, Ordering::SeqCst);
        self.maybe_callback();
    }

    /// Set total (for indeterminate that becomes determinate)
    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::SeqCst);
        self.maybe_callback();
    }

    /// Set message
    pub fn set_message(&self, message: &str) {
        *self.message.write() = Some(message.to_string());
        self.maybe_callback();
    }

    /// Set stage
    pub fn set_stage(&self, stage: &str) {
        *self.stage.write() = Some(stage.to_string());
        self.force_callback();
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.force_callback();
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Get current progress state
    pub fn progress(&self) -> Progress {
        let current = self.current.load(Ordering::SeqCst);
        let total = self.total.load(Ordering::SeqCst);
        let elapsed = self.start_time.elapsed();
        
        let bytes_per_second = if elapsed.as_secs() > 0 {
            Some(current / elapsed.as_secs())
        } else {
            None
        };
        
        let eta_seconds = if let Some(bps) = bytes_per_second {
            if bps > 0 && total > current {
                Some((total - current) / bps)
            } else {
                None
            }
        } else {
            None
        };
        
        Progress {
            current,
            total,
            message: self.message.read().clone(),
            stage: self.stage.read().clone(),
            bytes_per_second,
            eta_seconds,
            cancelled: self.cancelled.load(Ordering::SeqCst),
        }
    }

    /// Force a callback (bypass rate limiting)
    pub fn force_callback(&self) {
        if let Some(ref cb) = self.callback {
            cb(&self.progress());
            *self.last_callback.lock() = Instant::now();
        }
    }

    /// Complete the operation
    pub fn complete(&self) {
        let total = self.total.load(Ordering::SeqCst);
        self.current.store(total, Ordering::SeqCst);
        self.force_callback();
    }

    /// Maybe invoke callback (rate limited)
    fn maybe_callback(&self) {
        if let Some(ref cb) = self.callback {
            let should_callback = self.last_callback.lock().elapsed() >= self.min_interval;
            
            if should_callback {
                cb(&self.progress());
                *self.last_callback.lock() = Instant::now();
            }
        }
    }
}

// =============================================================================
// Arc-wrapped Tracker
// =============================================================================

/// Thread-safe progress tracker wrapped in Arc for sharing
pub type SharedProgressTracker = Arc<ProgressTracker>;

/// Create a new shared progress tracker
pub fn shared_tracker(total: u64, callback: impl Fn(&Progress) + Send + Sync + 'static) -> SharedProgressTracker {
    Arc::new(ProgressTracker::new(total, callback))
}

// =============================================================================
// Progress Builder
// =============================================================================

/// Builder for creating progress callbacks that emit to channels
pub struct ProgressChannelBuilder;

impl ProgressChannelBuilder {
    /// Create a progress callback that sends to a std mpsc channel
    #[allow(dead_code)]
    pub fn std_channel() -> (ProgressCallback, std::sync::mpsc::Receiver<Progress>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let callback: ProgressCallback = Box::new(move |p: &Progress| {
            let _ = tx.send(p.clone());
        });
        (callback, rx)
    }

    /// Create a progress callback that stores last progress
    pub fn last_value() -> (ProgressCallback, Arc<RwLock<Option<Progress>>>) {
        let last = Arc::new(RwLock::new(None));
        let last_clone = Arc::clone(&last);
        let callback: ProgressCallback = Box::new(move |p: &Progress| {
            *last_clone.write() = Some(p.clone());
        });
        (callback, last)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_percent() {
        let p = Progress::new(100);
        assert_eq!(p.percent(), 0.0);
        
        let p = Progress { current: 50, total: 100, ..Default::default() };
        assert_eq!(p.percent(), 50.0);
        
        let p = Progress { current: 100, total: 100, ..Default::default() };
        assert_eq!(p.percent(), 100.0);
    }

    #[test]
    fn test_progress_indeterminate() {
        let p = Progress::indeterminate();
        assert!(!p.is_determinate());
        assert_eq!(p.percent(), 0.0);
    }

    #[test]
    fn test_progress_complete() {
        let p = Progress { current: 100, total: 100, ..Default::default() };
        assert!(p.is_complete());
        
        let p = Progress { current: 50, total: 100, ..Default::default() };
        assert!(!p.is_complete());
    }

    #[test]
    fn test_tracker_advance() {
        let tracker = ProgressTracker::silent(100);
        tracker.advance(25);
        tracker.advance(25);
        let p = tracker.progress();
        assert_eq!(p.current, 50);
    }

    #[test]
    fn test_tracker_cancel() {
        let tracker = ProgressTracker::silent(100);
        assert!(!tracker.is_cancelled());
        tracker.cancel();
        assert!(tracker.is_cancelled());
    }

    #[test]
    fn test_tracker_complete() {
        let tracker = ProgressTracker::silent(100);
        tracker.advance(50);
        tracker.complete();
        let p = tracker.progress();
        assert!(p.is_complete());
    }

    #[test]
    fn test_progress_builder_last_value() {
        let (callback, last) = ProgressChannelBuilder::last_value();
        let progress = Progress { current: 42, total: 100, ..Default::default() };
        callback(&progress);
        
        let stored = last.read();
        assert!(stored.is_some());
        assert_eq!(stored.as_ref().unwrap().current, 42);
    }
}
