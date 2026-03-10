// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF/E01 export commands
//!
//! Provides Tauri commands for creating E01 forensic images from evidence files.
//! Uses the libewf-ffi crate for native EWF format writing with full metadata
//! support (case info, examiner, hashes, compression).
//!
//! Types are in [`super::ewf_export_types`], helpers in [`super::ewf_helpers`],
//! and the EWF reader command in [`super::ewf_read`].

use libewf_ffi::{
    EwfCaseInfo, EwfCompression, EwfCompressionMethod, EwfFormat, EwfWriter, EwfWriterConfig,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

// Re-export types for backward compatibility (mod.rs does `pub use ewf_export::*`)
pub use super::ewf_export_types::{EwfExportOptions, EwfExportProgress, EwfExportResult};
pub use super::ewf_read::{ewf_read_image_info, EwfImageInfoResponse, EwfReadCaseInfoResponse};

use super::ewf_export_types::EWF_CANCEL_FLAGS;
use super::ewf_helpers::{
    format_byte_size, is_system_boot_volume, nix_stat, parse_compression, parse_compression_method,
    parse_format, walk_dir_files,
};

// =============================================================================
// Tauri Commands
// =============================================================================

/// Get the libewf library version
#[tauri::command]
pub fn ewf_get_version() -> String {
    libewf_ffi::libewf_version()
}

/// Create an E01 forensic image from source files
///
/// This command reads source files and writes them into an E01/L01 container
/// with full forensic metadata (case info, hashes, compression).
#[tauri::command]
pub async fn ewf_create_image(
    options: EwfExportOptions,
    window: Window,
) -> Result<EwfExportResult, String> {
    let start = std::time::Instant::now();

    // Parse format
    let format = match &options.format {
        Some(f) => parse_format(f)?,
        None => EwfFormat::Encase5,
    };

    // Parse compression
    let compression = match &options.compression {
        Some(c) => parse_compression(c)?,
        None => EwfCompression::Fast,
    };

    // Parse compression method
    let compression_method = match &options.compression_method {
        Some(m) => parse_compression_method(m)?,
        None => EwfCompressionMethod::Deflate,
    };

    info!(
        "Creating {} image at: {} (format={:?}, compression={:?}, method={:?}, files={})",
        format.extension(),
        options.output_path,
        format,
        compression,
        compression_method,
        options.source_paths.len()
    );

    // Set up cancel flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(options.output_path.clone(), cancel_flag.clone());
    }

    // --- Safety validations ---

    // Refuse to image the running system's boot volume
    for path_str in &options.source_paths {
        let canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| Path::new(path_str).to_path_buf());
        if is_system_boot_volume(&canon) {
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
            return Err(format!("Source file does not exist: {}", path_str));
        }
        if path.is_dir() {
            // Recursively enumerate all files in the directory
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

    // Check destination has enough free space (use total_bytes as a conservative
    // upper bound — compression will typically reduce it, but we can't predict the ratio).
    if let Ok(dest_meta) = nix_stat(&output_canon) {
        let avail = dest_meta.available_space;
        if avail > 0 && total_bytes > avail {
            let need = format_byte_size(total_bytes);
            let have = format_byte_size(avail);
            warn!(
                "Destination may not have enough space: need ~{} but only {} free",
                need, have
            );
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
        "ewf-export-progress",
        EwfExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: 0,
            total_files: file_sizes.len(),
            bytes_written: 0,
            total_bytes,
            percent: 0.0,
            phase: "Initializing".to_string(),
        },
    );

    // Build config
    let config = EwfWriterConfig {
        format,
        compression,
        compression_method,
        segment_size: options
            .segment_size
            .unwrap_or(libewf_ffi::ffi::LIBEWF_DEFAULT_SEGMENT_FILE_SIZE),
        media_size: Some(total_bytes),
        case_info: EwfCaseInfo {
            case_number: options.case_number.clone(),
            evidence_number: options.evidence_number.clone(),
            examiner_name: options.examiner_name.clone(),
            description: options.description.clone(),
            notes: options.notes.clone(),
            ..Default::default()
        },
        ..Default::default()
    };

    // Create the writer
    let mut writer = EwfWriter::create(&options.output_path, config)
        .map_err(|e| format!("Failed to create EWF writer: {}", e))?;

    // Write source files
    let compute_md5 = options.compute_md5.unwrap_or(true);
    let compute_sha1 = options.compute_sha1.unwrap_or(false);
    let mut global_bytes_written: u64 = 0;
    let chunk_size = 64 * 1024; // 64 KB read chunks

    // Set up streaming hashers
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

    for (file_idx, (path_str, _file_size)) in file_sizes.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            warn!("EWF export cancelled");
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
            "ewf-export-progress",
            EwfExportProgress {
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
            },
        );

        // Read and write file in chunks
        let mut file = std::fs::File::open(path_str)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut buf = vec![0u8; chunk_size];

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                warn!("EWF export cancelled during write");
                return Err("Export cancelled".to_string());
            }

            use std::io::Read;
            let bytes_read = file
                .read(&mut buf)
                .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;
            if bytes_read == 0 {
                break;
            }

            let data = &buf[..bytes_read];

            // Update hashers
            if let Some(ref mut hasher) = md5_hasher {
                hasher.update(data);
            }
            if let Some(ref mut h) = sha1_hasher {
                h.update(data);
            }

            // Write to EWF
            writer
                .write_all(data)
                .map_err(|e| format!("Failed to write to EWF: {}", e))?;

            global_bytes_written += bytes_read as u64;

            // Emit progress every 1 MB
            if global_bytes_written % (1024 * 1024) < chunk_size as u64 {
                let _ = window.emit(
                    "ewf-export-progress",
                    EwfExportProgress {
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
                    },
                );
            }
        }
    }

    // Compute final hashes
    let md5_hex = md5_hasher.map(|h| hex::encode(h.finalize()));
    let sha1_hex = sha1_hasher.map(|h| hex::encode(h.finalize()));

    // Set hash values in the EWF container
    if let Some(ref hash) = md5_hex {
        writer
            .set_md5_hash(hash)
            .map_err(|e| format!("Failed to set MD5 hash: {}", e))?;
    }
    if let Some(ref hash) = sha1_hex {
        writer
            .set_sha1_hash(hash)
            .map_err(|e| format!("Failed to set SHA1 hash: {}", e))?;
    }

    // Finalize
    let _ = window.emit(
        "ewf-export-progress",
        EwfExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: file_sizes.len(),
            total_files: file_sizes.len(),
            bytes_written: global_bytes_written,
            total_bytes,
            percent: 99.0,
            phase: "Finalizing container...".to_string(),
        },
    );

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize EWF container: {}", e))?;

    // Clean up cancel flag
    {
        let mut flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.remove(&options.output_path);
    }

    let duration = start.elapsed();
    let format_str = format.extension().trim_start_matches('.').to_string();

    // Emit completion
    let _ = window.emit(
        "ewf-export-progress",
        EwfExportProgress {
            output_path: options.output_path.clone(),
            current_file: String::new(),
            file_index: file_sizes.len(),
            total_files: file_sizes.len(),
            bytes_written: global_bytes_written,
            total_bytes,
            percent: 100.0,
            phase: "Complete".to_string(),
        },
    );

    info!(
        "E01 export complete: {} bytes written in {:.1}s",
        global_bytes_written,
        duration.as_secs_f64()
    );

    Ok(EwfExportResult {
        output_path: format!("{}{}", options.output_path, format.extension()),
        format: format_str,
        bytes_written: global_bytes_written,
        files_included: file_sizes.len(),
        compressed: !matches!(compression, EwfCompression::None),
        md5_hash: md5_hex,
        sha1_hash: sha1_hex,
        duration_ms: duration.as_millis() as u64,
    })
}

/// Cancel an in-progress E01 export
#[tauri::command]
pub fn ewf_cancel_export(output_path: String) -> Result<bool, String> {
    let flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.get(&output_path) {
        flag.store(true, Ordering::Relaxed);
        info!("Cancelled EWF export: {}", output_path);
        Ok(true)
    } else {
        Ok(false)
    }
}
