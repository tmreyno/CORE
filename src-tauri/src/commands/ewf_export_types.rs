// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! EWF export types and cancel flag management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};

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
pub(super) static EWF_CANCEL_FLAGS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

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
}
