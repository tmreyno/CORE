// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Expert Witness Format (E01/L01/Ex01/Lx01) operations.

use std::time::Instant;
use tauri::Emitter;
use tracing::{debug, info};

use crate::ewf;

use super::VerifyProgress;

/// Verify an E01 container using libewf C library.
///
/// libewf handles multi-segment discovery, decompression, and media boundary
/// truncation natively.  Supports MD5, SHA-1, and SHA-256.
fn libewf_verify_e01(
    path: &str,
    algorithm: &str,
    progress_cb: &mut dyn FnMut(u64, u64),
) -> Result<String, String> {
    use libewf_ffi::EwfReader;

    let reader = EwfReader::open(path).map_err(|e| format!("libewf open failed: {e}"))?;

    reader
        .verify_media_hash(algorithm, |current, total| {
            progress_cb(current, total);
        })
        .map_err(|e| format!("libewf verify failed: {e}"))
}

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
        let mut progress_cb = |current: u64, total: u64| {
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
        };

        // Prefer libewf C library for E01 verification — it handles
        // multi-segment, decompression, and media boundaries natively.
        // Fall back to the pure-Rust parser if libewf is unavailable
        // (stub build) or if the algorithm is not supported.
        if libewf_ffi::is_available() {
            let libewf_result = libewf_verify_e01(&inputPath, &algorithm, &mut progress_cb);
            match libewf_result {
                Ok(hash) => Ok(hash),
                Err(e) => {
                    debug!(
                        error = %e,
                        "libewf verify failed, falling back to pure-Rust parser"
                    );
                    ewf::verify_with_progress(&inputPath, &algorithm, &mut progress_cb)
                        .map_err(|e| e.to_string())
                }
            }
        } else {
            ewf::verify_with_progress(&inputPath, &algorithm, &mut progress_cb)
                .map_err(|e| e.to_string())
        }
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?;
    let elapsed = start.elapsed();
    match &result {
        Ok(hash) => {
            info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, hash = %hash, "E01 verification completed")
        }
        Err(e) => {
            info!(path = %path_for_log, duration_ms = elapsed.as_millis() as u64, error = %e, "E01 verification failed")
        }
    }
    result
}
