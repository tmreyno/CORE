// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED container operations (UFD, UFDR, UFDX formats).
//!
//! Tauri command wrappers for the UFED backend module (`crate::ufed`).
//! Provides IPC surface for:
//! - Container metadata retrieval (full and fast modes)
//! - Hash verification with progress events
//! - Container statistics
//! - File extraction

use tauri::Emitter;

use crate::ufed;

use super::VerifyProgress;

/// Get full UFED container information.
///
/// Parses UFD/UFDX metadata, scans for associated files,
/// checks extraction set completeness, and reads collection info.
///
/// # Arguments
/// * `path` - Path to the UFED container file (.ufd, .ufdr, .ufdx, or .zip)
///
/// # Returns
/// Complete `UfedInfo` with case, device, extraction, and collection data.
#[tauri::command]
pub async fn ufed_info(path: String) -> Result<ufed::UfedInfo, String> {
    tauri::async_runtime::spawn_blocking(move || ufed::info(&path))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
        .map_err(|e| e.to_string())
}

/// Get fast UFED container information (lightweight).
///
/// Skips associated file scanning, sibling UFD parsing for ZIPs,
/// and collection UFDX parsing. Use for quick listing/detection.
///
/// # Arguments
/// * `path` - Path to the UFED container file
///
/// # Returns
/// Partial `UfedInfo` with basic metadata only.
#[tauri::command]
pub async fn ufed_info_fast(path: String) -> Result<ufed::UfedInfo, String> {
    tauri::async_runtime::spawn_blocking(move || ufed::info_fast(&path))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
        .map_err(|e| e.to_string())
}

/// Verify a UFED container by computing its hash.
///
/// Emits `verify-progress` events with `{ path, current, total, percent }`.
///
/// # Arguments
/// * `path` - Path to the UFED container file
/// * `algorithm` - Hash algorithm (e.g., "SHA-256", "MD5", "BLAKE3")
/// * `app` - Tauri app handle for progress events
///
/// # Returns
/// Computed hash string (hex-encoded).
#[tauri::command]
pub async fn ufed_verify(
    path: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let path_for_closure = path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ufed::verify_with_progress(&path, &algorithm, |current, total| {
            let percent = if total > 0 {
                (current as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            let _ = app.emit(
                "verify-progress",
                VerifyProgress {
                    path: path_for_closure.clone(),
                    current,
                    total,
                    percent,
                },
            );
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
    .map_err(|e| e.to_string())
}

/// Get UFED container statistics.
///
/// Returns aggregate stats: file size, associated file count/size,
/// stored hash count, and presence flags for metadata sections.
///
/// # Arguments
/// * `path` - Path to the UFED container file
///
/// # Returns
/// `UfedStats` with container statistics.
#[tauri::command]
pub async fn ufed_get_stats(path: String) -> Result<ufed::UfedStats, String> {
    tauri::async_runtime::spawn_blocking(move || ufed::get_stats(&path))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
        .map_err(|e| e.to_string())
}

/// Extract UFED container contents to output directory.
///
/// For ZIP containers, extracts the archive contents.
/// For UFD files, copies the file and any associated files.
/// For UFDR containers, copies the data to the output directory.
///
/// # Arguments
/// * `path` - Path to the UFED container file
/// * `output_dir` - Destination directory for extracted files
///
/// # Returns
/// `UfedExtractResult` with extraction summary.
#[tauri::command]
pub async fn ufed_extract(
    path: String,
    output_dir: String,
) -> Result<ufed::UfedExtractResult, String> {
    tauri::async_runtime::spawn_blocking(move || ufed::extract(&path, &output_dir))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
        .map_err(|e| e.to_string())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufed_info_missing_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ufed_info("/nonexistent/file.ufd".to_string()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("not found")
                || err.contains("Not found")
                || err.contains("UFED file not found"),
            "Unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_ufed_info_fast_missing_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ufed_info_fast("/nonexistent/file.ufd".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_ufed_get_stats_missing_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ufed_get_stats("/nonexistent/file.ufd".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_ufed_extract_missing_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(ufed_extract(
            "/nonexistent/file.ufd".to_string(),
            "/tmp/output".to_string(),
        ));
        assert!(result.is_err());
    }
}
