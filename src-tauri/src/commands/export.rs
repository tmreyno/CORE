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

use crate::database;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::Path;
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

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

fn default_true() -> bool { true }

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
) -> Result<(u64, Option<String>), String> {
    let source_meta = fs::metadata(source)
        .map_err(|e| format!("Failed to read source metadata: {}", e))?;
    let file_size = source_meta.len();
    
    // Open source and destination
    let src_file = File::open(source)
        .map_err(|e| format!("Failed to open source: {}", e))?;
    let dst_file = File::create(dest)
        .map_err(|e| format!("Failed to create destination: {}", e))?;
    
    let mut reader = BufReader::with_capacity(1024 * 1024, src_file); // 1MB buffer
    let mut writer = BufWriter::with_capacity(1024 * 1024, dst_file);
    let mut hasher = Sha256::new();
    
    let mut bytes_copied = 0u64;
    let mut buffer = vec![0u8; 256 * 1024]; // 256KB chunks
    let mut last_emit = std::time::Instant::now();
    
    let filename = source.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string_lossy().to_string());
    
    loop {
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        
        if bytes_read == 0 {
            break;
        }
        
        writer.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Write error: {}", e))?;
        
        hasher.update(&buffer[..bytes_read]);
        bytes_copied += bytes_read as u64;
        
        // Emit progress every 100ms
        if last_emit.elapsed().as_millis() > 100 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let total_copied = total_bytes_so_far + bytes_copied;
            let speed = if elapsed > 0.0 { (total_copied as f64 / elapsed) as u64 } else { 0 };
            let percent = if total_bytes > 0 {
                (total_copied as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            
            let _ = window.emit("copy-progress", CopyProgress {
                operation_id: operation_id.to_string(),
                current_file: filename.clone(),
                current_index: file_index,
                total_files,
                current_file_bytes: bytes_copied,
                current_file_total: file_size,
                total_bytes_copied: total_copied,
                total_bytes,
                percent,
                status: format!("Copying + Hashing: {}", filename),
                speed_bps: speed,
                phase: Some("copying".to_string()),
                hash_bytes_processed: Some(bytes_copied),
                hash_bytes_total: Some(file_size),
            });
            
            last_emit = std::time::Instant::now();
        }
    }
    
    writer.flush().map_err(|e| format!("Flush error: {}", e))?;
    
    let hash = format!("{:x}", hasher.finalize());
    
    Ok((bytes_copied, Some(hash)))
}

/// Verify a copied file matches the original hash
fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open file for verification: {}", e))?;
    
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    
    loop {
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| format!("Read error during verification: {}", e))?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    let actual_hash = format!("{:x}", hasher.finalize());
    Ok(actual_hash == expected_hash)
}

/// Calculate total size of files to copy
fn calculate_total_size(paths: &[String]) -> u64 {
    let mut total = 0u64;
    for path in paths {
        if let Ok(meta) = fs::metadata(path) {
            if meta.is_file() {
                total += meta.len();
            } else if meta.is_dir() {
                total += calculate_dir_size(Path::new(path));
            }
        }
    }
    total
}

/// Calculate directory size recursively
fn calculate_dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += calculate_dir_size(&path);
            }
        }
    }
    total
}

/// Collect all files from paths (expanding directories)
fn collect_files(paths: &[String]) -> Vec<(String, String)> {
    let mut files = Vec::new();
    
    for path in paths {
        let path_obj = Path::new(path);
        if path_obj.is_file() {
            let filename = path_obj.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            files.push((path.clone(), filename));
        } else if path_obj.is_dir() {
            collect_dir_files(path_obj, path_obj, &mut files);
        }
    }
    
    files
}

/// Recursively collect files from a directory
fn collect_dir_files(base: &Path, dir: &Path, files: &mut Vec<(String, String)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let rel_path = path.strip_prefix(base)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default());
                files.push((path.to_string_lossy().to_string(), rel_path));
            } else if path.is_dir() {
                collect_dir_files(base, &path, files);
            }
        }
    }
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
    info!("Starting export operation: {} sources to {} (forensic: {})", 
          source_paths.len(), destination, options.as_ref().map(|o| o.compute_hashes).unwrap_or(false));
    
    let opts = options.unwrap_or_default();
    let operation_id = format!("export-{}", chrono::Utc::now().timestamp_millis());
    let start_time = std::time::Instant::now();
    let export_time = chrono::Utc::now().timestamp() as u64;
    
    // Create destination directory if needed
    let dest_path = Path::new(&destination);
    if opts.create_dirs && !dest_path.exists() {
        fs::create_dir_all(dest_path)
            .map_err(|e| format!("Failed to create destination directory: {}", e))?;
    }
    
    // Calculate total size and collect files
    let _ = window.emit("copy-progress", CopyProgress {
        operation_id: operation_id.clone(),
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
    });
    
    let total_bytes = calculate_total_size(&source_paths);
    let files = collect_files(&source_paths);
    let total_files = files.len();
    
    debug!("Copying {} files, {} bytes total", total_files, total_bytes);
    
    let mut files_copied = 0usize;
    let mut files_failed = 0usize;
    let mut bytes_copied = 0u64;
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut metadata_list: Vec<ExportMetadata> = Vec::new();
    let mut files_verified_known = 0usize;
    let mut files_mismatch_known = 0usize;
    
    // Copy/export each file
    for (index, (source, rel_path)) in files.iter().enumerate() {
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
            failures.push((source.clone(), "File exists (overwrite disabled)".to_string()));
            files_failed += 1;
            continue;
        }
        
        // Copy the file
        match copy_file_with_progress(
            source_path,
            &dest_file,
            &window,
            &operation_id,
            index + 1,
            total_files,
            bytes_copied,
            total_bytes,
            start_time,
        ) {
            Ok((copied, hash)) => {
                bytes_copied += copied;
                
                // Handle forensic metadata if hashing is enabled
                if opts.compute_hashes {
                    if let (Some(sha256), Ok(source_meta)) = (&hash, fs::metadata(source_path)) {
                        let verified = if opts.verify_after_copy {
                            verify_file_hash(&dest_file, sha256).unwrap_or_default()
                        } else {
                            true  // Not verified, just copied
                        };
                        
                        if !verified {
                            warn!("Verification failed for {}", rel_path);
                            failures.push((source.clone(), "Hash verification failed".to_string()));
                            files_failed += 1;
                            continue;
                        }
                        
                        let modified_time = source_meta.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        
                        // Check against known hashes if requested
                        let (known_hash, matches_known, known_hash_source) = if opts.verify_against_known {
                            let db = database::get_db();
                            match db.lookup_known_hash_by_path(source) {
                                Ok(Some((stored_hash, hash_source))) => {
                                    let matches = stored_hash.eq_ignore_ascii_case(sha256);
                                    if matches {
                                        files_verified_known += 1;
                                        debug!("Known hash match for {}", rel_path);
                                    } else {
                                        files_mismatch_known += 1;
                                        warn!("Known hash MISMATCH for {}: expected={}, got={}", 
                                              rel_path, stored_hash, sha256);
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
                            match verify_file_hash(&dest_file, expected) {
                                Ok(true) => {
                                    debug!("Hash verified for {}", rel_path);
                                }
                                Ok(false) => {
                                    warn!("Hash mismatch for {}", rel_path);
                                    failures.push((source.clone(), "Hash verification failed".to_string()));
                                    files_failed += 1;
                                    continue;
                                }
                                Err(e) => {
                                    warn!("Hash verification error for {}: {}", rel_path, e);
                                }
                            }
                        }
                    }
                }
                
                // Preserve timestamps
                if opts.preserve_timestamps {
                    if let Ok(meta) = fs::metadata(source_path) {
                        if let Ok(mtime) = meta.modified() {
                            let _ = filetime::set_file_mtime(&dest_file, filetime::FileTime::from_system_time(mtime));
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
    let avg_speed = if duration_ms > 0 {
        (bytes_copied * 1000) / duration_ms
    } else {
        0
    };
    
    // Generate manifest and reports if requested
    let mut json_manifest_path = None;
    let mut txt_report_path = None;
    
    if opts.compute_hashes && !metadata_list.is_empty() {
        let export_name = opts.export_name.as_deref().unwrap_or("export");
        
        // Generate JSON manifest
        if opts.generate_json_manifest {
            let manifest_path = dest_path.join(format!("{}_manifest.json", export_name));
            let manifest = serde_json::json!({
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
            
            if let Err(e) = fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap_or_default()) {
                warn!("Failed to write JSON manifest: {}", e);
            } else {
                info!("JSON manifest written to {}", manifest_path.display());
                json_manifest_path = Some(manifest_path.to_string_lossy().to_string());
            }
        }
        
        // Generate TXT report
        if opts.generate_txt_report {
            let report_path = dest_path.join(format!("{}_report.txt", export_name));
            let mut report = String::new();
            report.push_str(&format!("Export Report: {}\n", export_name));
            report.push_str(&format!("Export Time: {}\n", chrono::Utc::now().to_rfc3339()));
            report.push_str(&format!("Total Files: {}\n", files_copied));
            report.push_str(&format!("Total Bytes: {}\n", bytes_copied));
            report.push_str(&format!("Duration: {}ms\n", duration_ms));
            report.push_str(&format!("Files Verified (Known): {}\n", files_verified_known));
            report.push_str(&format!("Files Mismatched (Known): {}\n", files_mismatch_known));
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
    let _ = window.emit("copy-progress", CopyProgress {
        operation_id: operation_id.clone(),
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
    });
    
    info!("Export complete: {} files, {} bytes in {}ms (forensic: {})", 
          files_copied, bytes_copied, duration_ms, opts.compute_hashes);
    
    Ok(CopyResult {
        files_copied,
        files_failed,
        bytes_copied,
        duration_ms,
        avg_speed_bps: avg_speed,
        failures,
        metadata: if opts.compute_hashes { Some(metadata_list) } else { None },
        json_manifest_path,
        txt_report_path,
        files_verified_known,
        files_mismatch_known,
    })
}
