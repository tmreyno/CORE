// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Raw disk image (.dd/.img) export commands
//!
//! Creates raw (dd-style) forensic images from evidence files or devices.
//! Unlike E01, raw images have no container format — they are a byte-for-byte
//! copy with optional segmentation and concurrent hash computation.
//!
//! Safety validations and helpers are shared with the EWF exporter via
//! [`super::ewf_helpers`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

use super::ewf_helpers::{format_byte_size, is_system_boot_volume, nix_stat, walk_dir_files};

// =============================================================================
// Types
// =============================================================================

/// Cancel flags for in-progress raw exports, keyed by output path.
static RAW_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Options for creating a raw disk image
#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportOptions {
    /// Source file/device paths to image
    pub source_paths: Vec<String>,
    /// Output path (base, without extension — .dd will be appended)
    pub output_path: String,
    /// Segment/split size in bytes (0 = no splitting)
    pub segment_size: Option<u64>,
    /// Compute MD5 hash during imaging
    pub compute_md5: Option<bool>,
    /// Compute SHA-1 hash during imaging
    pub compute_sha1: Option<bool>,
    /// Compute SHA-256 hash during imaging
    pub compute_sha256: Option<bool>,
    /// Case number (for companion file)
    pub case_number: Option<String>,
    /// Evidence number (for companion file)
    pub evidence_number: Option<String>,
    /// Examiner name (for companion file)
    pub examiner_name: Option<String>,
    /// Description (for companion file)
    pub description: Option<String>,
    /// Notes (for companion file)
    pub notes: Option<String>,
}

/// Progress event payload for raw export
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportProgress {
    pub output_path: String,
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub phase: String,
    pub current_segment: usize,
}

/// Result of a completed raw export
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawExportResult {
    pub output_path: String,
    pub bytes_written: u64,
    pub files_included: usize,
    pub segments_created: usize,
    pub md5_hash: Option<String>,
    pub sha1_hash: Option<String>,
    pub sha256_hash: Option<String>,
    pub duration_ms: u64,
}

// =============================================================================
// Helpers
// =============================================================================

/// Generate the path for a given segment number.
/// Segment 1 → base.dd, segment 2 → base.002, segment 3 → base.003, etc.
fn segment_path(base: &str, segment: usize) -> String {
    if segment <= 1 {
        format!("{}.dd", base)
    } else {
        format!("{}.{:03}", base, segment)
    }
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Create a raw disk image (.dd) from source files/devices
///
/// Streams source data to output with optional segmentation and integrity hashing.
/// Progress is reported via `"raw-export-progress"` events.
#[tauri::command]
pub async fn raw_create_image(
    options: RawExportOptions,
    window: Window,
) -> Result<RawExportResult, String> {
    let start = std::time::Instant::now();

    info!(
        "Creating raw image at: {} (sources={}, segment_size={:?})",
        options.output_path,
        options.source_paths.len(),
        options.segment_size
    );

    // Set up cancel flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = RAW_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(options.output_path.clone(), cancel_flag.clone());
    }

    // --- Safety validations ---

    // Refuse to image the running system's boot volume
    for path_str in &options.source_paths {
        let canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| Path::new(path_str).to_path_buf());
        if is_system_boot_volume(&canon) {
            cleanup_cancel_flag(&options.output_path);
            return Err(format!(
                "Refusing to image the system boot volume ({}). Imaging the running OS disk can produce inconsistent data. \
                 Use an external boot environment or a write-blocker for system drive acquisition.",
                path_str
            ));
        }
    }

    // Verify output destination is NOT on any of the source volumes
    let output_dir = Path::new(&options.output_path)
        .parent()
        .unwrap_or_else(|| Path::new(&options.output_path));
    let output_canon =
        std::fs::canonicalize(output_dir).unwrap_or_else(|_| output_dir.to_path_buf());
    for path_str in &options.source_paths {
        let source_canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| Path::new(path_str).to_path_buf());
        if output_canon.starts_with(&source_canon) || source_canon.starts_with(&output_canon) {
            cleanup_cancel_flag(&options.output_path);
            return Err(format!(
                "Output destination ({}) overlaps with source ({}). \
                 Writing the image to the same volume being imaged will corrupt the output. \
                 Please choose a destination on a different volume.",
                output_dir.display(),
                path_str,
            ));
        }
    }

    // Calculate total size of source files (directories are walked recursively)
    let mut total_bytes: u64 = 0;
    let mut file_sizes: Vec<(String, u64)> = Vec::new();
    for path_str in &options.source_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            cleanup_cancel_flag(&options.output_path);
            return Err(format!("Source file does not exist: {}", path_str));
        }
        if path.is_dir() {
            let dir_files = walk_dir_files(path)?;
            if dir_files.is_empty() {
                warn!("Directory contains no files: {}", path_str);
            }
            for (fpath, fsize) in dir_files {
                total_bytes += fsize;
                file_sizes.push((fpath, fsize));
            }
            info!(
                "Expanded directory {} into {} files",
                path_str,
                file_sizes.len()
            );
        } else {
            let metadata = std::fs::metadata(path)
                .map_err(|e| format!("Failed to read metadata for {}: {}", path_str, e))?;
            let size = metadata.len();
            total_bytes += size;
            file_sizes.push((path_str.clone(), size));
        }
    }

    // Check destination has enough free space
    if let Ok(dest_meta) = nix_stat(&output_canon) {
        let avail = dest_meta.available_space;
        if avail > 0 && total_bytes > avail {
            let need = format_byte_size(total_bytes);
            let have = format_byte_size(avail);
            cleanup_cancel_flag(&options.output_path);
            return Err(format!(
                "Insufficient disk space on the destination volume. \
                 The source data is approximately {} but only {} is available. \
                 Free up space or choose a different destination.",
                need, have
            ));
        }
    }

    // Emit initial progress
    let _ = window.emit(
        "raw-export-progress",
        RawExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: 0,
            total_files: file_sizes.len(),
            bytes_written: 0,
            total_bytes,
            percent: 0.0,
            phase: "Initializing".to_string(),
            current_segment: 1,
        },
    );

    // Set up streaming hashers
    let compute_md5 = options.compute_md5.unwrap_or(true);
    let compute_sha1 = options.compute_sha1.unwrap_or(false);
    let compute_sha256 = options.compute_sha256.unwrap_or(true);

    use md5::Digest as _;
    let mut md5_hasher: Option<md5::Md5> = if compute_md5 {
        Some(md5::Md5::new())
    } else {
        None
    };
    let mut sha1_hasher: Option<sha1::Sha1> = if compute_sha1 {
        Some(sha1::Sha1::new())
    } else {
        None
    };
    let mut sha256_hasher: Option<sha2::Sha256> = if compute_sha256 {
        Some(sha2::Sha256::new())
    } else {
        None
    };

    let segment_size = options.segment_size.unwrap_or(0);
    let chunk_size: usize = 1024 * 1024; // 1 MB read chunks

    let mut global_bytes_written: u64 = 0;
    let mut current_segment: usize = 1;
    let mut segment_bytes_written: u64 = 0;

    // Open first output segment
    let first_seg_path = segment_path(&options.output_path, current_segment);
    let mut output_file = std::fs::File::create(&first_seg_path)
        .map_err(|e| format!("Failed to create output file {}: {}", first_seg_path, e))?;

    for (file_idx, (path_str, _file_size)) in file_sizes.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            warn!("Raw export cancelled");
            cleanup_cancel_flag(&options.output_path);
            return Err("Export cancelled".to_string());
        }

        let filename = Path::new(path_str)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        debug!(
            "Writing file {}/{}: {}",
            file_idx + 1,
            file_sizes.len(),
            filename
        );

        // Emit file start progress
        let _ = window.emit(
            "raw-export-progress",
            RawExportProgress {
                output_path: options.output_path.clone(),
                current_file: filename.clone(),
                file_index: file_idx + 1,
                total_files: file_sizes.len(),
                bytes_written: global_bytes_written,
                total_bytes,
                percent: if total_bytes > 0 {
                    (global_bytes_written as f64 / total_bytes as f64) * 100.0
                } else {
                    0.0
                },
                phase: format!("Writing {}", filename),
                current_segment,
            },
        );

        // Read and write file in chunks
        let file = std::fs::File::open(path_str)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut reader = std::io::BufReader::with_capacity(chunk_size, file);
        let mut buf = vec![0u8; chunk_size];

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                warn!("Raw export cancelled during write");
                cleanup_cancel_flag(&options.output_path);
                return Err("Export cancelled".to_string());
            }

            use std::io::Read;
            let bytes_read = reader
                .read(&mut buf)
                .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;
            if bytes_read == 0 {
                break;
            }

            let data = &buf[..bytes_read];

            // Update hashers
            if let Some(ref mut h) = md5_hasher {
                h.update(data);
            }
            if let Some(ref mut h) = sha1_hasher {
                h.update(data);
            }
            if let Some(ref mut h) = sha256_hasher {
                h.update(data);
            }

            // Handle segmentation: split data across segments if needed
            let mut data_offset = 0;
            while data_offset < bytes_read {
                let remaining_in_chunk = bytes_read - data_offset;

                if segment_size > 0 {
                    let space_in_segment = segment_size - segment_bytes_written;
                    let write_len = remaining_in_chunk.min(space_in_segment as usize);

                    use std::io::Write;
                    output_file
                        .write_all(&data[data_offset..data_offset + write_len])
                        .map_err(|e| {
                            format!("Failed to write to segment {}: {}", current_segment, e)
                        })?;

                    data_offset += write_len;
                    segment_bytes_written += write_len as u64;
                    global_bytes_written += write_len as u64;

                    // Check if segment is full → open next segment
                    if segment_bytes_written >= segment_size {
                        drop(output_file);
                        current_segment += 1;
                        segment_bytes_written = 0;
                        let next_seg = segment_path(&options.output_path, current_segment);
                        output_file = std::fs::File::create(&next_seg).map_err(|e| {
                            format!("Failed to create segment file {}: {}", next_seg, e)
                        })?;
                        info!("Opened segment {} at {}", current_segment, next_seg);
                    }
                } else {
                    // No segmentation — write all at once
                    use std::io::Write;
                    output_file
                        .write_all(&data[data_offset..])
                        .map_err(|e| format!("Failed to write output: {}", e))?;
                    global_bytes_written += remaining_in_chunk as u64;
                    data_offset = bytes_read;
                }
            }

            // Emit progress every 1 MB
            if global_bytes_written % (1024 * 1024) < chunk_size as u64 {
                let _ = window.emit(
                    "raw-export-progress",
                    RawExportProgress {
                        output_path: options.output_path.clone(),
                        current_file: filename.clone(),
                        file_index: file_idx + 1,
                        total_files: file_sizes.len(),
                        bytes_written: global_bytes_written,
                        total_bytes,
                        percent: if total_bytes > 0 {
                            (global_bytes_written as f64 / total_bytes as f64) * 100.0
                        } else {
                            100.0
                        },
                        phase: format!("Writing {}", filename),
                        current_segment,
                    },
                );
            }
        }
    }

    // Flush and close last segment
    {
        use std::io::Write;
        output_file
            .flush()
            .map_err(|e| format!("Failed to flush output: {}", e))?;
    }
    drop(output_file);

    // Compute final hashes
    let md5_hex = md5_hasher.map(|h| hex::encode(h.finalize()));
    let sha1_hex = sha1_hasher.map(|h| hex::encode(h.finalize()));
    let sha256_hex = sha256_hasher.map(|h| hex::encode(h.finalize()));

    // Emit finalization progress
    let _ = window.emit(
        "raw-export-progress",
        RawExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: file_sizes.len(),
            total_files: file_sizes.len(),
            bytes_written: global_bytes_written,
            total_bytes,
            percent: 99.0,
            phase: "Finalizing...".to_string(),
            current_segment,
        },
    );

    // Clean up cancel flag
    cleanup_cancel_flag(&options.output_path);

    let duration = start.elapsed();

    // Emit completion
    let _ = window.emit(
        "raw-export-progress",
        RawExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: file_sizes.len(),
            total_files: file_sizes.len(),
            bytes_written: global_bytes_written,
            total_bytes,
            percent: 100.0,
            phase: "Complete".to_string(),
            current_segment,
        },
    );

    let output_path_final = segment_path(&options.output_path, 1);

    info!(
        "Raw export complete: {} bytes written across {} segment(s) in {:.1}s",
        global_bytes_written,
        current_segment,
        duration.as_secs_f64()
    );

    Ok(RawExportResult {
        output_path: output_path_final,
        bytes_written: global_bytes_written,
        files_included: file_sizes.len(),
        segments_created: current_segment,
        md5_hash: md5_hex,
        sha1_hash: sha1_hex,
        sha256_hash: sha256_hex,
        duration_ms: duration.as_millis() as u64,
    })
}

/// Cancel an in-progress raw image export
#[tauri::command]
pub fn raw_cancel_export(output_path: String) -> Result<bool, String> {
    let flags = RAW_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.get(&output_path) {
        flag.store(true, Ordering::Relaxed);
        info!("Cancelled raw export: {}", output_path);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Helper to clean up cancel flag on exit (success or error)
fn cleanup_cancel_flag(output_path: &str) {
    if let Ok(mut flags) = RAW_CANCEL_FLAGS.lock() {
        flags.remove(output_path);
    }
}
