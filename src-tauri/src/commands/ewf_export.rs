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
        "l01" | "logical" | "logical_encase5" => Ok(EwfFormat::LogicalEncase5),
        "l01v6" | "logical_encase6" => Ok(EwfFormat::LogicalEncase6),
        // LogicalEncase7 (0x12) produces .L01 (EWF1 segment type)
        "l01v7" | "logical_encase7" => Ok(EwfFormat::LogicalEncase7),
        // V2LogicalEncase7 (0x47) produces .Lx01 (EWF2 segment type)
        "lx01" | "v2logical_encase7" => Ok(EwfFormat::V2LogicalEncase7),
        "ftk" => Ok(EwfFormat::FtkImager),
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

    // Calculate total size of source files
    let mut total_bytes: u64 = 0;
    let mut file_sizes: Vec<(String, u64)> = Vec::new();
    for path_str in &options.source_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("Source file does not exist: {}", path_str));
        }
        let metadata = std::fs::metadata(path).map_err(|e| {
            format!("Failed to read metadata for {}: {}", path_str, e)
        })?;
        let size = metadata.len();
        total_bytes += size;
        file_sizes.push((path_str.clone(), size));
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
    fn test_parse_format_l01() {
        let result = parse_format("l01").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase5));
    }

    #[test]
    fn test_parse_format_logical() {
        let result = parse_format("logical").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase5));
    }

    #[test]
    fn test_parse_format_logical_encase5() {
        let result = parse_format("logical_encase5").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase5));
    }

    #[test]
    fn test_parse_format_l01v6() {
        let result = parse_format("l01v6").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase6));
    }

    #[test]
    fn test_parse_format_logical_encase6() {
        let result = parse_format("logical_encase6").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase6));
    }

    #[test]
    fn test_parse_format_l01v7() {
        let result = parse_format("l01v7").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase7));
    }

    #[test]
    fn test_parse_format_logical_encase7() {
        let result = parse_format("logical_encase7").unwrap();
        assert!(matches!(result, EwfFormat::LogicalEncase7));
    }

    #[test]
    fn test_parse_format_lx01() {
        let result = parse_format("lx01").unwrap();
        assert!(matches!(result, EwfFormat::V2LogicalEncase7));
    }

    #[test]
    fn test_parse_format_v2logical_encase7() {
        let result = parse_format("v2logical_encase7").unwrap();
        assert!(matches!(result, EwfFormat::V2LogicalEncase7));
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
    fn test_l01_produces_l01_extension() {
        let format = parse_format("l01").unwrap();
        assert_eq!(format.extension(), ".L01");
    }

    #[test]
    fn test_lx01_produces_lx01_extension() {
        let format = parse_format("lx01").unwrap();
        assert_eq!(format.extension(), ".Lx01");
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
        let v2l = parse_format("lx01").unwrap();
        assert!(v2l.is_v2());
        
        // Non-V2 formats (bzip2 would be invalid with these)
        let e5 = parse_format("e01").unwrap();
        assert!(!e5.is_v2());
    }
}
