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
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::info;

use super::ewf_helpers::{is_system_boot_volume, nix_stat};

const MAX_AFF4_TRAVERSAL_DEPTH: usize = 128;
const MAX_AFF4_SOURCE_PATHS: usize = 10_000;
const MAX_AFF4_LOGICAL_ENTRIES: usize = 250_000;
const MAX_AFF4_HASH_ALGORITHMS: usize = 8;

// =============================================================================
// Types
// =============================================================================

/// Cancel flags for in-progress AFF4 exports, keyed by output path.
static AFF4_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
struct Aff4CancelRegistration {
    output_path: String,
}

impl Drop for Aff4CancelRegistration {
    fn drop(&mut self) {
        cleanup_aff4_cancel_flag(&self.output_path);
    }
}

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

fn validate_aff4_export_options(options: &Aff4ExportOptions) -> Result<(), String> {
    if options.output_path.trim().is_empty() {
        return Err("AFF4 export output path is required".to_string());
    }
    if options.source_paths.is_empty() {
        return Err("AFF4 export requires at least one source path".to_string());
    }
    if options.source_paths.len() > MAX_AFF4_SOURCE_PATHS {
        return Err(format!(
            "AFF4 export requested {} source paths, exceeding limit {}",
            options.source_paths.len(),
            MAX_AFF4_SOURCE_PATHS
        ));
    }
    if options
        .source_paths
        .iter()
        .any(|source_path| source_path.trim().is_empty())
    {
        return Err("AFF4 export source paths cannot be empty".to_string());
    }
    if let Some(hash_algorithms) = options.hash_algorithms.as_ref() {
        if hash_algorithms.is_empty() {
            return Err("AFF4 export hashAlgorithms cannot be empty".to_string());
        }
        if hash_algorithms.len() > MAX_AFF4_HASH_ALGORITHMS {
            return Err(format!(
                "AFF4 export requested {} hash algorithms, exceeding limit {}",
                hash_algorithms.len(),
                MAX_AFF4_HASH_ALGORITHMS
            ));
        }
        if hash_algorithms
            .iter()
            .any(|algorithm| algorithm.trim().is_empty())
        {
            return Err("AFF4 export hashAlgorithms cannot contain empty entries".to_string());
        }
    }

    Ok(())
}

fn parse_hash_algorithms(
    hash_algorithms: Option<&[String]>,
) -> Result<Vec<Aff4HashAlgorithm>, String> {
    let Some(hash_algorithms) = hash_algorithms else {
        return Ok(vec![Aff4HashAlgorithm::Sha256]);
    };

    let mut parsed = Vec::with_capacity(hash_algorithms.len());
    let mut seen = HashSet::with_capacity(hash_algorithms.len());
    for algorithm in hash_algorithms {
        let algorithm = parse_hash_algorithm(algorithm)?;
        if !seen.insert(algorithm) {
            return Err(format!(
                "AFF4 export hashAlgorithms contains duplicate algorithm: {:?}",
                algorithm
            ));
        }
        parsed.push(algorithm);
    }

    Ok(parsed)
}

fn register_aff4_cancel_flag(
    output_path: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<Aff4CancelRegistration, String> {
    let mut flags = AFF4_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if flags.contains_key(output_path) {
        return Err(format!(
            "An AFF4 export is already running for output path: {}",
            output_path
        ));
    }
    flags.insert(output_path.to_string(), cancel_flag);
    Ok(Aff4CancelRegistration {
        output_path: output_path.to_string(),
    })
}

fn cleanup_aff4_cancel_flag(output_path: &str) {
    if let Ok(mut flags) = AFF4_CANCEL_FLAGS.lock() {
        flags.remove(output_path);
    }
}

fn checked_aff4_total_size_add(total: u64, addition: u64, path: &Path) -> Result<u64, String> {
    total.checked_add(addition).ok_or_else(|| {
        format!(
            "AFF4 export total size overflow while adding {} bytes from {} to current total {} bytes",
            addition,
            path.display(),
            total
        )
    })
}

fn aff4_logical_entry_name(path: &Path) -> Result<String, String> {
    let Some(name) = path.file_name().filter(|name| !name.is_empty()) else {
        return Err(format!(
            "AFF4 export source path has no file name: {}",
            path.display()
        ));
    };
    Ok(name.to_string_lossy().to_string())
}

fn push_aff4_logical_entry(
    entries: &mut Vec<Aff4LogicalEntry>,
    entry: Aff4LogicalEntry,
    source_path: &Path,
) -> Result<(), String> {
    if entries.len() >= MAX_AFF4_LOGICAL_ENTRIES {
        return Err(format!(
            "AFF4 export expanded to more than {} logical entries while adding {}",
            MAX_AFF4_LOGICAL_ENTRIES,
            source_path.display()
        ));
    }
    entries.push(entry);
    Ok(())
}

fn total_aff4_entry_bytes(entries: &[Aff4LogicalEntry]) -> Result<u64, String> {
    entries.iter().try_fold(0u64, |total, entry| {
        let source_path = entry
            .source_path
            .as_deref()
            .unwrap_or_else(|| Path::new(""));
        checked_aff4_total_size_add(total, entry.size, source_path)
    })
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
    collect_entries_recursive_at_depth(dir, base, entries, 0)
}

fn collect_entries_recursive_at_depth(
    dir: &Path,
    base: &Path,
    entries: &mut Vec<Aff4LogicalEntry>,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_AFF4_TRAVERSAL_DEPTH {
        tracing::warn!(
            "Skipping directory {}: maximum AFF4 traversal depth {} exceeded",
            dir.display(),
            MAX_AFF4_TRAVERSAL_DEPTH
        );
        return Ok(());
    }

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
            push_aff4_logical_entry(
                entries,
                Aff4LogicalEntry {
                    original_path: rel_path,
                    size: 0,
                    is_directory: true,
                    last_written: None,
                    last_accessed: None,
                    record_changed: None,
                    birth_time: None,
                    source_path: Some(path.clone()),
                    hashes: HashMap::new(),
                },
                &path,
            )?;
            collect_entries_recursive_at_depth(&path, base, entries, depth + 1)?;
        } else if path.is_file() {
            push_aff4_logical_entry(
                entries,
                Aff4LogicalEntry::from_source(path.clone(), rel_path),
                &path,
            )?;
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
    validate_aff4_export_options(&options)?;

    // Parse compression
    let compression = match &options.compression {
        Some(c) => parse_compression(c)?,
        None => Aff4Compression::Stored,
    };

    // Parse hash algorithms
    let hash_algorithms = parse_hash_algorithms(options.hash_algorithms.as_deref())?;

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
    let _cancel_registration =
        register_aff4_cancel_flag(&options.output_path, cancel_flag.clone())?;

    // Collect all logical entries from source paths
    let mut entries: Vec<Aff4LogicalEntry> = Vec::new();
    for path_str in &options.source_paths {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            // Use the parent as base so the directory name is preserved
            let base = path.parent().unwrap_or(&path);
            collect_entries_recursive(&path, base, &mut entries)?;
        } else {
            let rel = aff4_logical_entry_name(&path)?;
            push_aff4_logical_entry(
                &mut entries,
                Aff4LogicalEntry::from_source(path.clone(), rel),
                &path,
            )?;
        }
    }

    let total_bytes = total_aff4_entry_bytes(&entries)?;
    let total_files = entries.len();

    info!(
        "AFF4 writer ready: {} entries, {:.1} MB total",
        total_files,
        total_bytes as f64 / 1_048_576.0
    );

    // Emit early "preparing" event
    crate::eventing::log_emit_result(
        "aff4-export-progress",
        window.emit(
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
        ),
    );

    // Check destination has enough free space
    if let Ok(avail) = nix_stat(&output_canon).map(|info| info.available_space) {
        if avail > 0 && total_bytes > avail {
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
            crate::eventing::log_emit_result(
                "aff4-export-progress",
                window_clone.emit(
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
                ),
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

    cleanup_aff4_cancel_flag(&output_path_for_cleanup);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_options() -> Aff4ExportOptions {
        Aff4ExportOptions {
            source_paths: vec!["source.bin".to_string()],
            output_path: "/tmp/case/logical.aff4".to_string(),
            compression: None,
            hash_algorithms: None,
            case_number: None,
            evidence_number: None,
            examiner_name: None,
            description: None,
            notes: None,
        }
    }

    fn logical_entry(size: u64, source_path: &str) -> Aff4LogicalEntry {
        Aff4LogicalEntry {
            original_path: source_path.to_string(),
            size,
            is_directory: false,
            last_written: None,
            last_accessed: None,
            record_changed: None,
            birth_time: None,
            source_path: Some(PathBuf::from(source_path)),
            hashes: HashMap::new(),
        }
    }

    #[test]
    fn validate_aff4_export_options_rejects_missing_output_path() {
        let mut options = minimal_options();
        options.output_path = " ".to_string();

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("output path is required"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_missing_sources() {
        let mut options = minimal_options();
        options.source_paths.clear();

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("requires at least one source path"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_excessive_source_paths() {
        let mut options = minimal_options();
        options.source_paths = vec!["source.bin".to_string(); MAX_AFF4_SOURCE_PATHS + 1];

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_empty_source_path() {
        let mut options = minimal_options();
        options.source_paths.push(" ".to_string());

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("source paths cannot be empty"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_empty_hash_list() {
        let mut options = minimal_options();
        options.hash_algorithms = Some(Vec::new());

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("hashAlgorithms cannot be empty"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_excessive_hash_algorithms() {
        let mut options = minimal_options();
        options.hash_algorithms = Some(vec!["sha256".to_string(); MAX_AFF4_HASH_ALGORITHMS + 1]);

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("hash algorithms"));
        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn validate_aff4_export_options_rejects_empty_hash_algorithm() {
        let mut options = minimal_options();
        options.hash_algorithms = Some(vec!["sha256".to_string(), " ".to_string()]);

        let err = validate_aff4_export_options(&options).unwrap_err();

        assert!(err.contains("hashAlgorithms cannot contain empty entries"));
    }

    #[test]
    fn parse_hash_algorithms_defaults_to_sha256() {
        let algorithms = parse_hash_algorithms(None).unwrap();

        assert_eq!(algorithms, vec![Aff4HashAlgorithm::Sha256]);
    }

    #[test]
    fn parse_hash_algorithms_rejects_duplicates() {
        let algorithms = vec!["sha256".to_string(), "SHA-256".to_string()];

        let err = parse_hash_algorithms(Some(&algorithms)).unwrap_err();

        assert!(err.contains("duplicate algorithm"));
    }

    #[test]
    fn checked_aff4_total_size_add_rejects_overflow() {
        let path = Path::new("overflow-source.bin");
        let err = checked_aff4_total_size_add(u64::MAX, 1, path).unwrap_err();

        assert!(err.contains("AFF4 export total size overflow"));
        assert!(err.contains("overflow-source.bin"));
    }

    #[test]
    fn aff4_logical_entry_name_rejects_root_path() {
        let err = aff4_logical_entry_name(Path::new("/")).unwrap_err();

        assert!(err.contains("has no file name"));
    }

    #[test]
    fn aff4_logical_entry_name_uses_file_name() {
        assert_eq!(
            aff4_logical_entry_name(Path::new("/tmp/source.bin")).unwrap(),
            "source.bin"
        );
    }

    #[test]
    fn push_aff4_logical_entry_rejects_expansion_over_limit() {
        let mut entries = vec![logical_entry(1, "source.bin"); MAX_AFF4_LOGICAL_ENTRIES];

        let err = push_aff4_logical_entry(
            &mut entries,
            logical_entry(1, "extra.bin"),
            Path::new("extra.bin"),
        )
        .unwrap_err();

        assert!(err.contains("expanded to more than"));
        assert_eq!(entries.len(), MAX_AFF4_LOGICAL_ENTRIES);
    }

    #[test]
    fn total_aff4_entry_bytes_rejects_overflow() {
        let entries = vec![
            logical_entry(u64::MAX - 5, "first.bin"),
            logical_entry(10, "second.bin"),
        ];

        let err = total_aff4_entry_bytes(&entries).unwrap_err();

        assert!(err.contains("AFF4 export total size overflow"));
    }

    #[test]
    fn register_aff4_cancel_flag_rejects_duplicate_output_path() {
        let output_path = unique_output_path();
        cleanup_aff4_cancel_flag(&output_path);

        let registration =
            register_aff4_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
        let err =
            register_aff4_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap_err();
        drop(registration);

        assert!(err.contains("already running"));
    }

    #[test]
    fn aff4_cancel_registration_cleans_up_on_drop() {
        let output_path = unique_output_path();
        cleanup_aff4_cancel_flag(&output_path);

        {
            let _registration =
                register_aff4_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
            assert!(aff4_cancel_export(output_path.clone()).unwrap());
        }

        assert!(!aff4_cancel_export(output_path).unwrap());
    }

    fn unique_output_path() -> String {
        format!(
            "/tmp/core-ffx-aff4-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }
}
