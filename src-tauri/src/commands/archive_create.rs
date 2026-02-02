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
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

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
}

impl Default for CreateArchiveOptions {
    fn default() -> Self {
        Self {
            compression_level: 5, // Normal
            password: None,
            num_threads: Some(2),
            dict_size_mb: None,
            solid: Some(true),
            split_size_mb: None,
            chunk_size_mb: Some(64),
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
    
    // Spawn blocking task for compression
    let window_clone = window.clone();
    let archive_path_clone = archive_path.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
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
                let dir_size = calculate_dir_size(&path_obj).unwrap_or(0);
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
            
            sz.create_archive_streaming(
                &working_path,
                &input_paths_vec,
                compression_level,
                Some(&stream_opts),
                Some(Box::new(move |bytes_processed, bytes_total, current_file_bytes, 
                                     current_file_total, current_file_name| {
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
    .map_err(|e| format!("Task failed: {}", e))?
}

/// Test archive integrity after creation
///
/// # Arguments
/// * `archive_path` - Path to archive to test
/// * `password` - Optional password if encrypted
/// * `window` - Tauri window for progress events
///
/// # Returns
/// * `Ok(true)` - Archive is valid
/// * `Err(message)` - Archive is corrupted or invalid
#[tauri::command]
pub async fn test_7z_archive(
    archive_path: String,
    password: Option<String>,
    window: Window,
) -> Result<bool, String> {
    info!("Testing 7z archive integrity: {}", archive_path);
    
    let window_clone = window.clone();
    
    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new().map_err(|e| format!("Failed to initialize 7z library: {}", e))?;
        
        // Emit starting status
        let _ = window_clone.emit("archive-test-progress", ArchiveCreateProgress {
            archive_path: archive_path.clone(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 0.0,
            status: "Testing archive integrity...".to_string(),
        });
        
        // Test archive (no progress callback support in current wrapper)
        sz.test_archive(
            &archive_path,
            password.as_deref(),
        ).map_err(|e| format!("Archive test failed: {}", e))?;
        
        // Emit completion status
        let _ = window_clone.emit("archive-test-progress", ArchiveCreateProgress {
            archive_path: archive_path.clone(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 100.0,
            status: "Archive is valid".to_string(),
        });
        
        info!("Archive test passed: {}", archive_path);
        Ok(true)
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

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
/// Note: This is a placeholder for future implementation.
/// Currently, archive creation cannot be cancelled mid-operation.
#[tauri::command]
pub async fn cancel_archive_creation(
    archive_path: String,
) -> Result<(), String> {
    warn!("Archive creation cancellation requested for: {} (not yet implemented)", archive_path);
    Err("Archive creation cancellation not yet implemented".to_string())
}
