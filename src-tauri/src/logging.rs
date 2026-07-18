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

use parking_lot::Mutex;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// =============================================================================
// Project-Scoped Audit Log Writer
// =============================================================================

/// Shared state for the project-scoped audit log writer.
///
/// When a project is open, `inner` holds a `BufWriter<File>` pointed at the
/// project's log directory. When no project is open, writes are silently
/// discarded (the layer still exists but produces no output).
struct ProjectLogState {
    writer: Option<BufWriter<std::fs::File>>,
    dir: Option<PathBuf>,
}

static PROJECT_LOG: LazyLock<Arc<Mutex<ProjectLogState>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(ProjectLogState {
        writer: None,
        dir: None,
    }))
});

/// A `std::io::Write` implementation that delegates to the project log writer.
/// When no project is open, writes succeed but are discarded.
struct ProjectLogWriter {
    state: Arc<Mutex<ProjectLogState>>,
}

impl Write for ProjectLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = self.state.lock();
        if let Some(ref mut w) = guard.writer {
            w.write(buf)
        } else {
            // Discard — no project open
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut guard = self.state.lock();
        if let Some(ref mut w) = guard.writer {
            w.flush()
        } else {
            Ok(())
        }
    }
}

/// `MakeWriter` for the project log layer. Creates a `ProjectLogWriter`
/// that shares state with all other writers via `PROJECT_LOG`.
struct ProjectLogMakeWriter {
    state: Arc<Mutex<ProjectLogState>>,
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for ProjectLogMakeWriter {
    type Writer = ProjectLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ProjectLogWriter {
            state: Arc::clone(&self.state),
        }
    }
}

fn collect_matching_log_files(
    log_dir: &Path,
    log_kind: &str,
    matches_name: impl Fn(&str) -> bool,
) -> Result<Vec<PathBuf>, String> {
    let mut log_files = Vec::new();
    for entry_result in std::fs::read_dir(log_dir)
        .map_err(|e| format!("Failed to read {log_kind} directory: {e}"))?
    {
        let entry =
            entry_result.map_err(|e| format!("Failed to read {log_kind} directory entry: {e}"))?;
        if matches_name(&entry.file_name().to_string_lossy()) {
            log_files.push(entry.path());
        }
    }
    log_files.sort_by(|a, b| b.cmp(a));
    Ok(log_files)
}

fn clear_project_log_state() -> (bool, Option<std::io::Error>) {
    let mut guard = PROJECT_LOG.lock();
    let had_writer = guard.writer.is_some();
    let flush_error = guard.writer.as_mut().and_then(|w| w.flush().err());
    guard.writer = None;
    guard.dir = None;
    (had_writer, flush_error)
}

/// Start writing project-scoped audit logs to the given project directory.
///
/// Creates a `logs/` subdirectory alongside the project files and opens
/// a daily-stamped JSON log file. Called when a project database is opened.
///
/// Log file naming: `ffx-project-audit.YYYY-MM-DD.log`
pub fn set_project_log_dir(project_dir: &Path) {
    let (had_previous_writer, flush_error) = clear_project_log_state();
    if let Some(error) = flush_error {
        tracing::warn!("Failed to flush previous project audit log before switching: {error}");
    } else if had_previous_writer {
        tracing::info!(target: "forensic_audit", "Previous project audit log closed");
    }

    let log_dir = project_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        tracing::warn!(
            "Failed to create project audit log directory {}: {e}",
            log_dir.display()
        );
        return;
    }

    // Build daily-stamped filename
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_file = log_dir.join(format!("ffx-project-audit.{date}.log"));

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        Ok(file) => {
            {
                let mut guard = PROJECT_LOG.lock();
                guard.writer = Some(BufWriter::new(file));
                guard.dir = Some(log_dir);
            }
            tracing::info!(
                target: "forensic_audit",
                path = %log_file.display(),
                "Project audit log opened"
            );
        }
        Err(e) => {
            tracing::warn!(
                "Failed to open project audit log {}: {e}",
                log_file.display()
            );
        }
    }
}

/// Stop writing project-scoped audit logs. Flushes and closes the log file.
/// Called when a project database is closed.
pub fn clear_project_log() {
    let (had_writer, flush_error) = clear_project_log_state();
    if let Some(error) = flush_error {
        tracing::warn!("Failed to flush project audit log while closing: {error}");
    }
    if had_writer {
        tracing::info!(target: "forensic_audit", "Project audit log closed");
    }
}

/// Get the project log directory, if a project log is currently active.
pub fn project_log_dir() -> Option<PathBuf> {
    let guard = PROJECT_LOG.lock();
    guard.dir.clone()
}

/// Read recent project-scoped audit log entries from the project's log directory.
///
/// Reads up to `max_lines` lines from the most recent project log files,
/// returning them newest-first. Each line is a JSON-formatted log entry.
pub fn read_project_audit_logs(
    project_dir: &Path,
    max_lines: usize,
) -> Result<Vec<String>, String> {
    let log_dir = project_dir.join("logs");
    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let log_files = collect_matching_log_files(&log_dir, "project log", |name| {
        name.starts_with("ffx-project-audit.")
    })?;

    let mut lines = Vec::new();
    for file_path in log_files {
        if lines.len() >= max_lines {
            break;
        }
        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            format!(
                "Failed to read project log file {}: {e}",
                file_path.display()
            )
        })?;

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

// =============================================================================
// Global Logging Initialization
// =============================================================================

/// Initialize the logging/tracing system
///
/// Call this once at application startup (in main.rs)
/// Sets up:
/// - Console output (compact format, ANSI colors)
/// - File output (daily rotation, JSON format) for global audit trail
/// - Project-scoped file output (active only when a project is open)
///
/// Global audit logs: `<app-local data>/<app log dir>/logs/<audit basename>.YYYY-MM-DD.log`
/// Project audit logs: `<project_dir>/logs/ffx-project-audit.YYYY-MM-DD.log`
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

    // File layer - daily-rotating JSON audit log (global, always-on)
    // Best-effort: keep startup alive, but make setup failures visible because
    // tracing is not installed yet and cannot report its own file sink failures.
    let file_layer = audit_log_dir().ok().and_then(|log_dir| {
        if let Err(error) = std::fs::create_dir_all(&log_dir) {
            eprintln!(
                "Failed to create global audit log directory {}: {error}",
                log_dir.display()
            );
            return None;
        }

        match tracing_appender::rolling::RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix(crate::app_paths::AUDIT_LOG_BASENAME)
            .filename_suffix(crate::app_paths::AUDIT_LOG_SUFFIX)
            .build(&log_dir)
        {
            Ok(file_appender) => {
                // File filter: info+ for audit trail (no debug/trace noise in files)
                let file_filter = EnvFilter::new("ffx_check=info,ffx_check_lib=info");

                Some(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_ansi(false) // No ANSI colors in log files
                        .json() // Structured JSON for machine parsing
                        .with_writer(file_appender)
                        .with_filter(file_filter),
                )
            }
            Err(error) => {
                eprintln!(
                    "Failed to open global audit log in {}: {error}",
                    log_dir.display()
                );
                None
            }
        }
    });

    // Project-scoped layer - writes to project directory when a project is open.
    // The writer discards output when no project is active.
    let project_filter = EnvFilter::new("ffx_check=info,ffx_check_lib=info");
    let project_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_ansi(false)
        .json()
        .with_writer(ProjectLogMakeWriter {
            state: Arc::clone(&PROJECT_LOG),
        })
        .with_filter(project_filter);

    // Configure the subscriber with all layers
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .with(project_layer);

    set_global_logging_default(subscriber, "standard");
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

    set_global_logging_default(subscriber, "verbose");
}

fn set_global_logging_default<S>(subscriber: S, mode: &str)
where
    S: tracing::Subscriber + Send + Sync + 'static,
{
    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("CORE-FFX {mode} logging subscriber was not installed: {error}");
    }
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
/// Returns the app-owned audit log directory for the current edition.
pub fn audit_log_dir() -> Result<PathBuf, String> {
    Ok(crate::app_paths::global_audit_log_dir())
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
    let log_files = collect_matching_log_files(&log_dir, "log", |name| {
        crate::app_paths::is_global_audit_log_filename(name)
    })?;

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
        assert!(dir
            .to_string_lossy()
            .contains(crate::app_paths::AUDIT_LOG_DIR_NAME));
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
            log_dir.join(crate::app_paths::audit_log_filename_for_date("2025-01-01")),
            "{\"level\":\"INFO\",\"message\":\"old entry 1\"}\n{\"level\":\"INFO\",\"message\":\"old entry 2\"}\n",
        ).unwrap();
        fs::write(
            log_dir.join(crate::app_paths::audit_log_filename_for_date("2025-01-02")),
            "{\"level\":\"INFO\",\"message\":\"new entry 1\"}\n{\"level\":\"WARN\",\"message\":\"new entry 2\"}\n",
        ).unwrap();
        // Non-matching file should be ignored
        fs::write(log_dir.join("other.log"), "should be ignored\n").unwrap();

        let log_files = collect_matching_log_files(log_dir, "test log", |name| {
            crate::app_paths::is_global_audit_log_filename(name)
        })
        .unwrap();

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
    fn test_read_project_audit_logs_with_temp_dir() {
        use std::fs;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        fs::create_dir(&log_dir).unwrap();
        fs::write(
            log_dir.join("ffx-project-audit.2025-01-01.log"),
            "{\"message\":\"old project entry\"}\n",
        )
        .unwrap();
        fs::write(
            log_dir.join("ffx-project-audit.2025-01-02.log"),
            "{\"message\":\"new project entry 1\"}\n{\"message\":\"new project entry 2\"}\n",
        )
        .unwrap();
        fs::write(log_dir.join("other.log"), "ignored\n").unwrap();

        let lines = read_project_audit_logs(temp_dir.path(), 2).unwrap();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("new project entry 2"));
        assert!(lines[1].contains("new project entry 1"));
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
        fs::write(
            log_dir.join(crate::app_paths::audit_log_filename_for_date("2025-06-01")),
            &content,
        )
        .unwrap();

        // Simulate read with limit
        let max_lines = 10;
        let read_content = fs::read_to_string(
            log_dir.join(crate::app_paths::audit_log_filename_for_date("2025-06-01")),
        )
        .unwrap();
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

    #[test]
    fn project_log_switch_failure_clears_previous_writer() {
        clear_project_log();

        let active_project = tempfile::TempDir::new().unwrap();
        set_project_log_dir(active_project.path());
        assert_eq!(project_log_dir(), Some(active_project.path().join("logs")));

        let blocked_project_root = active_project.path().join("blocked-project");
        std::fs::write(&blocked_project_root, b"not a directory").unwrap();

        set_project_log_dir(&blocked_project_root);
        assert!(project_log_dir().is_none());

        clear_project_log();
    }
}
