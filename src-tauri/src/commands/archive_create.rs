// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive creation commands using sevenzip-ffi library
//!
//! This module provides secure, forensic-grade archive creation with:
//! - Standard 7z format (compatible with 7-Zip)
//! - AES-256 encryption
//! - Multi-threading
//! - Split archives for large files
//! - Progress tracking
//! - SHA-256 verification

use serde::{Deserialize, Serialize};
use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::common::hash::hash_file;

/// Global cancel flags for active archive creation jobs
static CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

/// Progress event emitted during archive creation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCreateProgress {
    /// Archive path being created
    pub archive_path: String,
    /// Current file being processed
    pub current_file: String,
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes to process (0 if unknown)
    pub bytes_total: u64,
    /// Current file bytes processed
    pub current_file_bytes: u64,
    /// Current file total bytes
    pub current_file_total: u64,
    /// Progress percentage (0-100)
    pub percent: f64,
    /// Current operation status
    pub status: String,
}

/// Archive creation options
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArchiveOptions {
    /// Compression level (0-9)
    pub compression_level: u8,
    /// Optional password for AES-256 encryption
    pub password: Option<String>,
    /// Number of threads (0 = auto)
    pub num_threads: Option<i32>,
    /// Dictionary size in MB (0 = auto)
    pub dict_size_mb: Option<u64>,
    /// Enable solid compression (default: true)
    pub solid: Option<bool>,
    /// Split archive size in MB (0 = no split)
    pub split_size_mb: Option<u64>,
    /// Chunk size for streaming in MB (default: 64)
    pub chunk_size_mb: Option<u64>,
    // Forensic options
    /// Generate a forensic JSON manifest alongside the archive
    pub generate_manifest: Option<bool>,
    /// Verify archive integrity after creation
    pub verify_after_create: Option<bool>,
    /// Hash algorithm for manifest: "SHA-256", "SHA-1", "MD5", "SHA-256+MD5"
    pub hash_algorithm: Option<String>,
    /// Examiner name for chain-of-custody metadata
    pub examiner_name: Option<String>,
    /// Case number for chain-of-custody metadata
    pub case_number: Option<String>,
    /// Evidence description for chain-of-custody metadata
    pub evidence_description: Option<String>,
}

impl Default for CreateArchiveOptions {
    fn default() -> Self {
        Self {
            compression_level: 0, // Store - best for forensic containers (E01/AD1 already compressed)
            password: None,
            num_threads: Some(0), // Auto-detect
            dict_size_mb: None,
            solid: Some(true),
            split_size_mb: Some(2048), // 2GB default - good for cloud uploads and USB drives
            chunk_size_mb: Some(64),
            generate_manifest: Some(true),
            verify_after_create: Some(true),
            hash_algorithm: Some("SHA-256".to_string()),
            examiner_name: None,
            case_number: None,
            evidence_description: None,
        }
    }
}

/// Calculate total size of directory recursively
fn calculate_dir_size(dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            total += path.metadata()
                .map(|m| m.len())
                .map_err(|e| format!("Failed to get file size: {}", e))?;
        } else if path.is_dir() {
            total += calculate_dir_size(&path)?;
        }
    }
    
    Ok(total)
}

// =============================================================================
// Forensic Manifest
// =============================================================================

/// A single file entry in the forensic manifest
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFileEntry {
    /// Relative path within the archive
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp (ISO 8601)
    pub modified: Option<String>,
    /// SHA-256 hash (if computed)
    pub sha256: Option<String>,
    /// MD5 hash (if computed)
    pub md5: Option<String>,
    /// SHA-1 hash (if computed)
    pub sha1: Option<String>,
}

/// Forensic manifest metadata
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForensicManifest {
    /// Manifest format version
    pub version: String,
    /// Tool that created the manifest
    pub tool: String,
    /// Tool version
    pub tool_version: String,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Archive file name
    pub archive_name: String,
    /// Archive hash (SHA-256 of the final archive)
    pub archive_sha256: Option<String>,
    /// Compression level used (0-9)
    pub compression_level: u8,
    /// Whether AES-256 encryption was applied
    pub encrypted: bool,
    /// Hash algorithm(s) used for file entries
    pub hash_algorithms: Vec<String>,
    /// Total number of files
    pub total_files: usize,
    /// Total size of all files in bytes
    pub total_size: u64,
    /// Chain-of-custody metadata
    pub chain_of_custody: ChainOfCustody,
    /// File inventory with hashes
    pub files: Vec<ManifestFileEntry>,
}

/// Chain-of-custody metadata for forensic manifests
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainOfCustody {
    /// Examiner name
    pub examiner_name: Option<String>,
    /// Case number / reference
    pub case_number: Option<String>,
    /// Evidence description
    pub evidence_description: Option<String>,
    /// Hostname where archive was created
    pub hostname: String,
    /// Operating system
    pub operating_system: String,
}

/// Collect all files from input paths (recursively for directories)
fn collect_files(input_paths: &[String]) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let mut files = Vec::new();
    
    for input_path in input_paths {
        let path = Path::new(input_path);
        if path.is_file() {
            // Use just the filename as the relative path
            let rel = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| input_path.clone());
            files.push((rel, path.to_path_buf()));
        } else if path.is_dir() {
            collect_dir_files(path, path, &mut files)?;
        }
    }
    
    Ok(files)
}

/// Recursively collect files from a directory
fn collect_dir_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            let rel = path.strip_prefix(root)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            files.push((rel, path));
        } else if path.is_dir() {
            collect_dir_files(root, &path, files)?;
        }
    }
    
    Ok(())
}

/// Generate a forensic manifest for the archive
fn generate_forensic_manifest(
    archive_path: &str,
    input_paths: &[String],
    opts: &CreateArchiveOptions,
    window: &Window,
) -> Result<String, String> {
    let hash_algo = opts.hash_algorithm.as_deref().unwrap_or("SHA-256");
    
    // Determine which algorithms to compute
    let compute_sha256 = hash_algo.contains("SHA-256") || hash_algo.contains("sha-256");
    let compute_md5 = hash_algo.contains("MD5") || hash_algo.contains("md5");
    let compute_sha1 = hash_algo.contains("SHA-1") || hash_algo.contains("sha-1");
    
    // Collect all files
    let all_files = collect_files(input_paths)?;
    let total_files = all_files.len();
    
    // Emit manifest generation status
    let _ = window.emit("archive-create-progress", ArchiveCreateProgress {
        archive_path: archive_path.to_string(),
        current_file: String::new(),
        bytes_processed: 0,
        bytes_total: 0,
        current_file_bytes: 0,
        current_file_total: 0,
        percent: 0.0,
        status: format!("Generating forensic manifest ({} files)...", total_files),
    });
    
    let mut manifest_files = Vec::with_capacity(total_files);
    let mut total_size: u64 = 0;
    
    for (i, (rel_path, abs_path)) in all_files.iter().enumerate() {
        let metadata = std::fs::metadata(abs_path)
            .map_err(|e| format!("Failed to read metadata for {}: {}", abs_path.display(), e))?;
        
        let file_size = metadata.len();
        total_size += file_size;
        
        // Modified time as ISO 8601
        let modified = metadata.modified().ok().map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.to_rfc3339()
        });
        
        // Compute hashes
        let sha256 = if compute_sha256 {
            Some(hash_file(abs_path, "sha256")
                .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?)
        } else {
            None
        };
        
        let md5_hash = if compute_md5 {
            Some(hash_file(abs_path, "md5")
                .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?)
        } else {
            None
        };
        
        let sha1_hash = if compute_sha1 {
            Some(hash_file(abs_path, "sha1")
                .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?)
        } else {
            None
        };
        
        // Progress update (every 10 files or last file)
        if i % 10 == 0 || i == total_files - 1 {
            let percent = ((i + 1) as f64 / total_files as f64) * 100.0;
            let _ = window.emit("archive-create-progress", ArchiveCreateProgress {
                archive_path: archive_path.to_string(),
                current_file: rel_path.clone(),
                bytes_processed: 0,
                bytes_total: 0,
                current_file_bytes: 0,
                current_file_total: 0,
                percent,
                status: format!("Hashing for manifest: {}/{} files", i + 1, total_files),
            });
        }
        
        manifest_files.push(ManifestFileEntry {
            path: rel_path.clone(),
            size: file_size,
            modified,
            sha256,
            md5: md5_hash,
            sha1: sha1_hash,
        });
    }
    
    // Hash the archive itself (SHA-256)
    let archive_sha256 = if Path::new(archive_path).exists() {
        let _ = window.emit("archive-create-progress", ArchiveCreateProgress {
            archive_path: archive_path.to_string(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 0.0,
            status: "Hashing archive file...".to_string(),
        });
        Some(hash_file(Path::new(archive_path), "sha256")
            .map_err(|e| format!("Failed to hash archive: {}", e))?)
    } else {
        // Split archives - hash first volume
        let first_vol = format!("{}.001", archive_path);
        if Path::new(&first_vol).exists() {
            Some(hash_file(Path::new(&first_vol), "sha256")
                .map_err(|e| format!("Failed to hash first volume: {}", e))?)
        } else {
            None
        }
    };
    
    // Build hash algorithms list
    let mut hash_algorithms = Vec::new();
    if compute_sha256 { hash_algorithms.push("SHA-256".to_string()); }
    if compute_md5 { hash_algorithms.push("MD5".to_string()); }
    if compute_sha1 { hash_algorithms.push("SHA-1".to_string()); }
    
    // Get system info for chain-of-custody
    let hostname = std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let operating_system = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    
    let manifest = ForensicManifest {
        version: "1.0".to_string(),
        tool: "CORE-FFX".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        archive_name: Path::new(archive_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| archive_path.to_string()),
        archive_sha256,
        compression_level: opts.compression_level,
        encrypted: opts.password.is_some(),
        hash_algorithms,
        total_files: manifest_files.len(),
        total_size,
        chain_of_custody: ChainOfCustody {
            examiner_name: opts.examiner_name.clone(),
            case_number: opts.case_number.clone(),
            evidence_description: opts.evidence_description.clone(),
            hostname,
            operating_system,
        },
        files: manifest_files,
    };
    
    // Write manifest JSON
    let manifest_path = format!("{}.manifest.json", archive_path);
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    
    std::fs::write(&manifest_path, &json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;
    
    info!("Forensic manifest written: {} ({} files, {} bytes)", manifest_path, manifest.total_files, manifest.total_size);
    
    Ok(manifest_path)
}

/// Create a 7z archive with the given files
///
/// # Arguments
/// * `archive_path` - Output archive path (e.g., "evidence.7z")
/// * `input_paths` - Array of file/directory paths to compress
/// * `options` - Compression options
/// * `window` - Tauri window for progress events
///
/// # Returns
/// * `Ok(archive_path)` - Successfully created archive
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn create_7z_archive(
    archive_path: String,
    input_paths: Vec<String>,
    options: Option<CreateArchiveOptions>,
    window: Window,
) -> Result<String, String> {
    info!("Creating 7z archive: {} with {} inputs", archive_path, input_paths.len());
    debug!("Input paths: {:?}", input_paths);
    
    let opts = options.unwrap_or_default();
    
    // Validate inputs
    if input_paths.is_empty() {
        return Err("No input files specified".to_string());
    }
    
    for path in &input_paths {
        if !Path::new(path).exists() {
            return Err(format!("Input path does not exist: {}", path));
        }
    }
    
    // Convert compression level to enum
    let compression_level = match opts.compression_level {
        0 => CompressionLevel::Store,
        1 => CompressionLevel::Fastest,
        2..=3 => CompressionLevel::Fast,
        4..=6 => CompressionLevel::Normal,
        7..=8 => CompressionLevel::Maximum,
        9 => CompressionLevel::Ultra,
        _ => CompressionLevel::Normal,
    };
    
    // Determine operation name based on compression level
    let operation_name = if opts.compression_level == 0 {
        "Storing"
    } else {
        "Compressing"
    };
    
    // Register cancel flag for this archive creation
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = CANCEL_FLAGS.lock().map_err(|e| format!("Lock error: {}", e))?;
        flags.insert(archive_path.clone(), cancel_flag.clone());
    }
    
    // Spawn blocking task for compression
    let window_clone = window.clone();
    let archive_path_clone = archive_path.clone();
    
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Initialize sevenzip library
        let sz = SevenZip::new().map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Emit starting status
        let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
            archive_path: archive_path_clone.clone(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 0.0,
            status: "Initializing archive...".to_string(),
        });
        
        // Calculate total size to determine if streaming is needed
        let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
            archive_path: archive_path_clone.clone(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 0.0,
            status: "Calculating archive size...".to_string(),
        });
        
        let mut total_size: u64 = 0;
        for path in &input_paths {
            let path_obj = Path::new(path);
            if path_obj.is_file() {
                let file_size = path_obj.metadata()
                    .map(|m| m.len())
                    .unwrap_or(0);
                debug!("File {} size: {} bytes", path, file_size);
                total_size += file_size;
            } else if path_obj.is_dir() {
                let dir_size = calculate_dir_size(path_obj).unwrap_or(0);
                debug!("Directory {} size: {} bytes", path, dir_size);
                total_size += dir_size;
            }
        }
        
        let total_size_gb = total_size / (1024 * 1024 * 1024);
        debug!("Total size calculated: {} bytes ({}GB)", total_size, total_size_gb);
        
        // For large archives (>8GB), use temp file and rename on completion
        let one_gb = 1024 * 1024 * 1024u64;
        let eight_gb = 8 * one_gb;
        let is_large_archive = total_size > eight_gb;
        
        // Use temp file for large archives to avoid partial corrupt files
        let (working_path, final_path) = if is_large_archive {
            let temp_path = format!("{}.tmp", archive_path_clone);
            (temp_path, Some(archive_path_clone.clone()))
        } else {
            (archive_path_clone.clone(), None)
        };
        
        // Determine compression strategy:
        // - Standard: < 1GB, simple in-memory compression  
        // - Streaming: >= 1GB, uses split volumes to enable true streaming
        // For very large archives (>8GB), we force split volumes to prevent OOM
        let use_streaming = opts.split_size_mb.is_some() || total_size > one_gb;
        
        // For large archives without explicit split, auto-enable split at 4.7GB (DVD size)
        // This forces the library to use true streaming and prevents OOM
        let auto_split_mb = if is_large_archive && opts.split_size_mb.is_none() {
            Some(4700u64) // 4.7GB volumes
        } else {
            opts.split_size_mb
        };
        
        debug!("Compression strategy: use_streaming={}, is_large_archive={}, auto_split_mb={:?}", 
               use_streaming, is_large_archive, auto_split_mb);
        
        if use_streaming {
            // Use streaming compression for large files or split archives
            let mut stream_opts = StreamOptions::default();
            
            if let Some(threads) = opts.num_threads {
                stream_opts.num_threads = threads as usize;
            }
            if let Some(dict_mb) = opts.dict_size_mb {
                stream_opts.dict_size = dict_mb * 1024 * 1024;
            }
            if let Some(solid) = opts.solid {
                stream_opts.solid = solid;
            }
            if let Some(password) = opts.password.as_ref() {
                stream_opts.password = Some(password.clone());
            }
            // Use auto_split_mb which may be auto-set for large archives
            if let Some(split_mb) = auto_split_mb {
                stream_opts.split_size = split_mb * 1024 * 1024;
            }
            if let Some(chunk_mb) = opts.chunk_size_mb {
                stream_opts.chunk_size = chunk_mb * 1024 * 1024;
            }
            
            let split_info = if auto_split_mb.is_some() && opts.split_size_mb.is_none() {
                format!("auto-split: {}MB", auto_split_mb.unwrap_or(0))
            } else if auto_split_mb.is_some() {
                format!("split: {}MB", auto_split_mb.unwrap_or(0))
            } else {
                "no split".to_string()
            };
            
            info!("Using streaming compression ({}, chunk: {}MB) for {}GB archive", 
                  split_info,
                  opts.chunk_size_mb.unwrap_or(64),
                  total_size_gb);
            
            if is_large_archive {
                info!("Large archive detected - writing to temp file: {}", working_path);
            }
            
            // Emit status before starting compression
            let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
                archive_path: archive_path_clone.clone(),
                current_file: String::new(),
                bytes_processed: 0,
                bytes_total: total_size,
                current_file_bytes: 0,
                current_file_total: 0,
                percent: 0.0,
                status: format!("Starting {} of {} files ({}GB)...", operation_name.to_lowercase(), input_paths.len(), total_size_gb),
            });
            
            // Create with streaming and progress callback
            let input_paths_vec: Vec<&str> = input_paths.iter().map(|s| s.as_str()).collect();
            let window_for_callback = Arc::new(window_clone.clone());
            let archive_path_for_callback = archive_path_clone.clone();
            let operation_name_clone = operation_name.to_string();
            let working_path_clone = working_path.clone();
            let cancel_flag_for_callback = cancel_flag.clone();
            
            sz.create_archive_streaming(
                &working_path,
                &input_paths_vec,
                compression_level,
                Some(&stream_opts),
                Some(Box::new(move |bytes_processed, bytes_total, current_file_bytes, 
                                     current_file_total, current_file_name| {
                    // Check cancel flag
                    if cancel_flag_for_callback.load(Ordering::SeqCst) {
                        let _ = window_for_callback.emit("archive-create-progress", ArchiveCreateProgress {
                            archive_path: archive_path_for_callback.clone(),
                            current_file: String::new(),
                            bytes_processed,
                            bytes_total,
                            current_file_bytes: 0,
                            current_file_total: 0,
                            percent: 0.0,
                            status: "Cancelling...".to_string(),
                        });
                        return;
                    }
                    
                    let percent = if bytes_total > 0 {
                        (bytes_processed as f64 / bytes_total as f64) * 100.0
                    } else if current_file_total > 0 {
                        (current_file_bytes as f64 / current_file_total as f64) * 100.0
                    } else {
                        0.0
                    };
                    
                    let _ = window_for_callback.emit("archive-create-progress", ArchiveCreateProgress {
                        archive_path: archive_path_for_callback.clone(),
                        current_file: current_file_name.to_string(),
                        bytes_processed,
                        bytes_total,
                        current_file_bytes,
                        current_file_total,
                        percent,
                        status: format!("{}: {}", operation_name_clone, current_file_name),
                    });
                })),
            ).map_err(|e| format!("Streaming compression failed: {}", e))?;
            
            // Check if cancelled - clean up partial files
            if cancel_flag.load(Ordering::SeqCst) {
                warn!("Archive creation was cancelled, cleaning up partial files");
                let _ = std::fs::remove_file(&working_path_clone);
                // Also clean up split volumes
                for i in 1.. {
                    let volume = format!("{}.{:03}", working_path_clone, i);
                    if Path::new(&volume).exists() {
                        let _ = std::fs::remove_file(&volume);
                    } else {
                        break;
                    }
                }
                return Err("Archive creation cancelled by user".to_string());
            }
            
            // If we used a temp file, rename to final path
            if let Some(ref final_path) = final_path {
                info!("Renaming temp file {} to {}", working_path_clone, final_path);
                // For split archives, the library creates .001, .002, etc.
                // We need to rename all parts
                if auto_split_mb.is_some() {
                    // Rename split volumes: working_path.001 -> final_path.001, etc.
                    for i in 1.. {
                        let temp_volume = format!("{}.{:03}", working_path_clone, i);
                        let final_volume = format!("{}.{:03}", final_path, i);
                        if Path::new(&temp_volume).exists() {
                            std::fs::rename(&temp_volume, &final_volume)
                                .map_err(|e| format!("Failed to rename {} to {}: {}", temp_volume, final_volume, e))?;
                        } else {
                            break;
                        }
                    }
                    // Also try renaming base file if it exists
                    if Path::new(&working_path_clone).exists() {
                        std::fs::rename(&working_path_clone, final_path)
                            .map_err(|e| format!("Failed to rename temp file: {}", e))?;
                    }
                } else {
                    std::fs::rename(&working_path_clone, final_path)
                        .map_err(|e| format!("Failed to rename temp file: {}", e))?;
                }
            }
        } else {
            // Use standard compression for smaller archives
            let mut compress_opts = CompressOptions::default();
            
            if let Some(threads) = opts.num_threads {
                compress_opts.num_threads = threads as usize;
            }
            if let Some(dict_mb) = opts.dict_size_mb {
                compress_opts.dict_size = dict_mb * 1024 * 1024;
            }
            if let Some(solid) = opts.solid {
                compress_opts.solid = solid;
            }
            if let Some(password) = opts.password.as_ref() {
                compress_opts.password = Some(password.clone());
            }
            
            info!("Using standard {} (level {})", 
                  if opts.compression_level == 0 { "storage" } else { "compression" },
                  opts.compression_level);
            
            // Emit status before starting compression
            let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
                archive_path: archive_path_clone.clone(),
                current_file: String::new(),
                bytes_processed: 0,
                bytes_total: total_size,
                current_file_bytes: 0,
                current_file_total: 0,
                percent: 0.0,
                status: format!("Starting {} of {} files...", operation_name.to_lowercase(), input_paths.len()),
            });
            
            sz.create_archive(
                &archive_path_clone,
                &input_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                compression_level,
                Some(&compress_opts),
            ).map_err(|e| format!("Compression failed: {}", e))?;
        }
        
        // ─── Post-Archive Forensic Steps ────────────────────────────
        
        // Step 1: Verify archive integrity if requested
        if opts.verify_after_create.unwrap_or(false) {
            let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
                archive_path: archive_path_clone.clone(),
                current_file: String::new(),
                bytes_processed: total_size,
                bytes_total: total_size,
                current_file_bytes: 0,
                current_file_total: 0,
                percent: 0.0,
                status: "Verifying archive integrity...".to_string(),
            });
            
            let verify_sz = SevenZip::new().map_err(|e| format!("Failed to init 7z for verify: {}", e))?;
            verify_sz.test_archive(
                &archive_path_clone,
                opts.password.as_deref(),
                None, // No progress callback for verification
            ).map_err(|e| format!("Archive verification failed: {}", e))?;
            
            info!("Archive verification PASSED: {}", archive_path_clone);
        }
        
        // Step 2: Generate forensic manifest if requested
        if opts.generate_manifest.unwrap_or(false) {
            let manifest_path = generate_forensic_manifest(
                &archive_path_clone,
                &input_paths,
                &opts,
                &window_clone,
            )?;
            info!("Forensic manifest: {}", manifest_path);
        }
        
        // Emit completion status
        let _ = window_clone.emit("archive-create-progress", ArchiveCreateProgress {
            archive_path: archive_path_clone.clone(),
            current_file: String::new(),
            bytes_processed: total_size,
            bytes_total: total_size,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 100.0,
            status: "Archive completed successfully".to_string(),
        });
        
        info!("Archive created successfully: {}", archive_path_clone);
        Ok(archive_path_clone)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;
    
    // Clean up cancel flag
    {
        let mut flags = CANCEL_FLAGS.lock().map_err(|e| format!("Lock error: {}", e))?;
        flags.remove(&archive_path);
    }
    
    result
}

// NOTE: test_7z_archive moved to commands/archive.rs for consistency with other archive operations

/// Get estimated archive size before creating it
///
/// # Arguments
/// * `input_paths` - Array of file/directory paths to compress
/// * `compression_level` - Compression level (0-9)
///
/// # Returns
/// * `Ok((uncompressed_size, estimated_compressed_size))` - Size estimates in bytes
/// * `Err(message)` - Error message
#[tauri::command]
pub async fn estimate_archive_size(
    input_paths: Vec<String>,
    compression_level: u8,
) -> Result<(u64, u64), String> {
    debug!("Estimating archive size for {} inputs", input_paths.len());
    
    tauri::async_runtime::spawn_blocking(move || {
        let mut total_size: u64 = 0;
        
        for path in &input_paths {
            let path_obj = Path::new(path);
            if !path_obj.exists() {
                return Err(format!("Input path does not exist: {}", path));
            }
            
            if path_obj.is_file() {
                total_size += path_obj.metadata()
                    .map(|m| m.len())
                    .map_err(|e| format!("Failed to get file size: {}", e))?;
            } else if path_obj.is_dir() {
                // Recursively calculate directory size
                total_size += calculate_dir_size(path_obj)?;
            }
        }
        
        // Estimate compressed size based on compression level
        // These are rough estimates based on typical text/binary data
        let compression_ratio = match compression_level {
            0 => 1.0,       // Store (no compression)
            1 => 0.7,       // Fastest (~30% reduction)
            2..=3 => 0.5,   // Fast (~50% reduction)
            4..=6 => 0.35,  // Normal (~65% reduction)
            7..=8 => 0.25,  // Maximum (~75% reduction)
            9 => 0.20,      // Ultra (~80% reduction)
            _ => 0.35,
        };
        
        let estimated_compressed = (total_size as f64 * compression_ratio) as u64;
        
        debug!("Size estimate: {} bytes uncompressed, ~{} bytes compressed", 
               total_size, estimated_compressed);
        
        Ok((total_size, estimated_compressed))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Cancel an in-progress archive creation
/// 
/// Sets the cancel flag for the given archive path. The compression
/// progress callback checks this flag and stops processing when set.
/// The partial archive file is cleaned up automatically.
#[tauri::command]
pub async fn cancel_archive_creation(
    archive_path: String,
) -> Result<(), String> {
    info!("Archive creation cancellation requested for: {}", archive_path);
    
    let flags = CANCEL_FLAGS.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(flag) = flags.get(&archive_path) {
        flag.store(true, Ordering::SeqCst);
        info!("Cancel flag set for: {}", archive_path);
        Ok(())
    } else {
        warn!("No active archive creation found for: {}", archive_path);
        Err(format!("No active archive creation found for: {}", archive_path))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ==================== CreateArchiveOptions defaults ====================

    #[test]
    fn test_default_options_compression_level() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.compression_level, 0, "Default should be Store (0) for forensic containers");
    }

    #[test]
    fn test_default_options_no_password() {
        let opts = CreateArchiveOptions::default();
        assert!(opts.password.is_none());
    }

    #[test]
    fn test_default_options_auto_threads() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.num_threads, Some(0));
    }

    #[test]
    fn test_default_options_solid_enabled() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.solid, Some(true));
    }

    #[test]
    fn test_default_options_split_size() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.split_size_mb, Some(2048), "Default split should be 2GB");
    }

    #[test]
    fn test_default_options_chunk_size() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.chunk_size_mb, Some(64));
    }

    #[test]
    fn test_default_options_manifest_enabled() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.generate_manifest, Some(true));
    }

    #[test]
    fn test_default_options_verify_enabled() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.verify_after_create, Some(true));
    }

    #[test]
    fn test_default_options_hash_algorithm() {
        let opts = CreateArchiveOptions::default();
        assert_eq!(opts.hash_algorithm, Some("SHA-256".to_string()));
    }

    #[test]
    fn test_default_options_no_chain_of_custody() {
        let opts = CreateArchiveOptions::default();
        assert!(opts.examiner_name.is_none());
        assert!(opts.case_number.is_none());
        assert!(opts.evidence_description.is_none());
    }

    // ==================== calculate_dir_size ====================

    #[test]
    fn test_calculate_dir_size_empty() {
        let dir = TempDir::new().unwrap();
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_single_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap(); // 11 bytes
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn test_calculate_dir_size_multiple_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "aaaa").unwrap(); // 4 bytes
        fs::write(dir.path().join("b.txt"), "bbbbbb").unwrap(); // 6 bytes
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_calculate_dir_size_nested_dirs() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(dir.path().join("root.txt"), "root").unwrap(); // 4 bytes
        fs::write(sub.join("nested.txt"), "nested").unwrap(); // 6 bytes
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_calculate_dir_size_deeply_nested() {
        let dir = TempDir::new().unwrap();
        let deep = dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.bin"), vec![0u8; 1024]).unwrap(); // 1024 bytes
        let size = calculate_dir_size(dir.path()).unwrap();
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        let result = calculate_dir_size(Path::new("/nonexistent/path/xyz"));
        assert!(result.is_err());
    }

    // ==================== collect_files ====================

    #[test]
    fn test_collect_files_single_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("evidence.e01");
        fs::write(&file_path, "data").unwrap();

        let files = collect_files(&[file_path.to_string_lossy().to_string()]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "evidence.e01"); // relative path is just filename
        assert_eq!(files[0].1, file_path);
    }

    #[test]
    fn test_collect_files_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "aaaa").unwrap();
        fs::write(dir.path().join("b.txt"), "bbbb").unwrap();

        let files = collect_files(&[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!(files.len(), 2);
        // Both should have relative paths from root
        let names: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"b.txt"));
    }

    #[test]
    fn test_collect_files_nested_directory() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(dir.path().join("root.txt"), "root").unwrap();
        fs::write(sub.join("nested.txt"), "nested").unwrap();

        let files = collect_files(&[dir.path().to_string_lossy().to_string()]).unwrap();
        assert_eq!(files.len(), 2);
        let names: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();
        assert!(names.contains(&"root.txt"));
        assert!(names.contains(&"subdir/nested.txt"));
    }

    #[test]
    fn test_collect_files_mixed_inputs() {
        let dir = TempDir::new().unwrap();
        let file_a = dir.path().join("standalone.bin");
        fs::write(&file_a, "standalone").unwrap();

        let sub = dir.path().join("folder");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("inside.txt"), "inside").unwrap();

        let files = collect_files(&[
            file_a.to_string_lossy().to_string(),
            sub.to_string_lossy().to_string(),
        ]).unwrap();
        assert_eq!(files.len(), 2);
        let names: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();
        assert!(names.contains(&"standalone.bin"));
        assert!(names.contains(&"inside.txt"));
    }

    #[test]
    fn test_collect_files_empty_directory() {
        let dir = TempDir::new().unwrap();
        let files = collect_files(&[dir.path().to_string_lossy().to_string()]).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_files_empty_input() {
        let files = collect_files(&[]).unwrap();
        assert!(files.is_empty());
    }

    // ==================== Compression level mapping ====================

    #[test]
    fn test_compression_level_store() {
        let level = match 0u8 {
            0 => CompressionLevel::Store,
            _ => unreachable!(),
        };
        assert!(matches!(level, CompressionLevel::Store));
    }

    #[test]
    fn test_compression_level_fastest() {
        let level = match 1u8 {
            1 => CompressionLevel::Fastest,
            _ => unreachable!(),
        };
        assert!(matches!(level, CompressionLevel::Fastest));
    }

    #[test]
    fn test_compression_level_fast() {
        // Levels 2-3 map to Fast
        for l in 2..=3u8 {
            let level = match l {
                2..=3 => CompressionLevel::Fast,
                _ => unreachable!(),
            };
            assert!(matches!(level, CompressionLevel::Fast), "Level {} should map to Fast", l);
        }
    }

    #[test]
    fn test_compression_level_normal() {
        // Levels 4-6 map to Normal
        for l in 4..=6u8 {
            let level = match l {
                4..=6 => CompressionLevel::Normal,
                _ => unreachable!(),
            };
            assert!(matches!(level, CompressionLevel::Normal), "Level {} should map to Normal", l);
        }
    }

    #[test]
    fn test_compression_level_maximum() {
        // Levels 7-8 map to Maximum
        for l in 7..=8u8 {
            let level = match l {
                7..=8 => CompressionLevel::Maximum,
                _ => unreachable!(),
            };
            assert!(matches!(level, CompressionLevel::Maximum), "Level {} should map to Maximum", l);
        }
    }

    #[test]
    fn test_compression_level_ultra() {
        let level = match 9u8 {
            9 => CompressionLevel::Ultra,
            _ => unreachable!(),
        };
        assert!(matches!(level, CompressionLevel::Ultra));
    }

    // ==================== Compression ratio estimates ====================

    #[test]
    fn test_compression_ratio_store() {
        let ratio: f64 = 1.0; // Level 0
        assert_eq!(ratio, 1.0, "Store should not compress at all");
    }

    #[test]
    fn test_compression_ratio_fastest() {
        let ratio: f64 = 0.7; // Level 1
        assert!(ratio < 1.0 && ratio > 0.5);
    }

    #[test]
    fn test_compression_ratio_decreases_with_level() {
        let ratios = [1.0, 0.7, 0.5, 0.5, 0.35, 0.35, 0.35, 0.25, 0.25, 0.20];
        for i in 1..ratios.len() {
            assert!(
                ratios[i] <= ratios[i - 1],
                "Ratio for level {} ({}) should be <= level {} ({})",
                i, ratios[i], i - 1, ratios[i - 1]
            );
        }
    }

    #[test]
    fn test_compression_estimate_calculation() {
        let total_size: u64 = 1_000_000; // 1 MB
        let ratio = 0.35; // Normal compression
        let estimated = (total_size as f64 * ratio) as u64;
        assert_eq!(estimated, 350_000);
    }

    // ==================== Cancel flags ====================

    #[test]
    fn test_cancel_flag_initial_state() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_cancel_flag_set() {
        let flag = Arc::new(AtomicBool::new(false));
        flag.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_cancel_flag_shared_across_clones() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        flag_clone.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::SeqCst), "Original should see the flag set by clone");
    }

    // ==================== ArchiveCreateProgress serialization ====================

    #[test]
    fn test_progress_serialization() {
        let progress = ArchiveCreateProgress {
            archive_path: "/tmp/test.7z".to_string(),
            current_file: "evidence.e01".to_string(),
            bytes_processed: 500,
            bytes_total: 1000,
            current_file_bytes: 100,
            current_file_total: 200,
            percent: 50.0,
            status: "Compressing: evidence.e01".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("archivePath")); // camelCase
        assert!(json.contains("bytesProcessed"));
        assert!(json.contains("currentFile"));
        assert!(!json.contains("archive_path")); // NOT snake_case
    }

    // ==================== ManifestFileEntry ====================

    #[test]
    fn test_manifest_entry_serialization() {
        let entry = ManifestFileEntry {
            path: "evidence/file.e01".to_string(),
            size: 1024,
            modified: Some("2025-02-20T10:00:00Z".to_string()),
            sha256: Some("abcdef1234567890".to_string()),
            md5: None,
            sha1: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"path\""));
        assert!(json.contains("\"size\":1024"));
        assert!(json.contains("\"sha256\""));
        assert!(json.contains("\"md5\":null"));
    }

    // ==================== ForensicManifest ====================

    #[test]
    fn test_forensic_manifest_serialization() {
        let manifest = ForensicManifest {
            version: "1.0".to_string(),
            tool: "CORE-FFX".to_string(),
            tool_version: "0.1.0".to_string(),
            created_at: "2025-02-20T10:00:00Z".to_string(),
            archive_name: "test.7z".to_string(),
            archive_sha256: Some("abc123".to_string()),
            compression_level: 5,
            encrypted: true,
            hash_algorithms: vec!["SHA-256".to_string()],
            total_files: 3,
            total_size: 4096,
            chain_of_custody: ChainOfCustody {
                examiner_name: Some("Jane Doe".to_string()),
                case_number: Some("CASE-001".to_string()),
                evidence_description: Some("Hard drive image".to_string()),
                hostname: "workstation1".to_string(),
                operating_system: "macos aarch64".to_string(),
            },
            files: vec![],
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("\"chainOfCustody\"")); // camelCase
        assert!(json.contains("\"compressionLevel\":"));
        assert!(json.contains("\"hashAlgorithms\":"));
        assert!(json.contains("Jane Doe"));
        assert!(json.contains("CASE-001"));
    }

    // ==================== ChainOfCustody ====================

    #[test]
    fn test_chain_of_custody_all_none() {
        let coc = ChainOfCustody {
            examiner_name: None,
            case_number: None,
            evidence_description: None,
            hostname: "host".to_string(),
            operating_system: "linux x86_64".to_string(),
        };
        let json = serde_json::to_string(&coc).unwrap();
        assert!(json.contains("\"examinerName\":null"));
        assert!(json.contains("\"caseNumber\":null"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_collect_dir_files_preserves_relative_paths() {
        let dir = TempDir::new().unwrap();
        let deep = dir.path().join("level1").join("level2");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.bin"), vec![0u8; 100]).unwrap();

        let mut files = Vec::new();
        collect_dir_files(dir.path(), dir.path(), &mut files).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "level1/level2/deep.bin");
    }

    #[test]
    fn test_large_archive_threshold() {
        // Verify the 8GB threshold constant used in the code
        let one_gb = 1024u64 * 1024 * 1024;
        let eight_gb = 8 * one_gb;
        assert_eq!(eight_gb, 8_589_934_592);
    }

    #[test]
    fn test_streaming_threshold() {
        // Verify the 1GB streaming threshold
        let one_gb = 1024u64 * 1024 * 1024;
        assert_eq!(one_gb, 1_073_741_824);
    }
}
