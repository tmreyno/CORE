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

use libewf_ffi::{
    EwfCaseInfo, EwfCompression, EwfCompressionMethod, EwfFormat, EwfWriter, EwfWriterConfig,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::collections::HashMap;
use tauri::{Emitter, Window};
use tracing::{debug, info, warn};

// =============================================================================
// Types
// =============================================================================

/// Progress event emitted during E01 export
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfExportProgress {
    /// Output path
    pub output_path: String,
    /// Current file being processed (for multi-file exports)
    pub current_file: String,
    /// File index (1-based)
    pub file_index: usize,
    /// Total number of files
    pub total_files: usize,
    /// Bytes written so far
    pub bytes_written: u64,
    /// Total bytes to write
    pub total_bytes: u64,
    /// Progress percentage (0.0 - 100.0)
    pub percent: f64,
    /// Current phase description
    pub phase: String,
}

/// Result of an E01 export operation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfExportResult {
    /// Path to the created E01/L01 file
    pub output_path: String,
    /// Format used (e.g., "E01", "L01")
    pub format: String,
    /// Total bytes written to the container
    pub bytes_written: u64,
    /// Number of files included
    pub files_included: usize,
    /// Whether compression was used
    pub compressed: bool,
    /// MD5 hash of the data (if computed)
    pub md5_hash: Option<String>,
    /// SHA1 hash of the data (if computed)
    pub sha1_hash: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Options for creating an E01 export
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfExportOptions {
    /// Source files to include in the E01
    pub source_paths: Vec<String>,
    /// Output path (base name, extension added automatically)
    pub output_path: String,
    /// Output format: "e01" (default), "encase5", "encase6", "encase7",
    /// "v2encase7"/"ex01" (EWF2), "ftk"
    pub format: Option<String>,
    /// Compression level: "none", "fast" (default), "best"
    pub compression: Option<String>,
    /// Compression method: "deflate" (default), "bzip2" (requires V2 format)
    pub compression_method: Option<String>,
    /// Maximum segment file size in bytes
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
    /// Whether to compute MD5 hash
    pub compute_md5: Option<bool>,
    /// Whether to compute SHA1 hash
    pub compute_sha1: Option<bool>,
}

/// Global cancel flags for active E01 export jobs
static EWF_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// =============================================================================
// Helper functions
// =============================================================================

fn parse_format(format: &str) -> Result<EwfFormat, String> {
    match format.to_lowercase().as_str() {
        "e01" | "encase5" => Ok(EwfFormat::Encase5),
        "encase6" => Ok(EwfFormat::Encase6),
        // Encase7 (0x07) produces .E01 (EWF1 segment type)
        "encase7" => Ok(EwfFormat::Encase7),
        // V2Encase7 (0x37) produces .Ex01 (EWF2 segment type) — supports BZIP2
        "v2encase7" | "ex01" => Ok(EwfFormat::V2Encase7),
        "ftk" => Ok(EwfFormat::FtkImager),
        // Logical formats (L01, Lx01) are NOT supported for writing by libewf 20251220
        "l01" | "logical" | "logical_encase5" | "l01v6" | "logical_encase6"
        | "l01v7" | "logical_encase7" | "lx01" | "v2logical_encase7" => {
            Err("Logical EWF formats (L01/Lx01) are not supported for writing by libewf. Use a physical image format (E01/Ex01) instead.".to_string())
        }
        _ => Err(format!("Unknown EWF format: {}", format)),
    }
}

fn parse_compression(compression: &str) -> Result<EwfCompression, String> {
    match compression.to_lowercase().as_str() {
        "none" | "store" => Ok(EwfCompression::None),
        "fast" => Ok(EwfCompression::Fast),
        "best" | "maximum" => Ok(EwfCompression::Best),
        _ => Err(format!("Unknown compression level: {}", compression)),
    }
}

fn parse_compression_method(method: &str) -> Result<EwfCompressionMethod, String> {
    match method.to_lowercase().as_str() {
        "deflate" | "zlib" => Ok(EwfCompressionMethod::Deflate),
        "bzip2" | "bz2" => Ok(EwfCompressionMethod::Bzip2),
        "none" => Ok(EwfCompressionMethod::None),
        _ => Err(format!("Unknown compression method: {}", method)),
    }
}

/// Check if a canonicalized path is the system boot volume.
/// Cross-platform: detects macOS root, Windows C:\ drive, and Linux root.
fn is_system_boot_volume(canon: &Path) -> bool {
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
        if upper == "C:\\" || upper == "C:" || upper.starts_with("C:\\") && canon.parent().is_none() {
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

/// Result of a quick statvfs / disk-space query.
struct DiskSpaceInfo {
    available_space: u64,
}

/// Query the available space on the filesystem containing `path`.
fn nix_stat(path: &Path) -> Result<DiskSpaceInfo, String> {
    // Use std::fs metadata approach — works cross-platform.
    // On Unix we can use statvfs for accuracy.
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|e| format!("Invalid path: {e}"))?;
        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                let avail = stat.f_bavail as u64 * stat.f_frsize as u64;
                return Ok(DiskSpaceInfo { available_space: avail });
            }
        }
        Err("statvfs failed".into())
    }
    #[cfg(not(unix))]
    {
        // Fallback: sysinfo crate disk info (less precise but works)
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for d in disks.iter() {
            if path.starts_with(d.mount_point()) {
                return Ok(DiskSpaceInfo { available_space: d.available_space() });
            }
        }
        Err("Could not determine available space".into())
    }
}

/// Format a byte count as a human-readable string (e.g. "12.3 GB").
fn format_byte_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let exp = (bytes as f64).log(1024.0).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    let value = bytes as f64 / 1024_f64.powi(exp as i32);
    if exp == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

/// Recursively walk a directory and collect all files with their sizes.
/// Returned paths are absolute. Skips symlinks and unreadable entries.
fn walk_dir_files(dir: &Path) -> Result<Vec<(String, u64)>, String> {
    let mut results = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;
    for entry in entries {
        let entry = entry
            .map_err(|e| format!("Failed to read entry in {}: {}", dir.display(), e))?;
        let path = entry.path();
        let ft = entry.file_type()
            .map_err(|e| format!("Failed to get file type for {}: {}", path.display(), e))?;
        if ft.is_file() {
            let size = entry.metadata()
                .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?
                .len();
            results.push((path.to_string_lossy().into_owned(), size));
        } else if ft.is_dir() {
            let sub = walk_dir_files(&path)?;
            results.extend(sub);
        }
        // Skip symlinks and other special entries
    }
    Ok(results)
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
    {
        let mut flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.insert(options.output_path.clone(), cancel_flag.clone());
    }

    // --- Safety validations ---

    // Refuse to image the running system's boot volume
    for path_str in &options.source_paths {
        let canon = std::fs::canonicalize(path_str).unwrap_or_else(|_| Path::new(path_str).to_path_buf());
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
    let output_canon = std::fs::canonicalize(output_dir)
        .unwrap_or_else(|_| output_dir.to_path_buf());
    for path_str in &options.source_paths {
        let source_canon = std::fs::canonicalize(path_str)
            .unwrap_or_else(|_| Path::new(path_str).to_path_buf());
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
                total_bytes += fsize;
                file_sizes.push((fpath, fsize));
            }
            info!("Expanded directory {} into {} files", path_str, file_sizes.len());
        } else {
            let metadata = std::fs::metadata(path).map_err(|e| {
                format!("Failed to read metadata for {}: {}", path_str, e)
            })?;
            let size = metadata.len();
            total_bytes += size;
            file_sizes.push((path_str.clone(), size));
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
    let _ = window.emit(
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
    );

    // Build config
    let config = EwfWriterConfig {
        format,
        compression,
        compression_method,
        segment_size: options.segment_size.unwrap_or(libewf_ffi::ffi::LIBEWF_DEFAULT_SEGMENT_FILE_SIZE),
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
    let mut writer = EwfWriter::create(&options.output_path, config)
        .map_err(|e| format!("Failed to create EWF writer: {}", e))?;

    // Write source files
    let compute_md5 = options.compute_md5.unwrap_or(true);
    let compute_sha1 = options.compute_sha1.unwrap_or(false);
    let mut global_bytes_written: u64 = 0;
    let chunk_size = 64 * 1024; // 64 KB read chunks

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

    for (file_idx, (path_str, _file_size)) in file_sizes.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            warn!("EWF export cancelled");
            return Err("Export cancelled".to_string());
        }

        let filename = Path::new(path_str)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        debug!("Writing file {}/{}: {}", file_idx + 1, file_sizes.len(), filename);

        // Emit file start progress
        let _ = window.emit(
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
        );

        // Read and write file in chunks
        let mut file = std::fs::File::open(path_str)
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
        let mut buf = vec![0u8; chunk_size];

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                warn!("EWF export cancelled during write");
                return Err("Export cancelled".to_string());
            }

            use std::io::Read;
            let bytes_read = file
                .read(&mut buf)
                .map_err(|e| format!("Failed to read {}: {}", path_str, e))?;
            if bytes_read == 0 {
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

            global_bytes_written += bytes_read as u64;

            // Emit progress every 1 MB
            if global_bytes_written % (1024 * 1024) < chunk_size as u64 {
                let _ = window.emit(
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
                );
            }
        }
    }

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
    let _ = window.emit(
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
    );

    writer
        .finalize()
        .map_err(|e| format!("Failed to finalize EWF container: {}", e))?;

    // Clean up cancel flag
    {
        let mut flags = EWF_CANCEL_FLAGS.lock().map_err(|e| e.to_string())?;
        flags.remove(&options.output_path);
    }

    let duration = start.elapsed();
    let format_str = format.extension().trim_start_matches('.').to_string();

    // Emit completion
    let _ = window.emit(
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
    );

    info!(
        "E01 export complete: {} bytes written in {:.1}s",
        global_bytes_written,
        duration.as_secs_f64()
    );

    Ok(EwfExportResult {
        output_path: format!(
            "{}{}",
            options.output_path,
            format.extension()
        ),
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

// =============================================================================
// EWF Reader — Image Info Extraction
// =============================================================================

/// Serializable case metadata from an EWF container
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfReadCaseInfoResponse {
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquiry_software_version: Option<String>,
    pub acquiry_date: Option<String>,
    pub acquiry_operating_system: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
}

/// Serializable image metadata from an EWF container
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EwfImageInfoResponse {
    /// Detected format name (e.g., "EnCase 5", "EnCase 7 V2")
    pub format: String,
    /// File extension for this format (e.g., ".E01", ".Ex01")
    pub format_extension: String,
    /// Whether this is a logical evidence format
    pub is_logical: bool,
    /// Whether this is a V2 (EWF2) format
    pub is_v2: bool,
    /// Total media size in bytes
    pub media_size: u64,
    /// Bytes per sector
    pub bytes_per_sector: u32,
    /// Sectors per chunk
    pub sectors_per_chunk: u32,
    /// Compression level (-1=default, 0=none, 1=fast, 2=best)
    pub compression_level: i8,
    /// Compression method name (e.g., "Deflate", "BZIP2")
    pub compression_method: String,
    /// Media type constant
    pub media_type: u8,
    /// Media flags
    pub media_flags: u8,
    /// Segment file version (e.g., "1.0", "2.0")
    pub segment_file_version: Option<String>,
    /// Whether any segment files are corrupted
    pub is_corrupted: bool,
    /// Whether the image is encrypted
    pub is_encrypted: bool,
    /// Case/evidence metadata
    pub case_info: EwfReadCaseInfoResponse,
    /// Stored MD5 hash (hex string, if present)
    pub md5_hash: Option<String>,
    /// Stored SHA1 hash (hex string, if present)
    pub sha1_hash: Option<String>,
}

/// Read detailed image metadata from an E01/Ex01/L01/Lx01 container using libewf
///
/// Opens the EWF container (auto-discovers all segment files) and extracts
/// format info, case metadata, stored hashes, and image parameters. This uses
/// the libewf C library (via libewf-ffi) for comprehensive format support,
/// complementing the pure-Rust EWF parser used for tree browsing.
#[tauri::command]
pub async fn ewf_read_image_info(path: String) -> Result<EwfImageInfoResponse, String> {
    info!("Reading EWF image info via libewf: {}", path);

    // EwfReader::open is not Send — run on blocking thread
    let result = tokio::task::spawn_blocking(move || {
        let reader = libewf_ffi::EwfReader::open(&path)
            .map_err(|e| format!("Failed to open EWF container: {}", e))?;

        let info = reader
            .image_info()
            .map_err(|e| format!("Failed to read image info: {}", e))?;

        let case = &info.case_info;

        Ok::<EwfImageInfoResponse, String>(EwfImageInfoResponse {
            format: info.format.name().to_string(),
            format_extension: info.format.extension().to_string(),
            is_logical: info.format.is_logical(),
            is_v2: info.format.is_v2(),
            media_size: info.media_size,
            bytes_per_sector: info.bytes_per_sector,
            sectors_per_chunk: info.sectors_per_chunk,
            compression_level: info.compression_level,
            compression_method: info.compression_method.name().to_string(),
            media_type: info.media_type,
            media_flags: info.media_flags,
            segment_file_version: info
                .segment_file_version
                .map(|(major, minor)| format!("{}.{}", major, minor)),
            is_corrupted: info.is_corrupted,
            is_encrypted: info.is_encrypted,
            case_info: EwfReadCaseInfoResponse {
                case_number: case.case_number.clone(),
                evidence_number: case.evidence_number.clone(),
                examiner_name: case.examiner_name.clone(),
                description: case.description.clone(),
                notes: case.notes.clone(),
                acquiry_software_version: case.acquiry_software_version.clone(),
                acquiry_date: case.acquiry_date.clone(),
                acquiry_operating_system: case.acquiry_operating_system.clone(),
                model: case.model.clone(),
                serial_number: case.serial_number.clone(),
            },
            md5_hash: info.md5_hash,
            sha1_hash: info.sha1_hash,
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse_format ====================

    #[test]
    fn test_parse_format_e01() {
        let result = parse_format("e01").unwrap();
        assert!(matches!(result, EwfFormat::Encase5));
    }

    #[test]
    fn test_parse_format_encase5() {
        let result = parse_format("encase5").unwrap();
        assert!(matches!(result, EwfFormat::Encase5));
    }

    #[test]
    fn test_parse_format_encase6() {
        let result = parse_format("encase6").unwrap();
        assert!(matches!(result, EwfFormat::Encase6));
    }

    #[test]
    fn test_parse_format_encase7() {
        let result = parse_format("encase7").unwrap();
        assert!(matches!(result, EwfFormat::Encase7));
    }

    #[test]
    fn test_parse_format_v2encase7() {
        let result = parse_format("v2encase7").unwrap();
        assert!(matches!(result, EwfFormat::V2Encase7));
    }

    #[test]
    fn test_parse_format_ex01() {
        let result = parse_format("ex01").unwrap();
        assert!(matches!(result, EwfFormat::V2Encase7));
    }

    #[test]
    fn test_parse_format_l01_rejected() {
        let result = parse_format("l01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
    }

    #[test]
    fn test_parse_format_logical_rejected() {
        assert!(parse_format("logical").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase5_rejected() {
        assert!(parse_format("logical_encase5").is_err());
    }

    #[test]
    fn test_parse_format_l01v6_rejected() {
        assert!(parse_format("l01v6").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase6_rejected() {
        assert!(parse_format("logical_encase6").is_err());
    }

    #[test]
    fn test_parse_format_l01v7_rejected() {
        assert!(parse_format("l01v7").is_err());
    }

    #[test]
    fn test_parse_format_logical_encase7_rejected() {
        assert!(parse_format("logical_encase7").is_err());
    }

    #[test]
    fn test_parse_format_lx01_rejected() {
        assert!(parse_format("lx01").is_err());
    }

    #[test]
    fn test_parse_format_v2logical_encase7_rejected() {
        assert!(parse_format("v2logical_encase7").is_err());
    }

    #[test]
    fn test_parse_format_ftk() {
        let result = parse_format("ftk").unwrap();
        assert!(matches!(result, EwfFormat::FtkImager));
    }

    #[test]
    fn test_parse_format_case_insensitive() {
        assert!(matches!(parse_format("E01").unwrap(), EwfFormat::Encase5));
        assert!(matches!(parse_format("ENCASE7").unwrap(), EwfFormat::Encase7));
        assert!(matches!(parse_format("V2Encase7").unwrap(), EwfFormat::V2Encase7));
        assert!(matches!(parse_format("FTK").unwrap(), EwfFormat::FtkImager));
    }

    #[test]
    fn test_parse_format_unknown() {
        let result = parse_format("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown EWF format"));
    }

    #[test]
    fn test_parse_format_empty() {
        let result = parse_format("");
        assert!(result.is_err());
    }

    // ==================== parse_compression ====================

    #[test]
    fn test_parse_compression_none() {
        let result = parse_compression("none").unwrap();
        assert!(matches!(result, EwfCompression::None));
    }

    #[test]
    fn test_parse_compression_store() {
        let result = parse_compression("store").unwrap();
        assert!(matches!(result, EwfCompression::None));
    }

    #[test]
    fn test_parse_compression_fast() {
        let result = parse_compression("fast").unwrap();
        assert!(matches!(result, EwfCompression::Fast));
    }

    #[test]
    fn test_parse_compression_best() {
        let result = parse_compression("best").unwrap();
        assert!(matches!(result, EwfCompression::Best));
    }

    #[test]
    fn test_parse_compression_maximum() {
        let result = parse_compression("maximum").unwrap();
        assert!(matches!(result, EwfCompression::Best));
    }

    #[test]
    fn test_parse_compression_case_insensitive() {
        assert!(matches!(parse_compression("NONE").unwrap(), EwfCompression::None));
        assert!(matches!(parse_compression("Fast").unwrap(), EwfCompression::Fast));
        assert!(matches!(parse_compression("BEST").unwrap(), EwfCompression::Best));
    }

    #[test]
    fn test_parse_compression_unknown() {
        let result = parse_compression("turbo");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown compression level"));
    }

    // ==================== parse_compression_method ====================

    #[test]
    fn test_parse_compression_method_deflate() {
        let result = parse_compression_method("deflate").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Deflate));
    }

    #[test]
    fn test_parse_compression_method_zlib() {
        let result = parse_compression_method("zlib").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Deflate));
    }

    #[test]
    fn test_parse_compression_method_bzip2() {
        let result = parse_compression_method("bzip2").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Bzip2));
    }

    #[test]
    fn test_parse_compression_method_bz2() {
        let result = parse_compression_method("bz2").unwrap();
        assert!(matches!(result, EwfCompressionMethod::Bzip2));
    }

    #[test]
    fn test_parse_compression_method_none() {
        let result = parse_compression_method("none").unwrap();
        assert!(matches!(result, EwfCompressionMethod::None));
    }

    #[test]
    fn test_parse_compression_method_case_insensitive() {
        assert!(matches!(parse_compression_method("DEFLATE").unwrap(), EwfCompressionMethod::Deflate));
        assert!(matches!(parse_compression_method("BZIP2").unwrap(), EwfCompressionMethod::Bzip2));
        assert!(matches!(parse_compression_method("None").unwrap(), EwfCompressionMethod::None));
    }

    #[test]
    fn test_parse_compression_method_unknown() {
        let result = parse_compression_method("lzma");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown compression method"));
    }

    // ==================== Format extensions (critical forensic invariant) ====================

    #[test]
    fn test_encase5_produces_e01_extension() {
        let format = parse_format("e01").unwrap();
        assert_eq!(format.extension(), ".E01");
    }

    #[test]
    fn test_encase7_produces_e01_extension() {
        // Encase7 (0x07) uses EWF1 segment type → .E01, NOT .Ex01
        let format = parse_format("encase7").unwrap();
        assert_eq!(format.extension(), ".E01");
    }

    #[test]
    fn test_v2encase7_produces_ex01_extension() {
        // V2Encase7 (0x37) uses EWF2 segment type → .Ex01
        let format = parse_format("v2encase7").unwrap();
        assert_eq!(format.extension(), ".Ex01");
    }

    #[test]
    fn test_ex01_alias_produces_ex01_extension() {
        let format = parse_format("ex01").unwrap();
        assert_eq!(format.extension(), ".Ex01");
    }

    #[test]
    fn test_l01_rejected() {
        assert!(parse_format("l01").is_err());
    }

    #[test]
    fn test_lx01_rejected() {
        assert!(parse_format("lx01").is_err());
    }

    // ==================== EwfExportProgress serialization ====================

    #[test]
    fn test_export_progress_serialization() {
        let progress = EwfExportProgress {
            output_path: "/tmp/evidence.E01".to_string(),
            current_file: "disk.img".to_string(),
            file_index: 1,
            total_files: 3,
            bytes_written: 1024,
            total_bytes: 4096,
            percent: 25.0,
            phase: "Writing disk.img".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("outputPath")); // camelCase
        assert!(json.contains("bytesWritten"));
        assert!(json.contains("fileIndex"));
        assert!(json.contains("totalFiles"));
        assert!(!json.contains("output_path")); // NOT snake_case
    }

    // ==================== EwfExportResult serialization ====================

    #[test]
    fn test_export_result_serialization() {
        let result = EwfExportResult {
            output_path: "/tmp/evidence.E01".to_string(),
            format: "E01".to_string(),
            bytes_written: 1_048_576,
            files_included: 5,
            compressed: true,
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: None,
            duration_ms: 1500,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("outputPath"));
        assert!(json.contains("bytesWritten"));
        assert!(json.contains("filesIncluded"));
        assert!(json.contains("md5Hash"));
        assert!(json.contains("sha1Hash"));
        assert!(json.contains("durationMs"));
    }

    // ==================== EwfExportOptions deserialization ====================

    #[test]
    fn test_export_options_deserialization() {
        let json = r#"{
            "sourcePaths": ["/tmp/file1.bin", "/tmp/file2.bin"],
            "outputPath": "/tmp/output",
            "format": "e01",
            "compression": "fast",
            "compressionMethod": "deflate",
            "caseNumber": "CASE-001",
            "evidenceNumber": "EV-01",
            "examinerName": "Jane Smith",
            "description": "Test evidence",
            "notes": "Unit test",
            "computeMd5": true,
            "computeSha1": false
        }"#;
        let opts: EwfExportOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.source_paths.len(), 2);
        assert_eq!(opts.output_path, "/tmp/output");
        assert_eq!(opts.format, Some("e01".to_string()));
        assert_eq!(opts.compression, Some("fast".to_string()));
        assert_eq!(opts.compression_method, Some("deflate".to_string()));
        assert_eq!(opts.case_number, Some("CASE-001".to_string()));
        assert_eq!(opts.evidence_number, Some("EV-01".to_string()));
        assert_eq!(opts.examiner_name, Some("Jane Smith".to_string()));
        assert_eq!(opts.compute_md5, Some(true));
        assert_eq!(opts.compute_sha1, Some(false));
    }

    #[test]
    fn test_export_options_minimal_deserialization() {
        let json = r#"{
            "sourcePaths": ["/tmp/file.bin"],
            "outputPath": "/tmp/out"
        }"#;
        let opts: EwfExportOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.source_paths.len(), 1);
        assert!(opts.format.is_none());
        assert!(opts.compression.is_none());
        assert!(opts.compression_method.is_none());
        assert!(opts.case_number.is_none());
        assert!(opts.compute_md5.is_none());
        assert!(opts.compute_sha1.is_none());
    }

    // ==================== Cancel flags ====================

    #[test]
    fn test_cancel_flag_defaults_false() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::Relaxed));
    }

    #[test]
    fn test_cancel_flag_propagation() {
        let flag = Arc::new(AtomicBool::new(false));
        let clone = flag.clone();
        clone.store(true, Ordering::Relaxed);
        assert!(flag.load(Ordering::Relaxed));
    }

    // ==================== EwfImageInfoResponse serialization ====================

    #[test]
    fn test_image_info_response_serialization() {
        let response = EwfImageInfoResponse {
            format: "EnCase 5".to_string(),
            format_extension: ".E01".to_string(),
            is_logical: false,
            is_v2: false,
            media_size: 1_073_741_824,
            bytes_per_sector: 512,
            sectors_per_chunk: 64,
            compression_level: 1,
            compression_method: "Deflate".to_string(),
            media_type: 0,
            media_flags: 1,
            segment_file_version: Some("1.0".to_string()),
            is_corrupted: false,
            is_encrypted: false,
            case_info: EwfReadCaseInfoResponse {
                case_number: Some("CASE-001".to_string()),
                evidence_number: Some("EV-01".to_string()),
                examiner_name: Some("Jane Smith".to_string()),
                description: Some("Test disk image".to_string()),
                notes: None,
                acquiry_software_version: Some("EnCase 8.0".to_string()),
                acquiry_date: Some("2024-01-15".to_string()),
                acquiry_operating_system: Some("Windows 10".to_string()),
                model: Some("Samsung SSD 860 EVO".to_string()),
                serial_number: Some("S3Z9NB0K123456".to_string()),
            },
            md5_hash: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            sha1_hash: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        // Verify camelCase serialization
        assert!(json.contains("\"format\":"));
        assert!(json.contains("\"formatExtension\":"));
        assert!(json.contains("\"isLogical\":"));
        assert!(json.contains("\"isV2\":"));
        assert!(json.contains("\"mediaSize\":"));
        assert!(json.contains("\"bytesPerSector\":"));
        assert!(json.contains("\"sectorsPerChunk\":"));
        assert!(json.contains("\"compressionLevel\":"));
        assert!(json.contains("\"compressionMethod\":"));
        assert!(json.contains("\"mediaType\":"));
        assert!(json.contains("\"mediaFlags\":"));
        assert!(json.contains("\"segmentFileVersion\":"));
        assert!(json.contains("\"isCorrupted\":"));
        assert!(json.contains("\"isEncrypted\":"));
        assert!(json.contains("\"caseInfo\":"));
        assert!(json.contains("\"md5Hash\":"));
        assert!(json.contains("\"sha1Hash\":"));
        // Verify NOT snake_case
        assert!(!json.contains("format_extension"));
        assert!(!json.contains("is_logical"));
        assert!(!json.contains("media_size"));
    }

    #[test]
    fn test_case_info_response_serialization() {
        let case_info = EwfReadCaseInfoResponse {
            case_number: Some("2024-TEST".to_string()),
            evidence_number: Some("001".to_string()),
            examiner_name: Some("John Doe".to_string()),
            description: Some("Forensic image of suspect drive".to_string()),
            notes: Some("Evidence collected under warrant 2024-W-789".to_string()),
            acquiry_software_version: Some("FTK Imager 4.7".to_string()),
            acquiry_date: Some("2024-06-15T10:30:00".to_string()),
            acquiry_operating_system: Some("Windows 11".to_string()),
            model: None,
            serial_number: None,
        };
        let json = serde_json::to_string(&case_info).unwrap();
        assert!(json.contains("\"caseNumber\":"));
        assert!(json.contains("\"evidenceNumber\":"));
        assert!(json.contains("\"examinerName\":"));
        assert!(json.contains("\"acquirySoftwareVersion\":"));
        assert!(json.contains("\"acquiryDate\":"));
        assert!(json.contains("\"acquiryOperatingSystem\":"));
        assert!(!json.contains("case_number"));
        assert!(!json.contains("evidence_number"));
    }

    #[test]
    fn test_image_info_response_logical_v2() {
        let response = EwfImageInfoResponse {
            format: "Logical EnCase 7 V2".to_string(),
            format_extension: ".Lx01".to_string(),
            is_logical: true,
            is_v2: true,
            media_size: 0,
            bytes_per_sector: 512,
            sectors_per_chunk: 64,
            compression_level: 2,
            compression_method: "BZIP2".to_string(),
            media_type: 0,
            media_flags: 0,
            segment_file_version: Some("2.0".to_string()),
            is_corrupted: false,
            is_encrypted: true,
            case_info: EwfReadCaseInfoResponse {
                case_number: None,
                evidence_number: None,
                examiner_name: None,
                description: None,
                notes: None,
                acquiry_software_version: None,
                acquiry_date: None,
                acquiry_operating_system: None,
                model: None,
                serial_number: None,
            },
            md5_hash: None,
            sha1_hash: Some("da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"isLogical\":true"));
        assert!(json.contains("\"isV2\":true"));
        assert!(json.contains("\"isEncrypted\":true"));
        assert!(json.contains("\"sha1Hash\":"));
    }

    // ==================== Format-specific invariants ====================

    #[test]
    fn test_ex01_maps_to_v2encase7_not_encase7() {
        // Critical: "ex01" MUST map to V2Encase7, NOT Encase7
        // Encase7 produces .E01, V2Encase7 produces .Ex01
        let format = parse_format("ex01").unwrap();
        assert!(matches!(format, EwfFormat::V2Encase7));
        assert!(!matches!(format, EwfFormat::Encase7));
    }

    #[test]
    fn test_bzip2_requires_v2_format() {
        // bzip2 compression method is only valid with V2 formats
        let method = parse_compression_method("bzip2").unwrap();
        assert!(matches!(method, EwfCompressionMethod::Bzip2));
        
        // V2 formats that support bzip2
        let v2 = parse_format("v2encase7").unwrap();
        assert!(v2.is_v2());
        
        // Logical V2 (lx01) no longer supported for writing
        assert!(parse_format("lx01").is_err());
        
        // Non-V2 formats (bzip2 would be invalid with these)
        let e5 = parse_format("e01").unwrap();
        assert!(!e5.is_v2());
    }
}
