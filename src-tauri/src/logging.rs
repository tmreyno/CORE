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

use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

/// Initialize the logging/tracing system
/// 
/// Call this once at application startup (in main.rs)
pub fn init() {
    // Build filter from environment or use defaults
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default: info in release, debug in debug builds
            if cfg!(debug_assertions) {
                EnvFilter::new("ffx_check=debug,ffx_check_lib=debug")
            } else {
                EnvFilter::new("ffx_check=info,ffx_check_lib=info")
            }
        });
    
    // Configure the subscriber
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)      // Show module path
                .with_thread_ids(false) // Hide thread IDs (cleaner)
                .with_file(false)       // Hide file:line in normal mode
                .with_line_number(false)
                .compact()              // Compact format
        );
    
    // Set as global default (ignore error if already set)
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// Initialize logging with verbose output (file:line, thread IDs)
/// Useful for debugging during development
pub fn init_verbose() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("trace"));
    
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .pretty()  // Pretty multi-line format
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

// Re-export tracing macros for convenience
pub use tracing::{debug, error, info, trace, warn, instrument, span, Level as LogLevel};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init() {
        init();
        info!("Test log message");
        debug!(key = "value", "Structured log");
    }
}
