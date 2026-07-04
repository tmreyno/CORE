// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File copy and export commands
//!
//! Provides forensic-grade file copy operations with:
//! - Progress tracking
//! - Hash verification
//! - Metadata preservation
//! - Activity logging

use super::ewf_helpers::validate_snapshot_byte_count;
use crate::common::COPY_BUFFER_SIZE;
use crate::database;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

/// Cancel flags for in-progress export operations, keyed by operation_id
static EXPORT_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

const MAX_EXPORT_TRAVERSAL_DEPTH: usize = 128;

fn export_cancel_flags() -> std::sync::MutexGuard<'static, HashMap<String, Arc<AtomicBool>>> {
    match EXPORT_CANCEL_FLAGS.lock() {
        Ok(flags) => flags,
        Err(poisoned) => {
            warn!("EXPORT_CANCEL_FLAGS mutex poisoned; recovering cancel registry");
            poisoned.into_inner()
        }
    }
}

fn checked_export_copy_read_len(
    file_size: u64,
    bytes_copied: u64,
    source: &Path,
) -> Result<Option<usize>, String> {
    let remaining = file_size.checked_sub(bytes_copied).ok_or_else(|| {
        format!(
            "Copy byte counter exceeded source size for {}: copied {} bytes > expected {} bytes",
            source.display(),
            bytes_copied,
            file_size
        )
    })?;

    if remaining == 0 {
        return Ok(None);
    }

    let copy_buffer_size = u64::try_from(COPY_BUFFER_SIZE)
        .map_err(|_| "Copy buffer size does not fit in u64".to_string())?;
    usize::try_from(remaining.min(copy_buffer_size))
        .map(Some)
        .map_err(|_| "Copy read length does not fit in usize".to_string())
}

fn checked_export_copy_advance(
    bytes_copied: u64,
    bytes_read: usize,
    source: &Path,
) -> Result<u64, String> {
    let bytes_read = u64::try_from(bytes_read)
        .map_err(|_| "Copy read byte count does not fit in u64".to_string())?;
    bytes_copied.checked_add(bytes_read).ok_or_else(|| {
        format!(
            "Copy byte counter overflowed while reading {}: copied {} bytes, next chunk {} bytes",
            source.display(),
            bytes_copied,
            bytes_read
        )
    })
}

/// Progress event for copy/export operations
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyProgress {
    /// Operation ID
    pub operation_id: String,
    /// Current file being copied
    pub current_file: String,
    /// Current file index (1-based)
    pub current_index: usize,
    /// Total number of files
    pub total_files: usize,
    /// Bytes copied for current file
    pub current_file_bytes: u64,
    /// Total bytes for current file
    pub current_file_total: u64,
    /// Total bytes copied across all files
    pub total_bytes_copied: u64,
    /// Total bytes to copy
    pub total_bytes: u64,
    /// Progress percentage (0-100)
    pub percent: f64,
    /// Current operation status
    pub status: String,
    /// Copy speed in bytes per second
    pub speed_bps: u64,
    /// Current phase: "copying", "hashing", "verifying", "complete"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Bytes hashed so far (for hashing phase)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_bytes_processed: Option<u64>,
    /// Total bytes to hash (for hashing phase)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_bytes_total: Option<u64>,
}

/// Copy operation options
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyOptions {
    /// Compute SHA-256 hashes for all files
    #[serde(default)]
    pub compute_hashes: bool,
    /// Verify copied files match source hashes
    #[serde(default)]
    pub verify_after_copy: bool,
    /// Compare against known hashes (from hash cache/database)
    #[serde(default)]
    pub verify_against_known: bool,
    /// Generate JSON manifest file
    #[serde(default)]
    pub generate_json_manifest: bool,
    /// Generate TXT report file
    #[serde(default)]
    pub generate_txt_report: bool,
    /// Preserve file timestamps
    #[serde(default = "default_true")]
    pub preserve_timestamps: bool,
    /// Overwrite existing files
    #[serde(default)]
    pub overwrite: bool,
    /// Create parent directories
    #[serde(default = "default_true")]
    pub create_dirs: bool,
    /// Export name (for manifest/report filenames)
    #[serde(default)]
    pub export_name: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            compute_hashes: false,
            verify_after_copy: false,
            verify_against_known: false,
            generate_json_manifest: false,
            generate_txt_report: false,
            preserve_timestamps: true,
            overwrite: false,
            create_dirs: true,
            export_name: None,
        }
    }
}

/// Export metadata for forensic exports
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetadata {
    /// Source path
    pub source_path: String,
    /// Destination path
    pub destination_path: String,
    /// File size in bytes
    pub size: u64,
    /// SHA-256 hash of the file (if computed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Original modified time (Unix timestamp)
    pub modified_time: u64,
    /// Export timestamp (Unix timestamp)
    pub export_time: u64,
    /// Whether copy verification passed (destination matches source hash)
    pub copy_verified: bool,
    /// Known hash from database/cache (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_hash: Option<String>,
    /// Whether file matches known hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches_known: Option<bool>,
    /// Known hash source (e.g., "hash_cache", "database", "companion_log")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_hash_source: Option<String>,
}

/// Result of a copy/export operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    /// Unique operation ID for this export (e.g., "export-1719842300000")
    pub operation_id: String,
    /// Number of files copied successfully
    pub files_copied: usize,
    /// Number of files failed
    pub files_failed: usize,
    /// Total bytes copied
    pub bytes_copied: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Average speed in bytes per second
    pub avg_speed_bps: u64,
    /// Failed file paths with error messages
    pub failures: Vec<(String, String)>,
    /// Export metadata (when compute_hashes is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<ExportMetadata>>,
    /// Path to JSON manifest file (if generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_manifest_path: Option<String>,
    /// Path to TXT report file (if generated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txt_report_path: Option<String>,
    /// Number of files that match known hashes
    pub files_verified_known: usize,
    /// Number of files that don't match known hashes
    pub files_mismatch_known: usize,
}
/// Copy a single file with progress
#[allow(clippy::too_many_arguments)]
fn copy_file_with_progress(
    source: &Path,
    dest: &Path,
    window: &Window,
    operation_id: &str,
    file_index: usize,
    total_files: usize,
    total_bytes_so_far: u64,
    total_bytes: u64,
    start_time: std::time::Instant,
    cancel_flag: &AtomicBool,
    compute_hash: bool,
) -> Result<(u64, Option<String>), String> {
    // Check cancellation before starting
    if cancel_flag.load(Ordering::Relaxed) {
        return Err("Export cancelled".to_string());
    }

    let source_meta =
        fs::metadata(source).map_err(|e| format!("Failed to read source metadata: {}", e))?;
    let file_size = source_meta.len();

    // Open source and destination
    let src_file = File::open(source).map_err(|e| format!("Failed to open source: {}", e))?;
    let dst_file =
        File::create(dest).map_err(|e| format!("Failed to create destination: {}", e))?;

    let mut reader = BufReader::with_capacity(COPY_BUFFER_SIZE, src_file);
    let mut writer = BufWriter::with_capacity(COPY_BUFFER_SIZE, dst_file);
    let mut hasher = if compute_hash {
        Some(Sha256::new())
    } else {
        None
    };

    let mut bytes_copied = 0u64;
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
    let mut last_emit = std::time::Instant::now();

    let filename = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string_lossy().to_string());

    loop {
        // Check cancellation between chunks
        if cancel_flag.load(Ordering::Relaxed) {
            // Flush what's written so the partial file is valid
            let _ = writer.flush();
            return Err("Export cancelled".to_string());
        }
        let Some(read_size) = checked_export_copy_read_len(file_size, bytes_copied, source)? else {
            break;
        };
        let bytes_read = reader
            .read(&mut buffer[..read_size])
            .map_err(|e| format!("Read error: {}", e))?;

        if bytes_read == 0 {
            validate_snapshot_byte_count("Copy", source, file_size, bytes_copied)?;
            break;
        }

        writer
            .write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Write error: {}", e))?;

        if let Some(ref mut h) = hasher {
            h.update(&buffer[..bytes_read]);
        }
        bytes_copied = checked_export_copy_advance(bytes_copied, bytes_read, source)?;

        // Emit progress every 100ms
        if last_emit.elapsed().as_millis() > 100 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let total_copied = total_bytes_so_far.saturating_add(bytes_copied);
            let speed = if elapsed > 0.0 {
                (total_copied as f64 / elapsed) as u64
            } else {
                0
            };
            let percent = if total_bytes > 0 {
                (total_copied as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let _ = window.emit(
                "copy-progress",
                CopyProgress {
                    operation_id: operation_id.to_string(),
                    current_file: filename.clone(),
                    current_index: file_index,
                    total_files,
                    current_file_bytes: bytes_copied,
                    current_file_total: file_size,
                    total_bytes_copied: total_copied,
                    total_bytes,
                    percent,
                    status: if compute_hash {
                        format!("Copying + Hashing: {}", filename)
                    } else {
                        format!("Copying: {}", filename)
                    },
                    speed_bps: speed,
                    phase: Some("copying".to_string()),
                    hash_bytes_processed: if compute_hash {
                        Some(bytes_copied)
                    } else {
                        None
                    },
                    hash_bytes_total: if compute_hash { Some(file_size) } else { None },
                },
            );

            last_emit = std::time::Instant::now();
        }
    }

    validate_snapshot_byte_count("Copy", source, file_size, bytes_copied)?;

    writer.flush().map_err(|e| format!("Flush error: {}", e))?;

    let hash = hasher.map(|h| format!("{:x}", h.finalize()));

    Ok((bytes_copied, hash))
}

/// Verify a copied file matches the original hash
fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool, String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open file for verification: {}", e))?;

    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Read error during verification: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let actual_hash = format!("{:x}", hasher.finalize());
    Ok(actual_hash == expected_hash)
}

fn verify_copied_file(path: &Path, expected_hash: &str) -> Result<(), String> {
    match verify_file_hash(path, expected_hash) {
        Ok(true) => Ok(()),
        Ok(false) => Err("Hash verification failed".to_string()),
        Err(e) => Err(format!("Hash verification error: {e}")),
    }
}

/// Calculate total size of files to copy
fn calculate_total_size(paths: &[String]) -> Result<u64, String> {
    let mut total = 0u64;
    for path in paths {
        let source = Path::new(path);
        let meta = fs::metadata(source).map_err(|e| {
            format!(
                "Failed to read source metadata for {}: {}",
                source.display(),
                e
            )
        })?;
        if meta.is_file() {
            total = total.saturating_add(meta.len());
        } else if meta.is_dir() {
            total = total.saturating_add(calculate_dir_size(source)?);
        } else {
            return Err(format!(
                "Unsupported export source {}: not a file or directory",
                source.display()
            ));
        }
    }
    Ok(total)
}

/// Calculate directory size recursively
fn calculate_dir_size(dir: &Path) -> Result<u64, String> {
    calculate_dir_size_at_depth(dir, 0)
}

fn calculate_dir_size_at_depth(dir: &Path, depth: usize) -> Result<u64, String> {
    if depth > MAX_EXPORT_TRAVERSAL_DEPTH {
        warn!(
            "Skipping directory {}: maximum export traversal depth {} exceeded",
            dir.display(),
            MAX_EXPORT_TRAVERSAL_DEPTH
        );
        return Ok(0);
    }

    let mut total = 0u64;
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read export directory {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to read export directory entry in {}: {}",
                dir.display(),
                e
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| {
            format!(
                "Failed to read export source type for {}: {}",
                path.display(),
                e
            )
        })?;
        if file_type.is_file() {
            let meta = entry.metadata().map_err(|e| {
                format!(
                    "Failed to read export file metadata for {}: {}",
                    path.display(),
                    e
                )
            })?;
            total = total.saturating_add(meta.len());
        } else if file_type.is_dir() {
            total = total.saturating_add(calculate_dir_size_at_depth(&path, depth + 1)?);
        }
    }
    Ok(total)
}

fn required_export_space(total_bytes: u64) -> u64 {
    let headroom = (total_bytes / 10).max(1024 * 1024);
    total_bytes.saturating_add(headroom)
}

/// Collect all files from paths (expanding directories)
fn collect_files(paths: &[String]) -> Result<Vec<(String, String)>, String> {
    let mut files = Vec::new();

    for path in paths {
        let path_obj = Path::new(path);
        let meta = fs::metadata(path_obj).map_err(|e| {
            format!(
                "Failed to read source metadata for {}: {}",
                path_obj.display(),
                e
            )
        })?;
        if meta.is_file() {
            let filename = path_obj
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            files.push((path.clone(), filename));
        } else if meta.is_dir() {
            // Use the parent directory as base so the selected folder name is preserved
            // in the relative paths. e.g. selecting /path/to/Evidence produces
            // Evidence/file1.txt instead of just file1.txt
            let base = path_obj.parent().unwrap_or(path_obj);
            collect_dir_files(base, path_obj, &mut files)?;
        } else {
            return Err(format!(
                "Unsupported export source {}: not a file or directory",
                path_obj.display()
            ));
        }
    }

    Ok(files)
}

/// Recursively collect files from a directory
fn collect_dir_files(
    base: &Path,
    dir: &Path,
    files: &mut Vec<(String, String)>,
) -> Result<(), String> {
    collect_dir_files_at_depth(base, dir, files, 0)
}

fn collect_dir_files_at_depth(
    base: &Path,
    dir: &Path,
    files: &mut Vec<(String, String)>,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_EXPORT_TRAVERSAL_DEPTH {
        warn!(
            "Skipping directory {}: maximum export traversal depth {} exceeded",
            dir.display(),
            MAX_EXPORT_TRAVERSAL_DEPTH
        );
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read export directory {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to read export directory entry in {}: {}",
                dir.display(),
                e
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| {
            format!(
                "Failed to read export source type for {}: {}",
                path.display(),
                e
            )
        })?;
        if file_type.is_file() {
            let rel_path = path
                .strip_prefix(base)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            files.push((path.to_string_lossy().to_string(), rel_path));
        } else if file_type.is_dir() {
            collect_dir_files_at_depth(base, &path, files, depth + 1)?;
        }
    }
    Ok(())
}

/// Export/copy files to a destination directory with optional forensic features
///
/// Unified command that supports:
/// - Simple copy (when compute_hashes = false)
/// - Forensic export with hashing and manifests (when compute_hashes = true)
/// - Hash verification against known values
/// - JSON and TXT report generation
///
/// # Arguments
/// * `source_paths` - Array of file/directory paths to export
/// * `destination` - Destination directory
/// * `options` - Export options controlling hashing, verification, and reports
/// * `window` - Tauri window for progress events
///
/// # Returns
/// * `Ok(CopyResult)` - Export completed (may have partial failures)
/// * `Err(message)` - Fatal error
#[tauri::command]
pub async fn export_files(
    source_paths: Vec<String>,
    destination: String,
    options: Option<CopyOptions>,
    window: Window,
) -> Result<CopyResult, String> {
    info!(
        "Starting export operation: {} sources to {} (forensic: {})",
        source_paths.len(),
        destination,
        options.as_ref().map(|o| o.compute_hashes).unwrap_or(false)
    );

    let opts = options.unwrap_or_default();
    let operation_id = format!("export-{}", chrono::Utc::now().timestamp_millis());
    let start_time = std::time::Instant::now();
    let export_time = chrono::Utc::now().timestamp() as u64;

    // Register cancellation flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = export_cancel_flags();
        flags.insert(operation_id.clone(), cancel_flag.clone());
    }

    // Run the export, cleaning up the cancel flag regardless of outcome
    let dest_path = Path::new(&destination);
    let result = run_export_inner(
        &source_paths,
        dest_path,
        &opts,
        &operation_id,
        start_time,
        export_time,
        &cancel_flag,
        &window,
    );

    // Always clean up the cancel flag
    {
        let mut flags = export_cancel_flags();
        flags.remove(&operation_id);
    }

    result
}

/// Inner export logic separated to allow cancel-flag cleanup via RAII-like pattern
#[allow(clippy::too_many_arguments)]
fn run_export_inner(
    source_paths: &[String],
    dest_path: &Path,
    opts: &CopyOptions,
    operation_id: &str,
    start_time: std::time::Instant,
    export_time: u64,
    cancel_flag: &Arc<AtomicBool>,
    window: &Window,
) -> Result<CopyResult, String> {
    // Create destination directory if needed
    if opts.create_dirs && !dest_path.exists() {
        fs::create_dir_all(dest_path)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }

    // Calculate total size and collect files
    let _ = window.emit(
        "copy-progress",
        CopyProgress {
            operation_id: operation_id.to_string(),
            current_file: String::new(),
            current_index: 0,
            total_files: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            total_bytes_copied: 0,
            total_bytes: 0,
            percent: 0.0,
            status: "Calculating size...".to_string(),
            speed_bps: 0,
            phase: Some("calculating".to_string()),
            hash_bytes_processed: None,
            hash_bytes_total: None,
        },
    );

    let total_bytes = calculate_total_size(source_paths)?;
    let files = collect_files(source_paths)?;
    let total_files = files.len();

    // Check destination free space
    if let Some(free_bytes) = get_available_space(dest_path) {
        // Require at least 10% headroom beyond the files to account for metadata/manifests
        let required = required_export_space(total_bytes);
        if free_bytes < required {
            let free_mb = free_bytes / (1024 * 1024);
            let required_mb = required / (1024 * 1024);
            return Err(format!(
                "Insufficient disk space: {} MB available, {} MB required",
                free_mb, required_mb
            ));
        }
    }

    debug!("Copying {} files, {} bytes total", total_files, total_bytes);

    let mut files_copied = 0usize;
    let mut files_failed = 0usize;
    let mut bytes_copied = 0u64;
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut metadata_list: Vec<ExportMetadata> = Vec::new();
    let mut files_verified_known = 0usize;
    let mut files_mismatch_known = 0usize;

    // Copy/export each file
    let compute_hash = opts.compute_hashes || opts.verify_after_copy;
    for (index, (source, rel_path)) in files.iter().enumerate() {
        // Check cancellation before each file
        if cancel_flag.load(Ordering::Relaxed) {
            info!("Export cancelled after {} files", files_copied);
            failures.push(("*".to_string(), "Export cancelled by user".to_string()));
            break;
        }

        let source_path = Path::new(source);
        let dest_file = dest_path.join(rel_path);

        // Create parent directories
        if let Some(parent) = dest_file.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    warn!("Failed to create directory {}: {}", parent.display(), e);
                    failures.push((source.clone(), format!("Failed to create directory: {}", e)));
                    files_failed += 1;
                    continue;
                }
            }
        }

        // Check if destination exists
        if dest_file.exists() && !opts.overwrite {
            warn!("Skipping existing file: {}", dest_file.display());
            failures.push((
                source.clone(),
                "File exists (overwrite disabled)".to_string(),
            ));
            files_failed += 1;
            continue;
        }

        // Copy the file
        match copy_file_with_progress(
            source_path,
            &dest_file,
            window,
            operation_id,
            index + 1,
            total_files,
            bytes_copied,
            total_bytes,
            start_time,
            cancel_flag,
            compute_hash,
        ) {
            Ok((copied, hash)) => {
                bytes_copied = bytes_copied.saturating_add(copied);

                // Handle forensic metadata if hashing is enabled
                if opts.compute_hashes {
                    if let (Some(sha256), Ok(source_meta)) = (&hash, fs::metadata(source_path)) {
                        let verified = if opts.verify_after_copy {
                            if let Err(e) = verify_copied_file(&dest_file, sha256) {
                                warn!("Verification failed for {}: {}", rel_path, e);
                                failures.push((source.clone(), e));
                                files_failed += 1;
                                continue;
                            }
                            debug!("Hash verified for {}", rel_path);
                            true
                        } else {
                            false
                        };

                        if !verified {
                            debug!(
                                "Hash computed for {} without post-copy verification",
                                rel_path
                            );
                        }

                        let modified_time = source_meta
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);

                        // Check against known hashes if requested
                        let (known_hash, matches_known, known_hash_source) = if opts
                            .verify_against_known
                        {
                            let db = database::get_db();
                            match db.lookup_known_hash_by_path(source) {
                                Ok(Some((stored_hash, hash_source))) => {
                                    let matches = stored_hash.eq_ignore_ascii_case(sha256);
                                    if matches {
                                        files_verified_known += 1;
                                        debug!("Known hash match for {}", rel_path);
                                    } else {
                                        files_mismatch_known += 1;
                                        warn!(
                                            "Known hash MISMATCH for {}: expected={}, got={}",
                                            rel_path, stored_hash, sha256
                                        );
                                    }
                                    (Some(stored_hash), Some(matches), Some(hash_source))
                                }
                                Ok(None) => {
                                    debug!("No known hash in database for {}", rel_path);
                                    (None, None, None)
                                }
                                Err(e) => {
                                    warn!("Failed to look up known hash for {}: {}", rel_path, e);
                                    (None, None, None)
                                }
                            }
                        } else {
                            (None, None, None)
                        };

                        metadata_list.push(ExportMetadata {
                            source_path: source.clone(),
                            destination_path: dest_file.to_string_lossy().to_string(),
                            size: copied,
                            sha256: Some(sha256.clone()),
                            modified_time,
                            export_time,
                            copy_verified: verified,
                            known_hash,
                            matches_known,
                            known_hash_source,
                        });
                    }
                } else {
                    // Simple copy mode - verify hash if requested
                    if opts.verify_after_copy {
                        if let Some(ref expected) = hash {
                            if let Err(e) = verify_copied_file(&dest_file, expected) {
                                warn!("Verification failed for {}: {}", rel_path, e);
                                failures.push((source.clone(), e));
                                files_failed += 1;
                                continue;
                            }
                            debug!("Hash verified for {}", rel_path);
                        }
                    }
                }

                // Preserve timestamps
                if opts.preserve_timestamps {
                    if let Ok(meta) = fs::metadata(source_path) {
                        if let Ok(mtime) = meta.modified() {
                            let _ = filetime::set_file_mtime(
                                &dest_file,
                                filetime::FileTime::from_system_time(mtime),
                            );
                        }
                    }
                }

                files_copied += 1;
            }
            Err(e) => {
                warn!("Failed to copy {}: {}", source, e);
                failures.push((source.clone(), e));
                files_failed += 1;
            }
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let avg_speed = bytes_copied
        .saturating_mul(1000)
        .checked_div(duration_ms)
        .unwrap_or(0);

    // Generate manifest and reports if requested
    let mut json_manifest_path = None;
    let mut txt_report_path = None;

    if opts.compute_hashes && !metadata_list.is_empty() {
        let export_name = opts.export_name.as_deref().unwrap_or("export");

        // Generate JSON manifest
        if opts.generate_json_manifest {
            let manifest_path =
                dest_path.join(format!("{}_{}_manifest.json", export_name, operation_id));
            let manifest = serde_json::json!({
                "operation_id": operation_id,
                "export_name": export_name,
                "export_time": export_time,
                "export_time_iso": chrono::Utc::now().to_rfc3339(),
                "total_files": files_copied,
                "total_bytes": bytes_copied,
                "duration_ms": duration_ms,
                "files_verified_known": files_verified_known,
                "files_mismatch_known": files_mismatch_known,
                "files": metadata_list,
                "failures": failures,
            });

            let manifest_json = match serde_json::to_string_pretty(&manifest) {
                Ok(json) => json,
                Err(e) => {
                    warn!("Failed to serialize JSON manifest: {}", e);
                    String::new()
                }
            };

            if manifest_json.is_empty() {
                warn!("Skipping empty JSON manifest for {}", export_name);
            } else if let Err(e) = fs::write(&manifest_path, manifest_json) {
                warn!("Failed to write JSON manifest: {}", e);
            } else {
                info!("JSON manifest written to {}", manifest_path.display());
                json_manifest_path = Some(manifest_path.to_string_lossy().to_string());
            }
        }

        // Generate TXT report
        if opts.generate_txt_report {
            let report_path =
                dest_path.join(format!("{}_{}_report.txt", export_name, operation_id));
            let mut report = String::new();
            report.push_str(&format!("Export Report: {}\n", export_name));
            report.push_str(&format!("Operation ID: {}\n", operation_id));
            report.push_str(&format!(
                "Export Time: {}\n",
                chrono::Utc::now().to_rfc3339()
            ));
            report.push_str(&format!("Total Files: {}\n", files_copied));
            report.push_str(&format!("Total Bytes: {}\n", bytes_copied));
            report.push_str(&format!("Duration: {}ms\n", duration_ms));
            report.push_str(&format!(
                "Files Verified (Known): {}\n",
                files_verified_known
            ));
            report.push_str(&format!(
                "Files Mismatched (Known): {}\n",
                files_mismatch_known
            ));
            report.push_str("\n--- Files ---\n\n");

            for meta in &metadata_list {
                report.push_str(&format!("Source: {}\n", meta.source_path));
                report.push_str(&format!("Destination: {}\n", meta.destination_path));
                report.push_str(&format!("Size: {} bytes\n", meta.size));
                if let Some(ref hash) = meta.sha256 {
                    report.push_str(&format!("SHA-256: {}\n", hash));
                }
                report.push_str(&format!("Copy Verified: {}\n", meta.copy_verified));
                if let Some(ref known) = meta.known_hash {
                    report.push_str(&format!("Known Hash: {}\n", known));
                }
                if let Some(matches) = meta.matches_known {
                    report.push_str(&format!("Matches Known: {}\n", matches));
                }
                report.push('\n');
            }

            if !failures.is_empty() {
                report.push_str("\n--- Failures ---\n\n");
                for (path, error) in &failures {
                    report.push_str(&format!("{}: {}\n", path, error));
                }
            }

            if let Err(e) = fs::write(&report_path, report) {
                warn!("Failed to write TXT report: {}", e);
            } else {
                info!("TXT report written to {}", report_path.display());
                txt_report_path = Some(report_path.to_string_lossy().to_string());
            }
        }
    }

    // Emit completion
    let _ = window.emit(
        "copy-progress",
        CopyProgress {
            operation_id: operation_id.to_string(),
            current_file: String::new(),
            current_index: total_files,
            total_files,
            current_file_bytes: 0,
            current_file_total: 0,
            total_bytes_copied: bytes_copied,
            total_bytes,
            percent: 100.0,
            status: "Complete".to_string(),
            speed_bps: avg_speed,
            phase: Some("complete".to_string()),
            hash_bytes_processed: None,
            hash_bytes_total: None,
        },
    );

    info!(
        "Export complete: {} files, {} bytes in {}ms (forensic: {})",
        files_copied, bytes_copied, duration_ms, opts.compute_hashes
    );

    Ok(CopyResult {
        operation_id: operation_id.to_string(),
        files_copied,
        files_failed,
        bytes_copied,
        duration_ms,
        avg_speed_bps: avg_speed,
        failures,
        metadata: if opts.compute_hashes {
            Some(metadata_list)
        } else {
            None
        },
        json_manifest_path,
        txt_report_path,
        files_verified_known,
        files_mismatch_known,
    })
}

/// Cancel an in-progress export operation
#[tauri::command]
pub async fn cancel_export(operation_id: String) -> Result<bool, String> {
    let flags = export_cancel_flags();
    if let Some(flag) = flags.get(&operation_id) {
        flag.store(true, Ordering::Relaxed);
        info!("Cancel requested for export operation: {}", operation_id);
        Ok(true)
    } else {
        warn!(
            "No active export with operation_id: {} (may have already completed)",
            operation_id
        );
        Ok(false)
    }
}

/// Get available disk space at the given path (in bytes)
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
fn get_available_space(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
            Some(stat.f_bavail as u64 * stat.f_frsize as u64)
        } else {
            None
        }
    }
}

#[cfg(not(unix))]
fn get_available_space(_path: &Path) -> Option<u64> {
    // On non-Unix systems, skip the space check (it will just proceed)
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn required_export_space_saturates_for_huge_totals() {
        assert_eq!(required_export_space(u64::MAX), u64::MAX);
    }

    #[test]
    fn required_export_space_adds_minimum_headroom() {
        assert_eq!(required_export_space(512), 512 + 1024 * 1024);
    }

    #[test]
    fn calculate_total_size_sums_regular_files() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        std::fs::write(&a, [0u8; 3]).unwrap();
        std::fs::write(&b, [0u8; 5]).unwrap();

        let paths = vec![
            a.to_string_lossy().to_string(),
            b.to_string_lossy().to_string(),
        ];

        assert_eq!(calculate_total_size(&paths).unwrap(), 8);
    }

    #[test]
    fn calculate_total_size_rejects_missing_source() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("missing.bin");
        let paths = vec![missing.to_string_lossy().to_string()];

        let err = calculate_total_size(&paths).unwrap_err();
        assert!(err.contains("Failed to read source metadata"));
    }

    #[test]
    fn collect_files_rejects_missing_source() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("missing.bin");
        let paths = vec![missing.to_string_lossy().to_string()];

        let err = collect_files(&paths).unwrap_err();
        assert!(err.contains("Failed to read source metadata"));
    }

    #[test]
    fn collect_files_expands_directory_with_selected_folder_name() {
        let dir = TempDir::new().unwrap();
        let evidence_dir = dir.path().join("Evidence");
        std::fs::create_dir(&evidence_dir).unwrap();
        let file = evidence_dir.join("a.bin");
        std::fs::write(&file, [0u8; 3]).unwrap();
        let paths = vec![evidence_dir.to_string_lossy().to_string()];

        let files = collect_files(&paths).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, "Evidence/a.bin");
    }

    #[test]
    fn checked_export_copy_read_len_returns_none_when_file_complete() {
        let source = std::path::Path::new("complete.bin");

        assert_eq!(
            checked_export_copy_read_len(1024, 1024, source).unwrap(),
            None
        );
    }

    #[test]
    fn checked_export_copy_read_len_clamps_to_remaining_tail() {
        let source = std::path::Path::new("tail.bin");

        assert_eq!(
            checked_export_copy_read_len(1024, 1000, source).unwrap(),
            Some(24)
        );
    }

    #[test]
    fn checked_export_copy_read_len_caps_large_remaining_to_buffer_size() {
        let source = std::path::Path::new("large.bin");

        assert_eq!(
            checked_export_copy_read_len(u64::MAX, 0, source).unwrap(),
            Some(COPY_BUFFER_SIZE)
        );
    }

    #[test]
    fn checked_export_copy_read_len_rejects_counter_past_source_size() {
        let source = std::path::Path::new("drifted.bin");
        let err = checked_export_copy_read_len(10, 11, source).unwrap_err();

        assert!(err.contains("exceeded source size"));
    }

    #[test]
    fn checked_export_copy_advance_rejects_overflow() {
        let source = std::path::Path::new("overflow.bin");
        let err = checked_export_copy_advance(u64::MAX, 1, source).unwrap_err();

        assert!(err.contains("overflowed"));
    }

    #[test]
    fn checked_export_copy_advance_adds_bytes_read() {
        let source = std::path::Path::new("advance.bin");

        assert_eq!(checked_export_copy_advance(40, 2, source).unwrap(), 42);
    }

    #[test]
    fn verify_copied_file_accepts_matching_hash() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("copied.bin");
        std::fs::write(&path, b"evidence").unwrap();
        let expected = format!("{:x}", Sha256::digest(b"evidence"));

        assert!(verify_copied_file(&path, &expected).is_ok());
    }

    #[test]
    fn verify_copied_file_rejects_hash_mismatch() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("copied.bin");
        std::fs::write(&path, b"evidence").unwrap();
        let wrong = format!("{:x}", Sha256::digest(b"changed"));

        let err = verify_copied_file(&path, &wrong).unwrap_err();
        assert_eq!(err, "Hash verification failed");
    }

    #[test]
    fn verify_copied_file_preserves_read_error() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("missing.bin");
        let expected = format!("{:x}", Sha256::digest(b"evidence"));

        let err = verify_copied_file(&missing, &expected).unwrap_err();
        assert!(err.starts_with("Hash verification error:"));
        assert!(err.contains("Failed to open file for verification"));
    }
}
