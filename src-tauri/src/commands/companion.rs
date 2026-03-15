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
    serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse companion file: {e}"))?
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
