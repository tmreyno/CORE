// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Unified Segment Hashing
//!
//! This module provides consolidated hashing functions for forensic container
//! segments. Previously, similar implementations existed in:
//! - `ewf/operations.rs`
//! - `raw.rs`
//! - `ufed/mod.rs`
//! - `ad1/operations.rs`
//!
//! All format modules should use these functions instead of duplicating code.
//!
//! # Features
//!
//! - Memory-mapped I/O for large files (>64MB)
//! - BLAKE3 parallel hashing with rayon
//! - Progress callbacks for UI integration
//! - Consistent error handling across formats
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::common::segment_hash::hash_segment_with_progress;
//!
//! let hash = hash_segment_with_progress(
//!     "/path/to/segment.E01",
//!     "sha256",
//!     |current, total| println!("{}/{}", current, total)
//! )?;
//! ```

use memmap2::Mmap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, instrument, trace};

use super::hash::StreamingHasher;
use super::{BUFFER_SIZE, MMAP_THRESHOLD};
use crate::containers::ContainerError;

// =============================================================================
// Primary Hashing Function
// =============================================================================

/// Hash a single segment file with progress reporting.
///
/// This is the canonical implementation for hashing forensic container segments.
/// All format-specific modules (EWF, RAW, UFED, AD1) should use this function
/// instead of implementing their own.
///
/// # Algorithm Optimization
///
/// | Algorithm | Strategy |
/// |-----------|----------|
/// | BLAKE3 | Memory-mapped I/O + rayon parallel hashing for large files |
/// | Others | Buffered streaming with periodic progress updates |
///
/// # Arguments
///
/// * `segment_path` - Path to the segment file
/// * `algorithm` - Hash algorithm name (md5, sha1, sha256, sha512, blake3, etc.)
/// * `progress_callback` - Called periodically with (bytes_processed, total_bytes)
///
/// # Returns
///
/// Hex-encoded hash string on success
///
/// # Errors
///
/// Returns `ContainerError` for:
/// - File not found
/// - I/O errors
/// - Invalid algorithm name
#[instrument(skip(progress_callback), fields(path = %segment_path))]
pub fn hash_segment_with_progress<F>(
    segment_path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    let path = Path::new(segment_path);
    if !path.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "Segment file not found: {}",
            segment_path
        )));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to get file metadata: {}", e)))?;
    let total_size = metadata.len();

    debug!(algorithm, total_size, "Hashing segment");

    let file = File::open(path)
        .map_err(|e| ContainerError::IoError(format!("Failed to open segment: {}", e)))?;

    let algorithm_lower = algorithm.to_lowercase();

    // For BLAKE3 with large files, use mmap + parallel hashing
    if algorithm_lower == "blake3" && total_size >= MMAP_THRESHOLD {
        trace!("Using BLAKE3 with memory-mapped parallel hashing");
        return hash_blake3_mmap(&file, total_size, &mut progress_callback);
    }

    // For other algorithms with large files, use mmap for better I/O
    if total_size >= MMAP_THRESHOLD {
        trace!("Using memory-mapped I/O for large file");
        return hash_mmap(&file, total_size, algorithm, &mut progress_callback);
    }

    // Standard buffered path for smaller files
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

    // For BLAKE3 without mmap, still use parallel hashing
    if algorithm_lower == "blake3" {
        trace!("Using BLAKE3 parallel hashing (buffered)");
        return hash_blake3_buffered(&mut reader, total_size, &mut progress_callback);
    }

    // For other algorithms, use StreamingHasher
    trace!("Using streaming hasher for {}", algorithm);
    hash_streaming(&mut reader, total_size, algorithm, &mut progress_callback)
}

/// Hash a segment file without progress reporting (convenience wrapper)
pub fn hash_segment(segment_path: &str, algorithm: &str) -> Result<String, ContainerError> {
    hash_segment_with_progress(segment_path, algorithm, |_, _| {})
}

// =============================================================================
// Implementation Details
// =============================================================================

/// BLAKE3 hashing with memory-mapped I/O and parallel processing
fn hash_blake3_mmap<F>(
    file: &File,
    total_size: u64,
    progress_callback: &mut F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    let mmap = unsafe { Mmap::map(file) }
        .map_err(|e| ContainerError::IoError(format!("Failed to memory-map file: {}", e)))?;

    let mut hasher = blake3::Hasher::new();
    let chunk_size = 64 * 1024 * 1024; // 64MB chunks for progress
    let mut bytes_processed = 0u64;

    for chunk in mmap.chunks(chunk_size) {
        hasher.update_rayon(chunk);
        bytes_processed += chunk.len() as u64;
        progress_callback(bytes_processed, total_size);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Generic hashing with memory-mapped I/O
fn hash_mmap<F>(
    file: &File,
    total_size: u64,
    algorithm: &str,
    progress_callback: &mut F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    let mmap = unsafe { Mmap::map(file) }
        .map_err(|e| ContainerError::IoError(format!("Failed to memory-map file: {}", e)))?;

    let mut hasher: StreamingHasher = algorithm.parse()?;
    let chunk_size = 64 * 1024 * 1024;
    let mut bytes_processed = 0u64;

    for chunk in mmap.chunks(chunk_size) {
        hasher.update(chunk);
        bytes_processed += chunk.len() as u64;
        progress_callback(bytes_processed, total_size);
    }

    Ok(hasher.finalize())
}

/// BLAKE3 hashing with buffered I/O and parallel processing
fn hash_blake3_buffered<R, F>(
    reader: &mut BufReader<R>,
    total_size: u64,
    progress_callback: &mut F,
) -> Result<String, ContainerError>
where
    R: std::io::Read,
    F: FnMut(u64, u64),
{
    let mut hasher = blake3::Hasher::new();
    let mut bytes_read_total = 0u64;
    let report_interval = (total_size / 20).max(BUFFER_SIZE as u64);
    let mut last_report = 0u64;

    loop {
        let buf = reader
            .fill_buf()
            .map_err(|e| ContainerError::IoError(format!("Read error: {}", e)))?;
        let len = buf.len();
        if len == 0 {
            break;
        }

        hasher.update_rayon(buf);
        reader.consume(len);

        bytes_read_total += len as u64;
        if bytes_read_total - last_report >= report_interval {
            progress_callback(bytes_read_total, total_size);
            last_report = bytes_read_total;
        }
    }

    progress_callback(total_size, total_size);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Generic streaming hash with buffered I/O
fn hash_streaming<R, F>(
    reader: &mut BufReader<R>,
    total_size: u64,
    algorithm: &str,
    progress_callback: &mut F,
) -> Result<String, ContainerError>
where
    R: std::io::Read,
    F: FnMut(u64, u64),
{
    let mut hasher: StreamingHasher = algorithm.parse()?;
    let mut bytes_read_total = 0u64;
    let report_interval = (total_size / 20).max(BUFFER_SIZE as u64);
    let mut last_report = 0u64;

    loop {
        let buf = reader
            .fill_buf()
            .map_err(|e| ContainerError::IoError(format!("Read error: {}", e)))?;
        let len = buf.len();
        if len == 0 {
            break;
        }

        hasher.update(buf);
        reader.consume(len);
        bytes_read_total += len as u64;

        if bytes_read_total - last_report >= report_interval {
            progress_callback(bytes_read_total, total_size);
            last_report = bytes_read_total;
        }
    }

    progress_callback(total_size, total_size);
    Ok(hasher.finalize())
}

// =============================================================================
// Multi-Segment Hashing
// =============================================================================

/// Hash multiple segments and combine into single hash.
///
/// This streams through all segments in order, feeding data into a single
/// hasher to produce a combined hash of the entire container.
///
/// # Arguments
///
/// * `segment_paths` - Ordered list of segment file paths
/// * `algorithm` - Hash algorithm name
/// * `progress_callback` - Called with (current_bytes, total_bytes) across all segments
///
/// # Returns
///
/// Hex-encoded hash of the combined segment data
#[instrument(skip(segment_paths, progress_callback))]
pub fn hash_segments_combined<F>(
    segment_paths: &[std::path::PathBuf],
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    if segment_paths.is_empty() {
        return Err(ContainerError::InvalidFormat(
            "No segments provided".to_string(),
        ));
    }

    // Calculate total size across all segments
    let total_size: u64 = segment_paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();

    debug!(
        segments = segment_paths.len(),
        total_size, algorithm, "Hashing combined segments"
    );

    let algorithm_lower = algorithm.to_lowercase();
    let mut bytes_processed = 0u64;

    // For BLAKE3, use parallel hashing
    if algorithm_lower == "blake3" {
        let mut hasher = blake3::Hasher::new();

        for segment_path in segment_paths {
            let file = File::open(segment_path).map_err(|e| {
                ContainerError::IoError(format!(
                    "Failed to open segment {}: {}",
                    segment_path.display(),
                    e
                ))
            })?;
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

            loop {
                let buf = reader
                    .fill_buf()
                    .map_err(|e| ContainerError::IoError(format!("Read error: {}", e)))?;
                let len = buf.len();
                if len == 0 {
                    break;
                }

                hasher.update_rayon(buf);
                reader.consume(len);

                bytes_processed += len as u64;
                progress_callback(bytes_processed, total_size);
            }
        }

        return Ok(hasher.finalize().to_hex().to_string());
    }

    // For other algorithms, use StreamingHasher
    let mut hasher: StreamingHasher = algorithm.parse()?;

    for segment_path in segment_paths {
        let file = File::open(segment_path).map_err(|e| {
            ContainerError::IoError(format!(
                "Failed to open segment {}: {}",
                segment_path.display(),
                e
            ))
        })?;
        let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

        loop {
            let buf = reader
                .fill_buf()
                .map_err(|e| ContainerError::IoError(format!("Read error: {}", e)))?;
            let len = buf.len();
            if len == 0 {
                break;
            }

            hasher.update(buf);
            reader.consume(len);

            bytes_processed += len as u64;
            progress_callback(bytes_processed, total_size);
        }
    }

    Ok(hasher.finalize())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_segment_md5() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data for hashing").unwrap();
        file.flush().unwrap();

        let hash = hash_segment(file.path().to_str().unwrap(), "md5").unwrap();
        assert_eq!(hash.len(), 32); // MD5 = 128 bits = 32 hex chars
    }

    #[test]
    fn test_hash_segment_sha256() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data for hashing").unwrap();
        file.flush().unwrap();

        let hash = hash_segment(file.path().to_str().unwrap(), "sha256").unwrap();
        assert_eq!(hash.len(), 64); // SHA256 = 256 bits = 64 hex chars
    }

    #[test]
    fn test_hash_segment_blake3() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data for hashing").unwrap();
        file.flush().unwrap();

        let hash = hash_segment(file.path().to_str().unwrap(), "blake3").unwrap();
        assert_eq!(hash.len(), 64); // BLAKE3 = 256 bits = 64 hex chars
    }

    #[test]
    fn test_hash_segment_not_found() {
        let result = hash_segment("/nonexistent/path.bin", "sha256");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_segment_with_progress() {
        let mut file = NamedTempFile::new().unwrap();
        let data = vec![0u8; 1024 * 1024]; // 1MB
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        let mut progress_calls = 0;
        let hash = hash_segment_with_progress(
            file.path().to_str().unwrap(),
            "sha256",
            |_current, _total| {
                progress_calls += 1;
            },
        )
        .unwrap();

        assert_eq!(hash.len(), 64);
        assert!(progress_calls > 0);
    }
}
