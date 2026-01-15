// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED format detection
//!
//! Detection functions for UFED container formats based on
//! file extensions and sibling file patterns.

use std::path::Path;
use tracing::{trace, debug, instrument};

use super::types::{UfedFormat, UFED_EXTENSIONS};

/// Check if a file is a UFED format
/// 
/// This includes:
/// - .ufd, .ufdr, .ufdx by extension
/// - .zip files that have a sibling .ufd file with the same basename
#[instrument]
pub fn is_ufed(path: &str) -> bool {
    let lower = path.to_lowercase();
    
    // Check standard UFED extensions
    if UFED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) {
        debug!("is_ufed: {} -> true (extension match)", path);
        return true;
    }
    
    // Check if it's a ZIP file with a sibling UFD file
    if lower.ends_with(".zip") {
        debug!("is_ufed: {} checking for sibling UFD", path);
        if let Some(ufd_path) = find_sibling_ufd(path) {
            let exists = ufd_path.exists();
            debug!("is_ufed: {} sibling UFD check: {:?} exists={}", path, ufd_path, exists);
            return exists;
        } else {
            debug!("is_ufed: {} no sibling UFD found", path);
        }
    }
    
    debug!("is_ufed: {} -> false", path);
    false
}

/// Check if a filename has a UFED extension
pub fn is_ufed_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    UFED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Detect UFED format from file extension and context
#[instrument]
pub fn detect_format(path: &str) -> Option<UfedFormat> {
    let lower = path.to_lowercase();
    
    let format = if lower.ends_with(".ufdr") {
        Some(UfedFormat::Ufdr)
    } else if lower.ends_with(".ufdx") {
        Some(UfedFormat::Ufdx)
    } else if lower.ends_with(".ufd") {
        Some(UfedFormat::Ufd)
    } else if lower.ends_with(".zip") {
        // Check if it has a sibling UFD file
        if let Some(ufd_path) = find_sibling_ufd(path) {
            if ufd_path.exists() {
                return Some(UfedFormat::UfedZip);
            }
        }
        None
    } else {
        None
    };
    
    trace!(?format, "Detected UFED format");
    format
}

/// Find the sibling UFD file for a given path
/// 
/// Searches for UFD files in the same directory that either:
/// 1. Match exactly: `<basename>.ufd`
/// 2. Start with the ZIP's basename: `<basename>_<suffix>.ufd` (e.g., `Device_AdvancedLogical.ufd`)
/// 
/// This handles Cellebrite's naming convention where the UFD file often has
/// an extraction type suffix like `_AdvancedLogical` or `_FileSystem`.
/// 
/// Returns the path to an existing UFD file if found, or the expected exact-match
/// path if the directory doesn't exist or no UFD file is found.
pub fn find_sibling_ufd(path: &str) -> Option<std::path::PathBuf> {
    let path_obj = Path::new(path);
    let stem = path_obj.file_stem()?.to_string_lossy();
    let parent = path_obj.parent()?;
    
    // Build the exact-match path: <basename>.ufd
    let exact_path = parent.join(format!("{}.ufd", stem));
    
    // If exact match exists, return it
    if exact_path.exists() {
        trace!(?exact_path, "Found exact UFD match");
        return Some(exact_path);
    }
    
    // Search for UFD files starting with the basename (handles _AdvancedLogical suffix)
    // Only search if the directory exists
    if parent.exists() {
        if let Ok(entries) = std::fs::read_dir(parent) {
            let stem_lower = stem.to_lowercase();
            for entry in entries.flatten() {
                let entry_name = entry.file_name();
                let name_str = entry_name.to_string_lossy().to_lowercase();
                
                // Check if it's a UFD file that starts with our basename
                if name_str.ends_with(".ufd") && name_str.starts_with(&stem_lower) {
                    trace!(ufd_path = ?entry.path(), stem = %stem, "Found UFD file with matching prefix");
                    return Some(entry.path());
                }
            }
        }
    }
    
    // Return the exact-match path even if it doesn't exist
    // This allows detection logic to check if the path *would* exist
    Some(exact_path)
}

/// Extract device hint from filename or path
/// 
/// Looks for device-like patterns in UFED folder names such as:
/// - "Apple_iPhone SE (A2275)"
/// - "Samsung GSM_SM-S918U Galaxy S23 Ultra"
pub fn extract_device_hint(path: &str) -> Option<String> {
    let path_obj = Path::new(path);
    let filename = path_obj.file_stem()?.to_str()?;
    
    // Common patterns in UFED filenames
    let lower = filename.to_lowercase();
    if lower.contains("iphone") || lower.contains("ipad") || lower.contains("samsung") 
        || lower.contains("galaxy") || lower.contains("pixel") || lower.contains("android")
        || lower.contains("apple") || lower.contains("huawei") || lower.contains("oneplus")
    {
        return Some(filename.to_string());
    }
    
    // Check parent folder for UFED extraction pattern
    if let Some(parent) = path_obj.parent() {
        if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
            let parent_lower = parent_name.to_lowercase();
            if parent_lower.contains("ufed") || parent_lower.contains("advancedlogical") 
                || parent_lower.contains("file system")
            {
                // Go up one more level to find device info
                if let Some(grandparent) = parent.parent() {
                    if let Some(gp_name) = grandparent.file_name().and_then(|n| n.to_str()) {
                        if gp_name.to_lowercase().contains("ufed") {
                            return extract_device_from_ufed_folder(gp_name);
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Extract device name from UFED folder naming convention
/// 
/// Pattern: "UFED <Device Name> <Date> (<Number>)"
/// Example: "UFED Apple iPhone SE (A2275) 2024_08_26 (001)"
fn extract_device_from_ufed_folder(folder_name: &str) -> Option<String> {
    let name = folder_name.trim();
    
    // Remove "UFED " prefix
    let without_prefix = if name.to_lowercase().starts_with("ufed ") {
        &name[5..]
    } else {
        name
    };
    
    // Try to find the date pattern (YYYY_MM_DD or similar) and extract what's before it
    if let Some(date_pos) = find_date_pattern(without_prefix) {
        let device = without_prefix[..date_pos].trim();
        if !device.is_empty() {
            return Some(device.to_string());
        }
    }
    
    // Fallback: return the whole thing without UFED prefix
    if !without_prefix.is_empty() {
        return Some(without_prefix.to_string());
    }
    
    None
}

/// Find position of date pattern in string (YYYY_MM_DD or YYYY-MM-DD)
fn find_date_pattern(s: &str) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    
    for i in 0..chars.len().saturating_sub(9) {
        // Check for YYYY_MM_DD or YYYY-MM-DD
        if chars[i].is_ascii_digit() 
            && chars.get(i + 4).map(|&c| c == '_' || c == '-').unwrap_or(false)
            && chars.get(i + 7).map(|&c| c == '_' || c == '-').unwrap_or(false)
        {
            // Verify it's a valid date-like pattern
            let is_year = (i..i+4).all(|j| chars.get(j).map(|c| c.is_ascii_digit()).unwrap_or(false));
            let is_month = (i+5..i+7).all(|j| chars.get(j).map(|c| c.is_ascii_digit()).unwrap_or(false));
            let is_day = (i+8..i+10).all(|j| chars.get(j).map(|c| c.is_ascii_digit()).unwrap_or(false));
            
            if is_year && is_month && is_day {
                return Some(i);
            }
        }
    }
    
    None
}

/// Extract evidence number from folder structure
/// 
/// Looks for patterns like "02606-0900_1E_BTPLJM" in parent folders
pub fn extract_evidence_number(path: &Path) -> Option<String> {
    let mut current = path.parent();
    
    // Walk up the directory tree looking for evidence number patterns
    while let Some(dir) = current {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            // Skip extraction-related folder names
            let lower = name.to_lowercase();
            if lower.contains("ufed") || lower.contains("file system") || lower.contains("advancedlogical") {
                current = dir.parent();
                continue;
            }
            
            // Evidence number patterns: contain underscores, dashes, alphanumeric
            // e.g., "02606-0900_1E_BTPLJM", "12345-0001_A_XYZ123"
            if name.contains('_') && name.contains('-') && name.len() >= 10 {
                return Some(name.to_string());
            }
            
            // Also check for simpler patterns like case numbers
            if name.chars().filter(|c| c.is_ascii_digit()).count() >= 4 {
                // Has at least 4 digits, might be a case/evidence number
                if name.contains('-') || name.contains('_') {
                    return Some(name.to_string());
                }
            }
        }
        current = dir.parent();
    }
    
    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_ufed_file_ufd() {
        assert!(is_ufed_file("extraction.ufd"));
        assert!(is_ufed_file("extraction.UFD"));
        assert!(is_ufed_file("EXTRACTION.ufd"));
    }

    #[test]
    fn test_is_ufed_file_ufdr() {
        assert!(is_ufed_file("report.ufdr"));
        assert!(is_ufed_file("REPORT.UFDR"));
    }

    #[test]
    fn test_is_ufed_file_ufdx() {
        assert!(is_ufed_file("collection.ufdx"));
        assert!(is_ufed_file("COLLECTION.UFDX"));
    }

    #[test]
    fn test_is_ufed_file_not_ufed() {
        assert!(!is_ufed_file("document.pdf"));
        assert!(!is_ufed_file("archive.zip"));
        assert!(!is_ufed_file("image.e01"));
    }

    #[test]
    fn test_is_ufed_by_extension() {
        assert!(is_ufed("/path/to/file.ufd"));
        assert!(is_ufed("/path/to/file.ufdr"));
        assert!(is_ufed("/path/to/file.ufdx"));
    }

    #[test]
    fn test_is_ufed_not_ufed() {
        assert!(!is_ufed("/path/to/file.pdf"));
        assert!(!is_ufed("/path/to/file.e01"));
    }

    #[test]
    fn test_detect_format_ufd() {
        let format = detect_format("/path/to/extraction.ufd");
        assert_eq!(format, Some(UfedFormat::Ufd));
    }

    #[test]
    fn test_detect_format_ufdr() {
        let format = detect_format("/path/to/report.ufdr");
        assert_eq!(format, Some(UfedFormat::Ufdr));
    }

    #[test]
    fn test_detect_format_ufdx() {
        let format = detect_format("/path/to/collection.ufdx");
        assert_eq!(format, Some(UfedFormat::Ufdx));
    }

    #[test]
    fn test_detect_format_none() {
        let format = detect_format("/path/to/document.pdf");
        assert!(format.is_none());
    }

    #[test]
    fn test_find_sibling_ufd() {
        let result = find_sibling_ufd("/path/to/extraction.zip");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("extraction.ufd"));
    }

    #[test]
    fn test_find_sibling_ufd_with_parent() {
        let result = find_sibling_ufd("/evidence/case001/data.zip");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("case001"));
        assert!(path.to_string_lossy().ends_with("data.ufd"));
    }

    #[test]
    fn test_extract_device_hint_iphone() {
        let hint = extract_device_hint("/path/iPhone_backup.ufd");
        assert!(hint.is_some());
        assert!(hint.unwrap().to_lowercase().contains("iphone"));
    }

    #[test]
    fn test_extract_device_hint_samsung() {
        let hint = extract_device_hint("/path/Samsung_Galaxy_S21.ufd");
        assert!(hint.is_some());
    }

    #[test]
    fn test_extract_device_hint_no_device() {
        let hint = extract_device_hint("/path/random_file.ufd");
        assert!(hint.is_none());
    }

    #[test]
    fn test_extract_device_from_ufed_folder() {
        let result = extract_device_from_ufed_folder("UFED Apple iPhone SE (A2275) 2024_08_26 (001)");
        assert!(result.is_some());
        let device = result.unwrap();
        assert!(device.contains("iPhone"));
    }

    #[test]
    fn test_extract_device_from_ufed_folder_no_date() {
        let result = extract_device_from_ufed_folder("UFED Samsung Galaxy");
        assert!(result.is_some());
        let device = result.unwrap();
        assert!(device.contains("Samsung"));
    }

    #[test]
    fn test_find_date_pattern() {
        let result = find_date_pattern("Device 2024_08_26 extra");
        assert_eq!(result, Some(7)); // Position of "2024"
    }

    #[test]
    fn test_find_date_pattern_with_dashes() {
        let result = find_date_pattern("Device 2024-08-26 extra");
        assert_eq!(result, Some(7));
    }

    #[test]
    fn test_find_date_pattern_none() {
        let result = find_date_pattern("Device name no date");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_evidence_number() {
        let temp_dir = TempDir::new().unwrap();
        let evidence_path = temp_dir.path().join("02606-0900_1E_BTPLJM/UFED/data.ufd");
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        
        let result = extract_evidence_number(&evidence_path);
        assert!(result.is_some());
        assert!(result.unwrap().contains("02606"));
    }

    #[test]
    fn test_extract_evidence_number_simple() {
        let temp_dir = TempDir::new().unwrap();
        let evidence_path = temp_dir.path().join("CASE-2024-001/data.ufd");
        fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        
        let result = extract_evidence_number(&evidence_path);
        assert!(result.is_some());
    }

    #[test]
    fn test_is_ufed_zip_with_sibling() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("extraction.zip");
        let ufd_path = temp_dir.path().join("extraction.ufd");
        
        fs::write(&zip_path, b"PK").unwrap();
        fs::write(&ufd_path, b"UFD data").unwrap();
        
        assert!(is_ufed(zip_path.to_str().unwrap()));
    }

    #[test]
    fn test_detect_format_zip_with_sibling() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("extraction.zip");
        let ufd_path = temp_dir.path().join("extraction.ufd");
        
        fs::write(&zip_path, b"PK").unwrap();
        fs::write(&ufd_path, b"UFD data").unwrap();
        
        let format = detect_format(zip_path.to_str().unwrap());
        assert_eq!(format, Some(UfedFormat::UfedZip));
    }
}
