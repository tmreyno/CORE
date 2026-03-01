// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Cellebrite Physical Analyzer (PA) parser
//!
//! Handles extracted UFDR/PA output folders containing:
//! - `report.xml` — Main XML report with device info, extractions, artifacts
//! - `UFD Report/` — Report folder with HTML/PDF exports
//! - `UFED_Reader/` — Reader application folder
//! - SQLite databases (cellebrite.db, pa.db) with parsed artifacts
//!
//! Cellebrite PA exports typically follow this structure:
//! ```text
//! <case_folder>/
//!   ├── report.xml             # Main XML report
//!   ├── UFD Report/            # Optional HTML/PDF report
//!   ├── UFED_Reader/           # Optional reader app
//!   ├── cellebrite.db          # SQLite artifact database
//!   ├── *.ufd                  # UFDR extraction files
//!   └── files/                 # Extracted file artifacts
//! ```

use crate::containers::ContainerError;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

// =============================================================================
// Types
// =============================================================================

/// Cellebrite PA case information parsed from report.xml and databases
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellebriteCaseInfo {
    /// Case name
    pub case_name: String,
    /// Case number
    pub case_number: Option<String>,
    /// Examiner name
    pub examiner: Option<String>,
    /// Agency/organization
    pub agency: Option<String>,
    /// Device name/model
    pub device_name: Option<String>,
    /// Device model
    pub device_model: Option<String>,
    /// Device OS version
    pub os_version: Option<String>,
    /// IMEI/MEID
    pub imei: Option<String>,
    /// Serial number
    pub serial_number: Option<String>,
    /// ICCID (SIM card identifier)
    pub iccid: Option<String>,
    /// MSISDN (phone number)
    pub msisdn: Option<String>,
    /// Extraction type (Physical, Logical, File System, etc.)
    pub extraction_type: Option<String>,
    /// Extraction date
    pub extraction_date: Option<String>,
    /// PA version used
    pub pa_version: Option<String>,
    /// UFED version used
    pub ufed_version: Option<String>,
    /// Total artifact count
    pub total_artifacts: u64,
    /// Artifact categories with counts
    pub artifact_categories: Vec<CellebriteArtifactCategory>,
    /// Data sources in the extraction
    pub data_sources: Vec<CellebriteDataSource>,
    /// Case folder path
    pub case_path: Option<String>,
    /// Whether report.xml was found
    pub has_report_xml: bool,
    /// Whether SQLite database was found
    pub has_database: bool,
}

/// Artifact category with count
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellebriteArtifactCategory {
    /// Category name (e.g., "Chat", "Contacts", "Web History")
    pub name: String,
    /// Number of artifacts in this category
    pub count: u64,
}

/// Data source within a Cellebrite extraction
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellebriteDataSource {
    /// Source name
    pub name: String,
    /// Source type (e.g., "Physical", "Logical", "File System")
    pub source_type: String,
    /// Extraction timestamp
    pub timestamp: Option<String>,
    /// Source path or identifier
    pub path: Option<String>,
}

// =============================================================================
// Public API
// =============================================================================

/// Parse Cellebrite PA case from a folder or report.xml path
pub fn parse_cellebrite_case(path: &Path) -> Result<CellebriteCaseInfo, ContainerError> {
    let case_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    };

    let mut info = CellebriteCaseInfo {
        case_path: Some(case_dir.to_string_lossy().to_string()),
        ..Default::default()
    };

    // 1. Try report.xml (primary source)
    let report_xml = case_dir.join("report.xml");
    if report_xml.exists() {
        debug!("Parsing Cellebrite report.xml: {}", report_xml.display());
        info.has_report_xml = true;
        if let Ok(xml_info) = parse_report_xml(&report_xml) {
            info = merge_xml_info(info, xml_info);
        }
    }

    // 2. Try SQLite databases
    let db_files = find_cellebrite_databases(&case_dir);
    if !db_files.is_empty() {
        info.has_database = true;
        for db_path in &db_files {
            debug!("Parsing Cellebrite database: {}", db_path.display());
            if let Ok(db_info) = parse_cellebrite_db(db_path) {
                info = merge_db_info(info, db_info);
            }
        }
    }

    // 3. Default case name from folder
    if info.case_name.is_empty() {
        info.case_name = case_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Cellebrite Case")
            .to_string();
    }

    // 4. Calculate total from categories
    if info.total_artifacts == 0 && !info.artifact_categories.is_empty() {
        info.total_artifacts = info.artifact_categories.iter().map(|c| c.count).sum();
    }

    Ok(info)
}

/// Get artifact category summary from a Cellebrite database
pub fn get_cellebrite_categories(
    path: &Path,
) -> Result<Vec<CellebriteArtifactCategory>, ContainerError> {
    let case_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    };

    let db_files = find_cellebrite_databases(&case_dir);
    for db_path in &db_files {
        if let Ok(categories) = query_artifact_categories_from_db(db_path) {
            if !categories.is_empty() {
                return Ok(categories);
            }
        }
    }

    Ok(Vec::new())
}

// =============================================================================
// report.xml parsing
// =============================================================================

/// Partial info extracted from report.xml
#[derive(Default)]
struct XmlCaseInfo {
    case_name: String,
    case_number: Option<String>,
    examiner: Option<String>,
    device_name: Option<String>,
    device_model: Option<String>,
    os_version: Option<String>,
    imei: Option<String>,
    serial_number: Option<String>,
    iccid: Option<String>,
    msisdn: Option<String>,
    extraction_type: Option<String>,
    extraction_date: Option<String>,
    pa_version: Option<String>,
    ufed_version: Option<String>,
    artifact_categories: Vec<CellebriteArtifactCategory>,
    data_sources: Vec<CellebriteDataSource>,
}

/// Parse report.xml for case metadata
fn parse_report_xml(path: &Path) -> Result<XmlCaseInfo, ContainerError> {
    let content = fs::read_to_string(path)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut info = XmlCaseInfo::default();
    let mut current_element = String::new();
    let mut in_device_info = false;
    let mut in_case_info = false;
    let mut in_extraction_data = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_element = tag.clone();

                match tag.as_str() {
                    "deviceInfo" | "DeviceInfo" | "device" => in_device_info = true,
                    "caseInformation" | "CaseInformation" | "case" => in_case_info = true,
                    "extractionData" | "ExtractionData" | "extraction" => {
                        in_extraction_data = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "deviceInfo" | "DeviceInfo" | "device" => in_device_info = false,
                    "caseInformation" | "CaseInformation" | "case" => in_case_info = false,
                    "extractionData" | "ExtractionData" | "extraction" => {
                        in_extraction_data = false;
                    }
                    _ => {}
                }
                current_element.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }

                // Device info fields
                if in_device_info {
                    match current_element.as_str() {
                        "deviceName" | "DeviceName" | "name" => {
                            info.device_name = Some(text.clone());
                        }
                        "deviceModel" | "DeviceModel" | "model" => {
                            info.device_model = Some(text.clone());
                        }
                        "osVersion" | "OSVersion" | "os" => {
                            info.os_version = Some(text.clone());
                        }
                        "imei" | "IMEI" => {
                            info.imei = Some(text.clone());
                        }
                        "serialNumber" | "SerialNumber" | "serial" => {
                            info.serial_number = Some(text.clone());
                        }
                        "iccid" | "ICCID" => {
                            info.iccid = Some(text.clone());
                        }
                        "msisdn" | "MSISDN" | "phoneNumber" => {
                            info.msisdn = Some(text.clone());
                        }
                        _ => {}
                    }
                }

                // Case info fields
                if in_case_info {
                    match current_element.as_str() {
                        "caseName" | "CaseName" | "name" => {
                            info.case_name = text.clone();
                        }
                        "caseNumber" | "CaseNumber" | "number" => {
                            info.case_number = Some(text.clone());
                        }
                        "examiner" | "Examiner" | "examinerName" => {
                            info.examiner = Some(text.clone());
                        }
                        _ => {}
                    }
                }

                // Extraction fields
                if in_extraction_data {
                    match current_element.as_str() {
                        "extractionType" | "ExtractionType" | "type" => {
                            info.extraction_type = Some(text.clone());
                        }
                        "extractionDate" | "ExtractionDate" | "date" | "timestamp" => {
                            info.extraction_date = Some(text.clone());
                        }
                        "ufedVersion" | "UfedVersion" => {
                            info.ufed_version = Some(text.clone());
                        }
                        "paVersion" | "PAVersion" | "version" => {
                            info.pa_version = Some(text.clone());
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing Cellebrite report.xml: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(info)
}

// =============================================================================
// SQLite database parsing
// =============================================================================

/// Find Cellebrite SQLite databases in a folder
fn find_cellebrite_databases(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut dbs = Vec::new();

    // Known Cellebrite database names
    let known_names = [
        "cellebrite.db",
        "pa.db",
        "extraction.db",
        "report.db",
        "ufed_data.db",
    ];

    for name in &known_names {
        let db_path = dir.join(name);
        if db_path.exists() {
            dbs.push(db_path);
        }
    }

    // Also check for any .db files with "cellebrite" in the name
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "db").unwrap_or(false) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if name.contains("cellebrite") && !dbs.contains(&path) {
                    dbs.push(path);
                }
            }
        }
    }

    dbs
}

/// Partial info from database
#[derive(Default)]
struct DbCaseInfo {
    device_name: Option<String>,
    device_model: Option<String>,
    os_version: Option<String>,
    imei: Option<String>,
    extraction_type: Option<String>,
    categories: Vec<CellebriteArtifactCategory>,
    total_artifacts: u64,
}

/// Parse a Cellebrite SQLite database for case info
fn parse_cellebrite_db(path: &Path) -> Result<DbCaseInfo, ContainerError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut info = DbCaseInfo::default();

    // Try to get device info from common table names
    let device_tables = ["device_info", "DeviceInfo", "deviceinfo", "metadata"];

    for table in &device_tables {
        if table_exists(&conn, table) {
            // Try common column patterns
            if let Ok(val) = query_single_string(&conn, table, "device_name") {
                info.device_name = Some(val);
            } else if let Ok(val) = query_single_string(&conn, table, "name") {
                info.device_name = Some(val);
            }

            if let Ok(val) = query_single_string(&conn, table, "model") {
                info.device_model = Some(val);
            }

            if let Ok(val) = query_single_string(&conn, table, "os_version") {
                info.os_version = Some(val);
            }

            if let Ok(val) = query_single_string(&conn, table, "imei") {
                info.imei = Some(val);
            }

            break;
        }
    }

    // Try to get artifact categories
    info.categories = query_artifact_categories_from_conn(&conn);
    info.total_artifacts = info.categories.iter().map(|c| c.count).sum();

    Ok(info)
}

/// Query artifact categories from a Cellebrite database connection
fn query_artifact_categories_from_conn(conn: &Connection) -> Vec<CellebriteArtifactCategory> {
    // Try common artifact table patterns
    let queries = [
        // Pattern 1: artifact table with category column
        "SELECT category, COUNT(*) as cnt FROM artifacts GROUP BY category ORDER BY cnt DESC",
        // Pattern 2: separate category table
        "SELECT name, item_count FROM categories ORDER BY item_count DESC",
        // Pattern 3: type-based grouping
        "SELECT type, COUNT(*) as cnt FROM items GROUP BY type ORDER BY cnt DESC",
        // Pattern 4: source table
        "SELECT source_type, COUNT(*) as cnt FROM data_sources GROUP BY source_type ORDER BY cnt DESC",
    ];

    for query in &queries {
        if let Ok(mut stmt) = conn.prepare(query) {
            let mut categories = Vec::new();
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(CellebriteArtifactCategory {
                    name: row.get::<_, String>(0).unwrap_or_default(),
                    count: row.get::<_, i64>(1).unwrap_or(0) as u64,
                })
            }) {
                for row in rows.flatten() {
                    if !row.name.is_empty() {
                        categories.push(row);
                    }
                }
                if !categories.is_empty() {
                    return categories;
                }
            }
        }
    }

    Vec::new()
}

/// Query artifact categories from a database file path
fn query_artifact_categories_from_db(
    path: &Path,
) -> Result<Vec<CellebriteArtifactCategory>, ContainerError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(query_artifact_categories_from_conn(&conn))
}

// =============================================================================
// Helpers
// =============================================================================

/// Check if a table exists in a SQLite database
fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.prepare(&format!(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}'",
        table
    ))
    .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
    .is_ok()
}

/// Query a single string value from a table
fn query_single_string(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<String, ContainerError> {
    let sql = format!("SELECT {} FROM {} LIMIT 1", column, table);
    let val = conn.query_row(&sql, [], |row| row.get::<_, String>(0))?;
    Ok(val)
}

/// Merge XML-parsed info into the main struct
fn merge_xml_info(mut base: CellebriteCaseInfo, xml: XmlCaseInfo) -> CellebriteCaseInfo {
    if !xml.case_name.is_empty() {
        base.case_name = xml.case_name;
    }
    base.case_number = base.case_number.or(xml.case_number);
    base.examiner = base.examiner.or(xml.examiner);
    base.device_name = base.device_name.or(xml.device_name);
    base.device_model = base.device_model.or(xml.device_model);
    base.os_version = base.os_version.or(xml.os_version);
    base.imei = base.imei.or(xml.imei);
    base.serial_number = base.serial_number.or(xml.serial_number);
    base.iccid = base.iccid.or(xml.iccid);
    base.msisdn = base.msisdn.or(xml.msisdn);
    base.extraction_type = base.extraction_type.or(xml.extraction_type);
    base.extraction_date = base.extraction_date.or(xml.extraction_date);
    base.pa_version = base.pa_version.or(xml.pa_version);
    base.ufed_version = base.ufed_version.or(xml.ufed_version);

    if !xml.artifact_categories.is_empty() {
        base.artifact_categories = xml.artifact_categories;
    }
    if !xml.data_sources.is_empty() {
        base.data_sources = xml.data_sources;
    }

    base
}

/// Merge database info into the main struct (only fill blanks)
fn merge_db_info(mut base: CellebriteCaseInfo, db: DbCaseInfo) -> CellebriteCaseInfo {
    base.device_name = base.device_name.or(db.device_name);
    base.device_model = base.device_model.or(db.device_model);
    base.os_version = base.os_version.or(db.os_version);
    base.imei = base.imei.or(db.imei);
    base.extraction_type = base.extraction_type.or(db.extraction_type);

    if base.artifact_categories.is_empty() && !db.categories.is_empty() {
        base.artifact_categories = db.categories;
    }
    if base.total_artifacts == 0 && db.total_artifacts > 0 {
        base.total_artifacts = db.total_artifacts;
    }

    base
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cellebrite_case_info_default() {
        let info = CellebriteCaseInfo::default();
        assert!(info.case_name.is_empty());
        assert_eq!(info.total_artifacts, 0);
        assert!(!info.has_report_xml);
        assert!(!info.has_database);
    }

    #[test]
    fn test_cellebrite_case_info_serialization() {
        let info = CellebriteCaseInfo {
            case_name: "Test Case".to_string(),
            case_number: Some("2024-001".to_string()),
            device_name: Some("iPhone 15 Pro".to_string()),
            total_artifacts: 5000,
            ..Default::default()
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("caseName"));
        assert!(json.contains("caseNumber"));
        assert!(json.contains("deviceName"));
        assert!(json.contains("totalArtifacts"));

        let deser: CellebriteCaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.case_name, "Test Case");
        assert_eq!(deser.case_number, Some("2024-001".to_string()));
        assert_eq!(deser.total_artifacts, 5000);
    }

    #[test]
    fn test_parse_report_xml_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<report>
  <caseInformation>
    <caseName>Smith Investigation</caseName>
    <caseNumber>2024-042</caseNumber>
    <examiner>Jane Doe</examiner>
  </caseInformation>
  <deviceInfo>
    <deviceName>Samsung Galaxy S23</deviceName>
    <deviceModel>SM-S911B</deviceModel>
    <osVersion>Android 14</osVersion>
    <IMEI>123456789012345</IMEI>
    <serialNumber>R5CT12ABC</serialNumber>
  </deviceInfo>
  <extractionData>
    <extractionType>Physical</extractionType>
    <extractionDate>2024-03-15T10:30:00</extractionDate>
    <paVersion>7.65</paVersion>
  </extractionData>
</report>"#;

        let dir = TempDir::new().unwrap();
        let xml_path = dir.path().join("report.xml");
        fs::write(&xml_path, xml).unwrap();

        let info = parse_report_xml(&xml_path).unwrap();
        assert_eq!(info.case_name, "Smith Investigation");
        assert_eq!(info.case_number, Some("2024-042".to_string()));
        assert_eq!(info.examiner, Some("Jane Doe".to_string()));
        assert_eq!(info.device_name, Some("Samsung Galaxy S23".to_string()));
        assert_eq!(info.device_model, Some("SM-S911B".to_string()));
        assert_eq!(info.os_version, Some("Android 14".to_string()));
        assert_eq!(info.imei, Some("123456789012345".to_string()));
        assert_eq!(info.serial_number, Some("R5CT12ABC".to_string()));
        assert_eq!(info.extraction_type, Some("Physical".to_string()));
        assert_eq!(info.pa_version, Some("7.65".to_string()));
    }

    #[test]
    fn test_parse_cellebrite_case_folder_with_xml() {
        let dir = TempDir::new().unwrap();
        let xml = r#"<?xml version="1.0"?>
<report>
  <caseInformation>
    <caseName>Mobile Extract</caseName>
  </caseInformation>
  <deviceInfo>
    <deviceName>Pixel 8</deviceName>
  </deviceInfo>
</report>"#;

        fs::write(dir.path().join("report.xml"), xml).unwrap();

        let result = parse_cellebrite_case(dir.path()).unwrap();
        assert_eq!(result.case_name, "Mobile Extract");
        assert_eq!(result.device_name, Some("Pixel 8".to_string()));
        assert!(result.has_report_xml);
    }

    #[test]
    fn test_parse_cellebrite_case_no_xml_uses_folder_name() {
        let dir = TempDir::new().unwrap();
        let result = parse_cellebrite_case(dir.path()).unwrap();
        // Should use folder name as case name
        assert!(!result.case_name.is_empty());
        assert!(!result.has_report_xml);
    }

    #[test]
    fn test_find_cellebrite_databases_empty() {
        let dir = TempDir::new().unwrap();
        let dbs = find_cellebrite_databases(dir.path());
        assert!(dbs.is_empty());
    }

    #[test]
    fn test_find_cellebrite_databases_known_names() {
        let dir = TempDir::new().unwrap();

        // Create known database files
        fs::write(dir.path().join("cellebrite.db"), b"dummy").unwrap();
        fs::write(dir.path().join("pa.db"), b"dummy").unwrap();
        fs::write(dir.path().join("unrelated.db"), b"dummy").unwrap();

        let dbs = find_cellebrite_databases(dir.path());
        assert_eq!(dbs.len(), 2);
    }

    #[test]
    fn test_merge_xml_info_fills_blanks() {
        let base = CellebriteCaseInfo::default();
        let xml = XmlCaseInfo {
            case_name: "XML Case".to_string(),
            device_name: Some("iPhone".to_string()),
            ..Default::default()
        };

        let merged = merge_xml_info(base, xml);
        assert_eq!(merged.case_name, "XML Case");
        assert_eq!(merged.device_name, Some("iPhone".to_string()));
    }

    #[test]
    fn test_merge_xml_info_does_not_overwrite() {
        let base = CellebriteCaseInfo {
            case_number: Some("BASE-001".to_string()),
            ..Default::default()
        };
        let xml = XmlCaseInfo {
            case_number: Some("XML-002".to_string()),
            ..Default::default()
        };

        let merged = merge_xml_info(base, xml);
        // base value takes precedence (already set)
        assert_eq!(merged.case_number, Some("BASE-001".to_string()));
    }

    #[test]
    fn test_merge_db_info_fills_blanks() {
        let base = CellebriteCaseInfo::default();
        let db = DbCaseInfo {
            device_name: Some("Galaxy S24".to_string()),
            total_artifacts: 1234,
            ..Default::default()
        };

        let merged = merge_db_info(base, db);
        assert_eq!(merged.device_name, Some("Galaxy S24".to_string()));
        assert_eq!(merged.total_artifacts, 1234);
    }

    #[test]
    fn test_cellebrite_artifact_category_serialization() {
        let cat = CellebriteArtifactCategory {
            name: "Chat Messages".to_string(),
            count: 4567,
        };

        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("4567"));
    }

    #[test]
    fn test_cellebrite_data_source_serialization() {
        let src = CellebriteDataSource {
            name: "Physical Extraction".to_string(),
            source_type: "Physical".to_string(),
            timestamp: Some("2024-03-15T10:30:00".to_string()),
            path: Some("/evidence/phone.bin".to_string()),
        };

        let json = serde_json::to_string(&src).unwrap();
        let deser: CellebriteDataSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Physical Extraction");
        assert_eq!(deser.source_type, "Physical");
    }

    #[test]
    fn test_table_exists_false_for_missing() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(!table_exists(&conn, "nonexistent"));
    }

    #[test]
    fn test_table_exists_true_for_present() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE test_table (id INTEGER);")
            .unwrap();
        assert!(table_exists(&conn, "test_table"));
    }

    #[test]
    fn test_parse_report_xml_empty() {
        let dir = TempDir::new().unwrap();
        let xml_path = dir.path().join("report.xml");
        fs::write(&xml_path, "<?xml version=\"1.0\"?><report></report>").unwrap();

        let info = parse_report_xml(&xml_path).unwrap();
        assert!(info.case_name.is_empty());
        assert!(info.device_name.is_none());
    }

    #[test]
    fn test_query_artifact_categories_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        let cats = query_artifact_categories_from_conn(&conn);
        assert!(cats.is_empty());
    }

    #[test]
    fn test_query_artifact_categories_with_data() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE artifacts (id INTEGER, category TEXT);
             INSERT INTO artifacts VALUES (1, 'Chat');
             INSERT INTO artifacts VALUES (2, 'Chat');
             INSERT INTO artifacts VALUES (3, 'Contacts');
             INSERT INTO artifacts VALUES (4, 'Web History');",
        )
        .unwrap();

        let cats = query_artifact_categories_from_conn(&conn);
        assert_eq!(cats.len(), 3);
        // Sorted by count DESC
        assert_eq!(cats[0].name, "Chat");
        assert_eq!(cats[0].count, 2);
    }
}
