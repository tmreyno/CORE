// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! UFED collection and extraction set handling
//!
//! Functions for finding associated files, collection metadata,
//! and managing complete extraction sets.

use std::path::Path;

use super::detection::is_ufed;
use super::parsing::parse_ufdx_file;
use super::types::{AssociatedFile, CollectionInfo, StoredHash, UfedFormat};

/// Find and parse EvidenceCollection.ufdx in parent directories
///
/// Walks up the directory tree (up to 3 levels) looking for the collection file.
pub fn find_collection_ufdx(path: &Path) -> Option<CollectionInfo> {
    let mut current = path.parent();

    // Walk up to 3 levels looking for EvidenceCollection.ufdx
    for _ in 0..3 {
        let Some(dir) = current else { break };

        // Look for EvidenceCollection.ufdx in this directory
        let ufdx_path = dir.join("EvidenceCollection.ufdx");
        if ufdx_path.exists() {
            if let Some(info) = parse_ufdx_file(&ufdx_path) {
                return Some(info);
            }
        }

        current = dir.parent();
    }

    None
}

/// Find associated files in the same directory and parent
///
/// Lists ALL files for complete visibility of the extraction set,
/// including stored hash lookups from the UFD file.
pub fn find_associated_files(
    path: &Path,
    stored_hashes: Option<&Vec<StoredHash>>,
) -> Vec<AssociatedFile> {
    let mut associated = Vec::new();

    let Some(parent) = path.parent() else {
        return associated;
    };

    // First, scan the same directory (sibling files)
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let entry_path = entry.path();

            // Skip the file itself
            if entry_path == path {
                continue;
            }

            // Skip directories
            if entry_path.is_dir() {
                continue;
            }

            let Some(entry_name) = entry_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            // Skip macOS resource fork files and .DS_Store
            if entry_name.starts_with("._") || entry_name == ".DS_Store" {
                continue;
            }

            let entry_lower = entry_name.to_lowercase();
            let file_type = determine_file_type(&entry_lower);
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            // Look up stored hash for this file if available
            let stored_hash = stored_hashes.and_then(|hashes| {
                hashes
                    .iter()
                    .find(|h| {
                        h.filename.to_lowercase() == entry_lower
                            || entry_lower.contains(&h.filename.to_lowercase())
                    })
                    .map(|h| h.hash.clone())
            });

            associated.push(AssociatedFile {
                filename: entry_name.to_string(),
                file_type,
                size,
                stored_hash,
            });
        }
    }

    // Also check parent folder for UFDX collection files
    if let Some(grandparent) = parent.parent() {
        if let Ok(entries) = std::fs::read_dir(grandparent) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // Only look at files, not directories
                if entry_path.is_dir() {
                    continue;
                }

                let Some(entry_name) = entry_path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };

                let entry_lower = entry_name.to_lowercase();

                // Only include UFDX files from parent (collection-level metadata)
                if entry_lower.ends_with(".ufdx") {
                    let file_type = "UFDX".to_string();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                    associated.push(AssociatedFile {
                        filename: format!("../{}", entry_name), // Indicate it's from parent folder
                        file_type,
                        size,
                        stored_hash: None,
                    });
                }
            }
        }
    }

    // Sort by file type then name for consistent display
    associated.sort_by(|a, b| {
        a.file_type
            .cmp(&b.file_type)
            .then_with(|| a.filename.cmp(&b.filename))
    });

    associated
}

/// Determine file type from extension
fn determine_file_type(filename_lower: &str) -> String {
    if filename_lower.ends_with(".ufdr") {
        "UFDR".to_string()
    } else if filename_lower.ends_with(".ufdx") {
        "UFDX".to_string()
    } else if filename_lower.ends_with(".ufd") {
        "UFD".to_string()
    } else if filename_lower.ends_with(".zip") {
        "ZIP".to_string()
    } else if filename_lower.ends_with(".pdf") {
        "PDF".to_string()
    } else if filename_lower.ends_with(".xml") {
        "XML".to_string()
    } else if filename_lower.ends_with(".xlsx") {
        "XLSX".to_string()
    } else {
        "Other".to_string()
    }
}

/// Check if the associated files form a complete extraction set
///
/// A complete set typically has:
/// - A ZIP file (compressed extraction)
/// - A UFD or UFDX file (metadata)
/// - Optionally a PDF/XLSX report
pub fn check_extraction_set(associated: &[AssociatedFile], format: UfedFormat) -> bool {
    let has_zip = associated.iter().any(|f| f.file_type == "ZIP");
    let _has_ufd = associated
        .iter()
        .any(|f| f.file_type == "UFD" || f.file_type == "UFDX");
    let has_pdf = associated.iter().any(|f| f.file_type == "PDF");

    match format {
        UfedFormat::Ufd | UfedFormat::Ufdx => has_zip || has_pdf,
        UfedFormat::Ufdr => true,    // UFDR is self-contained
        UfedFormat::UfedZip => true, // UfedZip is the main evidence with sibling UFD
    }
}

/// Scan a directory for UFED extractions
///
/// Returns list of UFED file paths found
pub fn scan_for_ufed_files(dir: &Path, recursive: bool) -> Vec<String> {
    let mut ufed_files = Vec::new();
    scan_directory_for_ufed(dir, recursive, &mut ufed_files);
    ufed_files
}

fn scan_directory_for_ufed(dir: &Path, recursive: bool, results: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(path_str) = path.to_str() {
                if is_ufed(path_str) {
                    results.push(path_str.to_string());
                }
            }
        } else if recursive && path.is_dir() {
            scan_directory_for_ufed(&path, recursive, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufed::types::{AssociatedFile, UfedFormat};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // ==================== determine_file_type tests ====================

    #[test]
    fn test_determine_file_type_ufdr() {
        assert_eq!(determine_file_type("extraction.ufdr"), "UFDR");
        // Note: function expects lowercase input
        assert_eq!(determine_file_type("evidence.ufdr"), "UFDR");
    }

    #[test]
    fn test_determine_file_type_ufdx() {
        assert_eq!(determine_file_type("metadata.ufdx"), "UFDX");
        assert_eq!(determine_file_type("collection.ufdx"), "UFDX");
    }

    #[test]
    fn test_determine_file_type_ufd() {
        assert_eq!(determine_file_type("extraction.ufd"), "UFD");
    }

    #[test]
    fn test_determine_file_type_zip() {
        assert_eq!(determine_file_type("archive.zip"), "ZIP");
    }

    #[test]
    fn test_determine_file_type_pdf() {
        assert_eq!(determine_file_type("report.pdf"), "PDF");
    }

    #[test]
    fn test_determine_file_type_xml() {
        assert_eq!(determine_file_type("data.xml"), "XML");
    }

    #[test]
    fn test_determine_file_type_xlsx() {
        assert_eq!(determine_file_type("spreadsheet.xlsx"), "XLSX");
    }

    #[test]
    fn test_determine_file_type_other() {
        assert_eq!(determine_file_type("readme.txt"), "Other");
        assert_eq!(determine_file_type("image.png"), "Other");
        assert_eq!(determine_file_type("unknownfile"), "Other");
    }

    // ==================== check_extraction_set tests ====================

    #[test]
    fn test_check_extraction_set_ufd_with_zip() {
        let files = vec![AssociatedFile {
            filename: "extraction.zip".to_string(),
            file_type: "ZIP".to_string(),
            size: 1000,
            stored_hash: None,
        }];
        assert!(check_extraction_set(&files, UfedFormat::Ufd));
    }

    #[test]
    fn test_check_extraction_set_ufd_with_pdf() {
        let files = vec![AssociatedFile {
            filename: "report.pdf".to_string(),
            file_type: "PDF".to_string(),
            size: 500,
            stored_hash: None,
        }];
        assert!(check_extraction_set(&files, UfedFormat::Ufd));
    }

    #[test]
    fn test_check_extraction_set_ufd_empty() {
        let files: Vec<AssociatedFile> = vec![];
        assert!(!check_extraction_set(&files, UfedFormat::Ufd));
    }

    #[test]
    fn test_check_extraction_set_ufdx_with_zip() {
        let files = vec![AssociatedFile {
            filename: "data.zip".to_string(),
            file_type: "ZIP".to_string(),
            size: 2000,
            stored_hash: None,
        }];
        assert!(check_extraction_set(&files, UfedFormat::Ufdx));
    }

    #[test]
    fn test_check_extraction_set_ufdr_always_complete() {
        // UFDR is self-contained
        let files: Vec<AssociatedFile> = vec![];
        assert!(check_extraction_set(&files, UfedFormat::Ufdr));
    }

    #[test]
    fn test_check_extraction_set_ufed_zip_always_complete() {
        // UfedZip is the main evidence
        let files: Vec<AssociatedFile> = vec![];
        assert!(check_extraction_set(&files, UfedFormat::UfedZip));
    }

    // ==================== find_associated_files tests ====================

    #[test]
    fn test_find_associated_files_no_parent() {
        // Root path has no parent
        let path = Path::new("/");
        let files = find_associated_files(path, None);
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_associated_files_with_siblings() {
        let temp = TempDir::new().unwrap();
        let main_file = temp.path().join("evidence.ufd");
        let sibling_zip = temp.path().join("evidence.zip");
        let sibling_pdf = temp.path().join("report.pdf");

        File::create(&main_file).unwrap();
        File::create(&sibling_zip)
            .unwrap()
            .write_all(b"zipdata")
            .unwrap();
        File::create(&sibling_pdf)
            .unwrap()
            .write_all(b"pdfdata")
            .unwrap();

        let files = find_associated_files(&main_file, None);

        // Should find the sibling files but not the main file itself
        assert_eq!(files.len(), 2);
        let filenames: Vec<&str> = files.iter().map(|f| f.filename.as_str()).collect();
        assert!(filenames.contains(&"evidence.zip"));
        assert!(filenames.contains(&"report.pdf"));
    }

    #[test]
    fn test_find_associated_files_skips_ds_store() {
        let temp = TempDir::new().unwrap();
        let main_file = temp.path().join("evidence.ufd");
        let ds_store = temp.path().join(".DS_Store");
        let resource_fork = temp.path().join("._evidence.ufd");

        File::create(&main_file).unwrap();
        File::create(&ds_store).unwrap();
        File::create(&resource_fork).unwrap();

        let files = find_associated_files(&main_file, None);

        // Should skip .DS_Store and ._ files
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_associated_files_skips_directories() {
        let temp = TempDir::new().unwrap();
        let main_file = temp.path().join("evidence.ufd");
        let subdir = temp.path().join("subdir");

        File::create(&main_file).unwrap();
        fs::create_dir(&subdir).unwrap();

        let files = find_associated_files(&main_file, None);

        // Should skip directories
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_associated_files_with_stored_hashes() {
        let temp = TempDir::new().unwrap();
        let main_file = temp.path().join("evidence.ufd");
        let sibling_zip = temp.path().join("evidence.zip");

        File::create(&main_file).unwrap();
        File::create(&sibling_zip)
            .unwrap()
            .write_all(b"zipdata")
            .unwrap();

        let stored_hashes = vec![StoredHash {
            filename: "evidence.zip".to_string(),
            algorithm: "SHA256".to_string(),
            hash: "abc123".to_string(),
            timestamp: None,
        }];

        let files = find_associated_files(&main_file, Some(&stored_hashes));

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "evidence.zip");
        assert_eq!(files[0].stored_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_find_associated_files_sorted_by_type_and_name() {
        let temp = TempDir::new().unwrap();
        let main_file = temp.path().join("evidence.ufd");
        let zip_a = temp.path().join("a.zip");
        let zip_b = temp.path().join("b.zip");
        let pdf_a = temp.path().join("a.pdf");

        File::create(&main_file).unwrap();
        File::create(&zip_a).unwrap();
        File::create(&zip_b).unwrap();
        File::create(&pdf_a).unwrap();

        let files = find_associated_files(&main_file, None);

        // Should be sorted by type, then name
        assert_eq!(files.len(), 3);
        // PDF comes before ZIP alphabetically
        assert_eq!(files[0].file_type, "PDF");
        assert_eq!(files[1].file_type, "ZIP");
        assert_eq!(files[2].file_type, "ZIP");
        // Within ZIP, should be alphabetical
        assert_eq!(files[1].filename, "a.zip");
        assert_eq!(files[2].filename, "b.zip");
    }

    // ==================== scan_for_ufed_files tests ====================

    #[test]
    fn test_scan_for_ufed_files_empty_dir() {
        let temp = TempDir::new().unwrap();
        let files = scan_for_ufed_files(temp.path(), false);
        assert!(files.is_empty());
    }

    #[test]
    fn test_scan_for_ufed_files_finds_ufed() {
        let temp = TempDir::new().unwrap();
        let ufd_file = temp.path().join("evidence.ufd");
        let ufdr_file = temp.path().join("evidence.ufdr");
        let txt_file = temp.path().join("readme.txt");

        File::create(&ufd_file).unwrap();
        File::create(&ufdr_file).unwrap();
        File::create(&txt_file).unwrap();

        let files = scan_for_ufed_files(temp.path(), false);

        // Should find UFD and UFDR files
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_for_ufed_files_recursive() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let root_file = temp.path().join("root.ufd");
        let nested_file = subdir.join("nested.ufdr");

        File::create(&root_file).unwrap();
        File::create(&nested_file).unwrap();

        // Non-recursive should only find root
        let files_non_recursive = scan_for_ufed_files(temp.path(), false);
        assert_eq!(files_non_recursive.len(), 1);

        // Recursive should find both
        let files_recursive = scan_for_ufed_files(temp.path(), true);
        assert_eq!(files_recursive.len(), 2);
    }

    #[test]
    fn test_scan_for_ufed_files_nonexistent_dir() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        let files = scan_for_ufed_files(path, false);
        assert!(files.is_empty());
    }

    // ==================== find_collection_ufdx tests ====================

    #[test]
    fn test_find_collection_ufdx_no_parent() {
        // A path with no parent shouldn't crash
        let path = Path::new("/");
        let result = find_collection_ufdx(path);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_collection_ufdx_not_found() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("somefile.txt");
        File::create(&path).unwrap();

        let result = find_collection_ufdx(&path);
        assert!(result.is_none());
    }
}
