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
use std::collections::HashSet;
use std::path::Path;
use tauri::{Emitter, Window};
use tracing::info;

use super::{ArchiveCreateProgress, CreateArchiveOptions};
use crate::common::hash::hash_file;

const ARCHIVE_CREATE_MAX_TRAVERSAL_DEPTH: usize = 128;
const ARCHIVE_CREATE_MAX_FILES: usize = 250_000;

fn checked_manifest_total_size_add(total: u64, addition: u64, path: &Path) -> Result<u64, String> {
    total.checked_add(addition).ok_or_else(|| {
        format!(
            "Archive manifest total size overflow while adding {} bytes from {} to current total {} bytes",
            addition,
            path.display(),
            total
        )
    })
}

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
    let mut used_manifest_paths = HashSet::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if path.is_file() {
            if files.len() >= ARCHIVE_CREATE_MAX_FILES {
                tracing::warn!("Archive manifest file collection reached file cap");
                break;
            }

            // Use just the filename as the relative path
            let rel = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| input_path.clone());
            files.push((
                unique_manifest_path(rel, &mut used_manifest_paths),
                path.to_path_buf(),
            ));
        } else if path.is_dir() {
            collect_dir_files_with_used_paths(path, path, &mut files, &mut used_manifest_paths)?;
            if files.len() >= ARCHIVE_CREATE_MAX_FILES {
                tracing::warn!("Archive manifest file collection reached file cap");
                break;
            }
        }
    }

    Ok(files)
}

/// Recursively collect files from a directory.
/// Skips unreadable directories (e.g. macOS TCC-protected folders) with a warning.
#[cfg(test)]
pub(super) fn collect_dir_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<(), String> {
    let mut used_manifest_paths = files
        .iter()
        .map(|(rel_path, _)| rel_path.clone())
        .collect::<HashSet<_>>();
    collect_dir_files_with_used_paths(root, dir, files, &mut used_manifest_paths)
}

fn collect_dir_files_with_used_paths(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
    used_manifest_paths: &mut HashSet<String>,
) -> Result<(), String> {
    collect_dir_files_at_depth(root, dir, files, used_manifest_paths, 0)
}

fn collect_dir_files_at_depth(
    root: &Path,
    dir: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
    used_manifest_paths: &mut HashSet<String>,
    depth: usize,
) -> Result<(), String> {
    if depth > ARCHIVE_CREATE_MAX_TRAVERSAL_DEPTH {
        tracing::warn!(
            "Archive manifest collection reached traversal depth cap at {}",
            dir.display()
        );
        return Ok(());
    }

    if files.len() >= ARCHIVE_CREATE_MAX_FILES {
        tracing::warn!(
            "Archive manifest collection reached file cap while scanning {}",
            dir.display()
        );
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Skipping unreadable directory {}: {}", dir.display(), e);
            return Ok(());
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if files.len() >= ARCHIVE_CREATE_MAX_FILES {
                tracing::warn!(
                    "Archive manifest collection reached file cap while scanning {}",
                    dir.display()
                );
                break;
            }

            let rel = path
                .strip_prefix(root)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            files.push((unique_manifest_path(rel, used_manifest_paths), path));
        } else if path.is_dir() {
            collect_dir_files_at_depth(root, &path, files, used_manifest_paths, depth + 1)?;
        }
    }

    Ok(())
}

fn unique_manifest_path(rel_path: String, used_manifest_paths: &mut HashSet<String>) -> String {
    if used_manifest_paths.insert(rel_path.clone()) {
        return rel_path;
    }

    for duplicate_index in 2usize.. {
        let candidate = disambiguated_manifest_path(&rel_path, duplicate_index);
        if used_manifest_paths.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("usize range is unbounded")
}

fn disambiguated_manifest_path(rel_path: &str, duplicate_index: usize) -> String {
    let path = Path::new(rel_path);
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return format!("{rel_path} ({duplicate_index})");
    };

    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or(file_name);
    let file_name = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if !extension.is_empty() => {
            format!("{stem} ({duplicate_index}).{extension}")
        }
        _ => format!("{stem} ({duplicate_index})"),
    };

    match path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        Some(parent) => parent.join(file_name).to_string_lossy().to_string(),
        None => file_name,
    }
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
        total_size = checked_manifest_total_size_add(total_size, file_size, abs_path)?;

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
    fn test_collect_files_disambiguates_duplicate_manifest_paths() {
        let dir = TempDir::new().unwrap();
        let first_dir = dir.path().join("first");
        let second_dir = dir.path().join("second");
        fs::create_dir_all(&first_dir).unwrap();
        fs::create_dir_all(&second_dir).unwrap();
        let first = first_dir.join("same.bin");
        let second = second_dir.join("same.bin");
        fs::write(&first, "first").unwrap();
        fs::write(&second, "second").unwrap();

        let files = collect_files(&[
            first.to_string_lossy().to_string(),
            second.to_string_lossy().to_string(),
        ])
        .unwrap();
        let names: Vec<&str> = files.iter().map(|(rel, _)| rel.as_str()).collect();

        assert_eq!(names, vec!["same.bin", "same (2).bin"]);
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

    #[test]
    fn test_checked_manifest_total_size_add_sums_regular_values() {
        let path = Path::new("manifest-entry.bin");

        assert_eq!(checked_manifest_total_size_add(12, 30, path).unwrap(), 42);
    }

    #[test]
    fn test_checked_manifest_total_size_add_rejects_overflow() {
        let path = Path::new("manifest-overflow.bin");
        let err = checked_manifest_total_size_add(u64::MAX, 1, path).unwrap_err();

        assert!(err.contains("manifest total size overflow"));
        assert!(err.contains("manifest-overflow.bin"));
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

    #[test]
    fn test_collect_dir_files_disambiguates_against_existing_manifest_paths() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("evidence.txt"), b"nested").unwrap();

        let mut files = vec![(
            "evidence.txt".to_string(),
            dir.path().join("existing-evidence.txt"),
        )];
        collect_dir_files(dir.path(), dir.path(), &mut files).unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files[1].0, "evidence (2).txt");
    }

    #[test]
    fn test_collect_dir_files_skips_beyond_depth_limit() {
        let dir = TempDir::new().unwrap();
        let mut deep = dir.path().to_path_buf();

        for _ in 0..=ARCHIVE_CREATE_MAX_TRAVERSAL_DEPTH {
            deep = deep.join("d");
            fs::create_dir(&deep).unwrap();
        }

        fs::write(deep.join("too-deep.bin"), vec![0u8; 100]).unwrap();

        let mut files = Vec::new();
        collect_dir_files(dir.path(), dir.path(), &mut files).unwrap();
        assert!(files.is_empty());
    }
}
