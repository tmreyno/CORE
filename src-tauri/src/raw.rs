// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! RAW Disk Image Parser
//!
//! This module provides parsing and verification for raw (dd-style) forensic
//! disk images, supporting both single-file and multi-segment formats.
//!
//! ## Supported Formats
//!
//! | Extension     | Description                           |
//! |---------------|---------------------------------------|
//! | `.dd`         | Standard dd-style raw image           |
//! | `.raw`        | Generic raw disk image                |
//! | `.img`        | Disk image (verify magic to disambiguate) |
//! | `.001`-`.999` | Multi-segment numbered format         |
//!
//! ## Multi-Segment Images
//!
//! Raw images can be split into numbered segments:
//!
//! ```text
//! evidence.001  ─┐
//! evidence.002   │  Combined: Single contiguous byte stream
//! evidence.003   │  representing the original disk image
//! evidence.004  ─┘
//! ```
//!
//! Segment discovery:
//! 1. Detect if input file has numeric extension (.001, .002, etc.)
//! 2. Scan directory for matching basename with sequential numbers
//! 3. Sort segments by number, verify no gaps
//! 4. Concatenate virtually for seamless reading
//!
//! ## RawHandle
//!
//! The `RawHandle` provides a virtual file-like interface over segmented images:
//!
//! ```rust,ignore
//! let mut handle = RawHandle::open("/evidence/disk.001")?;
//!
//! // Read seamlessly across segment boundaries
//! let mut buf = vec![0u8; 1024];
//! let bytes_read = handle.read(&mut buf)?;
//!
//! // Total size spans all segments
//! let total_size = handle.total_size();
//! ```
//!
//! ## Hash Verification
//!
//! High-performance hashing with algorithm-specific optimizations:
//!
//! | Algorithm | Implementation                              |
//! |-----------|---------------------------------------------|
//! | BLAKE3    | Memory-mapped I/O + rayon parallel hashing  |
//! | XXH3      | Memory-mapped I/O, extremely fast           |
//! | SHA-256   | Pipelined: async I/O → hasher thread        |
//! | MD5       | Pipelined I/O (legacy, not recommended)     |
//!
//! ```rust,ignore
//! // Verify with progress callback
//! raw::verify_with_progress("/evidence/disk.001", "sha256", |current, total| {
//!     let percent = (current as f64 / total as f64) * 100.0;
//!     println!("Progress: {:.1}%", percent);
//! })?;
//! ```
//!
//! ## Forensic Notes
//!
//! - Raw images preserve **physical** disk layout (sector-by-sector)
//! - No compression or metadata - pure byte-for-byte copy
//! - Hash of raw image = hash of original disk
//! - Segment boundaries have NO forensic significance
//!   (they're just split points, not disk boundaries)
//! - For evidentiary purposes, always hash the **complete** image
//!
//! ## Performance
//!
//! Buffer sizes and threading are tuned for modern storage:
//! - 16MB I/O buffers for sequential throughput
//! - Memory-mapped I/O for >64MB files
//! - Parallel hashing for BLAKE3
//! - Pipelined I/O for other algorithms

// RAW disk image parser (.dd, .raw, .img, .001, .002, etc.)
// Supports single and multi-segment raw forensic images

use serde::Serialize;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use tracing::{debug, trace, instrument};

use crate::common::{BUFFER_SIZE, hash::StreamingHasher, segments::discover_numbered_segments};
use crate::containers::ContainerError;

// =============================================================================
// Public Types
// =============================================================================

#[derive(Serialize, Clone)]
pub struct RawInfo {
    pub segment_count: u32,
    pub total_size: u64,
    pub segment_sizes: Vec<u64>,
    pub segment_names: Vec<String>,
    pub first_segment: String,
    pub last_segment: String,
}

/// Raw image statistics
#[derive(Serialize, Clone)]
pub struct RawStats {
    /// Total number of segments
    pub segment_count: u32,
    /// Total size in bytes
    pub total_size: u64,
    /// Formatted total size (human readable)
    pub total_size_formatted: String,
    /// Size of largest segment
    pub largest_segment: u64,
    /// Size of smallest segment
    pub smallest_segment: u64,
    /// Average segment size
    pub average_segment_size: u64,
    /// Whether segments are uniform in size
    pub uniform_segments: bool,
    /// First segment filename
    pub first_segment: String,
    /// Last segment filename
    pub last_segment: String,
}

#[derive(Serialize)]
pub struct VerifyResult {
    pub algorithm: String,
    pub hash: String,
    pub total_size: u64,
    pub duration_secs: f64,
    pub throughput_mbs: f64,
}

// =============================================================================
// Raw Image Handle
// =============================================================================

pub struct RawHandle {
    segments: Vec<PathBuf>,
    segment_sizes: Vec<u64>,
    total_size: u64,
    current_segment: usize,
    current_file: Option<File>,
    position: u64,
}

impl RawHandle {
    /// Open a raw image (single or multi-segment)
    #[instrument(skip_all, fields(path))]
    pub fn open(path: &str) -> Result<Self, ContainerError> {
        debug!(path, "Opening raw image handle");
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err(ContainerError::FileNotFound(path.to_string()));
        }

        let (segments, segment_sizes) = discover_segments(path)?;
        let total_size: u64 = segment_sizes.iter().sum();
        
        debug!(
            segment_count = segments.len(),
            total_size,
            "Raw handle opened"
        );

        Ok(RawHandle {
            segments,
            segment_sizes,
            total_size,
            current_segment: 0,
            current_file: None,
            position: 0,
        })
    }

    /// Get total size of all segments
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get segment count
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Read bytes at current position
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ContainerError> {
        if self.position >= self.total_size {
            return Ok(0);
        }

        let mut total_read = 0;
        let mut remaining = buf.len();

        while remaining > 0 && self.position < self.total_size {
            // Find which segment we're in
            let (seg_idx, seg_offset) = self.position_to_segment(self.position);
            
            // Open segment if needed
            if self.current_segment != seg_idx || self.current_file.is_none() {
                self.current_segment = seg_idx;
                let file = File::open(&self.segments[seg_idx])
                    .map_err(|e| format!("Failed to open segment {}: {}", seg_idx, e))?;
                self.current_file = Some(file);
            }

            let file = self.current_file.as_mut().unwrap();
            file.seek(SeekFrom::Start(seg_offset))
                .map_err(|e| format!("Seek failed: {}", e))?;

            // Calculate how much we can read from this segment
            let seg_remaining = self.segment_sizes[seg_idx] - seg_offset;
            let to_read = remaining.min(seg_remaining as usize);

            let bytes_read = file.read(&mut buf[total_read..total_read + to_read])
                .map_err(|e| format!("Read failed: {}", e))?;

            if bytes_read == 0 {
                break;
            }

            total_read += bytes_read;
            remaining -= bytes_read;
            self.position += bytes_read as u64;
        }

        Ok(total_read)
    }

    /// Convert absolute position to (segment_index, offset_within_segment)
    fn position_to_segment(&self, pos: u64) -> (usize, u64) {
        let mut offset = pos;
        for (idx, &size) in self.segment_sizes.iter().enumerate() {
            if offset < size {
                return (idx, offset);
            }
            offset -= size;
        }
        // Past end - return last segment
        let last = self.segments.len() - 1;
        (last, self.segment_sizes[last])
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Get information about a raw image
#[instrument]
pub fn info(path: &str) -> Result<RawInfo, ContainerError> {
    debug!("Getting raw image info");
    let handle = RawHandle::open(path)?;
    
    // Extract just filenames for display
    let segment_names: Vec<String> = handle.segments.iter()
        .map(|p| p.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default())
        .collect();
    
    debug!(
        segment_count = handle.segment_count(),
        total_size = handle.total_size(),
        "Raw image info loaded"
    );
    
    Ok(RawInfo {
        segment_count: handle.segment_count() as u32,
        total_size: handle.total_size(),
        segment_sizes: handle.segment_sizes.clone(),
        segment_names,
        first_segment: handle.segments.first()
            .map(|p| p.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default())
            .unwrap_or_default(),
        last_segment: handle.segments.last()
            .map(|p| p.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default())
            .unwrap_or_default(),
    })
}

/// Get statistics about a raw image
#[instrument]
pub fn get_stats(path: &str) -> Result<RawStats, ContainerError> {
    debug!("Getting raw image stats");
    let info = info(path)?;
    
    let largest = info.segment_sizes.iter().max().copied().unwrap_or(0);
    let smallest = info.segment_sizes.iter().min().copied().unwrap_or(0);
    let average = if info.segment_count > 0 {
        info.total_size / info.segment_count as u64
    } else {
        0
    };
    
    // Check if segments are uniform (within 1% of each other)
    let uniform = if info.segment_count > 1 {
        let variance_threshold = largest / 100; // 1% variance allowed
        largest - smallest <= variance_threshold
    } else {
        true
    };
    
    // Format size for display
    let total_size_formatted = crate::common::format_size(info.total_size);
    
    Ok(RawStats {
        segment_count: info.segment_count,
        total_size: info.total_size,
        total_size_formatted,
        largest_segment: largest,
        smallest_segment: smallest,
        average_segment_size: average,
        uniform_segments: uniform,
        first_segment: info.first_segment,
        last_segment: info.last_segment,
    })
}

/// Fast info - same as info() for raw images (no tree to skip)
#[instrument]
pub fn info_fast(path: &str) -> Result<RawInfo, ContainerError> {
    info(path)
}

/// Check if a file is a raw image (by extension)
pub fn is_raw(path: &str) -> Result<bool, ContainerError> {
    let lower = path.to_lowercase();
    
    // Check common raw extensions
    if lower.ends_with(".dd") || lower.ends_with(".raw") || lower.ends_with(".img") {
        trace!(path, "Detected as raw by extension");
        return Ok(true);
    }
    
    // Check numeric extensions (.001, .002, etc.)
    if let Some(ext_start) = lower.rfind('.') {
        let ext = &lower[ext_start + 1..];
        if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Verify raw image with specified hash algorithm
pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Verify with progress callback - OPTIMIZED with pipelined I/O and hashing
#[instrument(skip(progress_callback))]
pub fn verify_with_progress<F>(path: &str, algorithm: &str, progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    debug!("Starting raw image verification");
    let handle = RawHandle::open(path)?;
    let total_size = handle.total_size();
    let algorithm_lower = algorithm.to_lowercase();

    // Validate algorithm
    let valid_algo = matches!(algorithm_lower.as_str(), 
        "md5" | "sha1" | "sha-1" | "sha256" | "sha-256" | 
        "sha512" | "sha-512" | "blake3" | "blake2" | "blake2b" |
        "xxh3" | "xxh64" | "crc32");
    
    if !valid_algo {
        return Err(ContainerError::VerificationError(format!("Unsupported algorithm: {}. Supported: md5, sha1, sha256, sha512, blake3, blake2, xxh3, xxh64, crc32", algorithm)));
    }

    debug!(algorithm = algorithm_lower.as_str(), total_size, "Verifying with algorithm");

    // For BLAKE3, use its built-in parallel hashing with memory-mapped I/O
    if algorithm_lower == "blake3" {
        return verify_blake3_optimized(path, total_size, progress_callback);
    }
    
    // For XXH3, use memory-mapped I/O for maximum speed
    if algorithm_lower == "xxh3" {
        return verify_xxh3_optimized(path, total_size, progress_callback);
    }

    // For other algorithms, use pipelined I/O -> hashing
    verify_pipelined(path, &algorithm_lower, total_size, progress_callback)
}

/// BLAKE3 optimized path - uses memory-mapped I/O + rayon parallel hashing
fn verify_blake3_optimized<F>(path: &str, total_size: u64, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    use memmap2::Mmap;
    use crate::common::MMAP_THRESHOLD;
    
    let mut hasher = blake3::Hasher::new();
    let segments = discover_segments(path)?.0;
    let mut bytes_processed = 0u64;
    let report_interval = (total_size / 50).max(BUFFER_SIZE as u64); // Report ~50 times
    let mut last_report = 0u64;
    
    for seg_path in &segments {
        let file = File::open(seg_path)
            .map_err(|e| format!("Failed to open segment: {}", e))?;
        let seg_size = file.metadata()
            .map_err(|e| format!("Failed to get segment size: {}", e))?
            .len();
        
        // Use memory-mapped I/O for large segments (faster than buffered read)
        if seg_size >= MMAP_THRESHOLD {
            // SAFETY: File is opened read-only, mmap is safe for read access
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map segment: {}", e))?;
            
            // Process in chunks for progress reporting
            let chunk_size = BUFFER_SIZE;
            for chunk in mmap.chunks(chunk_size) {
                hasher.update_rayon(chunk);
                bytes_processed += chunk.len() as u64;
                
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            // Small files: use buffered read
            use std::io::BufRead;
            let mut reader = std::io::BufReader::with_capacity(BUFFER_SIZE, file);
            
            loop {
                let buf = reader.fill_buf()
                    .map_err(|e| format!("Read error: {}", e))?;
                let len = buf.len();
                if len == 0 { break; }
                
                hasher.update_rayon(buf);
                reader.consume(len);
                
                bytes_processed += len as u64;
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        }
    }
    
    progress_callback(total_size, total_size);
    Ok(hasher.finalize().to_hex().to_string())
}

/// XXH3 optimized path - uses memory-mapped I/O for maximum speed
/// XXH3 is ~10x faster than SHA-256 for non-cryptographic checksums
fn verify_xxh3_optimized<F>(path: &str, total_size: u64, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    use memmap2::Mmap;
    use xxhash_rust::xxh3::Xxh3;
    use crate::common::MMAP_THRESHOLD;
    
    let mut hasher = Xxh3::new();
    let segments = discover_segments(path)?.0;
    let mut bytes_processed = 0u64;
    let report_interval = (total_size / 50).max(BUFFER_SIZE as u64);
    let mut last_report = 0u64;
    
    for seg_path in &segments {
        let file = File::open(seg_path)
            .map_err(|e| format!("Failed to open segment: {}", e))?;
        let seg_size = file.metadata()
            .map_err(|e| format!("Failed to get segment size: {}", e))?
            .len();
        
        // Use memory-mapped I/O for large segments
        if seg_size >= MMAP_THRESHOLD {
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map segment: {}", e))?;
            
            // Process in chunks for progress reporting
            let chunk_size = BUFFER_SIZE;
            for chunk in mmap.chunks(chunk_size) {
                hasher.update(chunk);
                bytes_processed += chunk.len() as u64;
                
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            // Small files: use buffered read
            use std::io::BufRead;
            let mut reader = std::io::BufReader::with_capacity(BUFFER_SIZE, file);
            
            loop {
                let buf = reader.fill_buf()
                    .map_err(|e| format!("Read error: {}", e))?;
                let len = buf.len();
                if len == 0 { break; }
                
                hasher.update(buf);
                reader.consume(len);
                
                bytes_processed += len as u64;
                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        }
    }
    
    progress_callback(total_size, total_size);
    Ok(format!("{:016x}", hasher.digest128()))
}

/// Pipelined verification: I/O thread feeds data to hashing thread
fn verify_pipelined<F>(path: &str, algorithm: &str, total_size: u64, mut progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    
    let segments = discover_segments(path)?.0;
    let algo = algorithm.to_string();
    
    // Shared progress counter
    let bytes_hashed = Arc::new(AtomicU64::new(0));
    let bytes_hashed_clone = Arc::clone(&bytes_hashed);
    
    // Channel with 4 buffer slots for pipelining (allows I/O to stay ahead)
    let (tx, rx) = mpsc::sync_channel::<Option<Vec<u8>>>(4);
    
    // I/O thread: reads segments and sends buffers
    let io_handle = thread::spawn(move || -> Result<(), ContainerError> {
        for seg_path in &segments {
            let file = File::open(seg_path)
                .map_err(|e| format!("Failed to open segment {:?}: {}", seg_path, e))?;
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
            
            loop {
                let mut buf = vec![0u8; BUFFER_SIZE];
                let bytes_read = reader.read(&mut buf)
                    .map_err(|e| ContainerError::IoError(format!("Read error: {}", e)))?;
                
                if bytes_read == 0 { break; }
                
                buf.truncate(bytes_read);
                if tx.send(Some(buf)).is_err() {
                    return Err(ContainerError::InternalError("Hash thread terminated early".to_string()));
                }
            }
        }
        // Signal completion
        let _ = tx.send(None);
        Ok(())
    });
    
    // Hashing thread: receives buffers and updates hash using StreamingHasher
    let hash_handle = thread::spawn(move || -> Result<String, ContainerError> {
        let mut hasher: StreamingHasher = algo.parse()?;
        
        // Process incoming buffers
        while let Ok(Some(buf)) = rx.recv() {
            let len = buf.len() as u64;
            hasher.update(&buf);
            bytes_hashed_clone.fetch_add(len, Ordering::Relaxed);
        }
        
        // Finalize and return hash
        Ok(hasher.finalize())
    });
    
    // Progress reporting in main thread
    let report_interval = (total_size / 100).max(1);
    let mut last_reported = 0u64;
    
    loop {
        let current = bytes_hashed.load(Ordering::Relaxed);
        if current >= total_size { break; }
        
        if current - last_reported >= report_interval {
            progress_callback(current, total_size);
            last_reported = current;
        }
        
        // Check if I/O thread finished
        if io_handle.is_finished() {
            break;
        }
        
        thread::sleep(std::time::Duration::from_millis(50));
    }
    
    // Wait for threads
    io_handle.join()
        .map_err(|_| "I/O thread panicked")?
        .map_err(|e| format!("I/O error: {}", e))?;
    
    let hash = hash_handle.join()
        .map_err(|_| "Hash thread panicked")?
        .map_err(|e| format!("Hash error: {}", e))?;
    
    progress_callback(total_size, total_size);
    Ok(hash)
}

/// Result of verifying a single segment
#[derive(Serialize, Clone)]
pub struct SegmentVerifyResult {
    pub segment_name: String,
    pub segment_number: u32,
    pub algorithm: String,
    pub computed_hash: String,
    pub expected_hash: Option<String>,
    pub verified: Option<bool>,  // None = no expected hash, Some(true) = match, Some(false) = mismatch
    pub size: u64,
    pub duration_secs: f64,
}

/// Verify a single segment file and return hash.
///
/// This is a thin wrapper around `crate::common::hash_segment_with_progress`.
/// Use that function directly for new code.
#[instrument(skip(progress_callback))]
pub fn hash_single_segment<F>(segment_path: &str, algorithm: &str, progress_callback: F) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64)
{
    crate::common::hash_segment_with_progress(segment_path, algorithm, progress_callback)
}

/// Get all segment file paths for a raw image
pub fn get_segment_paths(path: &str) -> Result<Vec<PathBuf>, ContainerError> {
    let (segments, _) = discover_segments(path)?;
    Ok(segments)
}

/// Extract raw image to a single file (useful for reassembling multi-segment)
pub fn extract(path: &str, output_path: &str) -> Result<(), ContainerError> {
    extract_with_progress(path, output_path, |_, _| {})
}

/// Extract raw image with progress callback
#[instrument(skip(progress_callback))]
pub fn extract_with_progress<F>(path: &str, output_path: &str, mut progress_callback: F) -> Result<(), ContainerError>
where
    F: FnMut(u64, u64),
{
    use std::io::Write;
    
    debug!(path, output_path, "Extracting raw image");
    
    let mut handle = RawHandle::open(path)?;
    let total_size = handle.total_size();
    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    let mut buf = vec![0u8; BUFFER_SIZE];
    let mut bytes_written = 0u64;
    let mut last_report = 0u64;
    let report_interval = total_size / 100; // Report every 1%
    
    loop {
        let bytes_read = handle.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        output.write_all(&buf[..bytes_read])
            .map_err(|e| format!("Write failed: {}", e))?;
        
        bytes_written += bytes_read as u64;
        
        // Report progress at intervals
        if bytes_written - last_report >= report_interval || bytes_written == total_size {
            progress_callback(bytes_written, total_size);
            last_report = bytes_written;
        }
    }
    
    // Final progress report
    progress_callback(total_size, total_size);
    debug!(bytes_written, "Raw image extraction complete");

    Ok(())
}

// =============================================================================
// Metadata Export Functions
// =============================================================================

/// Export raw image metadata as JSON
#[instrument]
pub fn export_metadata_json(path: &str) -> Result<String, ContainerError> {
    debug!(path, "Exporting raw image metadata as JSON");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    #[derive(Serialize)]
    struct RawMetadata {
        format: String,
        segment_count: u32,
        total_size: u64,
        total_size_formatted: String,
        segments: Vec<SegmentDetail>,
        statistics: SegmentStatistics,
    }
    
    #[derive(Serialize)]
    struct SegmentDetail {
        index: usize,
        name: String,
        size: u64,
        size_formatted: String,
    }
    
    #[derive(Serialize)]
    struct SegmentStatistics {
        largest_segment: u64,
        smallest_segment: u64,
        average_segment_size: u64,
        uniform_segments: bool,
    }
    
    let segments: Vec<SegmentDetail> = info.segment_names.iter()
        .zip(info.segment_sizes.iter())
        .enumerate()
        .map(|(i, (name, &size))| SegmentDetail {
            index: i + 1,
            name: name.clone(),
            size,
            size_formatted: crate::common::format_size(size),
        })
        .collect();
    
    let metadata = RawMetadata {
        format: "RAW".to_string(),
        segment_count: info.segment_count,
        total_size: info.total_size,
        total_size_formatted: stats.total_size_formatted,
        segments,
        statistics: SegmentStatistics {
            largest_segment: stats.largest_segment,
            smallest_segment: stats.smallest_segment,
            average_segment_size: stats.average_segment_size,
            uniform_segments: stats.uniform_segments,
        },
    };
    
    serde_json::to_string_pretty(&metadata)
        .map_err(|e| ContainerError::SerializationError(format!("Failed to serialize metadata: {}", e)))
}

/// Export raw image metadata as CSV
#[instrument]
pub fn export_metadata_csv(path: &str) -> Result<String, ContainerError> {
    debug!(path, "Exporting raw image metadata as CSV");
    
    let info = info(path)?;
    let stats = get_stats(path)?;
    
    let mut csv = String::new();
    
    // Header section
    csv.push_str("# Raw Image Metadata\n");
    csv.push_str("Format,RAW\n");
    csv.push_str(&format!("Total Size,{}\n", info.total_size));
    csv.push_str(&format!("Total Size (Formatted),\"{}\"\n", stats.total_size_formatted));
    csv.push_str(&format!("Segment Count,{}\n", info.segment_count));
    csv.push_str(&format!("Largest Segment,{}\n", stats.largest_segment));
    csv.push_str(&format!("Smallest Segment,{}\n", stats.smallest_segment));
    csv.push_str(&format!("Average Segment Size,{}\n", stats.average_segment_size));
    csv.push_str(&format!("Uniform Segments,{}\n", stats.uniform_segments));
    csv.push('\n');
    
    // Segment details
    csv.push_str("# Segment Details\n");
    csv.push_str("Index,Name,Size,Size (Formatted)\n");
    
    for (i, (name, &size)) in info.segment_names.iter().zip(info.segment_sizes.iter()).enumerate() {
        csv.push_str(&format!(
            "{},\"{}\",{},\"{}\"\n",
            i + 1,
            name,
            size,
            crate::common::format_size(size)
        ));
    }
    
    Ok(csv)
}

// =============================================================================
// Helper Functions  
// =============================================================================

/// Discover all segments for a raw image - uses common segment discovery
fn discover_segments(path: &str) -> Result<(Vec<std::path::PathBuf>, Vec<u64>), ContainerError> {
    trace!(path, "Discovering raw image segments");
    discover_numbered_segments(path)
        .map_err(ContainerError::IoError)
}

// =============================================================================
// Virtual Filesystem Implementation
// =============================================================================

pub mod vfs {
    //! # Raw Image Virtual Filesystem Implementation
    //!
    //! Read-only virtual filesystem for raw disk images.
    //! Supports two modes:
    //! - Physical: Exposes the raw image as a single virtual file
    //! - Filesystem: Auto-detects partitions and mounts filesystems

    use std::sync::{Arc, RwLock};
    use crate::common::vfs::{
        VirtualFileSystem, VfsError, FileAttr, DirEntry, normalize_path,
        MountedPartition, find_partition,
    };
    use crate::common::filesystem::{
        SeekableBlockDevice, BlockReader, BlockDevice,
        detect_partition_table, mount_filesystem,
    };
    use crate::containers::ContainerError;
    use super::RawHandle;

    /// Operating mode for the Raw VFS
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RawVfsMode {
        /// Physical mode: expose disk image as a single file
        Physical,
        /// Filesystem mode: auto-mount filesystems from partitions
        Filesystem,
    }

    /// Block device adapter for RawHandle to work with filesystem drivers
    struct RawBlockDevice {
        handle: Arc<RwLock<RawHandle>>,
    }

    impl RawBlockDevice {
        fn new(handle: RawHandle) -> Self {
            Self {
                handle: Arc::new(RwLock::new(handle)),
            }
        }
    }

    impl crate::common::filesystem::BlockDevice for RawBlockDevice {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
            let mut handle = self.handle.write()
                .map_err(|e| ContainerError::InternalError(format!("Lock error: {}", e)))?;
            handle.position = offset;
            handle.read(buf)
        }

        fn size(&self) -> u64 {
            self.handle.read()
                .map(|h| h.total_size())
                .unwrap_or(0)
        }
    }

    impl SeekableBlockDevice for RawBlockDevice {
        fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
            Box::new(RawBlockReader {
                handle: Arc::clone(&self.handle),
                position: offset,
            })
        }
    }

    /// Block reader for specific offset in raw image
    struct RawBlockReader {
        handle: Arc<RwLock<RawHandle>>,
        position: u64,
    }

    impl std::io::Read for RawBlockReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let mut handle = self.handle.write()
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            handle.position = self.position;
            let bytes = handle.read(buf)
                .map_err(std::io::Error::other)?;
            self.position += bytes as u64;
            Ok(bytes)
        }
    }

    impl std::io::Seek for RawBlockReader {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            let size = self.handle.read()
                .map(|h| h.total_size())
                .unwrap_or(0);
            
            let new_offset = match pos {
                std::io::SeekFrom::Start(o) => o,
                std::io::SeekFrom::End(o) => {
                    if o >= 0 {
                        size.saturating_add(o as u64)
                    } else {
                        size.saturating_sub((-o) as u64)
                    }
                }
                std::io::SeekFrom::Current(o) => {
                    if o >= 0 {
                        self.position.saturating_add(o as u64)
                    } else {
                        self.position.saturating_sub((-o) as u64)
                    }
                }
            };
            self.position = new_offset.min(size);
            Ok(self.position)
        }
    }

    impl BlockReader for RawBlockReader {}

    /// Virtual filesystem implementation for raw disk images
    pub struct RawVfs {
        /// Operating mode
        mode: RawVfsMode,
        /// Image path
        #[allow(dead_code)]
        path: String,
        /// Block device for filesystem access
        device: Option<Arc<RawBlockDevice>>,
        /// Raw handle for Physical mode
        handle: Option<RwLock<RawHandle>>,
        /// Virtual file name (Physical mode)
        filename: String,
        /// Mounted partitions (Filesystem mode)
        partitions: Vec<MountedPartition>,
        /// Partition table info for display
        #[allow(dead_code)]
        partition_table: Option<crate::common::filesystem::PartitionTable>,
    }

    impl RawVfs {
        /// Open a raw image in Physical mode (single file view)
        pub fn open(path: &str) -> Result<Self, VfsError> {
            if !std::path::Path::new(path).exists() {
                return Err(VfsError::NotFound(path.to_string()));
            }
            
            let handle = RawHandle::open(path)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            // Generate filename from path
            let filename = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("disk");
            
            Ok(Self {
                mode: RawVfsMode::Physical,
                path: path.to_string(),
                device: None,
                handle: Some(RwLock::new(handle)),
                filename: format!("{}.raw", filename),
                partitions: Vec::new(),
                partition_table: None,
            })
        }

        /// Open a raw image in Filesystem mode (auto-mount partitions)
        pub fn open_filesystem(path: &str) -> Result<Self, VfsError> {
            if !std::path::Path::new(path).exists() {
                return Err(VfsError::NotFound(path.to_string()));
            }
            
            let handle = RawHandle::open(path)
                .map_err(|e| VfsError::IoError(e.to_string()))?;
            
            // Create block device adapter
            let device = Arc::new(RawBlockDevice::new(handle));
            
            // Detect partition table
            let partition_table = detect_partition_table(device.as_ref())
                .map_err(|e| VfsError::Internal(format!("Partition detection failed: {}", e)))?;
            
            tracing::info!(
                table_type = ?partition_table.table_type,
                partition_count = partition_table.partitions.len(),
                "Raw image partition table detected"
            );
            
            // Try to mount filesystems on each partition
            let mut partitions = Vec::new();
            for (idx, entry) in partition_table.partitions.iter().enumerate() {
                // Clone device for each partition (Arc handles shared access)
                let fs_device: Box<dyn SeekableBlockDevice> = Box::new(RawBlockDeviceWrapper {
                    inner: Arc::clone(&device),
                });
                
                match mount_filesystem(fs_device, entry.start_offset, entry.size) {
                    Ok(fs) => {
                        let fs_info = fs.info();
                        let mount_name = format!(
                            "Partition_{}_{:?}",
                            idx + 1,
                            fs_info.fs_type
                        );
                        tracing::info!(
                            partition = idx + 1,
                            fs_type = ?fs_info.fs_type,
                            label = ?fs_info.label,
                            "Mounted filesystem"
                        );
                        partitions.push(MountedPartition {
                            entry: entry.clone(),
                            fs,
                            mount_name,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            partition = idx + 1,
                            error = %e,
                            "Failed to mount partition filesystem"
                        );
                    }
                }
            }
            
            let filename = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("disk");
            
            Ok(Self {
                mode: RawVfsMode::Filesystem,
                path: path.to_string(),
                device: Some(device),
                handle: None,
                filename: format!("{}.raw", filename),
                partitions,
                partition_table: Some(partition_table),
            })
        }

        /// Get the current mode
        pub fn mode(&self) -> RawVfsMode {
            self.mode
        }

        /// Get number of mounted partitions
        pub fn partition_count(&self) -> usize {
            self.partitions.len()
        }

        /// Get the total disk size
        fn disk_size(&self) -> Result<u64, VfsError> {
            if let Some(ref device) = self.device {
                Ok(device.size())
            } else if let Some(ref handle) = self.handle {
                let h = handle.read()
                    .map_err(|e| VfsError::Internal(e.to_string()))?;
                Ok(h.total_size())
            } else {
                Err(VfsError::Internal("No device or handle available".to_string()))
            }
        }
    }

    /// Wrapper to make Arc<RawBlockDevice> implement SeekableBlockDevice
    struct RawBlockDeviceWrapper {
        inner: Arc<RawBlockDevice>,
    }

    impl crate::common::filesystem::BlockDevice for RawBlockDeviceWrapper {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ContainerError> {
            self.inner.read_at(offset, buf)
        }

        fn size(&self) -> u64 {
            self.inner.size()
        }
    }

    impl SeekableBlockDevice for RawBlockDeviceWrapper {
        fn reader_at(&self, offset: u64) -> Box<dyn BlockReader> {
            self.inner.reader_at(offset)
        }
    }

    impl VirtualFileSystem for RawVfs {
        fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
            let normalized = normalize_path(path);
            
            match self.mode {
                RawVfsMode::Physical => {
                    // Physical mode: single file view
                    if normalized == "/" {
                        Ok(FileAttr {
                            size: 0,
                            is_directory: true,
                            permissions: 0o555,
                            nlink: 2,
                            inode: 1,
                            ..Default::default()
                        })
                    } else if normalized == format!("/{}", self.filename) {
                        Ok(FileAttr {
                            size: self.disk_size()?,
                            is_directory: false,
                            permissions: 0o444,
                            nlink: 1,
                            inode: 2,
                            ..Default::default()
                        })
                    } else {
                        Err(VfsError::NotFound(normalized))
                    }
                }
                RawVfsMode::Filesystem => {
                    // Filesystem mode: mounted partitions
                    if normalized == "/" {
                        Ok(FileAttr {
                            size: 0,
                            is_directory: true,
                            permissions: 0o555,
                            nlink: 2 + self.partitions.len() as u32,
                            inode: 1,
                            ..Default::default()
                        })
                    } else if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                        partition.fs.getattr(&sub_path)
                    } else {
                        // Check if it's a partition mount point
                        for (idx, partition) in self.partitions.iter().enumerate() {
                            if normalized == format!("/{}", partition.mount_name) {
                                return Ok(FileAttr {
                                    size: 0,
                                    is_directory: true,
                                    permissions: 0o555,
                                    nlink: 2,
                                    inode: 100 + idx as u64,
                                    ..Default::default()
                                });
                            }
                        }
                        Err(VfsError::NotFound(normalized))
                    }
                }
            }
        }

        fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
            let normalized = normalize_path(path);
            
            match self.mode {
                RawVfsMode::Physical => {
                    if normalized == "/" {
                        Ok(vec![DirEntry {
                            name: self.filename.clone(),
                            is_directory: false,
                            inode: 2,
                            file_type: 8,
                        }])
                    } else {
                        Err(VfsError::NotADirectory(normalized))
                    }
                }
                RawVfsMode::Filesystem => {
                    if normalized == "/" {
                        // List mounted partitions
                        let entries: Vec<DirEntry> = self.partitions.iter()
                            .enumerate()
                            .map(|(idx, p)| DirEntry {
                                name: p.mount_name.clone(),
                                is_directory: true,
                                inode: 100 + idx as u64,
                                file_type: 4, // Directory
                            })
                            .collect();
                        Ok(entries)
                    } else if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                        partition.fs.readdir(&sub_path)
                    } else {
                        Err(VfsError::NotADirectory(normalized))
                    }
                }
            }
        }

        fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
            let normalized = normalize_path(path);
            
            match self.mode {
                RawVfsMode::Physical => {
                    if normalized == format!("/{}", self.filename) {
                        let handle = self.handle.as_ref()
                            .ok_or_else(|| VfsError::Internal("No handle in Physical mode".to_string()))?;
                        
                        let mut h = handle.write()
                            .map_err(|e| VfsError::Internal(e.to_string()))?;
                        
                        let total_size = h.total_size();
                        
                        if offset >= total_size {
                            return Ok(Vec::new());
                        }
                        
                        h.position = offset;
                        
                        let actual_size = size.min((total_size - offset) as usize);
                        let mut buf = vec![0u8; actual_size];
                        
                        let bytes_read = h.read(&mut buf)
                            .map_err(|e| VfsError::IoError(e.to_string()))?;
                        
                        buf.truncate(bytes_read);
                        Ok(buf)
                    } else if normalized == "/" {
                        Err(VfsError::NotAFile(normalized))
                    } else {
                        Err(VfsError::NotFound(normalized))
                    }
                }
                RawVfsMode::Filesystem => {
                    if let Some((partition, sub_path)) = find_partition(&self.partitions, &normalized) {
                        partition.fs.read(&sub_path, offset, size)
                    } else {
                        Err(VfsError::NotFound(normalized))
                    }
                }
            }
        }
    }
}

// Re-export VFS
pub use vfs::RawVfs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_raw() {
        assert!(is_raw("/path/to/image.dd").unwrap());
        assert!(is_raw("/path/to/image.raw").unwrap());
        assert!(is_raw("/path/to/image.img").unwrap());
        assert!(is_raw("/path/to/image.001").unwrap());
        assert!(is_raw("/path/to/image.002").unwrap());
        assert!(!is_raw("/path/to/image.e01").unwrap());
        assert!(!is_raw("/path/to/image.ad1").unwrap());
    }
}
