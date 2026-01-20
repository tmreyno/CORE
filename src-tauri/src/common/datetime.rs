// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Date/Time Utilities
//!
//! Provides standardized date/time formatting functions for consistent output
//! across the application. All timestamps are either UTC (for storage) or
//! local time (for display).
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`now_rfc3339`] | Current UTC timestamp in RFC 3339 format |
//! | [`now_local_display`] | Current local time in human-readable format |
//! | [`format_display`] | Format SystemTime for human-readable display |
//! | [`format_duration`] | Format std::time::Duration as human-readable |
//! | [`parse_rfc3339`] | Parse RFC 3339 string to DateTime<Utc> |
//!
//! # Examples
//!
//! ```rust,ignore
//! use crate::common::datetime::{now_rfc3339, now_local_display, format_duration};
//!
//! // Get current timestamp for storage
//! let timestamp = now_rfc3339(); // "2024-01-15T14:30:00.000Z"
//!
//! // Get current time for display
//! let display = now_local_display(); // "2024-01-15 09:30:00"
//!
//! // Format a duration
//! let elapsed = std::time::Duration::from_secs(3665);
//! let formatted = format_duration(elapsed); // "1h 1m 5s"
//! ```

use chrono::{DateTime, Local, Utc, TimeZone};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// =============================================================================
// Standard Format Strings
// =============================================================================

/// Format string for human-readable display (local time)
pub const DISPLAY_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Format string for human-readable display with timezone
pub const DISPLAY_FORMAT_TZ: &str = "%Y-%m-%d %H:%M:%S %Z";

/// Format string for file-safe timestamps (no colons)
pub const FILE_SAFE_FORMAT: &str = "%Y%m%d_%H%M%S";

// =============================================================================
// Current Time Functions
// =============================================================================

/// Get current UTC timestamp in RFC 3339 format.
///
/// This is the canonical format for storing timestamps in databases,
/// JSON files, and forensic reports. RFC 3339 is a profile of ISO 8601.
///
/// # Returns
///
/// A string like "2024-01-15T14:30:00.000000000+00:00"
///
/// # Example
///
/// ```rust,ignore
/// let timestamp = now_rfc3339();
/// project.created_at = timestamp;
/// ```
#[inline]
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Get current local time in human-readable format.
///
/// This format is suitable for display in the UI and reports.
///
/// # Returns
///
/// A string like "2024-01-15 09:30:00"
///
/// # Example
///
/// ```rust,ignore
/// println!("Report generated at: {}", now_local_display());
/// ```
#[inline]
pub fn now_local_display() -> String {
    Local::now().format(DISPLAY_FORMAT).to_string()
}

/// Get current local time with timezone in human-readable format.
///
/// # Returns
///
/// A string like "2024-01-15 09:30:00 PST"
#[inline]
pub fn now_local_display_tz() -> String {
    Local::now().format(DISPLAY_FORMAT_TZ).to_string()
}

/// Get current timestamp suitable for filenames.
///
/// Returns a format without colons that is safe for all filesystems.
///
/// # Returns
///
/// A string like "20240115_093000"
#[inline]
pub fn now_file_safe() -> String {
    Local::now().format(FILE_SAFE_FORMAT).to_string()
}

// =============================================================================
// Formatting Functions
// =============================================================================

/// Format a SystemTime for human-readable display.
///
/// Converts the system time to local time and formats it.
///
/// # Arguments
///
/// * `time` - The SystemTime to format
///
/// # Returns
///
/// A string like "2024-01-15 09:30:00"
pub fn format_display(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format(DISPLAY_FORMAT).to_string()
}

/// Format a SystemTime as UTC in human-readable format.
///
/// # Arguments
///
/// * `time` - The SystemTime to format
///
/// # Returns
///
/// A string like "2024-01-15 14:30:00"
pub fn format_display_utc(time: SystemTime) -> String {
    let datetime: DateTime<Utc> = time.into();
    datetime.format(DISPLAY_FORMAT).to_string()
}

/// Format a Duration as a human-readable string.
///
/// Automatically chooses appropriate units based on duration length.
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// Examples:
/// - `Duration::from_secs(30)` → "30s"
/// - `Duration::from_secs(90)` → "1m 30s"
/// - `Duration::from_secs(3665)` → "1h 1m 5s"
/// - `Duration::from_secs(90000)` → "1d 1h 0m"
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        return format!("{}s", total_secs);
    }

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else {
        format!("{}m {}s", minutes, seconds)
    }
}

/// Format a duration with millisecond precision.
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A string like "1.234s" or "125ms"
pub fn format_duration_precise(duration: Duration) -> String {
    let millis = duration.as_millis();
    
    if millis < 1000 {
        format!("{}ms", millis)
    } else {
        format!("{:.3}s", duration.as_secs_f64())
    }
}

// =============================================================================
// Parsing Functions
// =============================================================================

/// Parse an RFC 3339 timestamp string to DateTime<Utc>.
///
/// # Arguments
///
/// * `s` - The RFC 3339 timestamp string
///
/// # Returns
///
/// `Ok(DateTime<Utc>)` if parsing succeeds, `Err` otherwise
///
/// # Example
///
/// ```rust,ignore
/// let dt = parse_rfc3339("2024-01-15T14:30:00Z")?;
/// ```
pub fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Parse various datetime formats commonly found in forensic evidence.
///
/// Tries multiple formats in order of likelihood.
///
/// # Arguments
///
/// * `s` - The datetime string to parse
///
/// # Returns
///
/// `Some(DateTime<Utc>)` if any format matches, `None` otherwise
pub fn parse_flexible(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC 3339 first (most common)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try common formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%d/%m/%Y %H:%M:%S",
        "%m/%d/%Y %H:%M:%S",
        "%Y-%m-%d",
        "%d-%m-%Y",
    ];

    for fmt in formats {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return Some(Utc.from_utc_datetime(&dt));
        }
        // Try as date only
        if let Ok(date) = chrono::NaiveDate::parse_from_str(s, fmt) {
            let dt = date.and_hms_opt(0, 0, 0)?;
            return Some(Utc.from_utc_datetime(&dt));
        }
    }

    None
}

/// Convert Unix timestamp (seconds since epoch) to DateTime<Utc>.
///
/// # Arguments
///
/// * `timestamp` - Unix timestamp in seconds
///
/// # Returns
///
/// `Some(DateTime<Utc>)` if valid, `None` if out of range
pub fn from_unix_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single()
}

/// Convert Unix timestamp (milliseconds since epoch) to DateTime<Utc>.
///
/// # Arguments
///
/// * `timestamp_ms` - Unix timestamp in milliseconds
///
/// # Returns
///
/// `Some(DateTime<Utc>)` if valid, `None` if out of range
pub fn from_unix_timestamp_millis(timestamp_ms: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_millis_opt(timestamp_ms).single()
}

/// Convert SystemTime to Unix timestamp (seconds since epoch).
pub fn to_unix_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_rfc3339() {
        let ts = now_rfc3339();
        // Should parse back without error
        assert!(parse_rfc3339(&ts).is_ok());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m 5s");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1d 1h 0m");
    }

    #[test]
    fn test_format_duration_precise() {
        assert_eq!(format_duration_precise(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration_precise(Duration::from_millis(1234)), "1.234s");
    }

    #[test]
    fn test_parse_flexible() {
        // RFC 3339
        assert!(parse_flexible("2024-01-15T14:30:00Z").is_some());
        // Common format
        assert!(parse_flexible("2024-01-15 14:30:00").is_some());
        // Date only
        assert!(parse_flexible("2024-01-15").is_some());
        // Invalid
        assert!(parse_flexible("not a date").is_none());
    }

    #[test]
    fn test_from_unix_timestamp() {
        let dt = from_unix_timestamp(0).unwrap();
        assert_eq!(dt, Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap());
    }
}
