// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! L01 logical evidence export commands
//!
//! Provides Tauri commands for creating L01 logical evidence containers using
//! the pure-Rust `l01_writer` module. Supports recursive directory acquisition,
//! progress events, cancellation, and case metadata.

use crate::l01_writer::{
    CompressionLevel, L01CaseInfo, L01HashAlgorithm, L01WriteError, L01WritePhase,
    L01WriteProgress, L01WriteResult, L01Writer, L01WriterConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::{debug, info};

use super::ewf_helpers::{is_system_boot_volume, nix_stat};

const MAX_L01_TRAVERSAL_DEPTH: usize = 128;
const MAX_L01_SOURCE_PATHS: usize = 10_000;
const MAX_L01_LOGICAL_ENTRIES: usize = 250_000;
const MAX_L01_FILTER_EXTENSIONS: usize = 10_000;

fn checked_l01_total_size_add(
    total: u64,
    addition: u64,
    path: &Path,
) -> Result<u64, std::io::Error> {
    total.checked_add(addition).ok_or_else(|| {
        std::io::Error::other(format!(
            "L01 total size overflow while adding {} bytes from {} to current total {} bytes",
            addition,
            path.display(),
            total
        ))
    })
}

fn checked_l01_entry_count_add(count: &mut usize, path: &Path) -> Result<(), String> {
    if *count >= MAX_L01_LOGICAL_ENTRIES {
        return Err(format!(
            "L01 export expanded to more than {} logical entries while adding {}",
            MAX_L01_LOGICAL_ENTRIES,
            path.display()
        ));
    }
    *count += 1;
    Ok(())
}

fn l01_logical_entry_name(path: &Path) -> Result<String, String> {
    let Some(name) = path.file_name().filter(|name| !name.is_empty()) else {
        return Err(format!(
            "L01 export source path has no file or directory name: {}",
            path.display()
        ));
    };
    Ok(name.to_string_lossy().to_string())
}

// =============================================================================
// Types
// =============================================================================

/// Options for creating an L01 logical evidence container
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L01ExportOptions {
    /// Source directory or file paths to include
    pub source_paths: Vec<String>,
    /// Output path for the L01 file
    pub output_path: String,
    /// Compression level: "none", "fast" (default), "best"
    pub compression: Option<String>,
    /// Hash algorithm for image integrity: "md5" (default), "sha1"
    pub hash_algorithm: Option<String>,
    /// Maximum segment file size in bytes (0 = no splitting)
    pub segment_size: Option<u64>,
    /// Case number
    pub case_number: Option<String>,
    /// Evidence number
    pub evidence_number: Option<String>,
    /// Examiner name
    pub examiner_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Notes
    pub notes: Option<String>,
    /// Include only files matching these extensions (e.g. ["pdf", "docx"]). Empty = all.
    pub filter_extensions: Option<Vec<String>>,
    /// Exclude files matching these extensions (e.g. ["tmp", "log"]). Empty = none.
    pub exclude_extensions: Option<Vec<String>>,
    /// Minimum file size in bytes (files smaller are skipped)
    pub min_file_size: Option<u64>,
    /// Maximum file size in bytes (files larger are skipped)
    pub max_file_size: Option<u64>,
}

/// Serializable result returned to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct L01ExportResponse {
    /// Output file path(s) created
    pub output_paths: Vec<String>,
    /// Total files written into the L01
    pub total_files: usize,
    /// Total directories written
    pub total_directories: usize,
    /// Total bytes of file data written
    pub total_data_bytes: u64,
    /// Total compressed bytes
    pub total_compressed_bytes: u64,
    /// Compression ratio (compressed / original)
    pub compression_ratio: f64,
    /// Image MD5 hash (if computed)
    pub md5_hash: Option<String>,
    /// Image SHA-1 hash (if computed)
    pub sha1_hash: Option<String>,
    /// Number of segment files
    pub segment_count: u32,
    /// Number of data chunks
    pub chunk_count: u32,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl From<L01WriteResult> for L01ExportResponse {
    fn from(r: L01WriteResult) -> Self {
        Self {
            output_paths: r.output_paths,
            total_files: r.total_files,
            total_directories: r.total_directories,
            total_data_bytes: r.total_data_bytes,
            total_compressed_bytes: r.total_compressed_bytes,
            compression_ratio: r.compression_ratio,
            md5_hash: r.md5_hash,
            sha1_hash: r.sha1_hash,
            segment_count: r.segment_count,
            chunk_count: r.chunk_count,
            duration_ms: 0, // filled in by the command
        }
    }
}

/// Global cancel flags for active L01 export jobs
static L01_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
struct L01CancelRegistration {
    output_path: String,
}

impl Drop for L01CancelRegistration {
    fn drop(&mut self) {
        cleanup_l01_cancel_flag(&self.output_path);
    }
}

// =============================================================================
// Helper functions
// =============================================================================

fn validate_l01_source_paths(source_paths: &[String]) -> Result<(), String> {
    if source_paths.is_empty() {
        return Err("L01 export requires at least one source path".to_string());
    }
    if source_paths.len() > MAX_L01_SOURCE_PATHS {
        return Err(format!(
            "L01 export requested {} source paths, exceeding limit {}",
            source_paths.len(),
            MAX_L01_SOURCE_PATHS
        ));
    }
    if source_paths
        .iter()
        .any(|source_path| source_path.trim().is_empty())
    {
        return Err("L01 export source paths cannot be empty".to_string());
    }
    Ok(())
}

fn validate_l01_export_options(options: &L01ExportOptions) -> Result<(), String> {
    if options.output_path.trim().is_empty() {
        return Err("L01 export output path is required".to_string());
    }
    validate_l01_source_paths(&options.source_paths)?;
    validate_l01_filter_extensions(options.filter_extensions.as_deref(), "include")?;
    validate_l01_filter_extensions(options.exclude_extensions.as_deref(), "exclude")?;
    if let (Some(min_size), Some(max_size)) = (options.min_file_size, options.max_file_size) {
        if min_size > max_size {
            return Err(format!(
                "L01 export minFileSize ({min_size}) cannot exceed maxFileSize ({max_size})"
            ));
        }
    }
    Ok(())
}

fn validate_l01_filter_extensions(
    extensions: Option<&[String]>,
    label: &str,
) -> Result<(), String> {
    let Some(extensions) = extensions else {
        return Ok(());
    };
    if extensions.len() > MAX_L01_FILTER_EXTENSIONS {
        return Err(format!(
            "L01 export {label} extension filter has {} entries, exceeding limit {}",
            extensions.len(),
            MAX_L01_FILTER_EXTENSIONS
        ));
    }
    if extensions
        .iter()
        .any(|extension| extension.trim().is_empty())
    {
        return Err(format!(
            "L01 export {label} extension filter cannot contain empty entries"
        ));
    }
    Ok(())
}

fn register_l01_cancel_flag(
    output_path: &str,
    cancel_flag: Arc<AtomicBool>,
) -> Result<L01CancelRegistration, String> {
    let mut flags = L01_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if flags.contains_key(output_path) {
        return Err(format!(
            "An L01 export is already running for output path: {}",
            output_path
        ));
    }
    flags.insert(output_path.to_string(), cancel_flag);
    Ok(L01CancelRegistration {
        output_path: output_path.to_string(),
    })
}

fn cleanup_l01_cancel_flag(output_path: &str) {
    if let Ok(mut flags) = L01_CANCEL_FLAGS.lock() {
        flags.remove(output_path);
    }
}

fn parse_l01_compression(compression: &str) -> Result<CompressionLevel, String> {
    match compression.to_lowercase().as_str() {
        "none" | "store" => Ok(CompressionLevel::None),
        "fast" | "default" => Ok(CompressionLevel::Fast),
        "best" | "maximum" => Ok(CompressionLevel::Best),
        _ => Err(format!("Unknown L01 compression level: {}", compression)),
    }
}

fn parse_l01_hash_algorithm(algo: &str) -> Result<L01HashAlgorithm, String> {
    match algo.to_lowercase().as_str() {
        "md5" => Ok(L01HashAlgorithm::Md5),
        "sha1" | "sha-1" => Ok(L01HashAlgorithm::Sha1),
        _ => Err(format!(
            "Unknown hash algorithm: {}. Supported: md5, sha1",
            algo
        )),
    }
}

fn format_write_error(err: L01WriteError) -> String {
    match err {
        L01WriteError::Io(e) => format!("I/O error: {}", e),
        L01WriteError::Cancelled => "L01 export was cancelled".to_string(),
        other => format!("L01 write error: {}", other),
    }
}

/// Filter configuration for logical acquisition.
#[derive(Debug, Clone, Default)]
struct FileFilter {
    include_exts: Vec<String>,
    exclude_exts: Vec<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,
}

impl FileFilter {
    fn from_options(options: &L01ExportOptions) -> Self {
        Self {
            include_exts: options
                .filter_extensions
                .as_ref()
                .map(|v| v.iter().map(|s| s.to_lowercase()).collect())
                .unwrap_or_default(),
            exclude_exts: options
                .exclude_extensions
                .as_ref()
                .map(|v| v.iter().map(|s| s.to_lowercase()).collect())
                .unwrap_or_default(),
            min_size: options.min_file_size,
            max_size: options.max_file_size,
        }
    }

    fn is_empty(&self) -> bool {
        self.include_exts.is_empty()
            && self.exclude_exts.is_empty()
            && self.min_size.is_none()
            && self.max_size.is_none()
    }

    fn matches(&self, name: &str, size: u64) -> bool {
        // Extension filter
        let ext = std::path::Path::new(name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if !self.include_exts.is_empty() && !self.include_exts.contains(&ext) {
            return false;
        }
        if !self.exclude_exts.is_empty() && self.exclude_exts.contains(&ext) {
            return false;
        }

        // Size filter
        if let Some(min) = self.min_size {
            if size < min {
                return false;
            }
        }
        if let Some(max) = self.max_size {
            if size > max {
                return false;
            }
        }

        true
    }
}

/// Recursively compute total file size in a directory.
/// Skips unreadable directories (e.g. macOS TCC-protected folders) with a warning.
fn walk_dir_size(dir: &std::path::Path, filter: &FileFilter) -> Result<u64, std::io::Error> {
    walk_dir_size_at_depth(dir, filter, 0)
}

fn walk_dir_size_at_depth(
    dir: &std::path::Path,
    filter: &FileFilter,
    depth: usize,
) -> Result<u64, std::io::Error> {
    if depth > MAX_L01_TRAVERSAL_DEPTH {
        tracing::warn!(
            "Skipping directory {}: maximum L01 traversal depth {} exceeded",
            dir.display(),
            MAX_L01_TRAVERSAL_DEPTH
        );
        return Ok(0);
    }

    let mut total: u64 = 0;
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Skipping unreadable directory {}: {}", dir.display(), e);
            return Ok(0);
        }
    };
    for entry in entries.flatten() {
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_file() {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if !filter.is_empty() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !filter.matches(&name, size) {
                    continue;
                }
            }
            total = checked_l01_total_size_add(total, size, &entry.path())?;
        } else if ft.is_dir() {
            let dir_size = walk_dir_size_at_depth(&entry.path(), filter, depth + 1)?;
            total = checked_l01_total_size_add(total, dir_size, &entry.path())?;
        }
    }
    Ok(total)
}

/// Recursively walk a directory and add entries to the L01 writer under a parent.
/// This is used instead of `add_source_directory` to allow placing the contents
/// under a specific parent directory entry (preserving the selected folder name).
fn walk_dir_into_writer(
    writer: &mut L01Writer,
    dir_path: &std::path::Path,
    parent_id: u64,
    filter: &FileFilter,
    entry_count: &mut usize,
) -> Result<usize, String> {
    walk_dir_into_writer_at_depth(writer, dir_path, parent_id, filter, entry_count, 0)
}

fn walk_dir_into_writer_at_depth(
    writer: &mut L01Writer,
    dir_path: &std::path::Path,
    parent_id: u64,
    filter: &FileFilter,
    entry_count: &mut usize,
    depth: usize,
) -> Result<usize, String> {
    if depth > MAX_L01_TRAVERSAL_DEPTH {
        tracing::warn!(
            "Skipping directory {}: maximum L01 traversal depth {} exceeded",
            dir_path.display(),
            MAX_L01_TRAVERSAL_DEPTH
        );
        return Ok(0);
    }

    let mut count = 0;

    let mut entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(dir_path) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            tracing::warn!(
                "Skipping unreadable directory {}: {}",
                dir_path.display(),
                e
            );
            return Ok(0);
        }
    };

    // Sort for deterministic output
    entries.sort_by_key(|e| e.file_name());

    for dir_entry in entries {
        let path = dir_entry.path();
        let file_name = dir_entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/directories
        if file_name.starts_with('.') {
            continue;
        }

        let metadata = dir_entry
            .metadata()
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;

        if metadata.is_dir() {
            checked_l01_entry_count_add(entry_count, &path)?;
            let dir_id = writer.add_directory(file_name, parent_id);
            count += 1;
            count += walk_dir_into_writer_at_depth(
                writer,
                &path,
                dir_id,
                filter,
                entry_count,
                depth + 1,
            )?;
        } else if metadata.is_file() {
            let size = metadata.len();
            if !filter.is_empty() && !filter.matches(&file_name, size) {
                continue;
            }
            checked_l01_entry_count_add(entry_count, &path)?;
            writer.add_file(file_name, size, path.clone(), parent_id);
            count += 1;
        }
    }

    Ok(count)
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Create an L01 logical evidence container from source files/directories.
///
/// Emits `l01-export-progress` events during the operation.
/// Can be cancelled via `l01_cancel_export`.
#[tauri::command]
pub async fn l01_create_image(
    options: L01ExportOptions,
    window: Window,
) -> Result<L01ExportResponse, String> {
    let start = std::time::Instant::now();
    validate_l01_export_options(&options)?;

    // Parse options
    let compression = match &options.compression {
        Some(c) => parse_l01_compression(c)?,
        None => CompressionLevel::Fast,
    };

    let hash_algorithm = match &options.hash_algorithm {
        Some(a) => parse_l01_hash_algorithm(a)?,
        None => L01HashAlgorithm::Md5,
    };

    let segment_size = options.segment_size.unwrap_or(0);

    info!(
        "Creating L01 image at: {} (compression={:?}, hash={:?}, sources={})",
        options.output_path,
        compression,
        hash_algorithm,
        options.source_paths.len()
    );

    // Validate source paths exist
    for path_str in &options.source_paths {
        let path = std::path::Path::new(path_str);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", path_str));
        }
    }

    // --- Safety validations ---

    // Refuse to image the running system boot volume
    for path_str in &options.source_paths {
        let canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| std::path::PathBuf::from(path_str));
        if is_system_boot_volume(&canon) {
            return Err(format!(
                "Refusing to image the system boot volume ({}). Imaging the running OS disk can \
                 produce inconsistent data. Use an external boot environment for system drive acquisition.",
                path_str
            ));
        }
    }

    // Verify output destination does not overlap with any source path
    let output_dir = std::path::Path::new(&options.output_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new(&options.output_path));
    let output_canon =
        std::fs::canonicalize(output_dir).unwrap_or_else(|_| output_dir.to_path_buf());
    for path_str in &options.source_paths {
        let source_canon =
            std::fs::canonicalize(path_str).unwrap_or_else(|_| std::path::PathBuf::from(path_str));
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
    let _cancel_registration = register_l01_cancel_flag(&options.output_path, cancel_flag.clone())?;

    // Build config
    let config = L01WriterConfig {
        output_path: PathBuf::from(&options.output_path),
        case_info: L01CaseInfo {
            case_number: options.case_number.clone().unwrap_or_default(),
            evidence_number: options.evidence_number.clone().unwrap_or_default(),
            description: options.description.clone().unwrap_or_default(),
            examiner: options.examiner_name.clone().unwrap_or_default(),
            notes: options.notes.clone().unwrap_or_default(),
        },
        compression_level: compression,
        segment_size,
        hash_algorithm,
        ..Default::default()
    };

    // Create writer and add sources
    let mut writer = L01Writer::new(config);
    let filter = FileFilter::from_options(&options);
    let mut planned_entry_count = 0usize;

    for path_str in &options.source_paths {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            // Add the directory entry itself, then walk its contents under it
            let dir_name = l01_logical_entry_name(&path)?;
            checked_l01_entry_count_add(&mut planned_entry_count, &path)?;
            let parent_id = writer.add_directory(dir_name, 0);

            // Walk the directory contents and add under the parent directory entry
            let count = walk_dir_into_writer(
                &mut writer,
                &path,
                parent_id,
                &filter,
                &mut planned_entry_count,
            )?;
            info!(
                "Added directory {} ({} entries, parent_id={})",
                path_str,
                count + 1,
                parent_id
            );
        } else {
            // Add a single file
            let metadata = std::fs::metadata(&path)
                .map_err(|e| format!("Failed to read metadata for {}: {}", path_str, e))?;
            let file_name = l01_logical_entry_name(&path)?;
            checked_l01_entry_count_add(&mut planned_entry_count, &path)?;
            writer.add_file(file_name, metadata.len(), path.clone(), 0);
            debug!("Added file {} ({} bytes)", path_str, metadata.len());
        }
    }

    info!(
        "L01 writer ready: {} entries, {} bytes total",
        writer.entry_count(),
        writer.total_file_size()
    );

    // Emit an early "preparing" event so the frontend knows how many files/bytes
    // were found — before the writer starts its main phases.
    {
        let early_progress = L01WriteProgress {
            path: options.output_path.clone(),
            current_file: String::new(),
            files_processed: 0,
            total_files: writer.entry_count(),
            bytes_written: 0,
            total_bytes: writer.total_file_size(),
            percent: 0.0,
            phase: L01WritePhase::Preparing,
        };
        crate::eventing::log_emit_result(
            "l01-export-progress",
            window.emit("l01-export-progress", &early_progress),
        );
    }

    // Check destination has enough free space
    let total_source_bytes = writer.total_file_size();
    {
        let avail_result = nix_stat(&output_canon).map(|info| info.available_space);
        if let Ok(avail) = avail_result {
            if avail > 0 && total_source_bytes > avail {
                return Err(format!(
                    "Insufficient disk space on the destination volume. \
                     The source data is approximately {:.1} GB but only {:.1} GB is available. \
                     Free up space or choose a different destination.",
                    total_source_bytes as f64 / 1_073_741_824.0,
                    avail as f64 / 1_073_741_824.0,
                ));
            }
        }
    }

    // Set up progress callback
    let window_clone = window.clone();
    let progress_fn = Box::new(move |progress: L01WriteProgress| {
        crate::eventing::log_emit_result(
            "l01-export-progress",
            window_clone.emit("l01-export-progress", &progress),
        );
    });

    // Run the write operation in a blocking task
    let output_path_for_cleanup = options.output_path.clone();
    let result =
        tokio::task::spawn_blocking(move || writer.write(Some(&cancel_flag), Some(progress_fn)))
            .await
            .map_err(|e| format!("L01 write task panicked: {}", e))?
            .map_err(format_write_error)?;

    cleanup_l01_cancel_flag(&output_path_for_cleanup);

    let duration = start.elapsed();
    info!(
        "L01 export complete: {} files, {} dirs, {:.1} MB data, {:.1} MB compressed ({:.1}% ratio), {:.1}s",
        result.total_files,
        result.total_directories,
        result.total_data_bytes as f64 / 1_048_576.0,
        result.total_compressed_bytes as f64 / 1_048_576.0,
        result.compression_ratio * 100.0,
        duration.as_secs_f64()
    );

    let mut response = L01ExportResponse::from(result);
    response.duration_ms = duration.as_millis() as u64;
    Ok(response)
}

/// Cancel an in-progress L01 export
#[tauri::command]
pub fn l01_cancel_export(output_path: String) -> Result<bool, String> {
    let flags = L01_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = flags.get(&output_path) {
        flag.store(true, Ordering::Relaxed);
        info!("Cancelled L01 export: {}", output_path);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Estimate the output size for an L01 export.
///
/// Returns approximate total bytes based on source sizes and compression level.
/// This is a heuristic — actual size depends on data compressibility.
#[tauri::command]
pub fn l01_estimate_size(
    source_paths: Vec<String>,
    compression: Option<String>,
) -> Result<u64, String> {
    validate_l01_source_paths(&source_paths)?;
    let mut total_source_bytes: u64 = 0;

    for path_str in &source_paths {
        let path = std::path::Path::new(path_str);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", path_str));
        }
        if path.is_dir() {
            // Walk directory recursively
            let dir_size = walk_dir_size(path, &FileFilter::default())
                .map_err(|e| format!("Failed to walk directory {}: {}", path_str, e))?;
            total_source_bytes = checked_l01_total_size_add(total_source_bytes, dir_size, path)
                .map_err(|e| e.to_string())?;
        } else {
            let metadata = std::fs::metadata(path)
                .map_err(|e| format!("Failed to read metadata for {}: {}", path_str, e))?;
            total_source_bytes =
                checked_l01_total_size_add(total_source_bytes, metadata.len(), path)
                    .map_err(|e| e.to_string())?;
        }
    }

    // Estimate based on compression level
    let compression_level = match compression.as_deref() {
        Some(c) => parse_l01_compression(c)?,
        None => CompressionLevel::Fast,
    };

    let estimated = estimate_l01_output_size(total_source_bytes, compression_level);

    Ok(estimated)
}

fn estimate_l01_output_size(total_source_bytes: u64, compression_level: CompressionLevel) -> u64 {
    match compression_level {
        // No compression: data + ~5% overhead for headers/tables.
        CompressionLevel::None => {
            total_source_bytes.saturating_add(total_source_bytes.saturating_div(20))
        }
        // Fast compression: rough estimate ~75% of original.
        CompressionLevel::Fast => total_source_bytes.saturating_mul(75).saturating_div(100),
        // Best compression: rough estimate ~60% of original.
        CompressionLevel::Best => total_source_bytes.saturating_mul(60).saturating_div(100),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_options() -> L01ExportOptions {
        L01ExportOptions {
            source_paths: vec!["source.bin".to_string()],
            output_path: "/tmp/case/logical.L01".to_string(),
            compression: None,
            hash_algorithm: None,
            segment_size: None,
            case_number: None,
            evidence_number: None,
            examiner_name: None,
            description: None,
            notes: None,
            filter_extensions: None,
            exclude_extensions: None,
            min_file_size: None,
            max_file_size: None,
        }
    }

    #[test]
    fn test_parse_compression() {
        assert!(matches!(
            parse_l01_compression("none"),
            Ok(CompressionLevel::None)
        ));
        assert!(matches!(
            parse_l01_compression("fast"),
            Ok(CompressionLevel::Fast)
        ));
        assert!(matches!(
            parse_l01_compression("best"),
            Ok(CompressionLevel::Best)
        ));
        assert!(matches!(
            parse_l01_compression("FAST"),
            Ok(CompressionLevel::Fast)
        ));
        assert!(parse_l01_compression("invalid").is_err());
    }

    #[test]
    fn test_parse_hash_algorithm() {
        assert!(matches!(
            parse_l01_hash_algorithm("md5"),
            Ok(L01HashAlgorithm::Md5)
        ));
        assert!(matches!(
            parse_l01_hash_algorithm("sha1"),
            Ok(L01HashAlgorithm::Sha1)
        ));
        assert!(matches!(
            parse_l01_hash_algorithm("SHA-1"),
            Ok(L01HashAlgorithm::Sha1)
        ));
        assert!(parse_l01_hash_algorithm("sha256").is_err());
    }

    #[test]
    fn test_format_write_error() {
        let err = L01WriteError::NoFiles;
        let msg = format_write_error(err);
        assert!(msg.contains("No files"));

        let err = L01WriteError::Cancelled;
        let msg = format_write_error(err);
        assert!(msg.contains("cancelled"));
    }

    #[test]
    fn test_export_response_from_result() {
        let result = L01WriteResult {
            output_paths: vec!["/tmp/test.L01".to_string()],
            total_files: 10,
            total_directories: 3,
            total_data_bytes: 1024,
            total_compressed_bytes: 512,
            compression_ratio: 0.5,
            md5_hash: Some("abc123".to_string()),
            sha1_hash: None,
            segment_count: 1,
            chunk_count: 5,
        };
        let response = L01ExportResponse::from(result);
        assert_eq!(response.total_files, 10);
        assert_eq!(response.total_directories, 3);
        assert_eq!(response.compression_ratio, 0.5);
        assert_eq!(response.md5_hash, Some("abc123".to_string()));
        assert_eq!(response.duration_ms, 0); // not yet set
    }

    #[test]
    fn test_estimate_l01_output_size_none_adds_overhead() {
        assert_eq!(estimate_l01_output_size(100, CompressionLevel::None), 105);
    }

    #[test]
    fn test_estimate_l01_output_size_fast_uses_integer_ratio() {
        assert_eq!(estimate_l01_output_size(100, CompressionLevel::Fast), 75);
    }

    #[test]
    fn test_estimate_l01_output_size_saturates_no_compression() {
        assert_eq!(
            estimate_l01_output_size(u64::MAX, CompressionLevel::None),
            u64::MAX
        );
    }

    #[test]
    fn test_checked_l01_total_size_add_sums_regular_values() {
        let path = Path::new("source.bin");

        assert_eq!(checked_l01_total_size_add(40, 2, path).unwrap(), 42);
    }

    #[test]
    fn test_checked_l01_total_size_add_rejects_overflow() {
        let path = Path::new("overflow-source.bin");
        let err = checked_l01_total_size_add(u64::MAX, 1, path).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("L01 total size overflow"));
        assert!(message.contains("overflow-source.bin"));
    }

    #[test]
    fn test_checked_l01_entry_count_add_rejects_over_limit() {
        let path = Path::new("too-many.bin");
        let mut count = MAX_L01_LOGICAL_ENTRIES;

        let err = checked_l01_entry_count_add(&mut count, path).unwrap_err();

        assert!(err.contains("expanded to more than"));
        assert_eq!(count, MAX_L01_LOGICAL_ENTRIES);
    }

    #[test]
    fn test_l01_logical_entry_name_rejects_root_path() {
        let err = l01_logical_entry_name(Path::new("/")).unwrap_err();

        assert!(err.contains("has no file or directory name"));
    }

    #[test]
    fn test_l01_logical_entry_name_uses_file_name() {
        assert_eq!(
            l01_logical_entry_name(Path::new("/tmp/source.bin")).unwrap(),
            "source.bin"
        );
    }

    #[test]
    fn test_validate_l01_export_options_rejects_missing_output_path() {
        let mut options = minimal_options();
        options.output_path = " ".to_string();

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("output path is required"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_missing_sources() {
        let mut options = minimal_options();
        options.source_paths.clear();

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("requires at least one source path"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_excessive_source_paths() {
        let mut options = minimal_options();
        options.source_paths = vec!["source.bin".to_string(); MAX_L01_SOURCE_PATHS + 1];

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_empty_source_path() {
        let mut options = minimal_options();
        options.source_paths.push(" ".to_string());

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("source paths cannot be empty"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_invalid_size_filter() {
        let mut options = minimal_options();
        options.min_file_size = Some(100);
        options.max_file_size = Some(10);

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("minFileSize"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_excessive_filter_extensions() {
        let mut options = minimal_options();
        options.filter_extensions = Some(vec!["pdf".to_string(); MAX_L01_FILTER_EXTENSIONS + 1]);

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("extension filter"));
        assert!(err.contains("exceeding limit"));
    }

    #[test]
    fn test_validate_l01_export_options_rejects_empty_filter_extension() {
        let mut options = minimal_options();
        options.exclude_extensions = Some(vec!["tmp".to_string(), " ".to_string()]);

        let err = validate_l01_export_options(&options).unwrap_err();

        assert!(err.contains("extension filter cannot contain empty entries"));
    }

    #[test]
    fn test_register_l01_cancel_flag_rejects_duplicate_output_path() {
        let output_path = unique_output_path();
        cleanup_l01_cancel_flag(&output_path);

        let registration =
            register_l01_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
        let err =
            register_l01_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap_err();
        drop(registration);

        assert!(err.contains("already running"));
    }

    #[test]
    fn test_l01_cancel_registration_cleans_up_on_drop() {
        let output_path = unique_output_path();
        cleanup_l01_cancel_flag(&output_path);

        {
            let _registration =
                register_l01_cancel_flag(&output_path, Arc::new(AtomicBool::new(false))).unwrap();
            assert!(l01_cancel_export(output_path.clone()).unwrap());
        }

        assert!(!l01_cancel_export(output_path).unwrap());
    }

    #[test]
    fn test_walk_dir_size_skips_beyond_depth_limit() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut current = dir.path().to_path_buf();
        for index in 0..=MAX_L01_TRAVERSAL_DEPTH + 1 {
            current = current.join(format!("d{}", index));
            std::fs::create_dir(&current).unwrap();
        }
        std::fs::write(current.join("too-deep.bin"), b"data").unwrap();

        let size = walk_dir_size(dir.path(), &FileFilter::default()).unwrap();
        assert_eq!(size, 0);
    }

    fn unique_output_path() -> String {
        format!(
            "/tmp/core-ffx-l01-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }
}
