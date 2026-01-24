// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for performance profiling
//!
//! Provides frontend access to CPU profiling, flamegraph generation,
//! and performance analysis capabilities.

use crate::common::profiler::{
    clear_session_history, get_completed_sessions, get_profiling_stats, is_profiling_active,
    start_profiling, start_profiling_with_config, stop_profiling,
    stop_profiling_with_flamegraph, ProfileReport, ProfilingConfig, ProfilingStats,
};
use tauri::command;

/// Start a new profiling session
///
/// # Arguments
///
/// * `name` - Name for this profiling session
/// * `frequency_hz` - Sampling frequency (10-1000 Hz recommended)
///
/// # Example (TypeScript)
///
/// ```typescript
/// await invoke("profiler_start", { name: "extraction", frequencyHz: 100 });
/// ```
#[command]
pub async fn profiler_start(name: String, frequency_hz: i32) -> Result<(), String> {
    start_profiling(name, frequency_hz).map_err(|e| e.to_string())
}

/// Start profiling with custom configuration
///
/// # Example (TypeScript)
///
/// ```typescript
/// await invoke("profiler_start_custom", {
///   name: "heavy_operation",
///   config: { frequencyHz: 1000, includeBlockedTime: true, minSamples: 5 }
/// });
/// ```
#[command]
pub async fn profiler_start_custom(
    name: String,
    config: ProfilingConfigInput,
) -> Result<(), String> {
    let config = ProfilingConfig {
        frequency_hz: config.frequency_hz,
        include_blocked_time: config.include_blocked_time.unwrap_or(false),
        min_samples: config.min_samples.unwrap_or(1),
    };
    start_profiling_with_config(name, config).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingConfigInput {
    pub frequency_hz: i32,
    pub include_blocked_time: Option<bool>,
    pub min_samples: Option<usize>,
}

/// Stop the active profiling session and return the report
///
/// # Example (TypeScript)
///
/// ```typescript
/// const report = await invoke<ProfileReport>("profiler_stop");
/// console.log(`Collected ${report.sampleCount} samples`);
/// console.log(`Top function: ${report.topFunctions[0].name}`);
/// ```
#[command]
pub async fn profiler_stop() -> Result<ProfileReport, String> {
    stop_profiling().map_err(|e| e.to_string())
}

/// Stop profiling and generate a flamegraph SVG
///
/// # Example (TypeScript)
///
/// ```typescript
/// const report = await invoke<ProfileReport>("profiler_stop_with_flamegraph");
/// if (report.flamegraphSvg) {
///   // Display the flamegraph
///   document.getElementById("flamegraph").innerHTML = report.flamegraphSvg;
/// }
/// ```
#[command]
pub async fn profiler_stop_with_flamegraph() -> Result<ProfileReport, String> {
    stop_profiling_with_flamegraph().map_err(|e| e.to_string())
}

/// Check if profiling is currently active
///
/// # Example (TypeScript)
///
/// ```typescript
/// const active = await invoke<boolean>("profiler_is_active");
/// ```
#[command]
pub async fn profiler_is_active() -> Result<bool, String> {
    Ok(is_profiling_active())
}

/// Get statistics about profiling sessions
///
/// # Example (TypeScript)
///
/// ```typescript
/// const stats = await invoke<ProfilingStats>("profiler_get_stats");
/// console.log(`Active: ${stats.isActive}`);
/// console.log(`Completed sessions: ${stats.completedSessionCount}`);
/// ```
#[command]
pub async fn profiler_get_stats() -> Result<ProfilingStats, String> {
    Ok(get_profiling_stats())
}

/// Get all completed profiling sessions
///
/// # Example (TypeScript)
///
/// ```typescript
/// const sessions = await invoke<ProfileReport[]>("profiler_get_history");
/// for (const session of sessions) {
///   console.log(`${session.name}: ${session.durationSecs}s`);
/// }
/// ```
#[command]
pub async fn profiler_get_history() -> Result<Vec<ProfileReport>, String> {
    Ok(get_completed_sessions())
}

/// Clear the profiling session history
///
/// # Example (TypeScript)
///
/// ```typescript
/// await invoke("profiler_clear_history");
/// ```
#[command]
pub async fn profiler_clear_history() -> Result<(), String> {
    clear_session_history();
    Ok(())
}

/// Get hot path analysis from the latest report
///
/// # Arguments
///
/// * `limit` - Maximum number of hot paths to return
///
/// # Example (TypeScript)
///
/// ```typescript
/// const hotPaths = await invoke<FunctionSample[]>("profiler_get_hot_paths", { limit: 10 });
/// ```
#[command]
pub async fn profiler_get_hot_paths(limit: usize) -> Result<Vec<FunctionSample>, String> {
    let sessions = get_completed_sessions();
    if let Some(latest) = sessions.last() {
        Ok(latest
            .top_functions
            .iter()
            .take(limit)
            .map(|f| FunctionSample {
                name: f.name.clone(),
                sample_count: f.sample_count,
                percentage: f.percentage,
            })
            .collect())
    } else {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSample {
    pub name: String,
    pub sample_count: usize,
    pub percentage: f64,
}

/// Get a summary of profiling recommendations
///
/// # Example (TypeScript)
///
/// ```typescript
/// const summary = await invoke<ProfileSummary>("profiler_get_summary");
/// console.log(summary.recommendation);
/// ```
#[command]
pub async fn profiler_get_summary() -> Result<ProfileSummary, String> {
    let stats = get_profiling_stats();
    let sessions = get_completed_sessions();

    let avg_duration = if !sessions.is_empty() {
        sessions.iter().map(|s| s.duration_secs).sum::<f64>() / sessions.len() as f64
    } else {
        0.0
    };

    let avg_samples = if !sessions.is_empty() {
        sessions.iter().map(|s| s.sample_count).sum::<usize>() / sessions.len()
    } else {
        0
    };

    let recommendation = if stats.is_active {
        "Profiling is currently active. Stop it to see results.".to_string()
    } else if sessions.is_empty() {
        "No profiling sessions recorded yet. Start a session with profiler_start().".to_string()
    } else {
        format!(
            "Average session duration: {:.2}s with {} samples. Review hot paths for optimization opportunities.",
            avg_duration, avg_samples
        )
    };

    Ok(ProfileSummary {
        stats,
        avg_duration_secs: avg_duration,
        avg_sample_count: avg_samples,
        recommendation,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSummary {
    pub stats: ProfilingStats,
    pub avg_duration_secs: f64,
    pub avg_sample_count: usize,
    pub recommendation: String,
}
