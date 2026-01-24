// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for memory profiling
//!
//! Provides frontend access to memory allocation tracking, leak detection,
//! and heap analysis capabilities.

use crate::common::memory_profiler::{
    clear_snapshots, detect_leaks, format_bytes, get_current_memory_usage, get_memory_stats,
    get_snapshots, is_tracking_active, start_tracking, stop_tracking, take_snapshot, LeakReport,
    MemorySnapshot, MemoryStats,
};
use tauri::command;

/// Start memory tracking
///
/// # Example (TypeScript)
///
/// ```typescript
/// await invoke("memory_start_tracking");
/// ```
#[command]
pub async fn memory_start_tracking() -> Result<(), String> {
    start_tracking();
    Ok(())
}

/// Stop memory tracking and return statistics
///
/// # Example (TypeScript)
///
/// ```typescript
/// const stats = await invoke<MemoryStats>("memory_stop_tracking");
/// console.log(`Peak memory: ${stats.peakBytes} bytes`);
/// ```
#[command]
pub async fn memory_stop_tracking() -> Result<MemoryStats, String> {
    Ok(stop_tracking())
}

/// Check if memory tracking is active
///
/// # Example (TypeScript)
///
/// ```typescript
/// const active = await invoke<boolean>("memory_is_active");
/// ```
#[command]
pub async fn memory_is_active() -> Result<bool, String> {
    Ok(is_tracking_active())
}

/// Take a memory snapshot
///
/// # Example (TypeScript)
///
/// ```typescript
/// const snapshot = await invoke<MemorySnapshot>("memory_take_snapshot", {
///   name: "after_extraction"
/// });
/// console.log(`Memory at ${snapshot.name}: ${snapshot.currentBytes} bytes`);
/// ```
#[command]
pub async fn memory_take_snapshot(name: String) -> Result<MemorySnapshot, String> {
    Ok(take_snapshot(name))
}

/// Get current memory statistics
///
/// # Example (TypeScript)
///
/// ```typescript
/// const stats = await invoke<MemoryStats>("memory_get_stats");
/// console.log(`Current: ${stats.currentBytes}, Peak: ${stats.peakBytes}`);
/// ```
#[command]
pub async fn memory_get_stats() -> Result<MemoryStats, String> {
    get_memory_stats().map_err(|e| e.to_string())
}

/// Get all memory snapshots
///
/// # Example (TypeScript)
///
/// ```typescript
/// const snapshots = await invoke<MemorySnapshot[]>("memory_get_snapshots");
/// snapshots.forEach(s => console.log(`${s.name}: ${s.currentBytes} bytes`));
/// ```
#[command]
pub async fn memory_get_snapshots() -> Result<Vec<MemorySnapshot>, String> {
    Ok(get_snapshots())
}

/// Clear all memory snapshots
///
/// # Example (TypeScript)
///
/// ```typescript
/// await invoke("memory_clear_snapshots");
/// ```
#[command]
pub async fn memory_clear_snapshots() -> Result<(), String> {
    clear_snapshots();
    Ok(())
}

/// Detect potential memory leaks
///
/// # Example (TypeScript)
///
/// ```typescript
/// const report = await invoke<LeakReport>("memory_detect_leaks");
/// console.log(`Total leaked: ${report.totalLeakedBytes} bytes`);
/// report.potentialLeaks.forEach(leak => {
///   console.log(`  ${leak.name}: ${leak.bytes} bytes`);
/// });
/// ```
#[command]
pub async fn memory_detect_leaks() -> Result<LeakReport, String> {
    detect_leaks().map_err(|e| e.to_string())
}

/// Get current process memory usage
///
/// # Example (TypeScript)
///
/// ```typescript
/// const bytes = await invoke<number>("memory_get_current_usage");
/// console.log(`Current memory usage: ${bytes} bytes`);
/// ```
#[command]
pub async fn memory_get_current_usage() -> Result<u64, String> {
    Ok(get_current_memory_usage())
}

/// Format bytes as human-readable string
///
/// # Example (TypeScript)
///
/// ```typescript
/// const formatted = await invoke<string>("memory_format_bytes", { bytes: 5242880 });
/// console.log(formatted); // "5.00 MB"
/// ```
#[command]
pub async fn memory_format_bytes(bytes: u64) -> Result<String, String> {
    Ok(format_bytes(bytes))
}

/// Get memory profiling summary
///
/// # Example (TypeScript)
///
/// ```typescript
/// const summary = await invoke<MemorySummary>("memory_get_summary");
/// console.log(summary.recommendation);
/// ```
#[command]
pub async fn memory_get_summary() -> Result<MemorySummary, String> {
    let is_active = is_tracking_active();
    let current_usage = get_current_memory_usage();
    let snapshots = get_snapshots();

    let stats = if is_active {
        Some(get_memory_stats().map_err(|e| e.to_string())?)
    } else {
        None
    };

    let recommendation = if is_active {
        format!(
            "Memory tracking active. Current usage: {}. Take snapshots at key points for analysis.",
            format_bytes(current_usage)
        )
    } else if !snapshots.is_empty() {
        format!(
            "Tracking inactive. {} snapshots available. Use memory_detect_leaks() for analysis.",
            snapshots.len()
        )
    } else {
        "No memory tracking data. Start tracking with memory_start_tracking().".to_string()
    };

    Ok(MemorySummary {
        is_active,
        current_usage_bytes: current_usage,
        current_usage_formatted: format_bytes(current_usage),
        snapshot_count: snapshots.len(),
        stats,
        recommendation,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySummary {
    pub is_active: bool,
    pub current_usage_bytes: u64,
    pub current_usage_formatted: String,
    pub snapshot_count: usize,
    pub stats: Option<MemoryStats>,
    pub recommendation: String,
}
