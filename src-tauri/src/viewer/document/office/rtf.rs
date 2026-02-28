// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! RTF Format Handling
//!
//! Extracts plain text from Rich Text Format files by stripping
//! control words and group delimiters.

use std::path::Path;

use crate::viewer::document::error::{DocumentError, DocumentResult};
use super::{OfficeParagraph, OfficeTextSection};

// =============================================================================
// RTF Text Extraction
// =============================================================================

/// Extract text from an RTF file by stripping control words.
///
/// RTF is a text-based format with backslash control words.
/// We strip these to extract the readable text content.
pub(crate) fn extract_rtf_text(path: &Path) -> DocumentResult<Vec<OfficeTextSection>> {
    let data = std::fs::read_to_string(path)?;

    // Verify it's actually RTF
    if !data.starts_with("{\\rtf") {
        return Err(DocumentError::Parse("Not a valid RTF file".to_string()));
    }

    let text = strip_rtf_to_text(&data);

    let paragraphs: Vec<OfficeParagraph> = text
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(OfficeParagraph::normal)
        .collect();

    Ok(vec![OfficeTextSection {
        label: None,
        paragraphs,
    }])
}

/// Strip RTF control words to extract plain text.
///
/// Handles:
/// - `\par` → newline
/// - `\tab` → tab
/// - `\'XX` → hex-encoded character
/// - `{` and `}` → group delimiters (ignored)
/// - Other `\word` → ignored
fn strip_rtf_to_text(rtf: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = rtf.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut depth = 0i32;
    let mut skip_group = false;

    while i < len {
        let ch = chars[i];

        match ch {
            '{' => {
                depth += 1;
                // Skip certain destination groups
                if i + 1 < len && chars[i + 1] == '\\' {
                    // Look ahead for destination keywords to skip
                    let rest: String = chars[i+1..std::cmp::min(i+30, len)].iter().collect();
                    if rest.starts_with("\\fonttbl") || rest.starts_with("\\colortbl")
                        || rest.starts_with("\\stylesheet") || rest.starts_with("\\info")
                        || rest.starts_with("\\*\\")
                    {
                        skip_group = true;
                    }
                }
                i += 1;
            }
            '}' => {
                depth -= 1;
                if depth <= 0 {
                    skip_group = false;
                }
                if skip_group && depth <= 1 {
                    skip_group = false;
                }
                i += 1;
            }
            '\\' if !skip_group => {
                i += 1;
                if i >= len {
                    break;
                }

                let next = chars[i];

                // Escaped special characters
                if next == '{' || next == '}' || next == '\\' {
                    result.push(next);
                    i += 1;
                } else if next == '\'' {
                    // Hex-encoded character: \'XX
                    i += 1;
                    if i + 1 < len {
                        let hex: String = chars[i..i+2].iter().collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            if byte >= 0x20 {
                                result.push(byte as char);
                            }
                        }
                        i += 2;
                    }
                } else if next == '\n' || next == '\r' {
                    // Line break in RTF source — ignore
                    i += 1;
                } else {
                    // Control word: \wordN or \word-N
                    let mut word = String::new();
                    while i < len && chars[i].is_ascii_alphabetic() {
                        word.push(chars[i]);
                        i += 1;
                    }
                    // Skip optional numeric parameter
                    if i < len && (chars[i] == '-' || chars[i].is_ascii_digit()) {
                        while i < len && (chars[i] == '-' || chars[i].is_ascii_digit()) {
                            i += 1;
                        }
                    }
                    // Skip trailing space delimiter
                    if i < len && chars[i] == ' ' {
                        i += 1;
                    }

                    // Handle meaningful control words
                    match word.as_str() {
                        "par" | "line" => result.push('\n'),
                        "tab" => result.push('\t'),
                        "emspace" | "enspace" | "qmspace" => result.push(' '),
                        _ => {} // Ignore other control words
                    }
                }
            }
            _ if !skip_group => {
                // Regular character
                if ch != '\r' && ch != '\n' {
                    result.push(ch);
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_rtf_basic() {
        let rtf = r"{\rtf1\ansi Hello World}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("Hello World"), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_paragraphs() {
        let rtf = r"{\rtf1\ansi First paragraph\par Second paragraph\par Third}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("First paragraph"), "Got: {}", text);
        assert!(text.contains("Second paragraph"), "Got: {}", text);
        assert!(text.contains("Third"), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_special_chars() {
        let rtf = r"{\rtf1 Braces: \{ and \} and backslash: \\}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains('{'), "Got: {}", text);
        assert!(text.contains('}'), "Got: {}", text);
        assert!(text.contains('\\'), "Got: {}", text);
    }

    #[test]
    fn test_strip_rtf_skip_groups() {
        let rtf = r"{\rtf1{\fonttbl{\f0 Arial;}}{\colortbl;\red0\green0\blue0;}Hello from RTF}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("Hello from RTF"), "Got: {}", text);
        // Should NOT contain font table content
        assert!(!text.contains("Arial"), "Font table leaked: {}", text);
    }

    #[test]
    fn test_strip_rtf_hex_chars() {
        // \'e9 = é in Windows-1252
        let rtf = r"{\rtf1 caf\'e9}";
        let text = strip_rtf_to_text(rtf);
        assert!(text.contains("caf"), "Got: {}", text);
    }
}
