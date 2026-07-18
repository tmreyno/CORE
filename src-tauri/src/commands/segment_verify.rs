// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Post-acquisition segment verification commands.
//!
//! Hashes created container segments (L01/L02/..., E01/E02/...) to verify
//! integrity after imaging. Supports:
//! - Combined hash: all segments fed into one hasher → single container hash
//! - Individual hashes: each segment file hashed independently

use std::path::{Path, PathBuf};
use tauri::Emitter;
use tracing::{debug, info};

use crate::common::segment_hash;
use crate::common::segments;
use crate::common::HashAlgorithm;

const MAX_SEGMENT_VERIFY_SEGMENTS: usize = 10_000;

// =============================================================================
// Types
// =============================================================================

/// Progress event emitted during segment hashing.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentHashProgress {
    /// Current segment being hashed (1-based)
    pub current_segment: usize,
    /// Total segments
    pub total_segments: usize,
    /// Bytes hashed so far across all segments
    pub bytes_hashed: u64,
    /// Total bytes across all segments
    pub total_bytes: u64,
    /// Percentage (0–100)
    pub percent: f64,
    /// Phase: "combined" or "individual"
    pub phase: String,
}

/// Hash result for a single segment file.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentHashResult {
    /// Segment file name
    pub segment_name: String,
    /// Full path
    pub segment_path: String,
    /// Segment number (1-based)
    pub segment_number: usize,
    /// Hash algorithm used
    pub algorithm: String,
    /// Hex-encoded hash value
    pub hash: String,
    /// File size in bytes
    pub size: u64,
}

/// Full result from segment verification.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentVerifyResult {
    /// Combined hash across all segments (if requested)
    pub combined_hash: Option<String>,
    /// Algorithm used for combined hash
    pub combined_algorithm: Option<String>,
    /// Individual per-segment hashes (if requested)
    pub segment_hashes: Vec<SegmentHashResult>,
    /// Total segments found
    pub segment_count: usize,
    /// Total bytes hashed
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

// =============================================================================
// Commands
// =============================================================================

/// Hash container segments after acquisition.
///
/// Given the primary output file path, discovers all segments and computes:
/// - A combined hash (all segments as one continuous stream)
/// - Individual hashes per segment file (optional)
#[tauri::command]
pub async fn hash_container_segments(
    path: String,
    algorithm: String,
    hash_combined: bool,
    hash_individual: bool,
    window: tauri::Window,
) -> Result<SegmentVerifyResult, String> {
    let start = std::time::Instant::now();
    let algo = validate_segment_verify_request(&algorithm, hash_combined, hash_individual)?;

    // Discover all segment files
    let segment_paths = discover_segments(&path)?;
    ensure_segment_count_within_limit(segment_paths.len())?;
    let segment_count = segment_paths.len();

    if segment_count == 0 {
        return Err("No segment files found".to_string());
    }

    info!(
        path,
        segment_count,
        algorithm = %algo,
        hash_combined,
        hash_individual,
        "Starting post-acquisition segment verification"
    );

    let segment_sizes = segment_sizes(&segment_paths)?;
    let total_bytes = total_segment_bytes(&segment_sizes)?;

    let paths_clone = segment_paths.clone();
    let sizes_clone = segment_sizes.clone();
    let window_clone = window.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut combined_hash: Option<String> = None;
        let mut segment_hashes: Vec<SegmentHashResult> = Vec::new();

        // Phase 1: Combined hash
        if hash_combined {
            debug!("Computing combined hash across {} segments", segment_count);
            let mut current_seg = 0usize;

            let hash = segment_hash::hash_segments_combined(
                &paths_clone,
                &algo,
                |bytes_done, bytes_total| {
                    let seg_idx = current_segment_index(bytes_done, &sizes_clone);
                    if seg_idx != current_seg {
                        current_seg = seg_idx;
                    }

                    let percent = if bytes_total > 0 {
                        ((bytes_done as f64 / bytes_total as f64) * 100.0).min(100.0)
                    } else {
                        0.0
                    };

                    crate::eventing::log_emit_result(
                        "segment-hash-progress",
                        window_clone.emit(
                            "segment-hash-progress",
                            SegmentHashProgress {
                                current_segment: current_seg.min(segment_count - 1) + 1,
                                total_segments: segment_count,
                                bytes_hashed: bytes_done,
                                total_bytes: bytes_total,
                                percent,
                                phase: "combined".to_string(),
                            },
                        ),
                    );
                },
            )
            .map_err(|e| format!("Combined hash failed: {}", e))?;

            info!(hash = %hash, "Combined segment hash computed");
            combined_hash = Some(hash);
        }

        // Phase 2: Individual segment hashes
        if hash_individual {
            debug!("Computing individual hashes for {} segments", segment_count);

            for (i, seg_path) in paths_clone.iter().enumerate() {
                let seg_size = sizes_clone[i];
                let seg_path_str = seg_path.to_string_lossy().to_string();

                let hash = segment_hash::hash_segment_with_progress(
                    &seg_path_str,
                    &algo,
                    |bytes_done, _bytes_total| {
                        let overall_done =
                            bytes_before_segment(&sizes_clone, i).saturating_add(bytes_done);

                        let percent = if total_bytes > 0 {
                            ((overall_done as f64 / total_bytes as f64) * 100.0).min(100.0)
                        } else {
                            0.0
                        };

                        crate::eventing::log_emit_result(
                            "segment-hash-progress",
                            window_clone.emit(
                                "segment-hash-progress",
                                SegmentHashProgress {
                                    current_segment: i + 1,
                                    total_segments: segment_count,
                                    bytes_hashed: overall_done,
                                    total_bytes,
                                    percent,
                                    phase: "individual".to_string(),
                                },
                            ),
                        );
                    },
                )
                .map_err(|e| format!("Segment {} hash failed: {}", i + 1, e))?;

                let seg_name = segment_result_name(seg_path)?;

                segment_hashes.push(SegmentHashResult {
                    segment_name: seg_name,
                    segment_path: seg_path_str,
                    segment_number: i + 1,
                    algorithm: algo.clone(),
                    hash,
                    size: seg_size,
                });
            }
        }

        Ok::<_, String>(SegmentVerifyResult {
            combined_hash,
            combined_algorithm: if hash_combined { Some(algo) } else { None },
            segment_hashes,
            segment_count,
            total_bytes,
            duration_ms: 0, // Set below
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))??;

    let duration_ms = start.elapsed().as_millis() as u64;
    let mut final_result = result;
    final_result.duration_ms = duration_ms;

    info!(
        duration_ms,
        segment_count,
        combined = final_result.combined_hash.is_some(),
        individual = final_result.segment_hashes.len(),
        "Segment verification complete"
    );

    Ok(final_result)
}

// =============================================================================
// Helpers
// =============================================================================

fn segment_sizes(paths: &[PathBuf]) -> Result<Vec<u64>, String> {
    paths
        .iter()
        .map(|path| {
            std::fs::metadata(path)
                .map(|metadata| metadata.len())
                .map_err(|e| format!("Failed to get segment metadata {}: {}", path.display(), e))
        })
        .collect()
}

fn total_segment_bytes(sizes: &[u64]) -> Result<u64, String> {
    sizes.iter().try_fold(0u64, |total, size| {
        total
            .checked_add(*size)
            .ok_or_else(|| "Segment byte total exceeds u64".to_string())
    })
}

fn validate_segment_verify_request(
    algorithm: &str,
    hash_combined: bool,
    hash_individual: bool,
) -> Result<String, String> {
    if !hash_combined && !hash_individual {
        return Err(
            "Segment verification requires hashCombined, hashIndividual, or both".to_string(),
        );
    }

    let algorithm = algorithm
        .parse::<HashAlgorithm>()
        .map_err(|e| e.to_string())?;
    Ok(algorithm.name().to_string())
}

fn ensure_segment_count_within_limit(segment_count: usize) -> Result<(), String> {
    if segment_count > MAX_SEGMENT_VERIFY_SEGMENTS {
        Err(format!(
            "Segment verification discovered {segment_count} segments, exceeding limit {MAX_SEGMENT_VERIFY_SEGMENTS}"
        ))
    } else {
        Ok(())
    }
}

fn bytes_before_segment(sizes: &[u64], index: usize) -> u64 {
    sizes
        .iter()
        .take(index)
        .fold(0u64, |total, size| total.saturating_add(*size))
}

fn segment_result_name(path: &Path) -> Result<String, String> {
    let Some(name) = path.file_name().filter(|name| !name.is_empty()) else {
        return Err(format!(
            "Segment path has no file name for verification result: {}",
            path.display()
        ));
    };
    Ok(name.to_string_lossy().to_string())
}

fn current_segment_index(bytes_done: u64, sizes: &[u64]) -> usize {
    if sizes.is_empty() {
        return 0;
    }

    let mut cumulative = 0u64;
    for (index, size) in sizes.iter().enumerate() {
        cumulative = cumulative.saturating_add(*size);
        if bytes_done <= cumulative {
            return index;
        }
    }

    sizes.len() - 1
}

/// Discover segments for any container type based on the primary file extension.
fn discover_segments(path: &str) -> Result<Vec<PathBuf>, String> {
    let lower = path.to_lowercase();

    if lower.ends_with(".l01") || lower.ends_with(".l02") {
        segments::discover_l01_segments(path)
            .map_err(|e| format!("L01 segment discovery failed: {}", e))
    } else if lower.ends_with(".e01") || lower.ends_with(".e02") || lower.ends_with(".ex01") {
        segments::discover_e01_segments(path)
            .map_err(|e| format!("E01 segment discovery failed: {}", e))
    } else if lower.ends_with(".ad1") || lower.ends_with(".ad2") {
        // AD1: pass expected_count=999 to discover all that exist
        let (paths, _, _) = segments::discover_ad1_segments(path, 999);
        if paths.is_empty() {
            // Single file
            let p = PathBuf::from(path);
            if p.exists() {
                Ok(vec![p])
            } else {
                Err(format!("File not found: {}", path))
            }
        } else {
            Ok(paths)
        }
    } else {
        // Numbered segments (.001, .002, etc.) or single file
        let (paths, _) = segments::discover_numbered_segments(path)
            .map_err(|e| format!("Segment discovery failed: {}", e))?;
        if paths.is_empty() {
            let p = PathBuf::from(path);
            if p.exists() {
                Ok(vec![p])
            } else {
                Err(format!("File not found: {}", path))
            }
        } else {
            Ok(paths)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn segment_sizes_fail_on_missing_segment_metadata() {
        let mut segment = NamedTempFile::new().unwrap();
        segment.write_all(b"segment").unwrap();
        segment.flush().unwrap();

        let missing = segment.path().with_extension("missing");
        let err = segment_sizes(&[segment.path().to_path_buf(), missing]).unwrap_err();

        assert!(err.contains("Failed to get segment metadata"));
    }

    #[test]
    fn total_segment_bytes_rejects_overflowing_totals() {
        let sizes = [u64::MAX - 5, 10, 20];

        let err = total_segment_bytes(&sizes).unwrap_err();
        assert!(err.contains("exceeds u64"));
    }

    #[test]
    fn segment_size_helpers_preserve_representable_totals() {
        let sizes = [10, 20, 30];

        assert_eq!(total_segment_bytes(&sizes).unwrap(), 60);
        assert_eq!(bytes_before_segment(&sizes, 2), 30);
    }

    #[test]
    fn segment_result_name_rejects_path_without_file_name() {
        let err = segment_result_name(Path::new("/")).unwrap_err();

        assert!(err.contains("has no file name"));
    }

    #[test]
    fn segment_result_name_uses_terminal_file_name() {
        assert_eq!(
            segment_result_name(Path::new("/tmp/evidence.E01")).unwrap(),
            "evidence.E01"
        );
    }

    #[test]
    fn current_segment_index_handles_boundaries_and_overrun() {
        let sizes = [10, 20, 30];

        assert_eq!(current_segment_index(0, &sizes), 0);
        assert_eq!(current_segment_index(10, &sizes), 0);
        assert_eq!(current_segment_index(11, &sizes), 1);
        assert_eq!(current_segment_index(30, &sizes), 1);
        assert_eq!(current_segment_index(31, &sizes), 2);
        assert_eq!(current_segment_index(999, &sizes), 2);
        assert_eq!(current_segment_index(999, &[]), 0);
    }

    #[test]
    fn validate_segment_verify_request_rejects_empty_work() {
        let err = validate_segment_verify_request("sha256", false, false).unwrap_err();

        assert!(err.contains("requires hashCombined"));
    }

    #[test]
    fn validate_segment_verify_request_normalizes_algorithm() {
        let algo = validate_segment_verify_request("sha-256", true, false).unwrap();

        assert_eq!(algo, "SHA-256");
    }

    #[test]
    fn validate_segment_verify_request_rejects_unknown_algorithm() {
        let err = validate_segment_verify_request("rot13", true, false).unwrap_err();

        assert!(err.contains("Unsupported hash algorithm"));
    }

    #[test]
    fn ensure_segment_count_within_limit_rejects_excessive_segments() {
        let err = ensure_segment_count_within_limit(MAX_SEGMENT_VERIFY_SEGMENTS + 1).unwrap_err();

        assert!(err.contains("exceeding limit"));
    }
}
