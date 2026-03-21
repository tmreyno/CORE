// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Expert Witness Format (E01/L01/Ex01/Lx01) operations.

use std::time::Instant;
use tauri::Emitter;
use tracing::info;

use crate::ewf;

use super::VerifyProgress;

// EWF Commands - Expert Witness Format implementation (E01/L01/Ex01/Lx01)
#[tauri::command]
pub async fn e01_v3_verify(
    #[allow(non_snake_case)] inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    info!(path = %inputPath, algorithm = %algorithm, "Starting E01 verification");
    let start = Instant::now();
    let path_for_closure = inputPath.clone();
    let path_for_log = inputPath.clone();
    // Run on blocking thread pool to prevent UI freeze
    let result = tauri::async_runtime::spawn_blocking(move || {
        ewf::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
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
    .map_err(|e| e.to_string());
    let elapsed = start.elapsed();
    match &result {
        Ok(hash) => info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, hash = %hash, "E01 verification completed"),
        Err(e) => info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, error = %e, "E01 verification failed"),
    }
    result
}
