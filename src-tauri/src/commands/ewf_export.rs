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
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

// Re-export types for backward compatibility (mod.rs does `pub use ewf_export::*`)
pub use super::ewf_export_types::{EwfExportOptions, EwfExportProgress, EwfExportResult};
pub use super::ewf_read::{ewf_read_image_info, EwfImageInfoResponse, EwfReadCaseInfoResponse};

use super::ewf_export_types::EWF_CANCEL_FLAGS;
use super::ewf_helpers::{
    checked_stream_read_size, format_byte_size, is_system_boot_volume, nix_stat, parse_compression,
    parse_compression_method, parse_format, validate_snapshot_byte_count, walk_dir_files,
};

const MAX_EWF_SOURCE_PATHS: usize = 10_000;
const MAX_EWF_SOURCE_FILES: usize = 250_000;

#[derive(Debug)]
struct CreatedEwfOutputs {
    output_path: String,
    preexisting: HashSet<PathBuf>,
    cleanup_on_drop: bool,
}

impl CreatedEwfOutputs {
    fn new(output_path: &str, format: EwfFormat) -> Result<Self, String> {
        let primary_path = ewf_primary_output_path(output_path, format);
        if primary_path.exists() {
            return Err(format!(
                "EWF output already exists and will not be overwritten: {}",
                primary_path.display()
            ));
        }

        let preexisting = discover_ewf_output_candidates(output_path)?;
        if let Some(existing) = preexisting.iter().next() {
            return Err(format!(
                "EWF output segment already exists and will not be overwritten: {}",
                existing.display()
            ));
        }

        Ok(Self {
            output_path: output_path.to_string(),
            preexisting,
            cleanup_on_drop: true,
        })
    }

    fn disarm(&mut self) {
        self.cleanup_on_drop = false;
    }
}

impl Drop for CreatedEwfOutputs {
    fn drop(&mut self) {
        if !self.cleanup_on_drop {
            return;
        }

        match discover_ewf_output_candidates(&self.output_path) {
            Ok(paths) => {
                for path in paths {
                    if self.preexisting.contains(&path) {
                        continue;
                    }
                    if let Err(err) = std::fs::remove_file(&path) {
                        warn!(
                            path = %path.display(),
                            error = %err,
                            "Failed to remove incomplete EWF output segment"
                        );
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, "Failed to discover incomplete EWF output segments");
            }
        }
    }
}

#[derive(Debug)]
struct EwfCancelRegistration {
    output_path: String,
}

impl Drop for EwfCancelRegistration {
    fn drop(&mut self) {
        cleanup_ewf_cancel_flag(&self.output_path);
    }
}

fn validate_ewf_export_options(options: &EwfExportOptions) -> Result<(), String> {
    if options.output_path.trim().is_empty() {
        return Err("EWF export output path is required".to_string());
    }
    if options.source_paths.is_empty() {
        return Err("EWF export requires at least one source path".to_string());
    }
    if options.source_paths.len() > MAX_EWF_SOURCE_PATHS {
        return Err(format!(
            "EWF export requested {} source paths, exceeding limit {}",
            options.source_paths.len(),
            MAX_EWF_SOURCE_PATHS
        ));
    }
    if options
        .source_paths
        .iter()
        .any(|source_path| source_path.trim().is_empty())
    {
        return Err("EWF export source paths cannot be empty".to_string());
    }

    Ok(())
}

fn register_ewf_cancel_flag(
    output_path: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<EwfCancelRegistration, String> {
    let mut flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if flags.contains_key(output_path) {
        return Err(format!(
            "An EWF export is already running for output path: {}",
            output_path
        ));
    }
    flags.insert(output_path.to_string(), cancel_flag);
    Ok(EwfCancelRegistration {
        output_path: output_path.to_string(),
    })
}

fn cleanup_ewf_cancel_flag(output_path: &str) {
    if let Ok(mut flags) = EWF_CANCEL_FLAGS.lock() {
        flags.remove(output_path);
    }
}

fn checked_ewf_total_size_add(total: u64, addition: u64, path: &Path) -> Result<u64, String> {
    total.checked_add(addition).ok_or_else(|| {
        format!(
            "EWF export total size overflow while adding {} bytes from {} to current total {} bytes",
            addition,
            path.display(),
            total
        )
    })
}

fn push_ewf_source_file(
    file_sizes: &mut Vec<(String, u64)>,
    path: String,
    size: u64,
) -> Result<(), String> {
    if file_sizes.len() >= MAX_EWF_SOURCE_FILES {
        return Err(format!(
            "EWF export expanded to more than {} source files",
            MAX_EWF_SOURCE_FILES
        ));
    }
    file_sizes.push((path, size));
    Ok(())
}

fn ewf_primary_output_path(output_path: &str, format: EwfFormat) -> PathBuf {
    PathBuf::from(format!("{}{}", output_path, format.extension()))
}

fn discover_ewf_output_candidates(output_path: &str) -> Result<HashSet<PathBuf>, String> {
    let path = Path::new(output_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let base_name = path
        .file_name()
        .ok_or_else(|| format!("Invalid EWF output path: {}", output_path))?
        .to_string_lossy();
    let entries = std::fs::read_dir(parent).map_err(|e| {
        format!(
            "Failed to read EWF output directory {}: {}",
            parent.display(),
            e
        )
    })?;

    let mut candidates = HashSet::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to inspect EWF output directory entry in {}: {}",
                parent.display(),
                e
            )
        })?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if is_ewf_output_candidate_name(&file_name, &base_name) {
            candidates.insert(entry.path());
        }
    }

    Ok(candidates)
}

fn is_ewf_output_candidate_name(file_name: &str, base_name: &str) -> bool {
    let Some(extension) = file_name
        .strip_prefix(base_name)
        .and_then(|suffix| suffix.strip_prefix('.'))
    else {
        return false;
    };

    is_ewf_segment_extension(extension)
}

fn is_ewf_segment_extension(extension: &str) -> bool {
    let extension = extension.to_ascii_lowercase();
    if extension.len() == 3
        && extension.starts_with('e')
        && extension[1..].chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }

    extension.len() == 4
        && extension.starts_with("ex")
        && extension[2..].chars().all(|c| c.is_ascii_digit())
}

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
    validate_ewf_export_options(&options)?;

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
    let _cancel_registration = register_ewf_cancel_flag(&options.output_path, cancel_flag.clone())?;

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
                total_bytes = checked_ewf_total_size_add(total_bytes, fsize, Path::new(&fpath))?;
                push_ewf_source_file(&mut file_sizes, fpath, fsize)?;
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
            total_bytes = checked_ewf_total_size_add(total_bytes, size, path)?;
            push_ewf_source_file(&mut file_sizes, path_str.clone(), size)?;
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
    crate::eventing::log_emit_result(
        "ewf-export-progress",
        window.emit(
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
        ),
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
    let mut created_outputs = CreatedEwfOutputs::new(&options.output_path, format)?;
    let mut writer = EwfWriter::create(&options.output_path, config)
        .map_err(|e| format!("Failed to create EWF writer: {}", e))?;

    // Write source files
    let compute_md5 = options.compute_md5.unwrap_or(true);
    let compute_sha1 = options.compute_sha1.unwrap_or(false);
    let mut global_bytes_written: u64 = 0;
    let chunk_size = 1024 * 1024; // 1 MB read chunks (16x fewer syscalls than 64KB)

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

    for (file_idx, (path_str, file_size)) in file_sizes.iter().enumerate() {
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
        crate::eventing::log_emit_result(
            "ewf-export-progress",
            window.emit(
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
            ),
        );

        // Read and write file in chunks
        let file = std::fs::File::open(path_str)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut file = std::io::BufReader::with_capacity(chunk_size, file);
        let mut buf = vec![0u8; chunk_size];
        let mut file_bytes_written = 0u64;

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                warn!("EWF export cancelled during write");
                return Err("Export cancelled".to_string());
            }

            let remaining_for_file = file_size.saturating_sub(file_bytes_written);
            if remaining_for_file == 0 {
                break;
            }
            let read_size = checked_stream_read_size(remaining_for_file, chunk_size)?;

            use std::io::Read;
            let bytes_read = file
                .read(&mut buf[..read_size])
                .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;
            if bytes_read == 0 {
                validate_snapshot_byte_count(
                    "EWF export",
                    Path::new(path_str),
                    *file_size,
                    file_bytes_written,
                )?;
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

            global_bytes_written = global_bytes_written.saturating_add(bytes_read as u64);
            file_bytes_written = file_bytes_written.saturating_add(bytes_read as u64);

            // Emit progress every 1 MB
            if global_bytes_written % (1024 * 1024) < chunk_size as u64 {
                crate::eventing::log_emit_result(
                    "ewf-export-progress",
                    window.emit(
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
                    ),
                );
            }
        }

        validate_snapshot_byte_count(
            "EWF export",
            Path::new(path_str),
            *file_size,
            file_bytes_written,
        )?;
    }

    validate_snapshot_byte_count(
        "EWF export total",
        Path::new(&options.output_path),
        total_bytes,
        global_bytes_written,
    )?;

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
    crate::eventing::log_emit_result(
        "ewf-export-progress",
        window.emit(
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
        ),
    );

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize EWF container: {}", e))?;
    created_outputs.disarm();

    let duration = start.elapsed();
    let format_str = format.extension().trim_start_matches('.').to_string();

    // Emit completion
    crate::eventing::log_emit_result(
        "ewf-export-progress",
        window.emit(
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
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_options() -> EwfExportOptions {
        EwfExportOptions {
            source_paths: vec!["source.bin".to_string()],
            output_path: "/tmp/case/image".to_string(),
            format: None,
            compression: None,
            compression_method: None,
            segment_size: None,
            case_number: None,
            evidence_number: None,
            examiner_name: None,
            description: None,
            notes: None,
            compute_md5: None,
            compute_sha1: None,
        }
    }

    #[test]
    fn validate_ewf_export_options_rejects_missing_output_path() {
        let mut options = minimal_options();
        options.output_path = "  ".to_string();

        let err = validate_ewf_export_options(&options).unwrap_err();

        assert!(err.contains("output path is required"));
    }

    #[test]
    fn validate_ewf_export_options_rejects_missing_sources() {
        let mut options = minimal_options();
        options.source_paths.clear();

        let err = validate_ewf_export_options(&options).unwrap_err();

        assert!(err.contains("requires at least one source path"));
    }

    #[test]
    fn validate_ewf_export_options_rejects_excessive_source_paths() {
        let mut options = minimal_options();
        options.source_paths = vec!["source.bin".to_string(); MAX_EWF_SOURCE_PATHS + 1];

        let err = validate_ewf_export_options(&options).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn validate_ewf_export_options_rejects_empty_source_path() {
        let mut options = minimal_options();
        options.source_paths.push(" ".to_string());

        let err = validate_ewf_export_options(&options).unwrap_err();

        assert!(err.contains("source paths cannot be empty"));
    }

    #[test]
    fn checked_ewf_total_size_add_rejects_overflow() {
        let path = Path::new("overflow-source.bin");
        let err = checked_ewf_total_size_add(u64::MAX, 1, path).unwrap_err();

        assert!(err.contains("EWF export total size overflow"));
        assert!(err.contains("overflow-source.bin"));
    }

    #[test]
    fn push_ewf_source_file_rejects_expansion_over_limit() {
        let mut file_sizes = vec![("source.bin".to_string(), 1); MAX_EWF_SOURCE_FILES];

        let err = push_ewf_source_file(&mut file_sizes, "extra.bin".to_string(), 1).unwrap_err();

        assert!(err.contains("expanded to more than"));
        assert_eq!(file_sizes.len(), MAX_EWF_SOURCE_FILES);
    }

    #[test]
    fn register_ewf_cancel_flag_rejects_duplicate_output_path() {
        let output_path = unique_output_path();
        cleanup_ewf_cancel_flag(&output_path);

        let registration =
            register_ewf_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
        let err =
            register_ewf_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap_err();
        drop(registration);

        assert!(err.contains("already running"));
    }

    #[test]
    fn ewf_cancel_registration_cleans_up_on_drop() {
        let output_path = unique_output_path();
        cleanup_ewf_cancel_flag(&output_path);

        {
            let _registration =
                register_ewf_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
            assert!(ewf_cancel_export(output_path.clone()).unwrap());
        }

        assert!(!ewf_cancel_export(output_path).unwrap());
    }

    #[test]
    fn ewf_primary_output_path_uses_format_extension() {
        assert_eq!(
            ewf_primary_output_path("/tmp/case/image", EwfFormat::Encase5),
            PathBuf::from("/tmp/case/image.E01")
        );
        assert_eq!(
            ewf_primary_output_path("/tmp/case/image", EwfFormat::V2Encase7),
            PathBuf::from("/tmp/case/image.Ex01")
        );
    }

    #[test]
    fn is_ewf_output_candidate_name_matches_ewf_segments_only() {
        assert!(is_ewf_output_candidate_name("image.E01", "image"));
        assert!(is_ewf_output_candidate_name("image.E99", "image"));
        assert!(is_ewf_output_candidate_name("image.Ex00", "image"));
        assert!(is_ewf_output_candidate_name("image.ex01", "image"));
        assert!(!is_ewf_output_candidate_name("image.txt", "image"));
        assert!(!is_ewf_output_candidate_name("image.E", "image"));
        assert!(!is_ewf_output_candidate_name("image.EAA", "image"));
        assert!(!is_ewf_output_candidate_name("other.E01", "image"));
    }

    #[test]
    fn created_ewf_outputs_rejects_existing_primary_segment() {
        let dir = tempfile::TempDir::new().unwrap();
        let output_path = dir.path().join("case-image");
        std::fs::write(dir.path().join("case-image.E01"), b"existing").unwrap();

        let err =
            CreatedEwfOutputs::new(output_path.to_str().unwrap(), EwfFormat::Encase5).unwrap_err();

        assert!(err.contains("already exists"));
    }

    #[test]
    fn created_ewf_outputs_rejects_existing_later_segment() {
        let dir = tempfile::TempDir::new().unwrap();
        let output_path = dir.path().join("case-image");
        std::fs::write(dir.path().join("case-image.E02"), b"existing").unwrap();

        let err =
            CreatedEwfOutputs::new(output_path.to_str().unwrap(), EwfFormat::Encase5).unwrap_err();

        assert!(err.contains("output segment already exists"));
    }

    #[test]
    fn created_ewf_outputs_removes_created_segments_on_drop() {
        let dir = tempfile::TempDir::new().unwrap();
        let output_path = dir.path().join("case-image");
        let e01_path = dir.path().join("case-image.E01");
        let e02_path = dir.path().join("case-image.E02");
        let ex_path = dir.path().join("case-image.Ex00");
        let unrelated_path = dir.path().join("case-image.txt");

        {
            let _guard =
                CreatedEwfOutputs::new(output_path.to_str().unwrap(), EwfFormat::Encase5).unwrap();
            std::fs::write(&e01_path, b"partial").unwrap();
            std::fs::write(&e02_path, b"partial").unwrap();
            std::fs::write(&ex_path, b"partial").unwrap();
            std::fs::write(&unrelated_path, b"keep").unwrap();
        }

        assert!(!e01_path.exists());
        assert!(!e02_path.exists());
        assert!(!ex_path.exists());
        assert!(unrelated_path.exists());
    }

    #[test]
    fn created_ewf_outputs_preserves_segments_after_disarm() {
        let dir = tempfile::TempDir::new().unwrap();
        let output_path = dir.path().join("case-image");
        let e01_path = dir.path().join("case-image.E01");

        {
            let mut guard =
                CreatedEwfOutputs::new(output_path.to_str().unwrap(), EwfFormat::Encase5).unwrap();
            std::fs::write(&e01_path, b"complete").unwrap();
            guard.disarm();
        }

        assert!(e01_path.exists());
    }

    fn unique_output_path() -> String {
        format!(
            "/tmp/core-ffx-ewf-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }
}
