// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED verification and hashing operations
//!
//! Provides hash computation for UFED container files with support for
//! multiple algorithms (SHA-256, MD5, BLAKE3, XXH3) and progress callbacks.

use std::fs::File;
use std::io::BufReader;

use tracing::{debug, instrument};

use crate::common::hash::{HashAlgorithm, StreamingHasher};
use crate::containers::ContainerError;

/// Verify a UFED file by computing its hash
#[instrument]
pub fn verify(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify_with_progress(path, algorithm, |_, _| {})
}

/// Verify a UFED file with progress callback
///
/// Performance optimizations:
/// - Uses 16MB buffers for reduced syscall overhead
/// - Memory-mapped I/O for large files (≥64MB)
/// - BLAKE3 parallel hashing via rayon
/// - XXH3 optimized path for non-cryptographic checksums
#[instrument(skip(progress_callback))]
pub fn verify_with_progress<F>(
    path: &str,
    algorithm: &str,
    mut progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    use crate::common::{BUFFER_SIZE, MMAP_THRESHOLD};
    use std::io::BufRead;

    debug!(path = %path, algorithm = %algorithm, "Verifying UFED file (optimized)");

    let file =
        File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;

    let total_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file size: {e}"))?
        .len();

    let algorithm_lower = algorithm.to_lowercase();

    // Progress reporting interval (~50 updates total)
    let report_interval = (total_size / 50).max(BUFFER_SIZE as u64);
    let mut bytes_processed: u64 = 0;
    let mut last_report: u64 = 0;

    // BLAKE3: Use memory-mapped I/O + rayon parallel hashing
    if algorithm_lower == "blake3" {
        use memmap2::Mmap;

        let mut hasher = blake3::Hasher::new();

        if total_size >= MMAP_THRESHOLD {
            // Large file: memory-mapped I/O
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map file: {e}"))?;

            for chunk in mmap.chunks(BUFFER_SIZE) {
                hasher.update_rayon(chunk);
                bytes_processed += chunk.len() as u64;

                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            // Small file: buffered read with parallel hashing
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

            loop {
                let buf = reader
                    .fill_buf()
                    .map_err(|e| format!("Read error: {e}"))?;
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

        progress_callback(total_size, total_size);
        return Ok(hasher.finalize().to_hex().to_string());
    }

    // XXH3: Use memory-mapped I/O for maximum speed
    if algorithm_lower == "xxh3" || algorithm_lower == "xxhash3" {
        use memmap2::Mmap;
        use xxhash_rust::xxh3::Xxh3;

        let mut hasher = Xxh3::new();

        if total_size >= MMAP_THRESHOLD {
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| format!("Failed to memory-map file: {e}"))?;

            for chunk in mmap.chunks(BUFFER_SIZE) {
                hasher.update(chunk);
                bytes_processed += chunk.len() as u64;

                if bytes_processed - last_report >= report_interval {
                    progress_callback(bytes_processed, total_size);
                    last_report = bytes_processed;
                }
            }
        } else {
            let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

            loop {
                let buf = reader
                    .fill_buf()
                    .map_err(|e| format!("Read error: {e}"))?;
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

        progress_callback(total_size, total_size);
        return Ok(format!("{:032x}", hasher.digest128()));
    }

    // Other algorithms (SHA-256, MD5, etc.): Use optimized buffered I/O
    let algo = algorithm
        .parse::<HashAlgorithm>()
        .map_err(|e| format!("Unsupported algorithm: {e}"))?;
    let mut hasher = StreamingHasher::new(algo);

    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);

    loop {
        let buf = reader
            .fill_buf()
            .map_err(|e| format!("Read error: {e}"))?;
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

    progress_callback(total_size, total_size);
    Ok(hasher.finalize())
}

/// Verify a specific file referenced in UFED metadata
///
/// This computes the hash of a file and returns it for comparison
/// with stored hashes in the UFD metadata.
#[instrument]
pub fn verify_file(path: &str, algorithm: &str) -> Result<String, ContainerError> {
    verify(path, algorithm)
}

/// Hash a single file in a UFED container set.
///
/// This is a thin wrapper around `crate::common::hash_segment_with_progress`.
/// Use that function directly for new code.
#[instrument(skip(progress_callback))]
pub fn hash_single_segment<F>(
    segment_path: &str,
    algorithm: &str,
    progress_callback: F,
) -> Result<String, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(segment_path = %segment_path, algorithm = %algorithm, "Hashing single UFED segment");
    crate::common::hash_segment_with_progress(segment_path, algorithm, progress_callback)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_nonexistent_file() {
        let result = verify("/nonexistent/path/file.ufd", "sha256");
        assert!(result.is_err());
    }
}
