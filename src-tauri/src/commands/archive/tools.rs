// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Archive tools - testing, repair, validation, encryption, LZMA compression,
//! and split archive handling.
//!
//! Provides advanced 7z operations including integrity testing, corruption repair,
//! detailed error reporting, native AES-256 encryption/decryption, LZMA/LZMA2
//! raw compression/decompression, and multi-volume archive extraction.

use tauri::{Emitter, Window};
use tracing::info;

use seven_zip::advanced;
use seven_zip::CompressionLevel;
use seven_zip::EncryptionContext;
use seven_zip::SevenZip;

/// Test archive integrity without extracting
#[tauri::command]
pub async fn test_7z_archive(
    archive_path: String,
    password: Option<String>,
    window: Window,
) -> Result<bool, String> {
    info!("Testing archive integrity: {}", archive_path);

    let window_clone = window.clone();
    let archive_path_clone = archive_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new().map_err(|e| format!("Failed to initialize 7z library: {}", e))?;

        // Emit starting status
        let _ = window_clone.emit(
            "archive-test-progress",
            serde_json::json!({
                "archive_path": archive_path_clone,
                "status": "Testing archive integrity...",
                "percent": 0.0,
            }),
        );

        // Test archive
        sz.test_archive(
            &archive_path_clone,
            password.as_deref(),
            None, // No progress callback for now
        )
        .map_err(|e| format!("Archive test failed: {}", e))?;

        // Emit completion
        let _ = window_clone.emit(
            "archive-test-progress",
            serde_json::json!({
                "archive_path": archive_path_clone,
                "status": "Archive is valid",
                "percent": 100.0,
            }),
        );

        info!("Archive test passed: {}", archive_path);
        Ok(true)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Repair corrupted archive
#[tauri::command]
pub async fn repair_7z_archive(
    corrupted_path: String,
    repaired_path: String,
    window: Window,
) -> Result<String, String> {
    info!("Repairing archive: {} -> {}", corrupted_path, repaired_path);

    let window_clone = window.clone();
    let repaired_clone = repaired_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new().map_err(|e| format!("Failed to initialize 7z library: {}", e))?;

        // Emit start status
        let _ = window_clone.emit(
            "archive-repair-progress",
            serde_json::json!({
                "percent": 0.0,
                "status": "Repairing archive...",
            }),
        );

        // Repair archive
        sz.repair_archive(
            &corrupted_path,
            &repaired_clone,
            None, // No progress callback for now
        )
        .map_err(|e| format!("Archive repair failed: {}", e))?;

        // Emit completion
        let _ = window_clone.emit(
            "archive-repair-progress",
            serde_json::json!({
                "percent": 100.0,
                "status": "Archive repaired successfully",
            }),
        );

        info!("Archive repaired successfully: {}", repaired_clone);
        Ok(repaired_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get detailed information about the last archive error
#[tauri::command]
pub fn get_last_archive_error() -> Result<serde_json::Value, String> {
    advanced::DetailedError::get_last()
        .map(|err| {
            serde_json::json!({
                "code": err.code,
                "message": err.message,
                "file_context": err.file_context,
                "position": err.position,
                "suggestion": err.suggestion,
            })
        })
        .map_err(|e| format!("Failed to get error details: {}", e))
}

/// Clear last error
#[tauri::command]
pub fn clear_last_archive_error() {
    advanced::DetailedError::clear();
}

/// Compress a single file to .lzma format
#[tauri::command]
pub async fn compress_to_lzma(
    input_path: String,
    output_path: String,
    compression_level: u8,
) -> Result<String, String> {
    info!("Compressing to LZMA: {} -> {}", input_path, output_path);

    let output_clone = output_path.clone();

    tokio::task::spawn_blocking(move || {
        let level = match compression_level {
            0 => CompressionLevel::Store,
            1 => CompressionLevel::Fastest,
            2..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Normal,
            7..=8 => CompressionLevel::Maximum,
            9 => CompressionLevel::Ultra,
            _ => CompressionLevel::Normal,
        };

        advanced::compress_lzma(&input_path, &output_path, level)
            .map_err(|e| format!("LZMA compression failed: {}", e))?;

        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Decompress a .lzma file
#[tauri::command]
pub async fn decompress_lzma(lzma_path: String, output_path: String) -> Result<String, String> {
    info!("Decompressing LZMA: {} -> {}", lzma_path, output_path);

    let output_clone = output_path.clone();

    tokio::task::spawn_blocking(move || {
        advanced::decompress_lzma(&lzma_path, &output_path)
            .map_err(|e| format!("LZMA decompression failed: {}", e))?;

        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Compress a single file to .xz (LZMA2) format
#[tauri::command]
pub async fn compress_to_lzma2(
    input_path: String,
    output_path: String,
    compression_level: u8,
) -> Result<String, String> {
    info!("Compressing to LZMA2: {} -> {}", input_path, output_path);

    let output_clone = output_path.clone();

    tokio::task::spawn_blocking(move || {
        let level = match compression_level {
            0 => CompressionLevel::Store,
            1 => CompressionLevel::Fastest,
            2..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Normal,
            7..=8 => CompressionLevel::Maximum,
            9 => CompressionLevel::Ultra,
            _ => CompressionLevel::Normal,
        };

        advanced::compress_lzma2(&input_path, &output_path, level)
            .map_err(|e| format!("LZMA2 compression failed: {}", e))?;

        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Decompress a .xz (LZMA2) file
#[tauri::command]
pub async fn decompress_lzma2(xz_path: String, output_path: String) -> Result<String, String> {
    info!("Decompressing LZMA2: {} -> {}", xz_path, output_path);

    let output_clone = output_path.clone();

    tokio::task::spawn_blocking(move || {
        advanced::decompress_lzma2(&xz_path, &output_path)
            .map_err(|e| format!("LZMA2 decompression failed: {}", e))?;

        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Encrypt data using native Rust AES-256
#[tauri::command]
pub fn encrypt_data_native(data: Vec<u8>, password: String) -> Result<Vec<u8>, String> {
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;

    ctx.encrypt(&data)
        .map_err(|e| format!("Encryption failed: {}", e))
}

/// Decrypt data using native Rust AES-256
#[tauri::command]
pub fn decrypt_data_native(encrypted_data: Vec<u8>, password: String) -> Result<Vec<u8>, String> {
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize decryption: {}", e))?;

    ctx.decrypt(&encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Extract split/multi-volume archive
#[tauri::command]
pub async fn extract_split_7z_archive(
    first_volume_path: String,
    output_dir: String,
    password: Option<String>,
    window: Window,
) -> Result<String, String> {
    info!(
        "Extracting split archive: {} to {}",
        first_volume_path, output_dir
    );

    let window_clone = window.clone();
    let output_clone = output_dir.clone();

    tauri::async_runtime::spawn_blocking(move || {
        // Emit start status
        let _ = window_clone.emit(
            "split-extract-progress",
            serde_json::json!({
                "status": "Extracting split archive...",
                "percent": 0.0,
            }),
        );

        // Extract split archive (simple version without progress callback)
        advanced::extract_split_archive(&first_volume_path, &output_dir, password.as_deref())
            .map_err(|e| format!("Split archive extraction failed: {}", e))?;

        // Emit completion
        let _ = window_clone.emit(
            "split-extract-progress",
            serde_json::json!({
                "status": "Split archive extracted successfully",
                "percent": 100.0,
            }),
        );

        Ok(output_clone)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Validation result structure with detailed error context
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveValidationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub file_context: Option<String>,
    pub suggestion: Option<String>,
}

/// Validate archive with detailed error reporting
#[tauri::command]
pub async fn validate_7z_archive(archive_path: String) -> Result<ArchiveValidationResult, String> {
    info!("Validating archive: {}", archive_path);

    tauri::async_runtime::spawn_blocking(move || {
        let sz = SevenZip::new().map_err(|e| format!("Failed to initialize 7z: {}", e))?;

        match sz.validate_archive(&archive_path) {
            Ok(_) => Ok(ArchiveValidationResult {
                is_valid: true,
                error_message: None,
                file_context: None,
                suggestion: None,
            }),
            Err(e) => {
                // Try to get detailed error info
                let error_msg = e.to_string();
                Ok(ArchiveValidationResult {
                    is_valid: false,
                    error_message: Some(error_msg.clone()),
                    file_context: Some(archive_path.clone()),
                    suggestion: Some(match error_msg.as_str() {
                        msg if msg.contains("CRC") => {
                            "Archive may be corrupted. Try repair_7z_archive.".to_string()
                        }
                        msg if msg.contains("password") => {
                            "Archive requires password or password is incorrect.".to_string()
                        }
                        msg if msg.contains("header") => {
                            "Archive headers are damaged. Try repair_7z_archive.".to_string()
                        }
                        _ => "Archive validation failed. Check file integrity.".to_string(),
                    }),
                })
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CompressionLevel mapping ====================

    /// Helper to replicate the match logic from compress_to_lzma/lzma2 commands
    fn map_compression_level(level: u8) -> CompressionLevel {
        match level {
            0 => CompressionLevel::Store,
            1 => CompressionLevel::Fastest,
            2..=3 => CompressionLevel::Fast,
            4..=6 => CompressionLevel::Normal,
            7..=8 => CompressionLevel::Maximum,
            9 => CompressionLevel::Ultra,
            _ => CompressionLevel::Normal,
        }
    }

    #[test]
    fn test_compression_level_store() {
        assert!(matches!(map_compression_level(0), CompressionLevel::Store));
    }

    #[test]
    fn test_compression_level_fastest() {
        assert!(matches!(
            map_compression_level(1),
            CompressionLevel::Fastest
        ));
    }

    #[test]
    fn test_compression_level_fast() {
        assert!(matches!(map_compression_level(2), CompressionLevel::Fast));
        assert!(matches!(map_compression_level(3), CompressionLevel::Fast));
    }

    #[test]
    fn test_compression_level_normal() {
        assert!(matches!(map_compression_level(4), CompressionLevel::Normal));
        assert!(matches!(map_compression_level(5), CompressionLevel::Normal));
        assert!(matches!(map_compression_level(6), CompressionLevel::Normal));
    }

    #[test]
    fn test_compression_level_maximum() {
        assert!(matches!(
            map_compression_level(7),
            CompressionLevel::Maximum
        ));
        assert!(matches!(
            map_compression_level(8),
            CompressionLevel::Maximum
        ));
    }

    #[test]
    fn test_compression_level_ultra() {
        assert!(matches!(map_compression_level(9), CompressionLevel::Ultra));
    }

    #[test]
    fn test_compression_level_out_of_range_defaults_to_normal() {
        assert!(matches!(
            map_compression_level(10),
            CompressionLevel::Normal
        ));
        assert!(matches!(
            map_compression_level(255),
            CompressionLevel::Normal
        ));
    }

    // ==================== ArchiveValidationResult serialization ====================

    #[test]
    fn test_validation_result_valid_serialization() {
        let result = ArchiveValidationResult {
            is_valid: true,
            error_message: None,
            file_context: None,
            suggestion: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isValid\":true"));
        assert!(json.contains("\"errorMessage\":null"));
        assert!(json.contains("\"suggestion\":null"));
        assert!(!json.contains("is_valid")); // camelCase enforced
    }

    #[test]
    fn test_validation_result_invalid_serialization() {
        let result = ArchiveValidationResult {
            is_valid: false,
            error_message: Some("CRC mismatch in file header".to_string()),
            file_context: Some("/tmp/test.7z".to_string()),
            suggestion: Some("Archive may be corrupted. Try repair_7z_archive.".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"isValid\":false"));
        assert!(json.contains("CRC mismatch"));
        assert!(json.contains("\"fileContext\":"));
        assert!(json.contains("\"suggestion\":"));
    }
}
