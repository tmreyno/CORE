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
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{Emitter, Window};
use tracing::{debug, info};

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

// =============================================================================
// Helper functions
// =============================================================================

/// Check if a canonicalized path is the system boot volume.
/// Cross-platform: detects macOS root, Windows C:\ drive, and Linux root.
fn is_system_boot_volume(canon: &std::path::Path) -> bool {
    let canon_str = canon.to_string_lossy();
    #[cfg(target_os = "macos")]
    {
        if canon_str == "/" || canon_str == "/System/Volumes/Data" {
            return true;
        }
    }
    #[cfg(target_os = "windows")]
    {
        let upper = canon_str.to_uppercase();
        if upper == "C:\\" || upper == "C:" || upper.starts_with("C:\\") && canon.parent().is_none()
        {
            return true;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if canon_str == "/" {
            return true;
        }
    }
    false
}

/// Query available disk space at the given path (cross-platform).
/// On Unix uses libc::statvfs, on other platforms falls back to sysinfo::Disks.
fn check_available_space(path: &std::path::Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        if let Ok(c_path) = std::ffi::CString::new(path.as_os_str().as_bytes()) {
            unsafe {
                let mut stat: libc::statvfs = std::mem::zeroed();
                if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                    #[allow(clippy::unnecessary_cast)]
                    let avail = stat.f_bavail as u64 * stat.f_frsize as u64;
                    return Ok(avail);
                }
            }
        }
        Err("statvfs failed".into())
    }
    #[cfg(not(unix))]
    {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for d in disks.iter() {
            if path.starts_with(d.mount_point()) {
                return Ok(d.available_space());
            }
        }
        Err("Could not determine available space".into())
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

/// Recursively compute total file size in a directory.
fn walk_dir_size(dir: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total: u64 = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_file() {
            total += entry.metadata()?.len();
        } else if ft.is_dir() {
            total += walk_dir_size(&entry.path())?;
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
) -> Result<usize, String> {
    let mut count = 0;

    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?
        .filter_map(|e| e.ok())
        .collect();

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
            let dir_id = writer.add_directory(file_name, parent_id);
            count += 1;
            count += walk_dir_into_writer(writer, &path, dir_id)?;
        } else if metadata.is_file() {
            let size = metadata.len();
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
    {
        let mut flags = L01_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(options.output_path.clone(), cancel_flag.clone());
    }

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

    for path_str in &options.source_paths {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            // Add the directory entry itself, then walk its contents under it
            let dir_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let parent_id = writer.add_directory(dir_name, 0);

            // Walk the directory contents and add under the parent directory entry
            let count = walk_dir_into_writer(&mut writer, &path, parent_id)?;
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
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
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
        let _ = window.emit("l01-export-progress", &early_progress);
    }

    // Check destination has enough free space
    let total_source_bytes = writer.total_file_size();
    {
        let avail_result = check_available_space(&output_canon);
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
        let _ = window_clone.emit("l01-export-progress", &progress);
    });

    // Run the write operation in a blocking task
    let output_path_for_cleanup = options.output_path.clone();
    let result =
        tokio::task::spawn_blocking(move || writer.write(Some(&cancel_flag), Some(progress_fn)))
            .await
            .map_err(|e| format!("L01 write task panicked: {}", e))?
            .map_err(format_write_error)?;

    // Clean up cancel flag
    {
        let mut flags = L01_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.remove(&output_path_for_cleanup);
    }

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
    let mut total_source_bytes: u64 = 0;

    for path_str in &source_paths {
        let path = std::path::Path::new(path_str);
        if !path.exists() {
            return Err(format!("Source path does not exist: {}", path_str));
        }
        if path.is_dir() {
            // Walk directory recursively
            total_source_bytes += walk_dir_size(path)
                .map_err(|e| format!("Failed to walk directory {}: {}", path_str, e))?;
        } else {
            let metadata = std::fs::metadata(path)
                .map_err(|e| format!("Failed to read metadata for {}: {}", path_str, e))?;
            total_source_bytes += metadata.len();
        }
    }

    // Estimate based on compression level
    let compression_level = match compression.as_deref() {
        Some(c) => parse_l01_compression(c)?,
        None => CompressionLevel::Fast,
    };

    let estimated = match compression_level {
        CompressionLevel::None => {
            // No compression: data + ~5% overhead for headers/tables
            (total_source_bytes as f64 * 1.05) as u64
        }
        CompressionLevel::Fast => {
            // Fast compression: rough estimate ~70% of original + overhead
            (total_source_bytes as f64 * 0.75) as u64
        }
        CompressionLevel::Best => {
            // Best compression: rough estimate ~55% of original + overhead
            (total_source_bytes as f64 * 0.60) as u64
        }
    };

    Ok(estimated)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
