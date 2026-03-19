// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Companion file writer/reader for acquisition metadata.
//!
//! When an acquisition completes (E01, L01, 7z, memory, triage), a
//! `.ffx-companion.json` sidecar file is written alongside the output.
//! This file captures acquisition metadata (case info, hashes, timing,
//! source/output details) so it can be re-imported into CORE-FFX later
//! to auto-populate evidence collection forms.
//!
//! File naming:
//! - File outputs: `<output>.ffx-companion.json` (e.g., `evidence.E01.ffx-companion.json`)
//! - Directory outputs: `<dir>/ffx-companion.json`

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

// ─── Shared Sub-Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionCaseInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examiner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSourceInfo {
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_files: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionOutputInfo {
    pub format: String,
    pub primary_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<Vec<String>>,
    pub total_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionHashes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionTiming {
    pub started_at: String,
    pub completed_at: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionSystemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_drive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_capacity: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_drive_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_removable: Option<bool>,
    // System identification (from Identify phase)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_serial_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
}

// ─── Input from Frontend ──────────────────────────────────────────────────────

/// Data provided by the frontend. The backend adds envelope fields
/// (version, tool, toolVersion, createdAt) before writing.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionFileInput {
    pub acquisition_type: String,
    #[serde(default)]
    pub case: Option<CompanionCaseInfo>,
    pub source: CompanionSourceInfo,
    pub output: CompanionOutputInfo,
    #[serde(default)]
    pub hashes: Option<CompanionHashes>,
    pub timing: CompanionTiming,
    #[serde(default)]
    pub system: Option<CompanionSystemInfo>,
}

// ─── Full Companion File ──────────────────────────────────────────────────────

/// Complete companion file structure (written to disk / read back).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionFile {
    pub version: String,
    pub tool: String,
    pub tool_version: String,
    pub created_at: String,
    pub acquisition_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case: Option<CompanionCaseInfo>,
    pub source: CompanionSourceInfo,
    pub output: CompanionOutputInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<CompanionHashes>,
    pub timing: CompanionTiming,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<CompanionSystemInfo>,
}

// ─── Path Helpers ─────────────────────────────────────────────────────────────

/// Compute the companion file path for a given output path.
///
/// - File outputs: `<output_path>.ffx-companion.json`
/// - Directory outputs: `<output_path>/ffx-companion.json`
pub fn companion_path_for(output_path: &str) -> PathBuf {
    let p = Path::new(output_path);
    if p.is_dir() {
        p.join("ffx-companion.json")
    } else {
        let mut name = p.file_name().unwrap_or_default().to_os_string();
        name.push(".ffx-companion.json");
        p.with_file_name(name)
    }
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

/// Write a companion file alongside an acquisition output.
///
/// The backend adds envelope fields (version, tool name, tool version,
/// created_at timestamp) so the frontend only needs to provide the
/// acquisition-specific data.
#[tauri::command]
pub async fn write_companion_file(
    output_path: String,
    data: CompanionFileInput,
) -> Result<String, String> {
    let companion_path = companion_path_for(&output_path);

    let file = CompanionFile {
        version: "1.0".to_string(),
        tool: "CORE-FFX".to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: data.timing.completed_at.clone(),
        acquisition_type: data.acquisition_type,
        case: data.case,
        source: data.source,
        output: data.output,
        hashes: data.hashes,
        timing: data.timing,
        system: data.system,
    };

    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| format!("Failed to serialize companion file: {e}"))?;

    std::fs::write(&companion_path, json)
        .map_err(|e| format!("Failed to write companion file: {e}"))?;

    Ok(companion_path.to_string_lossy().to_string())
}

/// Read and parse a companion file.
#[tauri::command]
pub async fn read_companion_file(path: String) -> Result<CompanionFile, String> {
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read companion file: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse companion file: {e}"))?
}

/// Find a companion file for a given evidence file path.
///
/// Checks for `<evidence_path>.ffx-companion.json` (file sidecar)
/// and `<evidence_path>/ffx-companion.json` (directory companion).
/// Returns `None` if no companion file exists.
#[tauri::command]
pub async fn find_companion_file(evidence_path: String) -> Result<Option<String>, String> {
    let p = Path::new(&evidence_path);

    // Check file sidecar: <path>.ffx-companion.json
    let mut sidecar_name = p.file_name().unwrap_or_default().to_os_string();
    sidecar_name.push(".ffx-companion.json");
    let sidecar = p.with_file_name(sidecar_name);
    if sidecar.exists() {
        return Ok(Some(sidecar.to_string_lossy().to_string()));
    }

    // Check directory companion: <path>/ffx-companion.json
    let dir_companion = p.join("ffx-companion.json");
    if dir_companion.exists() {
        return Ok(Some(dir_companion.to_string_lossy().to_string()));
    }

    Ok(None)
}

// ─── Acquisition Scanner ──────────────────────────────────────────────────────

/// A companion file discovered during a directory scan, with metadata
/// about whether the acquisition output still exists on disk.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredAcquisition {
    /// Path to the `.ffx-companion.json` file
    pub companion_path: String,
    /// Parsed companion file contents
    pub companion: CompanionFile,
    /// Whether the primary output file/directory still exists on disk
    pub output_exists: bool,
    /// Size of the output file in bytes (if it exists and is a file)
    pub output_size: Option<u64>,
}

/// Recursively scan a directory for `.ffx-companion.json` sidecar files.
///
/// Returns a list of parsed companion files with existence checks on their
/// referenced output files. This enables the "Import Acquisitions" workflow
/// where a user points at a directory of past acquisitions and selectively
/// imports them into the current project.
#[tauri::command]
pub async fn scan_for_acquisitions(
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<DiscoveredAcquisition>, String> {
    let root = Path::new(&dirPath);
    if !root.exists() {
        return Err(format!("Path does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root.display()));
    }

    info!("Scanning for companion files in: {}", root.display());

    let mut results = Vec::new();
    scan_dir_recursive(root, &mut results, 0);

    info!(
        "Found {} companion files in {}",
        results.len(),
        root.display()
    );

    Ok(results)
}

/// Recursively walk a directory looking for `.ffx-companion.json` files.
/// Max depth of 10 to prevent runaway traversal.
fn scan_dir_recursive(dir: &Path, results: &mut Vec<DiscoveredAcquisition>, depth: usize) {
    const MAX_DEPTH: usize = 10;
    if depth > MAX_DEPTH {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) => {
            debug!("Cannot read directory {}: {}", dir.display(), err);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories/files
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(&path, results, depth + 1);
        } else if name.ends_with(".ffx-companion.json") {
            match parse_companion_at(&path) {
                Ok(acq) => {
                    debug!("Found companion: {}", path.display());
                    results.push(acq);
                }
                Err(err) => {
                    warn!("Failed to parse companion file {}: {}", path.display(), err);
                }
            }
        }
    }
}

/// Parse a single companion file and check whether its output still exists.
fn parse_companion_at(companion_path: &Path) -> Result<DiscoveredAcquisition, String> {
    let data = std::fs::read_to_string(companion_path).map_err(|e| format!("Read error: {e}"))?;

    let companion: CompanionFile =
        serde_json::from_str(&data).map_err(|e| format!("Parse error: {e}"))?;

    let primary = &companion.output.primary_path;
    let output_path = Path::new(primary);
    let output_exists = output_path.exists();
    let output_size = if output_exists && output_path.is_file() {
        std::fs::metadata(output_path).ok().map(|m| m.len())
    } else {
        None
    };

    Ok(DiscoveredAcquisition {
        companion_path: companion_path.to_string_lossy().to_string(),
        companion,
        output_exists,
        output_size,
    })
}
