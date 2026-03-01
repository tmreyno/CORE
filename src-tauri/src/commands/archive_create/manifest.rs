// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic manifest types and generation for archive creation.
//!
//! Generates a JSON manifest alongside 7z archives containing:
//! - Per-file hash inventory (SHA-256, MD5, SHA-1)
//! - Chain-of-custody metadata (examiner, case number, evidence description)
//! - Archive-level SHA-256 hash
//! - System provenance (hostname, OS)

use serde::Serialize;
use std::path::Path;
use tauri::{Emitter, Window};
use tracing::info;

use super::{ArchiveCreateProgress, CreateArchiveOptions};
use crate::common::hash::hash_file;

// =============================================================================
// Forensic Manifest Types
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

// =============================================================================
// File Collection Helpers
// =============================================================================

/// Collect all files from input paths (recursively for directories)
pub(super) fn collect_files(
    input_paths: &[String],
) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let mut files = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if path.is_file() {
            // Use just the filename as the relative path
            let rel = path
                .file_name()
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
pub(super) fn collect_dir_files(
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
            let rel = path
                .strip_prefix(root)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            files.push((rel, path));
        } else if path.is_dir() {
            collect_dir_files(root, &path, files)?;
        }
    }

    Ok(())
}

// =============================================================================
// Manifest Generation
// =============================================================================

/// Generate a forensic manifest for the archive
pub(super) fn generate_forensic_manifest(
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
    let _ = window.emit(
        "archive-create-progress",
        ArchiveCreateProgress {
            archive_path: archive_path.to_string(),
            current_file: String::new(),
            bytes_processed: 0,
            bytes_total: 0,
            current_file_bytes: 0,
            current_file_total: 0,
            percent: 0.0,
            status: format!("Generating forensic manifest ({} files)...", total_files),
        },
    );

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
            Some(
                hash_file(abs_path, "sha256")
                    .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?,
            )
        } else {
            None
        };

        let md5_hash = if compute_md5 {
            Some(
                hash_file(abs_path, "md5")
                    .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?,
            )
        } else {
            None
        };

        let sha1_hash = if compute_sha1 {
            Some(
                hash_file(abs_path, "sha1")
                    .map_err(|e| format!("Failed to hash {}: {}", rel_path, e))?,
            )
        } else {
            None
        };

        // Progress update (every 10 files or last file)
        if i % 10 == 0 || i == total_files - 1 {
            let percent = ((i + 1) as f64 / total_files as f64) * 100.0;
            let _ = window.emit(
                "archive-create-progress",
                ArchiveCreateProgress {
                    archive_path: archive_path.to_string(),
                    current_file: rel_path.clone(),
                    bytes_processed: 0,
                    bytes_total: 0,
                    current_file_bytes: 0,
                    current_file_total: 0,
                    percent,
                    status: format!("Hashing for manifest: {}/{} files", i + 1, total_files),
                },
            );
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
        let _ = window.emit(
            "archive-create-progress",
            ArchiveCreateProgress {
                archive_path: archive_path.to_string(),
                current_file: String::new(),
                bytes_processed: 0,
                bytes_total: 0,
                current_file_bytes: 0,
                current_file_total: 0,
                percent: 0.0,
                status: "Hashing archive file...".to_string(),
            },
        );
        Some(
            hash_file(Path::new(archive_path), "sha256")
                .map_err(|e| format!("Failed to hash archive: {}", e))?,
        )
    } else {
        // Split archives - hash first volume
        let first_vol = format!("{}.001", archive_path);
        if Path::new(&first_vol).exists() {
            Some(
                hash_file(Path::new(&first_vol), "sha256")
                    .map_err(|e| format!("Failed to hash first volume: {}", e))?,
            )
        } else {
            None
        }
    };

    // Build hash algorithms list
    let mut hash_algorithms = Vec::new();
    if compute_sha256 {
        hash_algorithms.push("SHA-256".to_string());
    }
    if compute_md5 {
        hash_algorithms.push("MD5".to_string());
    }
    if compute_sha1 {
        hash_algorithms.push("SHA-1".to_string());
    }

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

    info!(
        "Forensic manifest written: {} ({} files, {} bytes)",
        manifest_path, manifest.total_files, manifest.total_size
    );

    Ok(manifest_path)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
        // Normalize separators for cross-platform compatibility
        let names: Vec<String> = files
            .iter()
            .map(|(rel, _)| rel.replace('\\', "/"))
            .collect();
        assert!(names.iter().any(|n| n == "root.txt"));
        assert!(names.iter().any(|n| n == "subdir/nested.txt"));
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
        ])
        .unwrap();
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

    // ==================== collect_dir_files ====================

    #[test]
    fn test_collect_dir_files_preserves_relative_paths() {
        let dir = TempDir::new().unwrap();
        let deep = dir.path().join("level1").join("level2");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.bin"), vec![0u8; 100]).unwrap();

        let mut files = Vec::new();
        collect_dir_files(dir.path(), dir.path(), &mut files).unwrap();
        assert_eq!(files.len(), 1);
        // Normalize separators for cross-platform compatibility
        assert_eq!(files[0].0.replace('\\', "/"), "level1/level2/deep.bin");
    }
}
