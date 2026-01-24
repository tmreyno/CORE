// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Path Security Utilities
//!
//! Provides functions for validating and sanitizing file paths
//! to prevent path traversal attacks and other security issues.

use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{warn, info};

/// Error type for path security operations
#[derive(Debug, Clone, Error)]
pub enum PathSecurityError {
    /// Path traversal attempt detected
    #[error("Path traversal detected: {0}")]
    TraversalDetected(String),
    /// Path canonicalization failed
    #[error("Failed to canonicalize path: {0}")]
    CanonicalizationFailed(String),
    /// Path is not under expected base
    #[error("Path '{path}' is not under base '{base}'")]
    NotUnderBase { path: String, base: String },
    /// Invalid path component
    #[error("Invalid path component: {0}")]
    InvalidComponent(String),
}

/// Result type for path security operations
pub type PathSecurityResult<T> = Result<T, PathSecurityError>;

/// Safely join a base path with a filename, preventing path traversal.
///
/// This function:
/// 1. Validates the filename doesn't contain traversal sequences
/// 2. Joins the paths
/// 3. Canonicalizes the result
/// 4. Verifies the result is still under the base directory
///
/// # Arguments
/// * `base` - The base directory path
/// * `filename` - The filename or relative path to join
///
/// # Returns
/// The safely joined and canonicalized path, or an error if traversal was detected.
///
/// # Example
/// ```rust,ignore
/// let base = Path::new("/evidence/case1");
/// let safe = safe_join(base, "file.txt")?;  // Ok: /evidence/case1/file.txt
/// let bad = safe_join(base, "../secrets.txt");  // Err: TraversalDetected
/// ```
pub fn safe_join(base: &Path, filename: &str) -> PathSecurityResult<PathBuf> {
    // Check for obvious traversal patterns in the raw filename
    if contains_traversal_pattern(filename) {
        warn!(
            target: "security",
            filename = filename,
            "Path traversal pattern detected in filename"
        );
        return Err(PathSecurityError::TraversalDetected(filename.to_string()));
    }
    
    // Join the paths
    let joined = base.join(filename);
    
    // Canonicalize both paths for comparison
    // Note: base must exist for canonicalization
    let canonical_base = base.canonicalize()
        .map_err(|e| PathSecurityError::CanonicalizationFailed(
            format!("Base path: {}", e)
        ))?;
    
    // For the joined path, if it doesn't exist yet (new file), 
    // canonicalize the parent and append the filename
    let canonical_joined = if joined.exists() {
        joined.canonicalize()
            .map_err(|e| PathSecurityError::CanonicalizationFailed(
                format!("Joined path: {}", e)
            ))?
    } else {
        // Get canonical parent + filename for new files
        let parent = joined.parent()
            .ok_or_else(|| PathSecurityError::InvalidComponent("No parent directory".into()))?;
        
        let canonical_parent = parent.canonicalize()
            .map_err(|e| PathSecurityError::CanonicalizationFailed(
                format!("Parent path: {}", e)
            ))?;
        
        let filename_component = joined.file_name()
            .ok_or_else(|| PathSecurityError::InvalidComponent("No filename".into()))?;
        
        canonical_parent.join(filename_component)
    };
    
    // Verify the result is under the base directory
    if !canonical_joined.starts_with(&canonical_base) {
        warn!(
            target: "security",
            path = %canonical_joined.display(),
            base = %canonical_base.display(),
            "Path escapes base directory"
        );
        return Err(PathSecurityError::NotUnderBase {
            path: canonical_joined.display().to_string(),
            base: canonical_base.display().to_string(),
        });
    }
    
    info!(
        target: "security",
        path = %canonical_joined.display(),
        "Path validation successful"
    );
    
    Ok(canonical_joined)
}

/// Check if a filename contains path traversal patterns.
///
/// Detects:
/// - `..` sequences
/// - Absolute path indicators
/// - Null bytes
/// - Other dangerous patterns
pub fn contains_traversal_pattern(filename: &str) -> bool {
    // Check for null bytes (used in null byte injection attacks)
    if filename.contains('\0') {
        return true;
    }
    
    // Check for parent directory references - must be a path component
    // Valid: "../", "..\\", starts with "..", ends with "/.." or "\\.."
    // Invalid (not traversal): "file..name.txt", "test.."
    if filename == ".." 
        || filename.starts_with("../") 
        || filename.starts_with("..\\")
        || filename.contains("/../")
        || filename.contains("\\..\\")
        || filename.contains("\\..")
        || filename.ends_with("/..")
        || filename.ends_with("\\..")
    {
        return true;
    }
    
    // Check for absolute paths (Unix and Windows)
    if filename.starts_with('/') || filename.starts_with('\\') {
        return true;
    }
    
    // Check for Windows drive letters
    if filename.len() >= 2 {
        let bytes = filename.as_bytes();
        if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            return true;
        }
    }
    
    // Check for URL-encoded traversal
    let lower = filename.to_lowercase();
    if lower.contains("%2e%2e") || lower.contains("%2f") || lower.contains("%5c") {
        return true;
    }
    
    false
}

/// Sanitize a filename by removing dangerous characters.
///
/// Removes or replaces:
/// - Path separators
/// - Null bytes
/// - Control characters
/// - Windows reserved characters
///
/// Returns the sanitized filename.
pub fn sanitize_filename(filename: &str) -> String {
    let mut sanitized = String::with_capacity(filename.len());
    
    for c in filename.chars() {
        match c {
            // Remove null bytes and control characters
            '\0'..='\x1f' | '\x7f' => continue,
            // Replace path separators
            '/' | '\\' => sanitized.push('_'),
            // Remove Windows reserved characters
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => continue,
            // Keep everything else
            _ => sanitized.push(c),
        }
    }
    
    // Remove leading/trailing dots and spaces (Windows issues)
    sanitized.trim_matches(|c| c == '.' || c == ' ').to_string()
}

/// Validate a path for safe file operations.
///
/// Returns true if the path is safe to use for file operations.
pub fn is_safe_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check for traversal patterns
    if contains_traversal_pattern(&path_str) {
        return false;
    }
    
    // Check each component
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::ParentDir => return false,
            Component::Normal(s) => {
                if let Some(s) = s.to_str() {
                    if s.contains('\0') {
                        return false;
                    }
                }
            }
            _ => {}
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_traversal_pattern() {
        // Should detect traversal
        assert!(contains_traversal_pattern("../secret.txt"));
        assert!(contains_traversal_pattern("..\\secret.txt"));
        assert!(contains_traversal_pattern("foo/../bar"));
        assert!(contains_traversal_pattern("/etc/passwd"));
        assert!(contains_traversal_pattern("\\Windows\\System32"));
        assert!(contains_traversal_pattern("C:\\Windows"));
        assert!(contains_traversal_pattern("file\0.txt"));
        assert!(contains_traversal_pattern("%2e%2e/secret"));
        
        // Should be safe
        assert!(!contains_traversal_pattern("file.txt"));
        assert!(!contains_traversal_pattern("subdir/file.txt"));
        assert!(!contains_traversal_pattern("file..name.txt"));
        assert!(!contains_traversal_pattern("my-file_2024.pdf"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("path/to/file.txt"), "path_to_file.txt");
        assert_eq!(sanitize_filename("file<>:\"|?*.txt"), "file.txt");
        assert_eq!(sanitize_filename("...hidden..."), "hidden");
        assert_eq!(sanitize_filename("file\0name.txt"), "filename.txt");
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("file.txt")));
        assert!(is_safe_path(Path::new("subdir/file.txt")));
        assert!(!is_safe_path(Path::new("../secret.txt")));
        assert!(!is_safe_path(Path::new("/etc/passwd")));
    }
}
