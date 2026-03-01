// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Adaptive I/O Buffer Management
//!
//! This module provides intelligent buffer sizing based on file characteristics
//! and operation types to optimize I/O performance across diverse workloads.
//!
//! # Performance Strategy
//!
//! Different operations benefit from different buffer sizes:
//! - **Large file hashing**: Larger buffers (32MB) amortize syscall overhead
//! - **Small file hashing**: Smaller buffers (4MB) reduce memory pressure
//! - **Interactive viewing**: Medium buffers (8MB) balance latency and throughput
//! - **Streaming operations**: Tuned buffers for real-time feedback
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::common::io_adaptive::{AdaptiveBuffer, Operation};
//!
//! let file_size = 5_000_000_000; // 5 GB
//! let buffer_size = AdaptiveBuffer::optimal_size(file_size, Operation::Hash);
//! assert_eq!(buffer_size, 32 << 20); // 32 MB for large file hashing
//! ```

use std::cmp;

// =============================================================================
// Operation Types
// =============================================================================

/// Type of I/O operation being performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Cryptographic hash computation (maximize throughput)
    Hash,

    /// File content reading for viewing (balance latency and throughput)
    Read,

    /// File extraction/copying (optimize for sustained writes)
    Extract,

    /// Verification against stored hash (similar to Hash but may need smaller buffers)
    Verify,

    /// Streaming operation with progress updates (smaller buffers for responsiveness)
    Stream,
}

// =============================================================================
// Size Thresholds (in bytes)
// =============================================================================

/// Very small files (< 1 MB) - Reserved for future micro-optimization
#[allow(dead_code)]
const TINY_FILE: u64 = 1 << 20; // 1 MB

/// Small files (< 10 MB)
const SMALL_FILE: u64 = 10 << 20; // 10 MB

/// Medium files (< 100 MB)
const MEDIUM_FILE: u64 = 100 << 20; // 100 MB

/// Large files (< 1 GB)
const LARGE_FILE: u64 = 1_000 << 20; // 1 GB

/// Very large files (>= 1 GB)
const HUGE_FILE: u64 = 1_000 << 20; // 1 GB threshold

// =============================================================================
// Buffer Sizes (in bytes)
// =============================================================================

/// Minimum buffer size (for tiny files)
const MIN_BUFFER: usize = 512 << 10; // 512 KB

/// Small buffer size (for small files)
const SMALL_BUFFER: usize = 2 << 20; // 2 MB

/// Medium buffer size (for medium files)
const MEDIUM_BUFFER: usize = 8 << 20; // 8 MB

/// Large buffer size (for large files)
const LARGE_BUFFER: usize = 16 << 20; // 16 MB

/// Huge buffer size (for very large files)
const HUGE_BUFFER: usize = 32 << 20; // 32 MB

/// Maximum buffer size (safety limit)
const MAX_BUFFER: usize = 64 << 20; // 64 MB

// =============================================================================
// Adaptive Buffer
// =============================================================================

/// Adaptive buffer sizing utility
pub struct AdaptiveBuffer;

impl AdaptiveBuffer {
    /// Calculate optimal buffer size for a given file size and operation
    ///
    /// # Arguments
    /// * `file_size` - Size of file in bytes
    /// * `operation` - Type of I/O operation
    ///
    /// # Returns
    /// Optimal buffer size in bytes
    ///
    /// # Strategy
    /// - Hash operations use larger buffers to maximize throughput
    /// - Interactive operations use smaller buffers to minimize latency
    /// - Size scales with file size to avoid memory waste on small files
    pub fn optimal_size(file_size: u64, operation: Operation) -> usize {
        match operation {
            Operation::Hash | Operation::Extract => {
                // Maximize throughput for batch operations
                if file_size >= HUGE_FILE {
                    HUGE_BUFFER // 32 MB
                } else if file_size >= LARGE_FILE {
                    LARGE_BUFFER // 16 MB
                } else if file_size >= MEDIUM_FILE {
                    MEDIUM_BUFFER // 8 MB
                } else if file_size >= SMALL_FILE {
                    SMALL_BUFFER // 2 MB
                } else {
                    MIN_BUFFER // 512 KB
                }
            }

            Operation::Read | Operation::Verify => {
                // Balance latency and throughput for interactive operations
                if file_size >= HUGE_FILE {
                    LARGE_BUFFER // 16 MB (smaller than Hash)
                } else if file_size >= LARGE_FILE {
                    MEDIUM_BUFFER // 8 MB
                } else if file_size >= MEDIUM_FILE {
                    SMALL_BUFFER // 2 MB
                } else {
                    MIN_BUFFER // 512 KB
                }
            }

            Operation::Stream => {
                // Smaller buffers for responsive progress updates
                if file_size >= HUGE_FILE {
                    MEDIUM_BUFFER // 8 MB
                } else if file_size >= LARGE_FILE {
                    SMALL_BUFFER // 2 MB
                } else {
                    MIN_BUFFER // 512 KB
                }
            }
        }
    }

    /// Get buffer size with explicit bounds
    ///
    /// # Arguments
    /// * `file_size` - Size of file in bytes
    /// * `operation` - Type of I/O operation
    /// * `min_size` - Minimum buffer size (optional)
    /// * `max_size` - Maximum buffer size (optional)
    ///
    /// # Returns
    /// Bounded buffer size in bytes
    pub fn bounded_size(
        file_size: u64,
        operation: Operation,
        min_size: Option<usize>,
        max_size: Option<usize>,
    ) -> usize {
        let optimal = Self::optimal_size(file_size, operation);
        let min_bound = min_size.unwrap_or(MIN_BUFFER);
        let max_bound = max_size.unwrap_or(MAX_BUFFER);

        cmp::max(min_bound, cmp::min(optimal, max_bound))
    }

    /// Calculate number of chunks for progress reporting
    ///
    /// Returns optimal number of progress updates for the given file size
    /// to balance UI responsiveness with performance overhead.
    pub fn progress_chunks(file_size: u64) -> u64 {
        if file_size >= HUGE_FILE {
            100 // Update every 1% for large files
        } else if file_size >= LARGE_FILE {
            50 // Update every 2% for medium-large files
        } else if file_size >= MEDIUM_FILE {
            20 // Update every 5% for medium files
        } else {
            10 // Update every 10% for small files
        }
    }

    /// Check if file should use memory-mapped I/O
    ///
    /// Memory-mapped I/O is beneficial for:
    /// - Large files that fit in available RAM
    /// - Random access patterns
    /// - Multiple reads of the same data
    pub fn should_use_mmap(file_size: u64) -> bool {
        // Use mmap for files >= 64 MB (matches existing MMAP_THRESHOLD)
        file_size >= 64 << 20
    }

    /// Get recommended read-ahead size for sequential access
    pub fn readahead_size(file_size: u64) -> usize {
        if file_size >= HUGE_FILE {
            HUGE_BUFFER * 2 // 64 MB read-ahead
        } else if file_size >= LARGE_FILE {
            LARGE_BUFFER * 2 // 32 MB read-ahead
        } else {
            MEDIUM_BUFFER // 8 MB read-ahead
        }
    }
}

// =============================================================================
// Adaptive Statistics
// =============================================================================

/// Statistics for adaptive buffer performance tracking
#[derive(Debug, Clone, Default)]
pub struct AdaptiveStats {
    /// Total bytes processed
    pub bytes_processed: u64,

    /// Number of read operations
    pub read_count: u64,

    /// Average buffer utilization (0.0 - 1.0)
    pub avg_utilization: f64,

    /// Time spent in I/O (microseconds)
    pub io_time_us: u64,

    /// Calculated throughput (MB/s)
    pub throughput_mbs: f64,
}

impl AdaptiveStats {
    /// Create new statistics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a read operation
    pub fn record_read(&mut self, bytes: u64, elapsed_us: u64) {
        self.bytes_processed += bytes;
        self.read_count += 1;
        self.io_time_us += elapsed_us;

        // Recalculate throughput
        if self.io_time_us > 0 {
            let seconds = self.io_time_us as f64 / 1_000_000.0;
            let mb = self.bytes_processed as f64 / (1024.0 * 1024.0);
            self.throughput_mbs = mb / seconds;
        }
    }

    /// Get average read size
    pub fn avg_read_size(&self) -> u64 {
        if self.read_count > 0 {
            self.bytes_processed / self.read_count
        } else {
            0
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_size_hash_operations() {
        // Tiny file: 500 KB
        assert_eq!(
            AdaptiveBuffer::optimal_size(500 << 10, Operation::Hash),
            MIN_BUFFER // 512 KB
        );

        // Small file: 5 MB
        assert_eq!(
            AdaptiveBuffer::optimal_size(5 << 20, Operation::Hash),
            MIN_BUFFER // 512 KB (< 10 MB)
        );

        // Medium file: 50 MB
        assert_eq!(
            AdaptiveBuffer::optimal_size(50 << 20, Operation::Hash),
            SMALL_BUFFER // 2 MB (< 100 MB)
        );

        // Large file: 500 MB (but < 1 GB threshold, so MEDIUM_BUFFER)
        assert_eq!(
            AdaptiveBuffer::optimal_size(500 << 20, Operation::Hash),
            MEDIUM_BUFFER // 8 MB (< 1 GB, so still MEDIUM)
        );

        // Huge file: 5 GB
        assert_eq!(
            AdaptiveBuffer::optimal_size(5_000 << 20, Operation::Hash),
            HUGE_BUFFER // 32 MB
        );
    }

    #[test]
    fn test_optimal_size_read_operations() {
        // Interactive operations use smaller buffers
        let large_file = 5_000 << 20; // 5 GB

        assert!(
            AdaptiveBuffer::optimal_size(large_file, Operation::Read)
                < AdaptiveBuffer::optimal_size(large_file, Operation::Hash)
        );
    }

    #[test]
    fn test_optimal_size_stream_operations() {
        // Streaming uses smallest buffers for responsiveness
        let large_file = 5_000 << 20; // 5 GB

        assert!(
            AdaptiveBuffer::optimal_size(large_file, Operation::Stream)
                < AdaptiveBuffer::optimal_size(large_file, Operation::Read)
        );
    }

    #[test]
    fn test_bounded_size() {
        let file_size = 5_000 << 20; // 5 GB

        // Test minimum bound
        let size = AdaptiveBuffer::bounded_size(
            file_size,
            Operation::Hash,
            Some(1 << 20), // Min 1 MB
            None,
        );
        assert!(size >= 1 << 20);

        // Test maximum bound
        let size = AdaptiveBuffer::bounded_size(
            file_size,
            Operation::Hash,
            None,
            Some(10 << 20), // Max 10 MB
        );
        assert!(size <= 10 << 20);
    }

    #[test]
    fn test_progress_chunks() {
        assert_eq!(AdaptiveBuffer::progress_chunks(500 << 10), 10); // Small: < 100 MB
        assert_eq!(AdaptiveBuffer::progress_chunks(50 << 20), 10); // Small: < 100 MB
        assert_eq!(AdaptiveBuffer::progress_chunks(150 << 20), 20); // Medium: >= 100 MB
        assert_eq!(AdaptiveBuffer::progress_chunks(500 << 20), 20); // Medium: < 1 GB threshold
        assert_eq!(AdaptiveBuffer::progress_chunks(5_000 << 20), 100); // Huge: >= 1 GB
    }

    #[test]
    fn test_should_use_mmap() {
        assert!(!AdaptiveBuffer::should_use_mmap(10 << 20)); // 10 MB - no
        assert!(AdaptiveBuffer::should_use_mmap(100 << 20)); // 100 MB - yes
        assert!(AdaptiveBuffer::should_use_mmap(1_000 << 20)); // 1 GB - yes
    }

    #[test]
    fn test_readahead_size() {
        assert_eq!(
            AdaptiveBuffer::readahead_size(5 << 20), // 5 MB
            MEDIUM_BUFFER                            // 8 MB
        );

        assert_eq!(
            AdaptiveBuffer::readahead_size(500 << 20), // 500 MB (< 1 GB)
            MEDIUM_BUFFER                              // 8 MB
        );

        assert_eq!(
            AdaptiveBuffer::readahead_size(1_500 << 20), // 1.5 GB (>= 1 GB)
            HUGE_BUFFER * 2                              // 64 MB (HUGE_FILE threshold)
        );

        assert_eq!(
            AdaptiveBuffer::readahead_size(5_000 << 20), // 5 GB
            HUGE_BUFFER * 2                              // 64 MB
        );
    }

    #[test]
    fn test_adaptive_stats() {
        let mut stats = AdaptiveStats::new();

        // Record some reads
        stats.record_read(1 << 20, 10_000); // 1 MB in 10ms
        stats.record_read(2 << 20, 20_000); // 2 MB in 20ms

        assert_eq!(stats.bytes_processed, 3 << 20);
        assert_eq!(stats.read_count, 2);
        assert_eq!(stats.avg_read_size(), (3 << 20) / 2);

        // Throughput should be ~100 MB/s
        assert!(stats.throughput_mbs > 90.0 && stats.throughput_mbs < 110.0);
    }
}
