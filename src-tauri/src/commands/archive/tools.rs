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

const ARCHIVE_TOOL_BUFFER_MAX_BYTES: usize = 64 * 1024 * 1024;
const ARCHIVE_TOOL_ENCRYPTED_BUFFER_MAX_BYTES: usize = ARCHIVE_TOOL_BUFFER_MAX_BYTES + 4096;
const ARCHIVE_TOOL_FIELD_MAX_CHARS: usize = 4096;
const ARCHIVE_TOOL_TEXT_MAX_CHARS: usize = 16_384;
const ARCHIVE_TOOL_TRUNCATED_SUFFIX: &str = "... [truncated]";

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
        crate::eventing::log_emit_result(
            "archive-test-progress",
            window_clone.emit(
                "archive-test-progress",
                serde_json::json!({
                    "archive_path": archive_path_clone,
                    "status": "Testing archive integrity...",
                    "percent": 0.0,
                }),
            ),
        );

        // Test archive
        sz.test_archive(
            &archive_path_clone,
            password.as_deref(),
            None, // No progress callback for now
        )
        .map_err(|e| format!("Archive test failed: {}", e))?;

        // Emit completion
        crate::eventing::log_emit_result(
            "archive-test-progress",
            window_clone.emit(
                "archive-test-progress",
                serde_json::json!({
                    "archive_path": archive_path_clone,
                    "status": "Archive is valid",
                    "percent": 100.0,
                }),
            ),
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
        crate::eventing::log_emit_result(
            "archive-repair-progress",
            window_clone.emit(
                "archive-repair-progress",
                serde_json::json!({
                    "percent": 0.0,
                    "status": "Repairing archive...",
                }),
            ),
        );

        // Repair archive
        sz.repair_archive(
            &corrupted_path,
            &repaired_clone,
            None, // No progress callback for now
        )
        .map_err(|e| format!("Archive repair failed: {}", e))?;

        // Emit completion
        crate::eventing::log_emit_result(
            "archive-repair-progress",
            window_clone.emit(
                "archive-repair-progress",
                serde_json::json!({
                    "percent": 100.0,
                    "status": "Archive repaired successfully",
                }),
            ),
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
                "message": truncate_archive_tool_text(&err.message, ARCHIVE_TOOL_TEXT_MAX_CHARS),
                "file_context": truncate_archive_tool_text(&err.file_context, ARCHIVE_TOOL_FIELD_MAX_CHARS),
                "position": err.position,
                "suggestion": truncate_archive_tool_text(&err.suggestion, ARCHIVE_TOOL_TEXT_MAX_CHARS),
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
    checked_archive_tool_buffer_len(data.len(), "Plaintext data")?;
    let password = truncate_archive_tool_text(&password, ARCHIVE_TOOL_FIELD_MAX_CHARS);
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;

    let encrypted = ctx
        .encrypt(&data)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    checked_archive_tool_encrypted_buffer_len(encrypted.len(), "Encrypted data")?;
    Ok(encrypted)
}

/// Decrypt data using native Rust AES-256
#[tauri::command]
pub fn decrypt_data_native(encrypted_data: Vec<u8>, password: String) -> Result<Vec<u8>, String> {
    checked_archive_tool_encrypted_buffer_len(encrypted_data.len(), "Encrypted data")?;
    let password = truncate_archive_tool_text(&password, ARCHIVE_TOOL_FIELD_MAX_CHARS);
    let mut ctx = EncryptionContext::new(&password)
        .map_err(|e| format!("Failed to initialize decryption: {}", e))?;

    let decrypted = ctx
        .decrypt(&encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    checked_archive_tool_buffer_len(decrypted.len(), "Decrypted data")?;
    Ok(decrypted)
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
        crate::eventing::log_emit_result(
            "split-extract-progress",
            window_clone.emit(
                "split-extract-progress",
                serde_json::json!({
                    "status": "Extracting split archive...",
                    "percent": 0.0,
                }),
            ),
        );

        // Extract split archive (simple version without progress callback)
        advanced::extract_split_archive(&first_volume_path, &output_dir, password.as_deref())
            .map_err(|e| format!("Split archive extraction failed: {}", e))?;

        // Emit completion
        crate::eventing::log_emit_result(
            "split-extract-progress",
            window_clone.emit(
                "split-extract-progress",
                serde_json::json!({
                    "status": "Split archive extracted successfully",
                    "percent": 100.0,
                }),
            ),
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
                Ok(bounded_archive_validation_result(ArchiveValidationResult {
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
                }))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

fn checked_archive_tool_buffer_len(len: usize, context: &str) -> Result<(), String> {
    if len > ARCHIVE_TOOL_BUFFER_MAX_BYTES {
        return Err(format!(
            "{context} is too large: {len} bytes > {ARCHIVE_TOOL_BUFFER_MAX_BYTES} bytes"
        ));
    }
    Ok(())
}

fn checked_archive_tool_encrypted_buffer_len(len: usize, context: &str) -> Result<(), String> {
    if len > ARCHIVE_TOOL_ENCRYPTED_BUFFER_MAX_BYTES {
        return Err(format!(
            "{context} is too large: {len} bytes > {ARCHIVE_TOOL_ENCRYPTED_BUFFER_MAX_BYTES} bytes"
        ));
    }
    Ok(())
}

fn bounded_archive_validation_result(
    mut result: ArchiveValidationResult,
) -> ArchiveValidationResult {
    result.error_message = result
        .error_message
        .map(|value| truncate_archive_tool_text(&value, ARCHIVE_TOOL_TEXT_MAX_CHARS));
    result.file_context = result
        .file_context
        .map(|value| truncate_archive_tool_text(&value, ARCHIVE_TOOL_FIELD_MAX_CHARS));
    result.suggestion = result
        .suggestion
        .map(|value| truncate_archive_tool_text(&value, ARCHIVE_TOOL_TEXT_MAX_CHARS));
    result
}

fn truncate_archive_tool_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep_chars = max_chars.saturating_sub(ARCHIVE_TOOL_TRUNCATED_SUFFIX.chars().count());
    let mut truncated: String = value.chars().take(keep_chars).collect();
    truncated.push_str(ARCHIVE_TOOL_TRUNCATED_SUFFIX);
    truncated
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

    #[test]
    fn checked_archive_tool_buffer_len_allows_limit() {
        assert!(checked_archive_tool_buffer_len(ARCHIVE_TOOL_BUFFER_MAX_BYTES, "data").is_ok());
    }

    #[test]
    fn checked_archive_tool_buffer_len_rejects_oversized_plaintext() {
        let err =
            checked_archive_tool_buffer_len(ARCHIVE_TOOL_BUFFER_MAX_BYTES + 1, "Plaintext data")
                .unwrap_err();

        assert!(err.contains("Plaintext data is too large"));
    }

    #[test]
    fn checked_archive_tool_encrypted_buffer_len_allows_overhead_limit() {
        assert!(checked_archive_tool_encrypted_buffer_len(
            ARCHIVE_TOOL_ENCRYPTED_BUFFER_MAX_BYTES,
            "Encrypted data",
        )
        .is_ok());
    }

    #[test]
    fn checked_archive_tool_encrypted_buffer_len_rejects_oversized_ciphertext() {
        let err = checked_archive_tool_encrypted_buffer_len(
            ARCHIVE_TOOL_ENCRYPTED_BUFFER_MAX_BYTES + 1,
            "Encrypted data",
        )
        .unwrap_err();

        assert!(err.contains("Encrypted data is too large"));
    }

    #[test]
    fn bounded_archive_validation_result_caps_strings() {
        let long = "x".repeat(ARCHIVE_TOOL_TEXT_MAX_CHARS + 8);
        let long_field = "y".repeat(ARCHIVE_TOOL_FIELD_MAX_CHARS + 8);
        let result = bounded_archive_validation_result(ArchiveValidationResult {
            is_valid: false,
            error_message: Some(long.clone()),
            file_context: Some(long_field),
            suggestion: Some(long),
        });

        assert_eq!(
            result.error_message.unwrap().chars().count(),
            ARCHIVE_TOOL_TEXT_MAX_CHARS
        );
        assert_eq!(
            result.file_context.unwrap().chars().count(),
            ARCHIVE_TOOL_FIELD_MAX_CHARS
        );
        assert!(result
            .suggestion
            .unwrap()
            .ends_with(ARCHIVE_TOOL_TRUNCATED_SUFFIX));
    }

    #[test]
    fn truncate_archive_tool_text_preserves_short_text() {
        assert_eq!(truncate_archive_tool_text("short", 16), "short");
    }
}
