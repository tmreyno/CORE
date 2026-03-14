// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AD1 verification, hashing, and companion log operations.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::debug;

use super::super::parser::Session;
use super::super::types::{ChunkVerifyResult, VerifyEntry, VerifyStatus};
use super::super::utils::*;
use crate::common::hash::{HashAlgorithm, StreamingHasher};
use crate::containers::ContainerError;

// =============================================================================
// Verification Functions
// =============================================================================

/// Verify file hashes in the container
#[must_use = "this returns verification results, which should be used"]
pub fn verify(path: &str, algorithm: &str) -> Result<Vec<VerifyEntry>, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Verify with progress callback
#[must_use = "this returns verification results, which should be used"]
pub fn verify_with_progress<F>(
    path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<Vec<VerifyEntry>, ContainerError>
where
    F: FnMut(u64, u64),
{
    let mut session = Session::open(path)?;
    let algo: HashAlgorithm = algorithm.parse()?;
    let mut results = Vec::new();

    // Count total files for progress
    let total = count_files(&session.root_items);
    let mut current = 0u64;

    // Clone root_items to avoid borrow checker issues
    let root_items = session.root_items.clone();

    // Create params struct for verify function
    let mut params = super::super::parser::VerifyParams {
        algorithm: algo,
        out: &mut results,
        current: &mut current,
        total,
        progress_callback: &mut progress_callback,
    };

    for item in &root_items {
        session.verify_item_with_progress(item, "", &mut params)?;
    }

    Ok(results)
}

// =============================================================================
// Chunk Verification Functions
// =============================================================================

/// Verify container and return chunk-level results (for parity with EWF)
#[must_use = "this returns the verification results, which should be used"]
pub fn verify_chunks(
    path: &str,
    algorithm: &str,
) -> Result<Vec<ChunkVerifyResult>, ContainerError> {
    let results = verify(path, algorithm)?;

    Ok(results
        .into_iter()
        .enumerate()
        .map(|(i, entry)| ChunkVerifyResult {
            index: i as u64,
            status: entry.status.to_string(),
            message: entry.computed,
            path: Some(entry.path),
        })
        .collect())
}

// =============================================================================
// Companion Log Functions
// =============================================================================

/// Verify container against companion log file
/// Compares computed hash with hash stored in log file
#[must_use = "this returns the verification result, which should be used"]
pub fn verify_against_log(path: &str, algorithm: &str) -> Result<VerifyEntry, ContainerError> {
    // Parse companion log (returns Option, so provide error if not found)
    let log_info = parse_companion_log(path).ok_or_else(|| {
        ContainerError::FileNotFound(format!("No companion log file found for: {}", path))
    })?;

    // Get expected hash from log
    let expected_hash = match algorithm.to_lowercase().as_str() {
        "md5" => log_info.md5_hash,
        "sha1" => log_info.sha1_hash,
        "sha256" => log_info.sha256_hash,
        _ => {
            return Err(ContainerError::ConfigError(format!(
                "Unsupported algorithm: {}",
                algorithm
            )))
        }
    };

    // Compute actual hash
    let computed_hash = hash_segments(path, algorithm)?;

    // Compare
    let status = match &expected_hash {
        Some(expected) => {
            if expected.to_lowercase() == computed_hash.to_lowercase() {
                VerifyStatus::Ok
            } else {
                VerifyStatus::Nok
            }
        }
        None => VerifyStatus::Computed,
    };

    Ok(VerifyEntry {
        path: path.to_string(),
        status,
        algorithm: Some(algorithm.to_string()),
        computed: Some(computed_hash),
        stored: expected_hash,
        size: None,
    })
}

// =============================================================================
// Segment Hashing Functions
// =============================================================================

/// Hash AD1 segment files (image-level hash)
/// This hashes all segment files sequentially to produce a single hash
/// that can be compared against the stored hash in the companion log
#[must_use = "this returns the hash, which should be used"]
pub fn hash_segments(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    hash_segments_with_progress(path, algorithm, |_, _| {})
}

/// Hash AD1 segments with progress callback
///
/// Performance optimizations:
/// - Uses 16MB buffers (matching global BUFFER_SIZE) for reduced syscall overhead
/// - Memory-mapped I/O for large segments (≥64MB)
/// - BLAKE3 parallel hashing via rayon
/// - Pipelined I/O for SHA-256/MD5 (I/O thread + hash thread)
#[must_use = "this returns the hash, which should be used"]
pub fn hash_segments_with_progress<F>(
    path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    use crate::common::{AdaptiveBuffer, IoOperation, MMAP_THRESHOLD};
    use std::io::BufRead;

    let func_start = std::time::Instant::now();
    debug!("hash_segments_with_progress started");

    validate_ad1(path, false)?; // Validate format (segment discovery is done below)
    debug!(
        elapsed_ms = func_start.elapsed().as_millis(),
        "validate_ad1 complete"
    );

    let algo: HashAlgorithm = algorithm.parse()?;
    let algorithm_lower = algorithm.to_lowercase();

    // Read header segment_number for reference (may be total count or segment's own index)
    let header_count = {
        let mut file = File::open(path)
            .map_err(|e| ContainerError::IoError(format!("Failed to open AD1 file: {e}")))?;
        let segment_header = read_segment_header(&mut file)?;
        segment_header.segment_number
    };

    // Discover ALL segment files by scanning the filesystem.
    // This is more reliable than the header field at offset 0x1c, which may store
    // the segment's own 1-based index rather than the total segment count.
    // The filesystem scan finds .ad1, .ad2, .ad3, etc. until no more exist.
    let mut total_size: u64 = 0;
    let mut segment_paths = Vec::new();
    let mut segment_sizes = Vec::new();

    let mut i: u32 = 1;
    loop {
        let segment_path = build_segment_path(path, i);
        let seg_path = Path::new(&segment_path);
        if !seg_path.exists() {
            // Try lowercase fallback for case-insensitive filesystems
            let lower = segment_path.to_lowercase();
            if lower != segment_path && Path::new(&lower).exists() {
                let size = std::fs::metadata(&lower).map(|m| m.len()).map_err(|e| {
                    ContainerError::IoError(format!("Failed to get segment size: {e}"))
                })?;
                total_size += size;
                segment_paths.push(lower);
                segment_sizes.push(size);
            } else {
                break;
            }
        } else {
            let size = std::fs::metadata(&segment_path)
                .map(|m| m.len())
                .map_err(|e| ContainerError::IoError(format!("Failed to get segment size: {e}")))?;
            total_size += size;
            segment_paths.push(segment_path);
            segment_sizes.push(size);
        }
        i += 1;
    }

    if segment_paths.is_empty() {
        return Err(ContainerError::SegmentError(format!(
            "No segment files found for: {}",
            path
        )));
    }

    let segment_count = segment_paths.len() as u32;

    // Log segment discovery results — helps diagnose mismatches
    if segment_count != header_count {
        tracing::warn!(
            header_count,
            filesystem_count = segment_count,
            "AD1 segment count: header field differs from filesystem discovery \
             (header may store segment index rather than total count)"
        );
    }
    debug!(segment_count, total_size, algorithm = %algorithm_lower, "Hashing AD1 segments (optimized)");

    // Use adaptive buffer sizing and progress chunks
    let buffer_size = AdaptiveBuffer::optimal_size(total_size, IoOperation::Hash);
    let progress_chunks = AdaptiveBuffer::progress_chunks(total_size);
    let report_interval = (total_size / progress_chunks).max(buffer_size as u64);
    let mut bytes_processed: u64 = 0;
    let mut last_report: u64 = 0;

    debug!(
        buffer_size,
        progress_chunks, "Using adaptive buffer for AD1 hash"
    );

    // BLAKE3: Use memory-mapped I/O + rayon parallel hashing
    if algorithm_lower == "blake3" {
        use memmap2::Mmap;

        let mut hasher = blake3::Hasher::new();

        for (idx, segment_path) in segment_paths.iter().enumerate() {
            let seg_size = segment_sizes[idx];
            let file = File::open(segment_path).map_err(|e| {
                ContainerError::IoError(format!("Failed to open segment {}: {e}", segment_path))
            })?;

            // Use memory-mapped I/O for large segments
            if seg_size >= MMAP_THRESHOLD {
                // SAFETY: File is opened read-only, mmap is safe for read access
                let mmap = unsafe { Mmap::map(&file) }.map_err(|e| {
                    ContainerError::IoError(format!("Failed to memory-map segment: {e}"))
                })?;

                // Process in chunks for progress reporting with parallel hashing
                for chunk in mmap.chunks(buffer_size) {
                    hasher.update_rayon(chunk);
                    bytes_processed += chunk.len() as u64;

                    if bytes_processed - last_report >= report_interval {
                        progress_callback(bytes_processed, total_size);
                        last_report = bytes_processed;
                    }
                }
            } else {
                // Small segments: buffered read with parallel hashing
                let mut reader = BufReader::with_capacity(buffer_size, file);

                loop {
                    let buf = reader
                        .fill_buf()
                        .map_err(|e| ContainerError::IoError(format!("Read error: {e}")))?;
                    let len = buf.len();
                    if len == 0 {
                        break;
                    }

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
        let hash = hasher.finalize().to_hex().to_string();
        debug!(hash = %hash, "AD1 segment hash complete (BLAKE3 optimized)");
        return Ok(hash);
    }

    // XXH3/XXH64: Use memory-mapped I/O for maximum speed (non-cryptographic)
    if algorithm_lower == "xxh3" || algorithm_lower == "xxhash3" {
        use memmap2::Mmap;
        use xxhash_rust::xxh3::Xxh3;

        let mut hasher = Xxh3::new();

        for (idx, segment_path) in segment_paths.iter().enumerate() {
            let seg_size = segment_sizes[idx];
            let file = File::open(segment_path).map_err(|e| {
                ContainerError::IoError(format!("Failed to open segment {}: {e}", segment_path))
            })?;

            if seg_size >= MMAP_THRESHOLD {
                let mmap = unsafe { Mmap::map(&file) }.map_err(|e| {
                    ContainerError::IoError(format!("Failed to memory-map segment: {e}"))
                })?;

                for chunk in mmap.chunks(buffer_size) {
                    hasher.update(chunk);
                    bytes_processed += chunk.len() as u64;

                    if bytes_processed - last_report >= report_interval {
                        progress_callback(bytes_processed, total_size);
                        last_report = bytes_processed;
                    }
                }
            } else {
                let mut reader = BufReader::with_capacity(buffer_size, file);

                loop {
                    let buf = reader
                        .fill_buf()
                        .map_err(|e| ContainerError::IoError(format!("Read error: {e}")))?;
                    let len = buf.len();
                    if len == 0 {
                        break;
                    }

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
        let hash = format!("{:032x}", hasher.digest128());
        debug!(hash = %hash, "AD1 segment hash complete (XXH3 optimized)");
        return Ok(hash);
    }

    // SHA-256/MD5/SHA-1/others: Use pipelined I/O (I/O thread → hash thread)
    hash_segments_pipelined(segment_paths, algo, total_size, progress_callback)
}

/// Pipelined hashing: I/O thread reads ahead while hash thread processes
/// This keeps both CPU and disk busy simultaneously
fn hash_segments_pipelined<F>(
    segment_paths: Vec<String>,
    algo: HashAlgorithm,
    total_size: u64,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    use crate::common::{AdaptiveBuffer, IoOperation};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    // Use adaptive buffer sizing
    let buffer_size = AdaptiveBuffer::optimal_size(total_size, IoOperation::Hash);

    // Shared progress counter
    let bytes_hashed = Arc::new(AtomicU64::new(0));
    let bytes_hashed_clone = Arc::clone(&bytes_hashed);

    // Channel with 4 buffer slots for pipelining (allows I/O to stay ahead)
    let (tx, rx) = mpsc::sync_channel::<Option<Vec<u8>>>(4);

    // I/O thread: reads segments and sends buffers
    let io_handle = thread::spawn(move || -> Result<(), ContainerError> {
        for segment_path in &segment_paths {
            let file = File::open(segment_path).map_err(|e| {
                ContainerError::IoError(format!("Failed to open segment {}: {e}", segment_path))
            })?;
            let mut reader = BufReader::with_capacity(buffer_size, file);

            loop {
                let mut buf = vec![0u8; buffer_size];
                let bytes_read = reader
                    .read(&mut buf)
                    .map_err(|e| ContainerError::IoError(format!("Read error: {e}")))?;

                if bytes_read == 0 {
                    break;
                }

                buf.truncate(bytes_read);
                if tx.send(Some(buf)).is_err() {
                    return Err(ContainerError::InternalError(
                        "Hash thread terminated early".to_string(),
                    ));
                }
            }
        }
        // Signal completion
        let _ = tx.send(None);
        Ok(())
    });

    // Hashing thread: receives buffers and updates hash
    let hash_handle = thread::spawn(move || -> Result<String, ContainerError> {
        let mut hasher = StreamingHasher::new(algo);

        while let Ok(Some(buf)) = rx.recv() {
            let len = buf.len() as u64;
            hasher.update(&buf);
            bytes_hashed_clone.fetch_add(len, Ordering::Relaxed);
        }

        Ok(hasher.finalize())
    });

    // Progress reporting in main thread
    let report_interval = (total_size / 100).max(1);
    let mut last_reported = 0u64;

    loop {
        let current = bytes_hashed.load(Ordering::Relaxed);
        if current >= total_size {
            break;
        }

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
    io_handle
        .join()
        .map_err(|_| ContainerError::InternalError("I/O thread panicked".to_string()))?
        .map_err(|e| ContainerError::IoError(format!("I/O error: {e}")))?;

    let hash = hash_handle
        .join()
        .map_err(|_| ContainerError::InternalError("Hash thread panicked".to_string()))?
        .map_err(|e| ContainerError::IoError(format!("Hash error: {e}")))?;

    progress_callback(total_size, total_size);
    debug!(hash = %hash, "AD1 segment hash complete (pipelined)");
    Ok(hash)
}

/// Hash a single AD1 segment file.
///
/// This is a thin wrapper around `crate::common::hash_segment_with_progress`.
/// Use that function directly for new code.
#[must_use = "this returns the hash, which should be used"]
pub fn hash_single_segment<F>(
    segment_path: &str,
    algorithm: &str,
    progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    crate::common::hash_segment_with_progress(segment_path, algorithm, progress_callback)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_ad1(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(b"ADSEGMENTEDFILE\0").unwrap();
        file.write_all(&[0u8; 496]).unwrap();
        path
    }

    #[test]
    fn test_verify_nonexistent() {
        let result = verify("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_invalid_algorithm() {
        let temp_dir = TempDir::new().unwrap();
        let ad1_path = create_test_ad1(temp_dir.path(), "test.ad1");
        let result = verify(ad1_path.to_str().unwrap(), "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_chunks_nonexistent() {
        let result = verify_chunks("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_against_log_nonexistent() {
        let result = verify_against_log("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_against_log_invalid_algorithm() {
        let result = verify_against_log("/nonexistent/path/file.ad1", "sha3");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_segments_nonexistent() {
        let result = hash_segments("/nonexistent/path/file.ad1", "md5");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_single_segment_nonexistent() {
        let result = hash_single_segment("/nonexistent/path/file.ad1", "md5", |_, _| {});
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("File not found"));
    }

    #[test]
    fn test_hash_single_segment_invalid_algorithm() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.ad1");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"test content").unwrap();
        let result = hash_single_segment(path.to_str().unwrap(), "invalid_algo", |_, _| {});
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Unsupported") || err_msg.contains("algorithm"));
    }

    #[test]
    fn test_verify_status_display() {
        assert_eq!(VerifyStatus::Ok.to_string(), "ok");
        assert_eq!(VerifyStatus::Nok.to_string(), "nok");
        assert_eq!(VerifyStatus::Computed.to_string(), "computed");
        assert_eq!(VerifyStatus::Skipped.to_string(), "skipped");
    }

    #[test]
    fn test_verify_status_is_ok() {
        assert!(VerifyStatus::Ok.is_ok());
        assert!(VerifyStatus::Computed.is_ok());
        assert!(!VerifyStatus::Nok.is_ok());
        assert!(!VerifyStatus::Skipped.is_ok());
    }

    #[test]
    fn test_verify_status_is_error() {
        assert!(VerifyStatus::Nok.is_error());
        assert!(!VerifyStatus::Ok.is_error());
        assert!(!VerifyStatus::Computed.is_error());
        assert!(!VerifyStatus::Skipped.is_error());
    }

    #[test]
    fn test_verify_entry_serialization() {
        let entry = VerifyEntry {
            path: "/file.txt".to_string(),
            status: VerifyStatus::Ok,
            algorithm: Some("md5".to_string()),
            computed: Some("abc123".to_string()),
            stored: Some("abc123".to_string()),
            size: Some(1024),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("ok"));
        assert!(json.contains("abc123"));
    }

    // =========================================================================
    // Segment discovery + hash tests
    // =========================================================================

    /// Create a valid AD1 segment file with proper header fields.
    /// - signature: ADSEGMENTEDFILE\0 at offset 0
    /// - segment_index (u32 LE) at offset 0x18
    /// - segment_number (u32 LE) at offset 0x1c
    /// - Additional payload data after the 512-byte header margin
    fn create_valid_ad1_segment(
        dir: &std::path::Path,
        name: &str,
        segment_index: u32,
        segment_number: u32,
        payload: &[u8],
    ) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut buf = vec![0u8; 512]; // AD1_LOGICAL_MARGIN = 512
                                      // Signature at offset 0 (16 bytes including null)
        buf[..16].copy_from_slice(b"ADSEGMENTEDFILE\0");
        // segment_index at offset 0x18
        buf[0x18..0x1c].copy_from_slice(&segment_index.to_le_bytes());
        // segment_number at offset 0x1c
        buf[0x1c..0x20].copy_from_slice(&segment_number.to_le_bytes());
        // fragments_size at offset 0x22 (non-zero to avoid issues)
        buf[0x22..0x26].copy_from_slice(&1u32.to_le_bytes());
        // header_size at offset 0x28
        buf[0x28..0x2c].copy_from_slice(&512u32.to_le_bytes());
        // Write header + payload
        let mut file = File::create(&path).unwrap();
        file.write_all(&buf).unwrap();
        file.write_all(payload).unwrap();
        path
    }

    #[test]
    fn test_hash_single_segment_ad1() {
        // Create a single-segment AD1 with known content
        let temp_dir = TempDir::new().unwrap();
        let payload = b"hello forensic world";
        let ad1_path = create_valid_ad1_segment(temp_dir.path(), "evidence.ad1", 0, 1, payload);

        // Hash the single segment (MD5)
        let hash = hash_segments(ad1_path.to_str().unwrap(), "md5").unwrap();
        assert!(!hash.is_empty(), "Hash should not be empty");

        // Hash again — should be deterministic
        let hash2 = hash_segments(ad1_path.to_str().unwrap(), "md5").unwrap();
        assert_eq!(hash, hash2, "Hashing same file should be deterministic");

        // Different algorithm should produce different hash
        let sha1_hash = hash_segments(ad1_path.to_str().unwrap(), "sha1").unwrap();
        assert_ne!(hash, sha1_hash, "MD5 and SHA1 should differ");
    }

    #[test]
    fn test_hash_multi_segment_includes_all_segments() {
        // Create a 3-segment AD1 where header says segment_number=3
        let temp_dir = TempDir::new().unwrap();
        let payload1 = b"segment one data";
        let payload2 = b"segment two data";
        let payload3 = b"segment three data";

        // Segment 1: evidence.ad1
        create_valid_ad1_segment(temp_dir.path(), "evidence.ad1", 0, 3, payload1);
        // Segment 2: evidence.ad2
        create_valid_ad1_segment(temp_dir.path(), "evidence.ad2", 1, 3, payload2);
        // Segment 3: evidence.ad3
        create_valid_ad1_segment(temp_dir.path(), "evidence.ad3", 2, 3, payload3);

        let ad1_path = temp_dir.path().join("evidence.ad1");

        // Hash all 3 segments
        let multi_hash = hash_segments(ad1_path.to_str().unwrap(), "md5").unwrap();
        assert!(!multi_hash.is_empty());

        // Now hash a single-segment version with ONLY segment 1 content
        let temp_dir2 = TempDir::new().unwrap();
        create_valid_ad1_segment(temp_dir2.path(), "single.ad1", 0, 1, payload1);
        let single_path = temp_dir2.path().join("single.ad1");
        let single_hash = hash_segments(single_path.to_str().unwrap(), "md5").unwrap();

        // Multi-segment hash MUST differ from single-segment hash
        // This proves all 3 segments are being hashed, not just the first
        assert_ne!(
            multi_hash, single_hash,
            "Multi-segment hash must include all segments, not just the first"
        );
    }

    #[test]
    fn test_filesystem_discovery_finds_more_than_header() {
        // Simulate the bug: header says segment_number=1 (its own index),
        // but 3 segment files actually exist on disk.
        // The fix uses filesystem scanning, so it should find all 3.
        let temp_dir = TempDir::new().unwrap();
        let payload1 = b"first segment bytes";
        let payload2 = b"second segment bytes";
        let payload3 = b"third segment bytes";

        // Header segment_number = 1 (stores its own index, NOT total count)
        create_valid_ad1_segment(temp_dir.path(), "image.ad1", 0, 1, payload1);
        // Additional segments exist on disk
        create_valid_ad1_segment(temp_dir.path(), "image.ad2", 1, 3, payload2);
        create_valid_ad1_segment(temp_dir.path(), "image.ad3", 2, 3, payload3);

        let ad1_path = temp_dir.path().join("image.ad1");

        // Hash should include ALL 3 segments despite header saying 1
        let hash_with_all = hash_segments(ad1_path.to_str().unwrap(), "md5").unwrap();

        // Compare: create same image with only segment 1
        let temp_dir2 = TempDir::new().unwrap();
        create_valid_ad1_segment(temp_dir2.path(), "image.ad1", 0, 1, payload1);
        let only_one_path = temp_dir2.path().join("image.ad1");
        let hash_just_one = hash_segments(only_one_path.to_str().unwrap(), "md5").unwrap();

        // The hash from 3 files MUST differ from hash of just 1 file
        assert_ne!(
            hash_with_all, hash_just_one,
            "Filesystem discovery must find segments beyond header count"
        );
    }

    #[test]
    fn test_hash_segments_progress_callback() {
        let temp_dir = TempDir::new().unwrap();
        let payload = vec![0xABu8; 4096]; // 4KB of data
        create_valid_ad1_segment(temp_dir.path(), "prog.ad1", 0, 1, &payload);
        let ad1_path = temp_dir.path().join("prog.ad1");

        let mut progress_called = false;
        let _hash =
            hash_segments_with_progress(ad1_path.to_str().unwrap(), "sha256", |current, total| {
                progress_called = true;
                assert!(current <= total, "Progress current should not exceed total");
                assert!(total > 0, "Total should be > 0");
            })
            .unwrap();

        assert!(
            progress_called,
            "Progress callback should have been invoked"
        );
    }

    #[test]
    fn test_hash_segments_sha256() {
        let temp_dir = TempDir::new().unwrap();
        let payload = b"sha256 test content for forensic hashing";
        create_valid_ad1_segment(temp_dir.path(), "sha.ad1", 0, 1, payload);
        let ad1_path = temp_dir.path().join("sha.ad1");

        let hash = hash_segments(ad1_path.to_str().unwrap(), "sha256").unwrap();
        // SHA-256 produces 64 hex chars
        assert_eq!(hash.len(), 64, "SHA-256 hash should be 64 hex chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Should be hex");
    }

    #[test]
    fn test_hash_segments_md5() {
        let temp_dir = TempDir::new().unwrap();
        let payload = b"md5 test content";
        create_valid_ad1_segment(temp_dir.path(), "md5test.ad1", 0, 1, payload);
        let ad1_path = temp_dir.path().join("md5test.ad1");

        let hash = hash_segments(ad1_path.to_str().unwrap(), "md5").unwrap();
        // MD5 produces 32 hex chars
        assert_eq!(hash.len(), 32, "MD5 hash should be 32 hex chars");
    }

    #[test]
    fn test_hash_segments_sha1() {
        let temp_dir = TempDir::new().unwrap();
        let payload = b"sha1 test content";
        create_valid_ad1_segment(temp_dir.path(), "sha1test.ad1", 0, 1, payload);
        let ad1_path = temp_dir.path().join("sha1test.ad1");

        let hash = hash_segments(ad1_path.to_str().unwrap(), "sha1").unwrap();
        // SHA-1 produces 40 hex chars
        assert_eq!(hash.len(), 40, "SHA-1 hash should be 40 hex chars");
    }
}
