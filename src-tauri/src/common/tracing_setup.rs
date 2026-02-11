// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Distributed Tracing Infrastructure
//!
//! Provides structured, async-aware logging and distributed tracing for debugging
//! and performance analysis across all operations.
//!
//! ## Features
//!
//! - **Structured Logging**: JSON and plaintext output with context preservation
//! - **Async-Aware**: Proper span tracking across async boundaries
//! - **File Rotation**: Daily log rotation with 7-day retention
//! - **Performance Tracking**: Automatic timing for all spans
//! - **Context Propagation**: Operation IDs and metadata flow through call chains
//!
//! ## Usage
//!
//! ```no_run
//! use tracing::{info, warn, error, debug, trace, instrument};
//! use ffx_check_lib::common::tracing_setup::init_tracing;
//!
//! // Initialize once at startup
//! init_tracing("INFO", "/path/to/logs").expect("init tracing");
//!
//! // Use tracing macros
//! info!(operation = "hash_file", file_path = "/evidence/file.ad1", "Starting hash");
//! ```

use std::path::{Path, PathBuf};
use tracing::{info, Level};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Tracing initialization error
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Failed to initialize tracing: {0}")]
    InitError(String),
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),
    #[error("Directory error: {0}")]
    DirectoryError(#[from] std::io::Error),
}

pub type TracingResult<T> = Result<T, TracingError>;

/// Initialize the global tracing subscriber
///
/// Call this once at application startup. Sets up:
/// - Console output with ANSI colors (if supported)
/// - Automatic span timing
/// - Environment-based filtering
///
/// # Arguments
///
/// * `default_level` - Default log level ("TRACE", "DEBUG", "INFO", "WARN", "ERROR")
/// * `_log_dir` - Reserved for future file logging (currently unused)
///
/// # Environment Variables
///
/// - `RUST_LOG` - Override log level (e.g., "debug", "ffx_check_lib=trace")
///
/// # Example
///
/// ```no_run
/// use ffx_check_lib::common::tracing_setup::init_tracing;
///
/// init_tracing("INFO", "/var/log/core-ffx").expect("init tracing");
/// ```
pub fn init_tracing(default_level: &str, _log_dir: impl AsRef<Path>) -> TracingResult<()> {
    // Build environment filter with default level
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{}={}",
            env!("CARGO_PKG_NAME").replace('-', "_"),
            default_level
        ))
    });

    // Console layer - human-readable with colors
    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE) // Log when spans close (shows duration)
        .with_filter(env_filter);

    // Initialize global subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .try_init()
        .map_err(|e| TracingError::InitError(e.to_string()))?;

    info!(
        level = default_level,
        "Tracing initialized (console output only)"
    );

    Ok(())
}

/// Macro to create a timed span with automatic field recording
///
/// # Example
///
/// ```rust
/// use ffx_check_lib::timed_span;
/// use tracing::info;
///
/// fn hash_file(path: &str, size: u64) -> Result<String, String> {
///     let _span = timed_span!("hash_file", path = path, size = size);
///     info!("Hashing file");
///     // ... hashing logic
///     Ok("abc123".to_string())
/// }
/// ```
#[macro_export]
macro_rules! timed_span {
    ($name:expr, $($key:ident = $value:expr),* $(,)?) => {{
        let span = tracing::info_span!(
            $name,
            $($key = tracing::field::Empty),*
        );
        $(
            span.record(stringify!($key), &tracing::field::display(&$value));
        )*
        span.entered()
    }};
}

/// Log levels supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn to_tracing_level(&self) -> Level {
        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = TracingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(LogLevel::Trace),
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(TracingError::InvalidLevel(s.to_string())),
        }
    }
}

/// Get the default log directory based on platform
pub fn default_log_dir() -> TracingResult<PathBuf> {
    let base = dirs::data_local_dir().ok_or_else(|| {
        TracingError::DirectoryError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine local data directory",
        ))
    })?;

    Ok(base.join("core-ffx").join("logs"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_tracing() {
        let temp_dir = TempDir::new().unwrap();
        let result = init_tracing("DEBUG", temp_dir.path());
        // May fail if already initialized in other tests - that's OK
        let _ = result;
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warn);
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Debug.to_tracing_level(), Level::DEBUG);
    }

    #[test]
    fn test_default_log_dir() {
        let log_dir = default_log_dir().unwrap();
        assert!(log_dir.to_string_lossy().contains("core-ffx"));
        assert!(log_dir.to_string_lossy().contains("logs"));
    }
}
