// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED extraction operations
//!
//! Provides extraction of UFED container contents to output directories,
//! with support for ZIP, UFD, UFDR, and UFDX formats.

use std::path::Path;

use tracing::{debug, instrument};

use crate::containers::ContainerError;
use super::detection;
use super::types::UfedFormat;

/// Result from UFED extraction operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedExtractResult {
    /// Number of files extracted
    pub files_extracted: u64,
    /// Number of directories created
    pub directories_created: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// List of extraction errors
    pub errors: Vec<UfedExtractError>,
    /// Whether the extraction was successful
    pub success: bool,
}

/// Individual extraction error
#[derive(Debug, Clone, serde::Serialize)]
pub struct UfedExtractError {
    /// Path that failed to extract
    pub path: String,
    /// Error message
    pub error: String,
}

/// Extract UFED container contents to output directory
///
/// For ZIP containers, extracts the archive contents.
/// For UFD files, copies the file and any associated files.
/// For UFDR containers, copies the data to the output directory.
#[instrument]
pub fn extract(
    path: &str,
    output_dir: &str,
) -> Result<UfedExtractResult, ContainerError> {
    extract_with_progress(path, output_dir, |_, _| {})
}

/// Extract UFED container with progress callback
#[instrument(skip(progress_callback))]
pub fn extract_with_progress<F>(
    path: &str,
    output_dir: &str,
    mut progress_callback: F,
) -> Result<UfedExtractResult, ContainerError>
where
    F: FnMut(u64, u64),
{
    debug!(path = %path, output_dir = %output_dir, "Extracting UFED container");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "UFED file not found: {path}"
        )));
    }

    let format = detection::detect_format(path)
        .ok_or_else(|| format!("Not a recognized UFED format: {path}"))?;

    let output_path = Path::new(output_dir);
    std::fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;

    match format {
        UfedFormat::UfedZip => {
            // Extract ZIP using archive module
            let result = crate::archive::extract(path, output_dir)?;
            Ok(UfedExtractResult {
                files_extracted: result.files_extracted,
                directories_created: result.directories_created,
                bytes_written: result.bytes_written,
                errors: result
                    .failed_files
                    .into_iter()
                    .map(|e| UfedExtractError {
                        path: e.path,
                        error: e.error,
                    })
                    .collect(),
                success: result.success,
            })
        }
        UfedFormat::Ufd | UfedFormat::Ufdr | UfedFormat::Ufdx => {
            // Copy the file and associated files
            let info = super::info(path)?;
            let total_size =
                info.size + info.associated_files.iter().map(|f| f.size).sum::<u64>();
            let mut bytes_written = 0u64;
            let mut files_extracted = 0u64;
            let mut errors = Vec::new();

            // Copy main file
            let main_name = path_obj
                .file_name()
                .ok_or_else(|| "Invalid file path".to_string())?;
            let dest_main = output_path.join(main_name);

            if let Err(e) = std::fs::copy(path_obj, &dest_main) {
                errors.push(UfedExtractError {
                    path: path.to_string(),
                    error: e.to_string(),
                });
            } else {
                bytes_written += info.size;
                files_extracted += 1;
                progress_callback(bytes_written, total_size);
            }

            // Copy associated files - construct full path from parent dir + filename
            let parent_dir = path_obj.parent().unwrap_or(path_obj);
            for assoc in &info.associated_files {
                let assoc_path = parent_dir.join(&assoc.filename);
                let dest = output_path.join(&assoc.filename);
                if let Err(e) = std::fs::copy(&assoc_path, &dest) {
                    errors.push(UfedExtractError {
                        path: assoc.filename.clone(),
                        error: e.to_string(),
                    });
                } else {
                    bytes_written += assoc.size;
                    files_extracted += 1;
                    progress_callback(bytes_written, total_size);
                }
            }

            Ok(UfedExtractResult {
                files_extracted,
                directories_created: 0,
                bytes_written,
                errors: errors.clone(),
                success: errors.is_empty(),
            })
        }
    }
}

/// Get paths of all segments in a UFED extraction set
///
/// For UFED containers, segments are the associated files that make up
/// the complete extraction set.
#[instrument]
pub fn get_segment_paths(path: &str) -> Result<Vec<std::path::PathBuf>, ContainerError> {
    debug!(path = %path, "Getting UFED segment paths");

    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(ContainerError::FileNotFound(format!(
            "UFED file not found: {path}"
        )));
    }

    let info = super::info(path)?;
    let mut segments = vec![path_obj.to_path_buf()];

    // Add associated files as "segments" - construct full path from parent dir + filename
    let parent_dir = path_obj.parent().unwrap_or(path_obj);
    for assoc in info.associated_files {
        segments.push(parent_dir.join(&assoc.filename));
    }

    Ok(segments)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nonexistent_file() {
        let result = extract("/nonexistent/path/file.ufd", "/tmp/output");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_segment_paths_nonexistent() {
        let result = get_segment_paths("/nonexistent/path/file.ufd");
        assert!(result.is_err());
    }

    #[test]
    fn test_ufed_extract_result() {
        let result = UfedExtractResult {
            files_extracted: 5,
            directories_created: 2,
            bytes_written: 10000,
            errors: vec![],
            success: true,
        };

        assert_eq!(result.files_extracted, 5);
        assert!(result.success);
    }
}
