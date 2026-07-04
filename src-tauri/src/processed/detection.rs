// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Detection logic for processed forensic databases
//!
//! Identifies AXIOM, Cellebrite PA, X-Ways, Autopsy, EnCase, FTK databases.

use std::fs;
use std::path::Path;
use tracing::debug;

use super::types::*;

const PROCESSED_SCAN_MAX_DEPTH: usize = 128;
const PROCESSED_SCAN_MAX_RESULTS: usize = 10_000;
const PROCESSED_SIZE_MAX_FILES: usize = 250_000;
const PROCESSED_DATABASE_FILES_MAX: usize = 10_000;
const PROCESSED_TEXT_FIELD_MAX_CHARS: usize = 4096;

/// Detect if a path is a processed database and what type
pub fn detect_processed_db(path: &Path) -> Option<ProcessedDbType> {
    if !path.exists() {
        return None;
    }

    // Check for Magnet AXIOM
    if is_axiom_database(path) {
        return Some(ProcessedDbType::MagnetAxiom);
    }

    // Check for Cellebrite PA extracted data
    if is_cellebrite_pa(path) {
        return Some(ProcessedDbType::CellebritePA);
    }

    // Check for X-Ways
    if is_xways(path) {
        return Some(ProcessedDbType::XWays);
    }

    // Check for Autopsy
    if is_autopsy(path) {
        return Some(ProcessedDbType::Autopsy);
    }

    // Check for EnCase
    if is_encase(path) {
        return Some(ProcessedDbType::EnCase);
    }

    // Check for FTK
    if is_ftk(path) {
        return Some(ProcessedDbType::FTK);
    }

    None
}

/// Check if path is a Magnet AXIOM processed database
///
/// AXIOM structure:
/// - Folder containing Case.mfdb or Case.mcfc files
/// - Folder name starts with "AXIOM"
/// - Contains Case Information.xml or Case Information.txt
///
/// Note: Folders named "Processed.Database" are containers for cases,
/// not cases themselves, so we check for actual AXIOM files.
fn is_axiom_database(path: &Path) -> bool {
    if path.is_dir() {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let name_lower = name.to_lowercase();

        // Check for AXIOM-specific files first (most reliable)
        let case_mcfc = path.join("Case.mcfc");
        let case_mfdb = path.join("Case.mfdb");
        let case_info_xml = path.join("Case Information.xml");
        let case_info_txt = path.join("Case Information.txt");

        if case_mcfc.exists() {
            debug!("AXIOM detected by Case.mcfc: {}", path.display());
            return true;
        }

        if case_mfdb.exists() {
            debug!("AXIOM detected by Case.mfdb: {}", path.display());
            return true;
        }

        if case_info_xml.exists() || case_info_txt.exists() {
            debug!(
                "AXIOM detected by Case Information file: {}",
                path.display()
            );
            return true;
        }

        // Folder name starting with "AXIOM" is a strong indicator,
        // but only if it's not the parent container folder
        if name_lower.starts_with("axiom") && !name_lower.contains("processed") {
            // Double check it has some AXIOM-related content
            let has_mfdb = fs::read_dir(path)
                .ok()
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        let ext = e
                            .path()
                            .extension()
                            .map(|x| x.to_string_lossy().to_lowercase());
                        ext.as_deref() == Some("mfdb") || ext.as_deref() == Some("mcfc")
                    })
                })
                .unwrap_or(false);

            if has_mfdb {
                debug!("AXIOM detected by folder name + mfdb: {}", name);
                return true;
            }
        }

        // Check for any .mfdb file in the folder (but not in "Processed.Database" container folders)
        if !name_lower.contains("processed.database") && !name_lower.contains("processed database")
        {
            let mfdb_exists = fs::read_dir(path)
                .ok()
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "mfdb")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if mfdb_exists {
                debug!("AXIOM detected by .mfdb file: {}", path.display());
                return true;
            }
        }
    } else if path.is_file() {
        // Direct .mfdb file
        if let Some(ext) = path.extension() {
            if ext == "mfdb" || ext == "mcfc" {
                return true;
            }
        }
    }

    false
}

/// Check if path is Cellebrite PA extracted data
///
/// PA structure:
/// - Contains UFD Report/ or UFED_Reader/ folder
/// - Has cellebrite.db or pa.db
/// - XML reports
fn is_cellebrite_pa(path: &Path) -> bool {
    if path.is_dir() {
        // Check for PA-specific folders
        let ufd_report = path.join("UFD Report");
        let ufed_reader = path.join("UFED_Reader");
        let report_xml = path.join("report.xml");

        if ufd_report.exists() || ufed_reader.exists() || report_xml.exists() {
            return true;
        }

        // Check for cellebrite database files
        let has_cellebrite_db = fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.contains("cellebrite") || name == "pa.db"
                })
            })
            .unwrap_or(false);

        if has_cellebrite_db {
            return true;
        }
    }

    false
}

/// Check if path is X-Ways case
fn is_xways(path: &Path) -> bool {
    if path.is_file() {
        if let Some(ext) = path.extension() {
            // X-Ways case container
            if ext == "ctx" {
                return true;
            }
        }
    } else if path.is_dir() {
        // Check for X-Ways files
        let has_xways = fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "ctx" || ext == "xfc")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if has_xways {
            return true;
        }
    }

    false
}

/// Check if path is Autopsy case
fn is_autopsy(path: &Path) -> bool {
    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext == "aut" {
                return true;
            }
        }
    } else if path.is_dir() {
        // Check for Autopsy case folder structure
        let autopsy_db = path.join("autopsy.db");
        let case_aut = fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "aut")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if autopsy_db.exists() || case_aut {
            return true;
        }
    }

    false
}

/// Check if path is EnCase case
fn is_encase(path: &Path) -> bool {
    if path.is_file() {
        if let Some(ext) = path.extension() {
            // EnCase case file or logical evidence
            if ext == "case" || ext == "LEF" || ext == "L01" {
                // Note: L01 is also handled by EWF, but in case context it's EnCase
                return ext == "case" || ext == "LEF";
            }
        }
    } else if path.is_dir() {
        // Check for EnCase case structure
        let has_encase = fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "case" || ext == "LEF")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if has_encase {
            return true;
        }
    }

    false
}

/// Check if path is FTK case
fn is_ftk(path: &Path) -> bool {
    if path.is_dir() {
        // FTK has .ftk case files
        let has_ftk = fs::read_dir(path)
            .ok()
            .map(|entries| {
                entries.filter_map(|e| e.ok()).any(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    e.path()
                        .extension()
                        .map(|ext| ext == "ftk")
                        .unwrap_or(false)
                        || name.contains("ftk")
                })
            })
            .unwrap_or(false);

        if has_ftk {
            return true;
        }
    }

    false
}

/// Scan a directory for processed databases
pub fn scan_for_processed_dbs(root: &Path, recursive: bool) -> Vec<ProcessedDbInfo> {
    let mut results = Vec::new();
    scan_for_processed_dbs_internal(root, recursive, 0, &mut results);
    results
}

fn scan_for_processed_dbs_internal(
    root: &Path,
    recursive: bool,
    depth: usize,
    results: &mut Vec<ProcessedDbInfo>,
) {
    if results.len() >= PROCESSED_SCAN_MAX_RESULTS {
        return;
    }

    if depth > PROCESSED_SCAN_MAX_DEPTH {
        tracing::warn!(
            "Skipping processed DB scan directory {}: maximum depth {} exceeded",
            root.display(),
            PROCESSED_SCAN_MAX_DEPTH
        );
        return;
    }

    if !root.exists() || !root.is_dir() {
        return;
    }

    debug!("Scanning for processed databases: {}", root.display());

    // Check if root itself is a processed DB
    if let Some(db_type) = detect_processed_db(root) {
        if let Some(info) = get_processed_db_info(root, db_type) {
            results.push(info);
            return; // Don't recurse into a processed DB
        }
    }

    // Scan directory entries
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if let Some(db_type) = detect_processed_db(&path) {
                if let Some(info) = get_processed_db_info(&path, db_type) {
                    results.push(info);
                    if results.len() >= PROCESSED_SCAN_MAX_RESULTS {
                        tracing::warn!(
                            "Stopping processed DB scan at {} results",
                            PROCESSED_SCAN_MAX_RESULTS
                        );
                        break;
                    }
                }
            } else if recursive && path.is_dir() {
                // Recurse into subdirectories
                scan_for_processed_dbs_internal(&path, true, depth + 1, results);
                if results.len() >= PROCESSED_SCAN_MAX_RESULTS {
                    break;
                }
            }
        }
    }
}

/// Get detailed info about a processed database
pub fn get_processed_db_info(path: &Path, db_type: ProcessedDbType) -> Option<ProcessedDbInfo> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Calculate total size
    let total_size = calculate_dir_size(path);

    // Find database files
    let database_files = find_database_files(path, db_type);

    // Try to extract case info from the name
    let case_number = extract_case_number(&name);

    Some(bounded_processed_db_info(ProcessedDbInfo {
        db_type,
        path: path.to_path_buf(),
        name,
        case_number,
        examiner: None,
        created_date: None,
        total_size,
        artifact_count: None,
        database_files,
        notes: None,
    }))
}

/// Calculate total size of a directory
fn calculate_dir_size(path: &Path) -> u64 {
    let mut files_seen = 0usize;
    calculate_dir_size_bounded(path, 0, &mut files_seen)
}

fn calculate_dir_size_bounded(path: &Path, depth: usize, files_seen: &mut usize) -> u64 {
    if *files_seen >= PROCESSED_SIZE_MAX_FILES {
        return 0;
    }

    if depth > PROCESSED_SCAN_MAX_DEPTH {
        tracing::warn!(
            "Skipping processed DB size directory {}: maximum depth {} exceeded",
            path.display(),
            PROCESSED_SCAN_MAX_DEPTH
        );
        return 0;
    }

    if path.is_file() {
        *files_seen += 1;
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if *files_seen >= PROCESSED_SIZE_MAX_FILES {
                tracing::warn!(
                    "Stopping processed DB size calculation at {} files",
                    PROCESSED_SIZE_MAX_FILES
                );
                break;
            }
            let entry_path = entry.path();
            if entry_path.is_file() {
                *files_seen += 1;
                total = total.saturating_add(entry_path.metadata().map(|m| m.len()).unwrap_or(0));
            } else if entry_path.is_dir() {
                total = total.saturating_add(calculate_dir_size_bounded(
                    &entry_path,
                    depth + 1,
                    files_seen,
                ));
            }
        }
    }
    total
}

/// Find database files within a processed database
fn find_database_files(path: &Path, db_type: ProcessedDbType) -> Vec<DatabaseFile> {
    let mut files = Vec::new();

    if !path.is_dir() {
        // Single file
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            files.push(DatabaseFile {
                path: path.to_path_buf(),
                name: name.to_string(),
                size: path.metadata().map(|m| m.len()).unwrap_or(0),
                contents: classify_database_contents(name, db_type),
            });
        }
        return bounded_database_files(files);
    }

    // Scan for database files
    let extensions = match db_type {
        ProcessedDbType::MagnetAxiom => vec!["mfdb", "db", "sqlite", "mcfc"],
        ProcessedDbType::CellebritePA => vec!["db", "sqlite", "xml"],
        ProcessedDbType::XWays => vec!["ctx", "xfc", "db"],
        ProcessedDbType::Autopsy => vec!["db", "aut"],
        ProcessedDbType::EnCase => vec!["case", "LEF", "db"],
        ProcessedDbType::FTK => vec!["ftk", "db"],
        _ => vec!["db", "sqlite"],
    };

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if files.len() >= PROCESSED_DATABASE_FILES_MAX {
                tracing::warn!(
                    "Stopping processed DB file listing at {} files",
                    PROCESSED_DATABASE_FILES_MAX
                );
                break;
            }
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            files.push(DatabaseFile {
                                path: entry_path.clone(),
                                name: name.to_string(),
                                size: entry_path.metadata().map(|m| m.len()).unwrap_or(0),
                                contents: classify_database_contents(name, db_type),
                            });
                        }
                    }
                }
            }
        }
    }

    bounded_database_files(files)
}

fn bounded_processed_db_info(mut info: ProcessedDbInfo) -> ProcessedDbInfo {
    info.name = truncate_processed_text(info.name);
    info.case_number = info.case_number.map(truncate_processed_text);
    info.examiner = info.examiner.map(truncate_processed_text);
    info.created_date = info.created_date.map(truncate_processed_text);
    info.notes = info.notes.map(truncate_processed_text);
    info.database_files = bounded_database_files(info.database_files);
    info
}

fn bounded_database_files(mut files: Vec<DatabaseFile>) -> Vec<DatabaseFile> {
    files.truncate(PROCESSED_DATABASE_FILES_MAX);
    files
        .into_iter()
        .map(|mut file| {
            file.name = truncate_processed_text(file.name);
            file
        })
        .collect()
}

fn truncate_processed_text(value: String) -> String {
    if value.chars().count() <= PROCESSED_TEXT_FIELD_MAX_CHARS {
        value
    } else {
        value.chars().take(PROCESSED_TEXT_FIELD_MAX_CHARS).collect()
    }
}

/// Classify what a database file contains based on its name
fn classify_database_contents(name: &str, _db_type: ProcessedDbType) -> DatabaseContents {
    let lower = name.to_lowercase();

    if lower.contains("case") || lower.contains("mcfc") {
        DatabaseContents::CaseInfo
    } else if lower.contains("artifact") {
        DatabaseContents::Artifacts
    } else if lower.contains("file") || lower.contains("fs") {
        DatabaseContents::FileSystem
    } else if lower.contains("keyword") || lower.contains("search") {
        DatabaseContents::Keywords
    } else if lower.contains("hash") {
        DatabaseContents::Hashes
    } else if lower.contains("media") || lower.contains("thumb") {
        DatabaseContents::Media
    } else if lower.contains("timeline") || lower.contains("time") {
        DatabaseContents::Timeline
    } else if lower.contains("bookmark") || lower.contains("tag") {
        DatabaseContents::Bookmarks
    } else if lower.contains("report") {
        DatabaseContents::Reports
    } else if lower.contains("config") || lower.contains("setting") {
        DatabaseContents::Config
    } else {
        DatabaseContents::Unknown
    }
}

/// Try to extract case number from a name string
fn extract_case_number(name: &str) -> Option<String> {
    // Common patterns: 24-042, 2024-001, CASE-001, etc.
    let re_patterns = [
        r"\d{2,4}-\d{3,4}", // 24-042 or 2024-001
        r"CASE-?\d+",       // CASE001 or CASE-001
    ];

    for pattern in re_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(m) = re.find(name) {
                return Some(m.as_str().to_string());
            }
        }
    }

    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_nonexistent_path() {
        let path = Path::new("/nonexistent/path/to/database");
        assert!(detect_processed_db(path).is_none());
    }

    #[test]
    fn test_extract_case_number_24_format() {
        let result = extract_case_number("Case_24-042_Evidence");
        assert_eq!(result, Some("24-042".to_string()));
    }

    #[test]
    fn test_extract_case_number_year_format() {
        let result = extract_case_number("Investigation_2024-001");
        assert_eq!(result, Some("2024-001".to_string()));
    }

    #[test]
    fn test_extract_case_number_case_prefix() {
        let result = extract_case_number("CASE-123_Analysis");
        assert_eq!(result, Some("CASE-123".to_string()));
    }

    #[test]
    fn test_extract_case_number_case_no_dash() {
        let result = extract_case_number("CASE456_Files");
        assert_eq!(result, Some("CASE456".to_string()));
    }

    #[test]
    fn test_extract_case_number_none() {
        let result = extract_case_number("MyExtractionFolder");
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_database_contents_case_info() {
        let result = classify_database_contents("Case.mfdb", ProcessedDbType::MagnetAxiom);
        assert_eq!(result, DatabaseContents::CaseInfo);

        let result = classify_database_contents("case_info.mcfc", ProcessedDbType::MagnetAxiom);
        assert_eq!(result, DatabaseContents::CaseInfo);
    }

    #[test]
    fn test_classify_database_contents_artifacts() {
        let result = classify_database_contents("artifacts.db", ProcessedDbType::MagnetAxiom);
        assert_eq!(result, DatabaseContents::Artifacts);
    }

    #[test]
    fn test_classify_database_contents_filesystem() {
        let result = classify_database_contents("file_listing.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::FileSystem);

        let result = classify_database_contents("fs_data.sqlite", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::FileSystem);
    }

    #[test]
    fn test_classify_database_contents_keywords() {
        let result = classify_database_contents("keyword_search.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Keywords);

        let result = classify_database_contents("search_results.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Keywords);
    }

    #[test]
    fn test_classify_database_contents_hashes() {
        let result = classify_database_contents("hash_values.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Hashes);
    }

    #[test]
    fn test_classify_database_contents_media() {
        let result = classify_database_contents("media.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Media);

        let result = classify_database_contents("thumbnails.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Media);
    }

    #[test]
    fn test_classify_database_contents_timeline() {
        let result = classify_database_contents("timeline.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Timeline);
    }

    #[test]
    fn test_classify_database_contents_bookmarks() {
        let result = classify_database_contents("bookmarks.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Bookmarks);

        let result = classify_database_contents("tagged_items.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Bookmarks);
    }

    #[test]
    fn test_classify_database_contents_reports() {
        let result = classify_database_contents("report_data.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Reports);
    }

    #[test]
    fn test_classify_database_contents_config() {
        let result = classify_database_contents("config.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Config);

        let result = classify_database_contents("settings.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Config);
    }

    #[test]
    fn test_classify_database_contents_unknown() {
        let result = classify_database_contents("random_data.db", ProcessedDbType::Unknown);
        assert_eq!(result, DatabaseContents::Unknown);
    }

    #[test]
    fn test_is_axiom_database_file() {
        let temp_dir = TempDir::new().unwrap();
        let mfdb_path = temp_dir.path().join("Case.mfdb");
        fs::write(&mfdb_path, b"test").unwrap();

        assert!(is_axiom_database(&mfdb_path));
    }

    #[test]
    fn test_is_axiom_database_folder_with_mcfc() {
        let temp_dir = TempDir::new().unwrap();
        let axiom_dir = temp_dir.path().join("AXIOM_Case");
        fs::create_dir(&axiom_dir).unwrap();
        fs::write(axiom_dir.join("Case.mcfc"), b"test").unwrap();

        assert!(is_axiom_database(&axiom_dir));
    }

    #[test]
    fn test_is_cellebrite_pa_with_report() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("report.xml"), b"<report/>").unwrap();

        assert!(is_cellebrite_pa(temp_dir.path()));
    }

    #[test]
    fn test_is_xways_ctx_file() {
        let temp_dir = TempDir::new().unwrap();
        let ctx_path = temp_dir.path().join("case.ctx");
        fs::write(&ctx_path, b"test").unwrap();

        assert!(is_xways(&ctx_path));
    }

    #[test]
    fn test_is_autopsy_aut_file() {
        let temp_dir = TempDir::new().unwrap();
        let aut_path = temp_dir.path().join("mycase.aut");
        fs::write(&aut_path, b"test").unwrap();

        assert!(is_autopsy(&aut_path));
    }

    #[test]
    fn test_is_encase_case_file() {
        let temp_dir = TempDir::new().unwrap();
        let case_path = temp_dir.path().join("investigation.case");
        fs::write(&case_path, b"test").unwrap();

        assert!(is_encase(&case_path));
    }

    #[test]
    fn test_is_encase_lef_file() {
        let temp_dir = TempDir::new().unwrap();
        let lef_path = temp_dir.path().join("evidence.LEF");
        fs::write(&lef_path, b"test").unwrap();

        assert!(is_encase(&lef_path));
    }

    #[test]
    fn test_is_ftk_folder_with_ftk_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("case.ftk"), b"test").unwrap();

        assert!(is_ftk(temp_dir.path()));
    }

    #[test]
    fn test_scan_for_processed_dbs_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let results = scan_for_processed_dbs(temp_dir.path(), false);
        assert!(results.is_empty());
    }

    #[test]
    fn scan_for_processed_dbs_skips_beyond_depth_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut current = temp_dir.path().to_path_buf();
        for index in 0..=PROCESSED_SCAN_MAX_DEPTH + 1 {
            current = current.join(format!("d{}", index));
            fs::create_dir(&current).unwrap();
        }
        fs::write(current.join("case.aut"), b"test").unwrap();

        let results = scan_for_processed_dbs(temp_dir.path(), true);

        assert!(results.is_empty());
    }

    #[test]
    fn scan_for_processed_dbs_internal_respects_existing_result_cap() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("case.aut"), b"test").unwrap();
        let mut results = (0..PROCESSED_SCAN_MAX_RESULTS)
            .map(|index| ProcessedDbInfo {
                db_type: ProcessedDbType::Autopsy,
                path: std::path::PathBuf::from(format!("/case/{index}")),
                name: format!("case-{index}"),
                ..Default::default()
            })
            .collect();

        scan_for_processed_dbs_internal(temp_dir.path(), false, 0, &mut results);

        assert_eq!(results.len(), PROCESSED_SCAN_MAX_RESULTS);
    }

    #[test]
    fn bounded_processed_db_info_caps_database_files_and_text() {
        let long_text = "x".repeat(PROCESSED_TEXT_FIELD_MAX_CHARS + 1);
        let info = ProcessedDbInfo {
            db_type: ProcessedDbType::Autopsy,
            path: std::path::PathBuf::from("/case"),
            name: long_text.clone(),
            case_number: Some(long_text.clone()),
            examiner: Some(long_text.clone()),
            created_date: Some(long_text.clone()),
            total_size: 0,
            artifact_count: None,
            database_files: (0..PROCESSED_DATABASE_FILES_MAX + 1)
                .map(|index| DatabaseFile {
                    path: std::path::PathBuf::from(format!("/case/db-{index}.db")),
                    name: long_text.clone(),
                    size: 1,
                    contents: DatabaseContents::Unknown,
                })
                .collect(),
            notes: Some(long_text.clone()),
        };

        let bounded = bounded_processed_db_info(info);

        assert_eq!(bounded.database_files.len(), PROCESSED_DATABASE_FILES_MAX);
        assert_eq!(bounded.name.chars().count(), PROCESSED_TEXT_FIELD_MAX_CHARS);
        assert_eq!(
            bounded.case_number.as_ref().unwrap().chars().count(),
            PROCESSED_TEXT_FIELD_MAX_CHARS
        );
        assert!(bounded
            .database_files
            .iter()
            .all(|file| file.name.chars().count() <= PROCESSED_TEXT_FIELD_MAX_CHARS));
    }

    #[test]
    fn test_processed_db_type_as_str() {
        assert_eq!(ProcessedDbType::MagnetAxiom.as_str(), "Magnet AXIOM");
        assert_eq!(ProcessedDbType::CellebritePA.as_str(), "Cellebrite PA");
        assert_eq!(ProcessedDbType::XWays.as_str(), "X-Ways");
        assert_eq!(ProcessedDbType::Autopsy.as_str(), "Autopsy");
        assert_eq!(ProcessedDbType::EnCase.as_str(), "EnCase");
        assert_eq!(ProcessedDbType::FTK.as_str(), "FTK");
    }
}
