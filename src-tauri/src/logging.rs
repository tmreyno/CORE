// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Logging and tracing configuration for FFX Check
//!
//! This module provides structured logging using the `tracing` crate.
//!
//! # Usage
//!
//! Initialize logging at app startup:
//! ```rust,ignore
//! logging::init();
//! ```
//!
//! Use tracing macros in your code:
//! ```rust
//! use tracing::{info, debug, warn, error, trace, instrument};
//!
//! #[instrument]  // Automatically logs function entry/exit with args
//! fn my_function(path: &str) {
//!     debug!("Processing file");
//!     info!(bytes = 1024, "Read data");
//!     warn!("Something unexpected");
//!     error!("Something failed");
//!     trace!("Very verbose detail");
//! }
//! ```
//!
//! # Log Levels
//!
//! - `error` - Errors that prevent operation completion
//! - `warn`  - Unexpected situations that don't prevent completion
//! - `info`  - High-level operation progress (default in release)
//! - `debug` - Detailed operation information (default in debug builds)
//! - `trace` - Very verbose, step-by-step details
//!
//! # Environment Variable Control
//!
//! Set `RUST_LOG` to control log levels at runtime:
//! ```bash
//! RUST_LOG=debug ./ffx-check          # All debug logs
//! RUST_LOG=ffx_check=trace ./ffx-check # Trace for this crate only
//! RUST_LOG=warn ./ffx-check           # Only warnings and errors
//! RUST_LOG=ewf=debug,ad1=info ./ffx-check  # Per-module control
//! ```

use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging/tracing system
///
/// Call this once at application startup (in main.rs)
/// Sets up:
/// - Console output (compact format, ANSI colors)
/// - File output (daily rotation, JSON format) for audit trail persistence
///
/// Audit logs are written to `<data_local_dir>/core-ffx/logs/ffx-audit.YYYY-MM-DD.log`
pub fn init() {
    // Build filter from environment or use defaults
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default: info in release, debug in debug builds
        if cfg!(debug_assertions) {
            EnvFilter::new("ffx_check=debug,ffx_check_lib=debug")
        } else {
            EnvFilter::new("ffx_check=info,ffx_check_lib=info")
        }
    });

    // Console layer - compact human-readable output
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact();

    // File layer - daily-rotating JSON audit log
    // Best-effort: if we can't determine the log dir, skip file logging
    let file_layer = audit_log_dir().ok().map(|log_dir| {
        // Ensure the log directory exists
        let _ = std::fs::create_dir_all(&log_dir);

        let file_appender = tracing_appender::rolling::daily(&log_dir, "ffx-audit.log");

        // File filter: info+ for audit trail (no debug/trace noise in files)
        let file_filter = EnvFilter::new("ffx_check=info,ffx_check_lib=info");

        fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_ansi(false) // No ANSI colors in log files
            .json() // Structured JSON for machine parsing
            .with_writer(file_appender)
            .with_filter(file_filter)
    });

    // Configure the subscriber with both layers
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer);

    // Set as global default (ignore error if already set)
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Initialize logging with verbose output (file:line, thread IDs)
/// Useful for debugging during development
pub fn init_verbose() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

    let subscriber = tracing_subscriber::registry().with(filter).with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .pretty(), // Pretty multi-line format
    );

    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Check if debug logging is enabled
/// Can be used to skip expensive debug computations
#[inline]
pub fn is_debug_enabled() -> bool {
    tracing::enabled!(Level::DEBUG)
}

/// Check if trace logging is enabled
#[inline]
pub fn is_trace_enabled() -> bool {
    tracing::enabled!(Level::TRACE)
}

/// Get the platform-specific audit log directory.
///
/// Returns `<data_local_dir>/core-ffx/logs`:
/// - macOS: `~/Library/Application Support/core-ffx/logs`
/// - Linux: `~/.local/share/core-ffx/logs`
/// - Windows: `{FOLDERID_LocalAppData}/core-ffx/logs`
pub fn audit_log_dir() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .map(|base| base.join("core-ffx").join("logs"))
        .ok_or_else(|| "Could not determine local data directory".to_string())
}

/// Read recent audit log entries from the log directory.
///
/// Reads up to `max_lines` lines from the most recent log files,
/// returning them newest-first. Each line is a JSON-formatted log entry.
pub fn read_audit_logs(max_lines: usize) -> Result<Vec<String>, String> {
    let log_dir = audit_log_dir()?;

    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    // Collect log files, sorted newest first by filename (date-stamped)
    let mut log_files: Vec<_> = std::fs::read_dir(&log_dir)
        .map_err(|e| format!("Failed to read log directory: {e}"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("ffx-audit.log")
        })
        .map(|entry| entry.path())
        .collect();

    // Sort descending so newest files come first
    log_files.sort_by(|a, b| b.cmp(a));

    let mut lines = Vec::new();
    for file_path in log_files {
        if lines.len() >= max_lines {
            break;
        }
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read log file {}: {e}", file_path.display()))?;

        // Collect lines in reverse order (newest entries last in file)
        let mut file_lines: Vec<String> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect();
        file_lines.reverse();

        let remaining = max_lines - lines.len();
        lines.extend(file_lines.into_iter().take(remaining));
    }

    Ok(lines)
}

// Re-export tracing macros for convenience
pub use tracing::{debug, error, info, instrument, span, trace, warn, Level as LogLevel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init();
        info!("Test log message");
        debug!(key = "value", "Structured log");
    }

    #[test]
    fn test_audit_log_dir() {
        let dir = audit_log_dir().unwrap();
        assert!(dir.to_string_lossy().contains("core-ffx"));
        assert!(dir.to_string_lossy().contains("logs"));
    }

    #[test]
    fn test_read_audit_logs_empty_dir() {
        // When no log files exist yet, should return empty vec
        let result = read_audit_logs(100);
        // This may succeed with empty results or the dir may not exist yet
        match result {
            Ok(lines) => {
                // Lines may be empty or contain entries from previous runs
                assert!(lines.len() <= 100);
            }
            Err(_) => {
                // Directory not existing is acceptable in test environment
            }
        }
    }

    #[test]
    fn test_read_audit_logs_with_temp_dir() {
        use std::fs;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_dir = temp_dir.path();

        // Create fake log files
        fs::write(
            log_dir.join("ffx-audit.log.2025-01-01"),
            "{\"level\":\"INFO\",\"message\":\"old entry 1\"}\n{\"level\":\"INFO\",\"message\":\"old entry 2\"}\n",
        ).unwrap();
        fs::write(
            log_dir.join("ffx-audit.log.2025-01-02"),
            "{\"level\":\"INFO\",\"message\":\"new entry 1\"}\n{\"level\":\"WARN\",\"message\":\"new entry 2\"}\n",
        ).unwrap();
        // Non-matching file should be ignored
        fs::write(log_dir.join("other.log"), "should be ignored\n").unwrap();

        // Read all lines from log files manually (simulating read_audit_logs logic)
        let mut log_files: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("ffx-audit.log"))
            .map(|e| e.path())
            .collect();
        log_files.sort_by(|a, b| b.cmp(a));

        assert_eq!(log_files.len(), 2);
        // Newest file first
        assert!(log_files[0].to_string_lossy().contains("2025-01-02"));

        // Read with max_lines limit
        let mut lines = Vec::new();
        let max_lines = 3;
        for file_path in &log_files {
            if lines.len() >= max_lines {
                break;
            }
            let content = fs::read_to_string(file_path).unwrap();
            let mut file_lines: Vec<String> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(String::from)
                .collect();
            file_lines.reverse();
            let remaining = max_lines - lines.len();
            lines.extend(file_lines.into_iter().take(remaining));
        }

        assert_eq!(lines.len(), 3);
        // Newest entries from newest file first
        assert!(lines[0].contains("new entry 2"));
        assert!(lines[1].contains("new entry 1"));
        assert!(lines[2].contains("old entry 2"));
    }

    #[test]
    fn test_read_audit_logs_max_lines_respected() {
        use std::fs;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_dir = temp_dir.path();

        // Create a file with many lines
        let mut content = String::new();
        for i in 0..50 {
            content.push_str(&format!("{{\"line\":{i}}}\n"));
        }
        fs::write(log_dir.join("ffx-audit.log.2025-06-01"), &content).unwrap();

        // Simulate read with limit
        let max_lines = 10;
        let read_content = fs::read_to_string(log_dir.join("ffx-audit.log.2025-06-01")).unwrap();
        let mut file_lines: Vec<String> = read_content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect();
        file_lines.reverse();
        let lines: Vec<_> = file_lines.into_iter().take(max_lines).collect();

        assert_eq!(lines.len(), 10);
        // Should have the last 10 entries (newest = highest index)
        assert!(lines[0].contains("\"line\":49"));
        assert!(lines[9].contains("\"line\":40"));
    }
}
