// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! AFF4 logical evidence export commands
//!
//! Creates AFF4 forensic containers using the pure-Rust `ffx-aff4` crate.
//! Supports AFF4-L (v1.1) logical file collections with:
//!   - Multiple compression algorithms (Deflate, LZ4, Snappy, Stored)
//!   - Multiple hash algorithms (MD5, SHA-1, SHA-256, SHA-512, Blake2b)
//!   - RDF/Turtle metadata (case info, timestamps, tool identity)
//!   - Cancellation via `AtomicBool` cancel flags
//!   - Progress events emitted via Tauri window

use ffx_aff4::{
    Aff4Compression, Aff4HashAlgorithm, Aff4LogicalEntry, Aff4LogicalWriter, Aff4Phase,
    Aff4Version, Aff4WriterConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::info;

use super::ewf_helpers::{is_system_boot_volume, nix_stat};

// =============================================================================
// Types
// =============================================================================

/// Cancel flags for in-progress AFF4 exports, keyed by output path.
static AFF4_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Options for creating an AFF4 logical image (received from frontend).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aff4ExportOptions {
    /// Source file/directory paths to include.
    pub source_paths: Vec<String>,
    /// Output .aff4 file path.
    pub output_path: String,
    /// Compression: "deflate" (default), "lz4", "snappy", "stored"/"none".
    pub compression: Option<String>,
    /// Hash algorithms to compute: "md5", "sha1", "sha256", "sha512", "blake2b".
    /// Default: ["sha256"].
    pub hash_algorithms: Option<Vec<String>>,
    /// Case number (embedded in RDF metadata).
    pub case_number: Option<String>,
    /// Evidence number.
    pub evidence_number: Option<String>,
    /// Examiner name.
    pub examiner_name: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Progress event payload for AFF4 export.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aff4ExportProgress {
    pub output_path: String,
    pub phase: String,
    pub current_file: String,
    pub files_processed: usize,
    pub total_files: usize,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub percent: f64,
}

/// Result of a completed AFF4 export.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aff4ExportResult {
    pub output_path: String,
    pub volume_urn: String,
    pub image_urn: String,
    pub total_bytes: u64,
    pub container_bytes: u64,
    pub compression_ratio: f64,
    pub bevy_count: u32,
    pub file_count: usize,
    pub linear_hashes: HashMap<String, String>,
    pub duration_ms: u64,
}

// =============================================================================
// Helpers
// =============================================================================

fn parse_compression(s: &str) -> Result<Aff4Compression, String> {
    match s.to_lowercase().as_str() {
        "deflate" | "default" => Ok(Aff4Compression::Deflate),
        "lz4" => Ok(Aff4Compression::Lz4),
        "snappy" => Ok(Aff4Compression::Snappy),
        "stored" | "none" | "store" => Ok(Aff4Compression::Stored),
        other => Err(format!(
            "Unknown AFF4 compression: '{}'. Supported: deflate, lz4, snappy, stored",
            other
        )),
    }
}

fn parse_hash_algorithm(s: &str) -> Result<Aff4HashAlgorithm, String> {
    match s.to_lowercase().as_str() {
        "md5" => Ok(Aff4HashAlgorithm::Md5),
        "sha1" | "sha-1" => Ok(Aff4HashAlgorithm::Sha1),
        "sha256" | "sha-256" => Ok(Aff4HashAlgorithm::Sha256),
        "sha512" | "sha-512" => Ok(Aff4HashAlgorithm::Sha512),
        "blake2b" => Ok(Aff4HashAlgorithm::Blake2b),
        other => Err(format!(
            "Unknown hash algorithm: '{}'. Supported: md5, sha1, sha256, sha512, blake2b",
            other
        )),
    }
}

fn phase_to_string(phase: Aff4Phase) -> &'static str {
    match phase {
        Aff4Phase::Preparing => "Preparing",
        Aff4Phase::WritingData => "WritingData",
        Aff4Phase::WritingMetadata => "WritingMetadata",
        Aff4Phase::ComputingHashes => "ComputingHashes",
        Aff4Phase::Finalizing => "Finalizing",
        Aff4Phase::Reading => "Reading",
        Aff4Phase::Verifying => "Verifying",
    }
}

/// Recursively collect logical entries from a directory.
/// Skips unreadable directories (e.g. macOS TCC-protected folders) with a warning.
fn collect_entries_recursive(
    dir: &Path,
    base: &Path,
    entries: &mut Vec<Aff4LogicalEntry>,
) -> Result<(), String> {
    let mut dir_entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            tracing::warn!("Skipping unreadable directory {}: {}", dir.display(), e);
            return Ok(());
        }
    };

    // Sort for deterministic output
    dir_entries.sort_by_key(|e| e.file_name());

    for dir_entry in dir_entries {
        let path = dir_entry.path();
        let file_name = dir_entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories
        if file_name.starts_with('.') {
            continue;
        }

        let rel_path = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            // Add directory entry
            entries.push(Aff4LogicalEntry {
                original_path: rel_path,
                size: 0,
                is_directory: true,
                last_written: None,
                last_accessed: None,
                record_changed: None,
                birth_time: None,
                source_path: Some(path.clone()),
                hashes: HashMap::new(),
            });
            collect_entries_recursive(&path, base, entries)?;
        } else if path.is_file() {
            entries.push(Aff4LogicalEntry::from_source(path, rel_path));
        }
    }

    Ok(())
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Create an AFF4 logical evidence container from source files/directories.
///
/// Emits `aff4-export-progress` events during the operation.
/// Can be cancelled via `aff4_cancel_export`.
#[tauri::command]
pub async fn aff4_create_image(
    options: Aff4ExportOptions,
    window: Window,
) -> Result<Aff4ExportResult, String> {
    let start = std::time::Instant::now();

    // Parse compression
    let compression = match &options.compression {
        Some(c) => parse_compression(c)?,
        None => Aff4Compression::Deflate,
    };

    // Parse hash algorithms
    let hash_algorithms: Vec<Aff4HashAlgorithm> = match &options.hash_algorithms {
        Some(algos) => algos
            .iter()
            .map(|a| parse_hash_algorithm(a))
            .collect::<Result<Vec<_>, _>>()?,
        None => vec![Aff4HashAlgorithm::Sha256],
    };

    info!(
        "Creating AFF4 image at: {} (compression={:?}, hashes={:?}, sources={})",
        options.output_path,
        compression,
        hash_algorithms,
        options.source_paths.len()
    );

    // Validate source paths exist
    for path_str in &options.source_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", path_str));
        }
    }

    // --- Safety validations ---

    // Refuse to image the system boot volume
    for path_str in &options.source_paths {
        let canon = std::fs::canonicalize(path_str).unwrap_or_else(|_| PathBuf::from(path_str));
        if is_system_boot_volume(&canon) {
            return Err(format!(
                "Refusing to image the system boot volume ({}). Imaging the running OS disk can \
                 produce inconsistent data. Use an external boot environment for system drive acquisition.",
                path_str
            ));
        }
    }

    // Verify output does not overlap with source
    let output_dir = Path::new(&options.output_path)
        .parent()
        .unwrap_or_else(|| Path::new(&options.output_path));
    let output_canon =
        std::fs::canonicalize(output_dir).unwrap_or_else(|_| output_dir.to_path_buf());
    for path_str in &options.source_paths {
        let source_canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| PathBuf::from(path_str));
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

    // Set up cancel flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = AFF4_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(options.output_path.clone(), cancel_flag.clone());
    }

    // Collect all logical entries from source paths
    let mut entries: Vec<Aff4LogicalEntry> = Vec::new();
    for path_str in &options.source_paths {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            // Use the parent as base so the directory name is preserved
            let base = path.parent().unwrap_or(&path);
            collect_entries_recursive(&path, base, &mut entries)?;
        } else {
            let rel = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            entries.push(Aff4LogicalEntry::from_source(path, rel));
        }
    }

    let total_bytes: u64 = entries.iter().map(|e| e.size).sum();
    let total_files = entries.len();

    info!(
        "AFF4 writer ready: {} entries, {:.1} MB total",
        total_files,
        total_bytes as f64 / 1_048_576.0
    );

    // Emit early "preparing" event
    let _ = window.emit(
        "aff4-export-progress",
        Aff4ExportProgress {
            output_path: options.output_path.clone(),
            phase: "Preparing".to_string(),
            current_file: String::new(),
            files_processed: 0,
            total_files,
            bytes_processed: 0,
            total_bytes,
            percent: 0.0,
        },
    );

    // Check destination has enough free space
    if let Ok(avail) = nix_stat(&output_canon).map(|info| info.available_space) {
        if avail > 0 && total_bytes > avail {
            // Clean up cancel flag before returning error
            if let Ok(mut flags) = AFF4_CANCEL_FLAGS.lock() {
                flags.remove(&options.output_path);
            }
            return Err(format!(
                "Insufficient disk space on the destination volume. \
                 The source data is approximately {:.1} GB but only {:.1} GB is available. \
                 Free up space or choose a different destination.",
                total_bytes as f64 / 1_073_741_824.0,
                avail as f64 / 1_073_741_824.0,
            ));
        }
    }

    // Build AFF4 writer config
    let config = Aff4WriterConfig {
        output_path: PathBuf::from(&options.output_path),
        version: Aff4Version::Logical,
        compression,
        linear_hashes: hash_algorithms.clone(),
        block_hashes: hash_algorithms,
        case_number: options.case_number.unwrap_or_default(),
        evidence_number: options.evidence_number.unwrap_or_default(),
        examiner: options.examiner_name.unwrap_or_default(),
        description: options.description.unwrap_or_default(),
        notes: options.notes.unwrap_or_default(),
        ..Default::default()
    };

    // Set up progress callback
    let window_clone = window.clone();
    let output_path_for_progress = options.output_path.clone();
    let progress_fn: Box<dyn FnMut(ffx_aff4::Aff4Progress) + Send> =
        Box::new(move |progress: ffx_aff4::Aff4Progress| {
            let _ = window_clone.emit(
                "aff4-export-progress",
                Aff4ExportProgress {
                    output_path: output_path_for_progress.clone(),
                    phase: phase_to_string(progress.phase).to_string(),
                    current_file: progress.current_file.clone(),
                    files_processed: progress.files_processed,
                    total_files: progress.total_files,
                    bytes_processed: progress.bytes_processed,
                    total_bytes: progress.total_bytes,
                    percent: progress.percent(),
                },
            );
        });

    // Run write operation in blocking task
    let output_path_for_cleanup = options.output_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        Aff4LogicalWriter::write_logical(
            &config,
            &mut entries,
            Some(&cancel_flag),
            Some(progress_fn),
        )
    })
    .await
    .map_err(|e| format!("AFF4 write task panicked: {}", e))?
    .map_err(|e| format!("AFF4 write error: {}", e))?;

    // Clean up cancel flag
    {
        let mut flags = AFF4_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.remove(&output_path_for_cleanup);
    }

    let duration = start.elapsed();
    info!(
        "AFF4 export complete: {} files, {:.1} MB data, {:.1} MB container ({:.1}% ratio), {:.1}s",
        result.file_count,
        result.total_bytes as f64 / 1_048_576.0,
        result.container_bytes as f64 / 1_048_576.0,
        result.compression_ratio * 100.0,
        duration.as_secs_f64()
    );

    // Convert hash keys from enum to string for frontend
    let linear_hashes: HashMap<String, String> = result
        .linear_hashes
        .iter()
        .map(|(k, v)| (format!("{:?}", k).to_lowercase(), v.clone()))
        .collect();

    Ok(Aff4ExportResult {
        output_path: result.output_path.to_string_lossy().to_string(),
        volume_urn: result.volume_urn,
        image_urn: result.image_urn,
        total_bytes: result.total_bytes,
        container_bytes: result.container_bytes,
        compression_ratio: result.compression_ratio,
        bevy_count: result.bevy_count,
        file_count: result.file_count,
        linear_hashes,
        duration_ms: duration.as_millis() as u64,
    })
}

/// Cancel an in-progress AFF4 export.
#[tauri::command]
pub fn aff4_cancel_export(output_path: String) -> Result<bool, String> {
    let flags = AFF4_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.get(&output_path) {
        flag.store(true, Ordering::Relaxed);
        info!("Cancelled AFF4 export: {}", output_path);
        Ok(true)
    } else {
        Ok(false)
    }
}
