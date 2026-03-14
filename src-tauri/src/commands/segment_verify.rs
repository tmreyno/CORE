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

use std::path::PathBuf;
use tauri::Emitter;
use tracing::{debug, info};

use crate::common::segment_hash;
use crate::common::segments;

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

    // Discover all segment files
    let segment_paths = discover_segments(&path)?;
    let segment_count = segment_paths.len();

    if segment_count == 0 {
        return Err("No segment files found".to_string());
    }

    info!(
        path,
        segment_count,
        algorithm,
        hash_combined,
        hash_individual,
        "Starting post-acquisition segment verification"
    );

    // Calculate total size
    let total_bytes: u64 = segment_paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .sum();

    let algo = algorithm.clone();
    let paths_clone = segment_paths.clone();
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
                    // Determine which segment we're in
                    let mut cumulative = 0u64;
                    let mut seg_idx = 0;
                    for p in &paths_clone {
                        let seg_size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
                        cumulative += seg_size;
                        if bytes_done <= cumulative {
                            break;
                        }
                        seg_idx += 1;
                    }
                    if seg_idx != current_seg {
                        current_seg = seg_idx;
                    }

                    let percent = if bytes_total > 0 {
                        (bytes_done as f64 / bytes_total as f64) * 100.0
                    } else {
                        0.0
                    };

                    let _ = window_clone.emit(
                        "segment-hash-progress",
                        SegmentHashProgress {
                            current_segment: current_seg.min(segment_count - 1) + 1,
                            total_segments: segment_count,
                            bytes_hashed: bytes_done,
                            total_bytes: bytes_total,
                            percent,
                            phase: "combined".to_string(),
                        },
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
                let seg_size = std::fs::metadata(seg_path).map(|m| m.len()).unwrap_or(0);
                let seg_path_str = seg_path.to_string_lossy().to_string();

                let hash = segment_hash::hash_segment_with_progress(
                    &seg_path_str,
                    &algo,
                    |bytes_done, _bytes_total| {
                        let overall_done: u64 = paths_clone[..i]
                            .iter()
                            .map(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
                            .sum::<u64>()
                            + bytes_done;

                        let percent = if total_bytes > 0 {
                            (overall_done as f64 / total_bytes as f64) * 100.0
                        } else {
                            0.0
                        };

                        let _ = window_clone.emit(
                            "segment-hash-progress",
                            SegmentHashProgress {
                                current_segment: i + 1,
                                total_segments: segment_count,
                                bytes_hashed: overall_done,
                                total_bytes,
                                percent,
                                phase: "individual".to_string(),
                            },
                        );
                    },
                )
                .map_err(|e| format!("Segment {} hash failed: {}", i + 1, e))?;

                let seg_name = seg_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

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
