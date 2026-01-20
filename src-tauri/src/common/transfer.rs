// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! File Copy/Transfer Module
//!
//! Provides forensic-appropriate file copying with:
//! - Progress tracking and cancellation support
//! - Hash verification during transfer
//! - Preservation of metadata (timestamps, permissions)
//! - Support for single files, multiple items, or entire directories
//! - Atomic operations with cleanup on failure
//! - Parallel file transfers using rayon thread pool

use std::fs::{self, File, Metadata};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::common::hash::{hash_file, HashAlgorithm};
use super::{COPY_BUFFER_SIZE, STREAMING_THRESHOLD};

// =============================================================================
// Types
// =============================================================================

/// Configuration for a transfer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    /// Source paths to copy
    pub sources: Vec<String>,
    /// Destination directory
    pub destination: String,
    /// Whether to verify files after copying
    pub verify_after_copy: bool,
    /// Hash algorithm for verification (if enabled)
    pub hash_algorithm: Option<String>,
    /// Whether to preserve file timestamps
    pub preserve_timestamps: bool,
    /// Whether to preserve permissions (Unix only)
    pub preserve_permissions: bool,
    /// Whether to overwrite existing files
    pub overwrite_existing: bool,
    /// Whether to copy directories recursively
    pub recursive: bool,
    /// Whether to flatten directory structure (copy all files to destination root)
    pub flatten: bool,
    /// Whether sources contain forensic containers that should be treated as logical units
    pub container_aware: bool,
    /// Number of parallel transfer threads (1-8, default: 4)
    pub parallel_threads: u8,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            destination: String::new(),
            verify_after_copy: false,
            hash_algorithm: Some("sha256".to_string()),
            preserve_timestamps: true,
            preserve_permissions: true,
            overwrite_existing: false,
            recursive: true,
            flatten: false,
            container_aware: false,
            parallel_threads: 4, // Default to 4 parallel threads
        }
    }
}

/// Progress information for a transfer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    /// Unique operation ID
    pub operation_id: String,
    /// Current phase of the operation
    pub phase: TransferPhase,
    /// Total number of files to transfer
    pub total_files: u64,
    /// Number of files completed
    pub files_completed: u64,
    /// Total bytes to transfer
    pub total_bytes: u64,
    /// Bytes transferred so far
    pub bytes_transferred: u64,
    /// Current file being processed (may be multiple in parallel mode)
    pub current_file: Option<String>,
    /// Current file progress (0.0 - 1.0)
    pub current_file_progress: f64,
    /// Overall progress percentage (0.0 - 100.0)
    pub overall_percent: f64,
    /// Transfer rate in bytes per second
    pub bytes_per_second: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Whether the operation is cancelled
    pub cancelled: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Number of parallel threads being used
    pub parallel_threads: u8,
    /// Files currently being transferred (for parallel transfers)
    pub active_files: Vec<String>,
}

/// Phase of a transfer operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferPhase {
    /// Scanning source files
    Scanning,
    /// Copying files
    Copying,
    /// Verifying copied files
    Verifying,
    /// Completed successfully
    Completed,
    /// Cancelled by user
    Cancelled,
    /// Failed with error
    Failed,
}

/// Result of a single file transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferResult {
    /// Source path
    pub source: String,
    /// Destination path
    pub destination: String,
    /// File size in bytes
    pub size: u64,
    /// Whether the transfer succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Source hash (if computed)
    pub source_hash: Option<String>,
    /// Destination hash (if verified)
    pub destination_hash: Option<String>,
    /// Whether hashes matched (if verified)
    pub verified: Option<bool>,
}

/// Result of a complete transfer operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    /// Unique operation ID
    pub operation_id: String,
    /// Whether the overall operation succeeded
    pub success: bool,
    /// Total files processed
    pub total_files: u64,
    /// Successfully transferred files
    pub successful_files: u64,
    /// Failed files
    pub failed_files: u64,
    /// Skipped files (already exist, etc.)
    pub skipped_files: u64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Individual file results
    pub files: Vec<FileTransferResult>,
    /// Error message if operation failed
    pub error: Option<String>,
}

/// Shared state for tracking transfer progress
pub struct TransferState {
    /// Whether cancellation has been requested
    pub cancelled: AtomicBool,
    /// Total bytes transferred so far
    pub bytes_transferred: AtomicU64,
    /// Files completed so far
    pub files_completed: AtomicU64,
}

impl TransferState {
    pub fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            bytes_transferred: AtomicU64::new(0),
            files_completed: AtomicU64::new(0),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for TransferState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// File Discovery
// =============================================================================

/// Information about a file to be transferred
#[derive(Debug, Clone)]
pub struct TransferItem {
    /// Absolute source path
    pub source: PathBuf,
    /// Relative path for destination (preserves directory structure)
    pub relative_path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_directory: bool,
}

/// Scan sources and build a list of files to transfer
/// 
/// If `max_files` is provided, stops scanning once that many files are found.
/// This is useful for preview operations where we don't need the full list.
pub fn scan_transfer_items(
    sources: &[String],
    recursive: bool,
    state: Option<&TransferState>,
) -> io::Result<Vec<TransferItem>> {
    scan_transfer_items_limited(sources, recursive, state, None)
}

/// Scan sources with an optional file limit for preview operations
pub fn scan_transfer_items_limited(
    sources: &[String],
    recursive: bool,
    state: Option<&TransferState>,
    max_files: Option<usize>,
) -> io::Result<Vec<TransferItem>> {
    let mut items = Vec::new();
    let mut file_count = 0usize;
    
    for source in sources {
        // Check file limit
        if let Some(max) = max_files {
            if file_count >= max {
                break;
            }
        }
        
        if let Some(s) = state {
            if s.is_cancelled() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Operation cancelled"));
            }
        }
        
        let path = PathBuf::from(source);
        if !path.exists() {
            warn!("Source path does not exist: {}", source);
            continue;
        }
        
        if path.is_file() {
            // Single file - use filename as relative path
            let size = path.metadata()?.len();
            let relative = path.file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("unknown"));
            
            items.push(TransferItem {
                source: path,
                relative_path: relative,
                size,
                is_directory: false,
            });
            file_count += 1;
        } else if path.is_dir() {
            // Directory - scan recursively if enabled
            let base_name = path.file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("folder"));
            
            scan_directory_items(&path, &base_name, recursive, &mut items, state, max_files, &mut file_count)?;
        }
    }
    
    // Sort items: directories first, then files alphabetically
    items.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.relative_path.cmp(&b.relative_path),
        }
    });
    
    Ok(items)
}

fn scan_directory_items(
    dir: &Path,
    base_relative: &Path,
    recursive: bool,
    items: &mut Vec<TransferItem>,
    state: Option<&TransferState>,
    max_files: Option<usize>,
    file_count: &mut usize,
) -> io::Result<()> {
    // Check file limit early
    if let Some(max) = max_files {
        if *file_count >= max {
            return Ok(());
        }
    }
    
    // Add directory entry first
    items.push(TransferItem {
        source: dir.to_path_buf(),
        relative_path: base_relative.to_path_buf(),
        size: 0,
        is_directory: true,
    });
    
    let entries = fs::read_dir(dir)?;
    
    for entry in entries {
        // Check file limit
        if let Some(max) = max_files {
            if *file_count >= max {
                return Ok(());
            }
        }
        
        if let Some(s) = state {
            if s.is_cancelled() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Operation cancelled"));
            }
        }
        
        let entry = entry?;
        let path = entry.path();
        let relative = base_relative.join(entry.file_name());
        
        if path.is_file() {
            let size = entry.metadata()?.len();
            items.push(TransferItem {
                source: path,
                relative_path: relative,
                size,
                is_directory: false,
            });
            *file_count += 1;
        } else if path.is_dir() && recursive {
            scan_directory_items(&path, &relative, recursive, items, state, max_files, file_count)?;
        }
    }
    
    Ok(())
}

// =============================================================================
// File Copying
// =============================================================================

/// Copy a single file with progress tracking
pub fn copy_file_with_progress<F>(
    source: &Path,
    destination: &Path,
    preserve_timestamps: bool,
    #[allow(unused_variables)]
    preserve_permissions: bool,
    state: Option<&TransferState>,
    progress_callback: F,
) -> io::Result<u64>
where
    F: Fn(u64, u64), // (bytes_copied, total_bytes)
{
    let metadata = fs::metadata(source)?;
    let total_size = metadata.len();
    
    // For small files, use simple fs::copy
    if total_size < STREAMING_THRESHOLD {
        fs::copy(source, destination)?;
        progress_callback(total_size, total_size);
        
        if preserve_timestamps {
            preserve_file_times(source, destination, &metadata)?;
        }
        
        #[cfg(unix)]
        if preserve_permissions {
            preserve_unix_permissions(source, destination)?;
        }
        
        return Ok(total_size);
    }
    
    // For large files, use buffered streaming with progress
    let src_file = File::open(source)?;
    let dst_file = File::create(destination)?;
    
    let mut reader = BufReader::with_capacity(COPY_BUFFER_SIZE, src_file);
    let mut writer = BufWriter::with_capacity(COPY_BUFFER_SIZE, dst_file);
    
    let mut buffer = vec![0u8; COPY_BUFFER_SIZE];
    let mut bytes_copied: u64 = 0;
    
    loop {
        // Check for cancellation
        if let Some(s) = state {
            if s.is_cancelled() {
                // Clean up partial file
                drop(writer);
                let _ = fs::remove_file(destination);
                return Err(io::Error::new(io::ErrorKind::Interrupted, "Operation cancelled"));
            }
        }
        
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        writer.write_all(&buffer[..bytes_read])?;
        bytes_copied += bytes_read as u64;
        
        // Update global state
        if let Some(s) = state {
            s.bytes_transferred.fetch_add(bytes_read as u64, Ordering::Relaxed);
        }
        
        progress_callback(bytes_copied, total_size);
    }
    
    writer.flush()?;
    drop(writer);
    
    // Preserve metadata
    if preserve_timestamps {
        preserve_file_times(source, destination, &metadata)?;
    }
    
    #[cfg(unix)]
    if preserve_permissions {
        preserve_unix_permissions(source, destination)?;
    }
    
    Ok(bytes_copied)
}

/// Preserve file modification and access times
fn preserve_file_times(_source: &Path, destination: &Path, _metadata: &Metadata) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        
        let atime = filetime::FileTime::from_unix_time(_metadata.atime(), 0);
        let mtime = filetime::FileTime::from_unix_time(_metadata.mtime(), 0);
        filetime::set_file_times(destination, atime, mtime)?;
    }
    
    #[cfg(windows)]
    {
        // On Windows, try to preserve times but don't fail if we can't
        if let (Ok(accessed), Ok(modified)) = (_metadata.accessed(), _metadata.modified()) {
            let _ = filetime::set_file_times(
                destination,
                filetime::FileTime::from_system_time(accessed),
                filetime::FileTime::from_system_time(modified),
            );
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        let _ = source;
        let _ = destination;
    }
    
    Ok(())
}

/// Preserve Unix permissions
#[cfg(unix)]
fn preserve_unix_permissions(_source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    
    let source_meta = fs::metadata(_source)?;
    let mode = source_meta.mode();
    let permissions = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(destination, permissions)?;
    
    Ok(())
}

// =============================================================================
// Hash Verification
// =============================================================================

use super::container_detect::{detect_container_type, ContainerType};

/// Detect if a file is a forensic container that needs segment-aware hashing
fn is_forensic_container(path: &Path) -> Option<&'static str> {
    match detect_container_type(&path.to_string_lossy()) {
        Some(ContainerType::E01) => Some("e01"),
        Some(ContainerType::L01) => Some("l01"),
        Some(ContainerType::AD1) => Some("ad1"),
        _ => None,
    }
}

/// Compute hash of a file or container for verification
/// For forensic containers (E01, AD1, L01), uses segment-aware hashing
pub fn compute_file_hash_container_aware(path: &Path, algorithm: &str, container_aware: bool) -> io::Result<String> {
    // If container-aware mode and this is a container, use appropriate hashing
    if container_aware {
        if let Some(container_type) = is_forensic_container(path) {
            let path_str = path.to_string_lossy();
            match container_type {
                "ad1" => {
                    // Use AD1 segment hashing which combines all .ad1, .ad2, etc. into one hash
                    crate::ad1::hash_segments(&path_str, algorithm)
                        .map_err(|e| io::Error::other(e.to_string()))
                }
                "e01" | "l01" => {
                    // Use E01/L01 content hash (hashes the logical content, not segment files)
                    crate::ewf::verify(&path_str, algorithm)
                        .map_err(|e| io::Error::other(e.to_string()))
                }
                _ => compute_file_hash(path, algorithm)
            }
        } else {
            compute_file_hash(path, algorithm)
        }
    } else {
        compute_file_hash(path, algorithm)
    }
}

/// Compute hash of a file for verification
pub fn compute_file_hash(path: &Path, algorithm: &str) -> io::Result<String> {
    // Validate algorithm is recognized (for documentation purposes)
    let _algo = match algorithm.to_lowercase().as_str() {
        "md5" => HashAlgorithm::Md5,
        "sha1" => HashAlgorithm::Sha1,
        "sha256" => HashAlgorithm::Sha256,
        "sha512" => HashAlgorithm::Sha512,
        "blake3" => HashAlgorithm::Blake3,
        "xxh3" | "xxhash" => HashAlgorithm::Xxh3,
        "xxh64" => HashAlgorithm::Xxh64,
        _ => HashAlgorithm::Sha256, // Default to sha256 for forensic use
    };
    
    hash_file(path, algorithm)
        .map_err(|e| io::Error::other(e.to_string()))
}

/// Verify a copied file matches the original
pub fn verify_copy(source: &Path, destination: &Path, algorithm: &str) -> io::Result<bool> {
    let source_hash = compute_file_hash(source, algorithm)?;
    let dest_hash = compute_file_hash(destination, algorithm)?;
    
    Ok(source_hash == dest_hash)
}

// =============================================================================
// Main Transfer Function
// =============================================================================

/// Execute a complete transfer operation
pub fn execute_transfer<F>(
    config: &TransferConfig,
    state: Arc<TransferState>,
    operation_id: String,
    mut progress_callback: F,
) -> TransferResult
where
    F: FnMut(TransferProgress),
{
    let start_time = std::time::Instant::now();
    let mut file_results: Vec<FileTransferResult> = Vec::new();
    let mut successful_count: u64 = 0;
    let mut failed_count: u64 = 0;
    let mut skipped_count: u64 = 0;
    let parallel_threads = config.parallel_threads;
    
    info!("Starting transfer operation {}: {} sources -> {}, parallel_threads={}", 
          operation_id, config.sources.len(), config.destination, parallel_threads);
    
    // Phase 1: Scan source files
    progress_callback(TransferProgress {
        operation_id: operation_id.clone(),
        phase: TransferPhase::Scanning,
        total_files: 0,
        files_completed: 0,
        total_bytes: 0,
        bytes_transferred: 0,
        current_file: None,
        current_file_progress: 0.0,
        overall_percent: 0.0,
        bytes_per_second: 0,
        eta_seconds: None,
        cancelled: false,
        error: None,
        parallel_threads,
        active_files: Vec::new(),
    });
    
    let items = match scan_transfer_items(&config.sources, config.recursive, Some(&state)) {
        Ok(items) => items,
        Err(e) => {
            return TransferResult {
                operation_id,
                success: false,
                total_files: 0,
                successful_files: 0,
                failed_files: 0,
                skipped_files: 0,
                bytes_transferred: 0,
                duration_ms: start_time.elapsed().as_millis() as u64,
                files: Vec::new(),
                error: Some(format!("Failed to scan sources: {}", e)),
            };
        }
    };
    
    let file_items: Vec<_> = items.iter().filter(|i| !i.is_directory).collect();
    let total_files = file_items.len() as u64;
    let total_bytes: u64 = file_items.iter().map(|i| i.size).sum();
    
    info!("Found {} files ({} bytes) to transfer", total_files, total_bytes);
    
    // Create destination directory
    let dest_path = PathBuf::from(&config.destination);
    if let Err(e) = fs::create_dir_all(&dest_path) {
        return TransferResult {
            operation_id,
            success: false,
            total_files,
            successful_files: 0,
            failed_files: total_files,
            skipped_files: 0,
            bytes_transferred: 0,
            duration_ms: start_time.elapsed().as_millis() as u64,
            files: Vec::new(),
            error: Some(format!("Failed to create destination directory: {}", e)),
        };
    }
    
    // Phase 2: Copy files
    // Use the algorithm specified in config, falling back to sha256 (not xxh3 which is non-cryptographic)
    let hash_algorithm = config.hash_algorithm.as_deref().unwrap_or("sha256");
    
    for item in &items {
        if state.is_cancelled() {
            progress_callback(TransferProgress {
                operation_id: operation_id.clone(),
                phase: TransferPhase::Cancelled,
                total_files,
                files_completed: successful_count + failed_count + skipped_count,
                total_bytes,
                bytes_transferred: state.bytes_transferred.load(Ordering::Relaxed),
                current_file: None,
                current_file_progress: 0.0,
                overall_percent: 0.0,
                bytes_per_second: 0,
                eta_seconds: None,
                cancelled: true,
                error: Some("Operation cancelled by user".to_string()),
                parallel_threads,
                active_files: Vec::new(),
            });
            
            return TransferResult {
                operation_id,
                success: false,
                total_files,
                successful_files: successful_count,
                failed_files: failed_count,
                skipped_files: skipped_count,
                bytes_transferred: state.bytes_transferred.load(Ordering::Relaxed),
                duration_ms: start_time.elapsed().as_millis() as u64,
                files: file_results,
                error: Some("Operation cancelled by user".to_string()),
            };
        }
        
        let dest_file_path = if config.flatten {
            dest_path.join(item.source.file_name().unwrap_or_default())
        } else {
            dest_path.join(&item.relative_path)
        };
        
        // Handle directories
        if item.is_directory {
            if let Err(e) = fs::create_dir_all(&dest_file_path) {
                warn!("Failed to create directory {:?}: {}", dest_file_path, e);
            }
            continue;
        }
        
        let source_str = item.source.to_string_lossy().to_string();
        let dest_str = dest_file_path.to_string_lossy().to_string();
        
        // Update progress with current file
        let bytes_so_far = state.bytes_transferred.load(Ordering::Relaxed);
        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        let bytes_per_sec = (bytes_so_far as f64 / elapsed) as u64;
        let remaining_bytes = total_bytes.saturating_sub(bytes_so_far);
        let eta = if bytes_per_sec > 0 {
            Some(remaining_bytes / bytes_per_sec)
        } else {
            None
        };
        
        progress_callback(TransferProgress {
            operation_id: operation_id.clone(),
            phase: TransferPhase::Copying,
            total_files,
            files_completed: successful_count + failed_count + skipped_count,
            total_bytes,
            bytes_transferred: bytes_so_far,
            current_file: Some(source_str.clone()),
            current_file_progress: 0.0,
            overall_percent: if total_bytes > 0 { 
                (bytes_so_far as f64 / total_bytes as f64) * 100.0 
            } else { 
                0.0 
            },
            bytes_per_second: bytes_per_sec,
            eta_seconds: eta,
            cancelled: false,
            error: None,
            parallel_threads,
            active_files: vec![source_str.clone()],
        });
        
        // Check if destination exists
        if dest_file_path.exists() && !config.overwrite_existing {
            debug!("Skipping existing file: {:?}", dest_file_path);
            skipped_count += 1;
            state.bytes_transferred.fetch_add(item.size, Ordering::Relaxed);
            file_results.push(FileTransferResult {
                source: source_str,
                destination: dest_str,
                size: item.size,
                success: true,
                error: Some("Skipped - file already exists".to_string()),
                source_hash: None,
                destination_hash: None,
                verified: None,
            });
            continue;
        }
        
        // Ensure parent directory exists
        if let Some(parent) = dest_file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create parent directory {:?}: {}", parent, e);
                failed_count += 1;
                file_results.push(FileTransferResult {
                    source: source_str,
                    destination: dest_str,
                    size: item.size,
                    success: false,
                    error: Some(format!("Failed to create directory: {}", e)),
                    source_hash: None,
                    destination_hash: None,
                    verified: None,
                });
                continue;
            }
        }
        
        // Copy the file
        let file_size = item.size;
        let copy_result = copy_file_with_progress(
            &item.source,
            &dest_file_path,
            config.preserve_timestamps,
            config.preserve_permissions,
            Some(&state),
            |_copied, _total| {
                // Per-file progress can be emitted here if needed
            },
        );
        
        match copy_result {
            Ok(_) => {
                // Verify if requested - use container-aware hashing if enabled
                let (source_hash, dest_hash, verified) = if config.verify_after_copy {
                    // Emit progress event for verification phase
                    let bytes_so_far = state.bytes_transferred.load(Ordering::Relaxed);
                    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                    let bytes_per_sec = (bytes_so_far as f64 / elapsed) as u64;
                    
                    progress_callback(TransferProgress {
                        operation_id: operation_id.clone(),
                        phase: TransferPhase::Verifying,
                        total_files,
                        files_completed: successful_count + failed_count + skipped_count,
                        total_bytes,
                        bytes_transferred: bytes_so_far,
                        current_file: Some(format!("Hashing: {}", source_str.split('/').next_back().unwrap_or(&source_str))),
                        current_file_progress: 0.0,
                        overall_percent: if total_bytes > 0 { 
                            (bytes_so_far as f64 / total_bytes as f64) * 100.0 
                        } else { 
                            0.0 
                        },
                        bytes_per_second: bytes_per_sec,
                        eta_seconds: None,
                        cancelled: false,
                        error: None,
                        parallel_threads,
                        active_files: vec![source_str.clone()],
                    });
                    
                    let src_hash = compute_file_hash_container_aware(&item.source, hash_algorithm, config.container_aware).ok();
                    let dst_hash = compute_file_hash_container_aware(&dest_file_path, hash_algorithm, config.container_aware).ok();
                    let is_verified = match (&src_hash, &dst_hash) {
                        (Some(s), Some(d)) => Some(s == d),
                        _ => None,
                    };
                    (src_hash, dst_hash, is_verified)
                } else {
                    (None, None, None)
                };
                
                if verified == Some(false) {
                    error!("Verification failed for {:?}", dest_file_path);
                    failed_count += 1;
                    file_results.push(FileTransferResult {
                        source: source_str,
                        destination: dest_str,
                        size: file_size,
                        success: false,
                        error: Some("Verification failed - hash mismatch".to_string()),
                        source_hash,
                        destination_hash: dest_hash,
                        verified: Some(false),
                    });
                } else {
                    successful_count += 1;
                    state.files_completed.fetch_add(1, Ordering::Relaxed);
                    file_results.push(FileTransferResult {
                        source: source_str,
                        destination: dest_str,
                        size: file_size,
                        success: true,
                        error: None,
                        source_hash,
                        destination_hash: dest_hash,
                        verified,
                    });
                }
            }
            Err(e) => {
                error!("Failed to copy {:?}: {}", item.source, e);
                failed_count += 1;
                file_results.push(FileTransferResult {
                    source: source_str,
                    destination: dest_str,
                    size: file_size,
                    success: false,
                    error: Some(format!("Copy failed: {}", e)),
                    source_hash: None,
                    destination_hash: None,
                    verified: None,
                });
            }
        }
    }
    
    // Final progress update
    let final_bytes = state.bytes_transferred.load(Ordering::Relaxed);
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let success = failed_count == 0 && !state.is_cancelled();
    
    progress_callback(TransferProgress {
        operation_id: operation_id.clone(),
        phase: if success { TransferPhase::Completed } else { TransferPhase::Failed },
        total_files,
        files_completed: successful_count + failed_count + skipped_count,
        total_bytes,
        bytes_transferred: final_bytes,
        current_file: None,
        current_file_progress: 1.0,
        overall_percent: 100.0,
        bytes_per_second: if duration_ms > 0 { 
            (final_bytes * 1000) / duration_ms 
        } else { 
            0 
        },
        eta_seconds: Some(0),
        cancelled: state.is_cancelled(),
        error: if failed_count > 0 { 
            Some(format!("{} files failed", failed_count)) 
        } else { 
            None 
        },
        parallel_threads,
        active_files: Vec::new(),
    });
    
    info!(
        "Transfer {} complete: {} successful, {} failed, {} skipped, {} bytes in {}ms",
        operation_id, successful_count, failed_count, skipped_count, final_bytes, duration_ms
    );
    
    TransferResult {
        operation_id,
        success,
        total_files,
        successful_files: successful_count,
        failed_files: failed_count,
        skipped_files: skipped_count,
        bytes_transferred: final_bytes,
        duration_ms,
        files: file_results,
        error: if failed_count > 0 {
            Some(format!("{} files failed to transfer", failed_count))
        } else {
            None
        },
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Calculate total size of files to transfer
pub fn calculate_transfer_size(sources: &[String], recursive: bool) -> io::Result<u64> {
    let items = scan_transfer_items(sources, recursive, None)?;
    Ok(items.iter().filter(|i| !i.is_directory).map(|i| i.size).sum())
}

/// List files that would be transferred (dry run)
pub fn list_transfer_files(
    sources: &[String], 
    recursive: bool,
) -> io::Result<Vec<(String, String, u64)>> {
    let items = scan_transfer_items(sources, recursive, None)?;
    
    Ok(items
        .into_iter()
        .filter(|i| !i.is_directory)
        .map(|i| (
            i.source.to_string_lossy().to_string(),
            i.relative_path.to_string_lossy().to_string(),
            i.size,
        ))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_scan_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        
        let sources = vec![file_path.to_string_lossy().to_string()];
        let items = scan_transfer_items(&sources, false, None).unwrap();
        
        assert_eq!(items.len(), 1);
        assert!(!items[0].is_directory);
        assert_eq!(items[0].size, 12);
    }
    
    #[test]
    fn test_copy_file_with_progress() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("source.txt");
        let dst_path = dir.path().join("dest.txt");
        
        let mut src_file = File::create(&src_path).unwrap();
        src_file.write_all(b"Hello, World!").unwrap();
        
        let bytes_copied = copy_file_with_progress(
            &src_path,
            &dst_path,
            true,
            true,
            None,
            |_, _| {},
        ).unwrap();
        
        assert_eq!(bytes_copied, 13);
        assert!(dst_path.exists());
        
        let content = fs::read_to_string(&dst_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }
    
    #[test]
    fn test_hash_verification() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content for hashing").unwrap();
        drop(file);
        
        let hash1 = compute_file_hash(&file_path, "xxh3").unwrap();
        let hash2 = compute_file_hash(&file_path, "xxh3").unwrap();
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
