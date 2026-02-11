// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Advanced Performance Profiling
//!
//! Provides production-grade CPU profiling, memory tracking, and performance analysis
//! with minimal overhead using sample-based profiling.
//!
//! ## Features
//!
//! - **CPU Profiling**: Sample-based profiling with flamegraph generation
//! - **Hot Path Detection**: Identify frequently executed code paths
//! - **Call Stack Analysis**: Track function call hierarchies
//! - **Low Overhead**: ~1-2% performance impact when enabled
//!
//! ## Usage
//!
//! ```no_run
//! use ffx_check_lib::common::profiler::{start_profiling, stop_profiling};
//!
//! // Start profiling
//! start_profiling("operation_name", 100).expect("start profiling"); // 100 Hz sampling
//!
//! // ... do work ...
//!
//! // Stop and get report
//! let report = stop_profiling().expect("stop profiling");
//! println!("Profile report:\n{}", report.summary());
//! ```

use parking_lot::RwLock;
use pprof::ProfilerGuard;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Instant;

#[cfg(test)]
use std::time::Duration;

/// Global profiler state
static PROFILER_STATE: LazyLock<Arc<RwLock<ProfilerState>>> =
    LazyLock::new(|| Arc::new(RwLock::new(ProfilerState::new())));

/// Profiler error types
#[derive(Debug, thiserror::Error)]
pub enum ProfilerError {
    #[error("Profiler error: {0}")]
    Error(String),
    #[error("Profiling not active")]
    NotActive,
    #[error("Profiling already active")]
    AlreadyActive,
    #[error("Failed to generate flamegraph: {0}")]
    FlamegraphError(String),
    #[error("Failed to build report: {0}")]
    ReportError(String),
}

pub type ProfilerResult<T> = Result<T, ProfilerError>;

/// Profiling session configuration
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Sampling frequency in Hz (default: 100)
    pub frequency_hz: i32,
    /// Whether to include blocked time (default: false)
    pub include_blocked_time: bool,
    /// Minimum sample count to include in report (default: 1)
    pub min_samples: usize,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 100,
            include_blocked_time: false,
            min_samples: 1,
        }
    }
}

impl ProfilingConfig {
    pub fn new(frequency_hz: i32) -> Self {
        Self {
            frequency_hz,
            ..Default::default()
        }
    }

    pub fn high_frequency() -> Self {
        Self::new(1000) // 1kHz - very detailed but higher overhead
    }

    pub fn normal() -> Self {
        Self::new(100) // 100Hz - good balance
    }

    pub fn low_overhead() -> Self {
        Self::new(10) // 10Hz - minimal impact
    }
}

/// Active profiling session
struct ProfileSession {
    name: String,
    guard: ProfilerGuard<'static>,
    start_time: Instant,
    config: ProfilingConfig,
}

/// Profiler state management
struct ProfilerState {
    active_session: Option<ProfileSession>,
    completed_sessions: Vec<ProfileReport>,
}

impl ProfilerState {
    fn new() -> Self {
        Self {
            active_session: None,
            completed_sessions: Vec::new(),
        }
    }

    fn is_active(&self) -> bool {
        self.active_session.is_some()
    }
}

/// Profile report with analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileReport {
    pub name: String,
    pub duration_secs: f64,
    pub sample_count: usize,
    pub top_functions: Vec<FunctionSample>,
    pub flamegraph_svg: Option<String>,
}

impl ProfileReport {
    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut s = format!(
            "Profile: {}\nDuration: {:.2}s\nSamples: {}\n\nTop Functions:\n",
            self.name, self.duration_secs, self.sample_count
        );

        for (i, func) in self.top_functions.iter().take(10).enumerate() {
            s.push_str(&format!(
                "  {}. {} - {:.1}% ({} samples)\n",
                i + 1,
                func.name,
                func.percentage,
                func.sample_count
            ));
        }

        s
    }
}

/// Function sample statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSample {
    pub name: String,
    pub sample_count: usize,
    pub percentage: f64,
}

// =============================================================================
// Public API
// =============================================================================

/// Start a profiling session
///
/// # Arguments
///
/// * `name` - Name for this profiling session
/// * `frequency_hz` - Sampling frequency (higher = more detail, more overhead)
///
/// # Example
///
/// ```no_run
/// use ffx_check_lib::common::profiler::start_profiling;
///
/// start_profiling("hash_operation", 100).expect("start profiling");
/// // ... do work ...
/// ```
pub fn start_profiling(name: impl Into<String>, frequency_hz: i32) -> ProfilerResult<()> {
    let mut state = PROFILER_STATE.write();

    if state.is_active() {
        return Err(ProfilerError::AlreadyActive);
    }

    let config = ProfilingConfig::new(frequency_hz);

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(config.frequency_hz)
        .blocklist(&["libc", "libsystem", "libpthread"])
        .build()
        .map_err(|e| ProfilerError::Error(e.to_string()))?;

    state.active_session = Some(ProfileSession {
        name: name.into(),
        guard,
        start_time: Instant::now(),
        config,
    });

    Ok(())
}

/// Start profiling with custom configuration
pub fn start_profiling_with_config(
    name: impl Into<String>,
    config: ProfilingConfig,
) -> ProfilerResult<()> {
    let mut state = PROFILER_STATE.write();

    if state.is_active() {
        return Err(ProfilerError::AlreadyActive);
    }

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(config.frequency_hz)
        .blocklist(&["libc", "libsystem", "libpthread"])
        .build()
        .map_err(|e| ProfilerError::Error(e.to_string()))?;

    state.active_session = Some(ProfileSession {
        name: name.into(),
        guard,
        start_time: Instant::now(),
        config,
    });

    Ok(())
}

/// Stop the active profiling session and generate a report
pub fn stop_profiling() -> ProfilerResult<ProfileReport> {
    let mut state = PROFILER_STATE.write();

    let session = state
        .active_session
        .take()
        .ok_or(ProfilerError::NotActive)?;

    let duration = session.start_time.elapsed();

    // Build the profile report
    let pprof_report = session
        .guard
        .report()
        .build()
        .map_err(|e| ProfilerError::ReportError(e.to_string()))?;

    // Extract top functions
    let mut function_counts: HashMap<String, usize> = HashMap::new();
    let mut total_samples = 0usize;

    // Iterate through samples to count function occurrences
    for (key, count) in pprof_report.data.iter() {
        let count = *count as usize;  // Convert isize to usize
        total_samples += count;
        for frame in key.frames.iter() {
            for symbol in frame.iter() {
                let name = symbol.name();
                *function_counts.entry(name).or_insert(0) += count;
            }
        }
    }

    // Sort by sample count
    let mut top_functions: Vec<_> = function_counts
        .into_iter()
        .filter(|(_, count)| *count >= session.config.min_samples)
        .map(|(name, count)| FunctionSample {
            name,
            sample_count: count,
            percentage: if total_samples > 0 {
                (count as f64 / total_samples as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    top_functions.sort_by(|a, b| b.sample_count.cmp(&a.sample_count));

    let report = ProfileReport {
        name: session.name,
        duration_secs: duration.as_secs_f64(),
        sample_count: total_samples,
        top_functions,
        flamegraph_svg: None, // Generated separately if needed
    };

    // Keep history of completed sessions
    state.completed_sessions.push(report.clone());

    Ok(report)
}

/// Stop profiling and export flamegraph SVG
pub fn stop_profiling_with_flamegraph() -> ProfilerResult<ProfileReport> {
    let mut state = PROFILER_STATE.write();

    let session = state
        .active_session
        .take()
        .ok_or(ProfilerError::NotActive)?;

    let duration = session.start_time.elapsed();

    // Build the profile report
    let pprof_report = session
        .guard
        .report()
        .build()
        .map_err(|e| ProfilerError::ReportError(e.to_string()))?;

    // Generate flamegraph
    let mut flamegraph_data = Vec::new();
    pprof_report
        .flamegraph(&mut flamegraph_data)
        .map_err(|e| ProfilerError::FlamegraphError(e.to_string()))?;

    let flamegraph_svg = String::from_utf8(flamegraph_data)
        .map_err(|e| ProfilerError::FlamegraphError(e.to_string()))?;

    // Extract top functions (same as stop_profiling)
    let mut function_counts: HashMap<String, usize> = HashMap::new();
    let mut total_samples = 0usize;

    for (key, count) in pprof_report.data.iter() {
        let count = *count as usize;  // Convert isize to usize
        total_samples += count;
        for frame in key.frames.iter() {
            for symbol in frame.iter() {
                let name = symbol.name();
                *function_counts.entry(name).or_insert(0) += count;
            }
        }
    }

    let mut top_functions: Vec<_> = function_counts
        .into_iter()
        .filter(|(_, count)| *count >= session.config.min_samples)
        .map(|(name, count)| FunctionSample {
            name,
            sample_count: count,
            percentage: if total_samples > 0 {
                (count as f64 / total_samples as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();

    top_functions.sort_by(|a, b| b.sample_count.cmp(&a.sample_count));

    let report = ProfileReport {
        name: session.name,
        duration_secs: duration.as_secs_f64(),
        sample_count: total_samples,
        top_functions,
        flamegraph_svg: Some(flamegraph_svg),
    };

    state.completed_sessions.push(report.clone());

    Ok(report)
}

/// Check if profiling is currently active
pub fn is_profiling_active() -> bool {
    PROFILER_STATE.read().is_active()
}

/// Get history of completed profiling sessions
pub fn get_completed_sessions() -> Vec<ProfileReport> {
    PROFILER_STATE.read().completed_sessions.clone()
}

/// Clear completed session history
pub fn clear_session_history() {
    PROFILER_STATE.write().completed_sessions.clear();
}

/// Get profiling statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilingStats {
    pub is_active: bool,
    pub active_session_name: Option<String>,
    pub active_duration_secs: Option<f64>,
    pub completed_session_count: usize,
    pub total_samples_collected: usize,
}

pub fn get_profiling_stats() -> ProfilingStats {
    let state = PROFILER_STATE.read();

    let (is_active, active_session_name, active_duration_secs) = if let Some(session) = &state.active_session {
        (
            true,
            Some(session.name.clone()),
            Some(session.start_time.elapsed().as_secs_f64()),
        )
    } else {
        (false, None, None)
    };

    let total_samples_collected = state
        .completed_sessions
        .iter()
        .map(|s| s.sample_count)
        .sum();

    ProfilingStats {
        is_active,
        active_session_name,
        active_duration_secs,
        completed_session_count: state.completed_sessions.len(),
        total_samples_collected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Mutex;

    // Global test mutex to prevent parallel test execution (tests share PROFILER_STATE)
    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn test_profiling_lifecycle() {
        let _lock = TEST_MUTEX.lock().unwrap();
        // Clear any previous state
        clear_session_history();

        assert!(!is_profiling_active());

        start_profiling("unit_test_session", 100).unwrap();
        assert!(is_profiling_active());

        // Do meaningful work
        for _ in 0..100 {
            let _ = (0..10000u64).fold(0u64, |acc, x| acc.wrapping_add(x));
        }
        thread::sleep(Duration::from_millis(50));

        let report = stop_profiling().unwrap();
        assert!(!is_profiling_active());
        assert_eq!(report.name, "unit_test_session");
        // Don't assert on sample_count - depends on system load
    }

    #[test]
    fn test_cannot_start_twice() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_session_history();

        start_profiling("session1", 100).unwrap();
        let result = start_profiling("session2", 100);
        assert!(result.is_err());

        stop_profiling().unwrap();
    }

    #[test]
    fn test_profiling_stats() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_session_history();

        let stats = get_profiling_stats();
        assert!(!stats.is_active);
        assert_eq!(stats.completed_session_count, 0);
    }
}
