// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Raw disk image operations (.dd, .raw, .img, .001).

use std::time::Instant;
use tauri::Emitter;
use tracing::info;

use crate::raw;

use super::VerifyProgress;

// RAW Commands - Raw disk image implementation (.dd, .raw, .img, .001)
#[tauri::command]
pub async fn raw_verify(
    #[allow(non_snake_case)] inputPath: String,
    algorithm: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    info!(path = %inputPath, algorithm = %algorithm, "Starting Raw verification");
    let start = Instant::now();
    let path_for_closure = inputPath.clone();
    let path_for_log = inputPath.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        raw::verify_with_progress(&inputPath, &algorithm, |current, total| {
            let percent = (current as f64 / total as f64) * 100.0;
            crate::eventing::log_emit_result(
                "verify-progress",
                app.emit(
                    "verify-progress",
                    VerifyProgress {
                        path: path_for_closure.clone(),
                        current,
                        total,
                        percent,
                    },
                ),
            );
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;
    let elapsed = start.elapsed();
    match &result {
        Ok(hash) => {
            info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, hash = %hash, "Raw verification completed")
        }
        Err(e) => {
            info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, error = %e, "Raw verification failed")
        }
    }
    result
}
