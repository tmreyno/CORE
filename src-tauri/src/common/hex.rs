// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Hex dump formatting utilities
//!
//! Provides hex dump display for binary data viewing in forensic analysis.
//! Essential for examining file headers, raw bytes, and binary structures.

use crate::containers::ContainerError;

// =============================================================================
// Hex Formatting Utilities
// =============================================================================

/// Format a single line of hex (no offset, no ASCII) - useful for inline display
pub fn format_hex_inline(data: &[u8], uppercase: bool) -> String {
    if uppercase {
        data.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        data.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Format bytes as continuous hex string (no spaces)
pub fn format_hex_string(data: &[u8], uppercase: bool) -> String {
    if uppercase {
        data.iter().map(|b| format!("{:02X}", b)).collect()
    } else {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Parse hex string back to bytes
pub fn parse_hex_string(hex: &str) -> Result<Vec<u8>, ContainerError> {
    let hex = hex.replace(" ", "").replace("\n", "").replace("\r", "");

    if !hex.len().is_multiple_of(2) {
        return Err(ContainerError::ParseError(
            "Hex string must have even length".to_string(),
        ));
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| {
                ContainerError::ParseError(format!("Invalid hex at position {}: {}", i, e))
            })
        })
        .collect()
}

// =============================================================================
// Size Formatting
// =============================================================================

/// Format byte count with human-readable units (KB, MB, GB, TB)
///
/// Returns a string like "1.50 GB (1610612736 bytes)" for clarity.
/// This is the canonical implementation - use this instead of duplicating.
///
/// # Examples
/// ```
/// use ffx_check_lib::common::format_size;
/// assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
/// assert_eq!(format_size(1500000), "1.43 MB (1500000 bytes)");
/// ```
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB ({} bytes)", bytes as f64 / TB as f64, bytes)
    } else if bytes >= GB {
        format!("{:.2} GB ({} bytes)", bytes as f64 / GB as f64, bytes)
    } else if bytes >= MB {
        format!("{:.2} MB ({} bytes)", bytes as f64 / MB as f64, bytes)
    } else if bytes >= KB {
        format!("{:.2} KB ({} bytes)", bytes as f64 / KB as f64, bytes)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format byte count compactly without showing raw bytes
///
/// Returns just "1.50 GB" without the byte count for UI display.
pub fn format_size_compact(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// =============================================================================
// CSV Utilities
// =============================================================================

/// Escape a value for CSV output
///
/// Properly handles commas, quotes, and newlines in CSV values by:
/// - Wrapping the value in double quotes if it contains special characters
/// - Doubling any internal double quotes
///
/// # Examples
/// ```rust
/// use ffx_check_lib::common::escape_csv;
///
/// assert_eq!(escape_csv("simple"), "simple");
/// assert_eq!(escape_csv("has,comma"), "\"has,comma\"");
/// assert_eq!(escape_csv("has\"quote"), "\"has\"\"quote\"");
/// assert_eq!(escape_csv("has\nline"), "\"has\nline\"");
/// ```
pub fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Build a CSV row from multiple values
///
/// Escapes each value and joins them with commas.
///
/// # Examples
/// ```rust
/// use ffx_check_lib::common::csv_row;
///
/// let row = csv_row(&["Name", "Value", "has,comma"]);
/// assert_eq!(row, "Name,Value,\"has,comma\"\n");
/// ```
pub fn csv_row(values: &[&str]) -> String {
    let escaped: Vec<String> = values.iter().map(|v| escape_csv(v)).collect();
    format!("{}\n", escaped.join(","))
}

/// Build a CSV header row
pub fn csv_header(columns: &[&str]) -> String {
    csv_row(columns)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(100), "100 bytes");
        assert_eq!(format_size(1024), "1.00 KB (1024 bytes)");
        assert_eq!(format_size(1536), "1.50 KB (1536 bytes)");
        assert_eq!(format_size(1048576), "1.00 MB (1048576 bytes)");
        assert_eq!(format_size(1073741824), "1.00 GB (1073741824 bytes)");
        assert_eq!(format_size(1099511627776), "1.00 TB (1099511627776 bytes)");
    }

    #[test]
    fn test_format_size_compact() {
        assert_eq!(format_size_compact(0), "0 bytes");
        assert_eq!(format_size_compact(1024), "1.00 KB");
        assert_eq!(format_size_compact(1073741824), "1.00 GB");
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("has,comma"), "\"has,comma\"");
        assert_eq!(escape_csv("has\"quote"), "\"has\"\"quote\"");
        assert_eq!(escape_csv("has\nline"), "\"has\nline\"");
        assert_eq!(escape_csv(""), "");
    }

    #[test]
    fn test_csv_row() {
        assert_eq!(csv_row(&["a", "b", "c"]), "a,b,c\n");
        assert_eq!(csv_row(&["has,comma", "normal"]), "\"has,comma\",normal\n");
    }

    #[test]
    fn test_hex_inline() {
        let data = &[0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(format_hex_inline(data, true), "DE AD BE EF");
        assert_eq!(format_hex_inline(data, false), "de ad be ef");
    }

    #[test]
    fn test_hex_string() {
        let data = &[0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(format_hex_string(data, true), "DEADBEEF");
        assert_eq!(format_hex_string(data, false), "deadbeef");
    }

    #[test]
    fn test_parse_hex_string() {
        assert_eq!(
            parse_hex_string("DEADBEEF").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert_eq!(
            parse_hex_string("de ad be ef").unwrap(),
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        );
        assert!(parse_hex_string("DEA").is_err()); // Odd length
        assert!(parse_hex_string("GHIJ").is_err()); // Invalid hex
    }
}
