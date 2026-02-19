// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Autopsy Case Parser
//!
//! Handles Autopsy case folders and `.aut` case files.
//!
//! Autopsy is an open-source digital forensics platform that stores case data
//! in SQLite databases. The typical case structure is:
//!
//! ```text
//! <case_name>/
//!   ├── <case_name>.aut          # Case metadata file (XML)
//!   ├── autopsy.db               # Main SQLite case database
//!   ├── Reports/                  # Generated reports
//!   ├── ModuleOutput/            # Ingest module output
//!   │   ├── HashLookup/
//!   │   ├── KeywordSearch/
//!   │   ├── EmailParser/
//!   │   └── ...
//!   └── Export/                   # Exported files
//! ```
//!
//! The `.aut` file is a simple properties/XML file with case metadata.
//! The `autopsy.db` is a SQLite database with the full case schema.

use std::fs;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::{Connection, OpenFlags};
use tracing::{debug, warn};
use crate::containers::ContainerError;

// =============================================================================
// Types
// =============================================================================

/// Autopsy case information parsed from .aut file and autopsy.db
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopsyCaseInfo {
    /// Case name
    pub case_name: String,
    /// Case number
    pub case_number: Option<String>,
    /// Examiner name
    pub examiner: Option<String>,
    /// Case type (Single-user or Multi-user)
    pub case_type: Option<String>,
    /// Autopsy version
    pub autopsy_version: Option<String>,
    /// Case creation date
    pub created_date: Option<String>,
    /// Case database schema version
    pub schema_version: Option<String>,
    /// Data sources in the case
    pub data_sources: Vec<AutopsyDataSource>,
    /// Ingest modules that have been run
    pub ingest_modules: Vec<AutopsyIngestModule>,
    /// Artifact categories with counts
    pub artifact_categories: Vec<AutopsyArtifactCategory>,
    /// Total artifacts count
    pub total_artifacts: u64,
    /// Total files indexed
    pub total_files: u64,
    /// Total data sources
    pub total_data_sources: u64,
    /// Tags used in the case
    pub tags: Vec<AutopsyTag>,
    /// Case folder path
    pub case_path: Option<String>,
    /// Whether autopsy.db was found
    pub has_database: bool,
    /// Whether .aut file was found
    pub has_aut_file: bool,
}

/// Data source within an Autopsy case
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopsyDataSource {
    /// Data source ID
    pub id: i64,
    /// Display name
    pub name: String,
    /// Device ID
    pub device_id: Option<String>,
    /// Time zone
    pub timezone: Option<String>,
    /// Size in bytes (if known)
    pub size: Option<u64>,
}

/// Ingest module that was run
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopsyIngestModule {
    /// Module display name
    pub name: String,
    /// Module version
    pub version: Option<String>,
}

/// Artifact category with count
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopsyArtifactCategory {
    /// Artifact type name
    pub name: String,
    /// Artifact type ID
    pub type_id: i32,
    /// Count of artifacts
    pub count: u64,
}

/// Tag used in the case
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopsyTag {
    /// Tag name
    pub name: String,
    /// Number of items with this tag
    pub count: u64,
}

// =============================================================================
// Public API
// =============================================================================

/// Parse Autopsy case from a folder or .aut file path
pub fn parse_autopsy_case(path: &Path) -> Result<AutopsyCaseInfo, ContainerError> {
    let case_dir = if path.is_dir() {
        path.to_path_buf()
    } else if path.extension().map(|e| e == "aut").unwrap_or(false) {
        // .aut file — case dir is the parent
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    let mut info = AutopsyCaseInfo::default();
    info.case_path = Some(case_dir.to_string_lossy().to_string());

    // 1. Try .aut file
    let aut_file = find_aut_file(&case_dir);
    if let Some(aut_path) = &aut_file {
        debug!("Parsing Autopsy .aut file: {}", aut_path.display());
        info.has_aut_file = true;
        if let Ok(aut_info) = parse_aut_file(aut_path) {
            info.case_name = aut_info.case_name;
            info.case_number = aut_info.case_number;
            info.examiner = aut_info.examiner;
            info.case_type = aut_info.case_type;
            info.autopsy_version = aut_info.autopsy_version;
            info.created_date = aut_info.created_date;
            info.schema_version = aut_info.schema_version;
        }
    }

    // 2. Try autopsy.db
    let db_path = case_dir.join("autopsy.db");
    if db_path.exists() {
        debug!("Parsing Autopsy database: {}", db_path.display());
        info.has_database = true;
        if let Ok(db_info) = parse_autopsy_db(&db_path) {
            info.data_sources = db_info.data_sources;
            info.ingest_modules = db_info.ingest_modules;
            info.artifact_categories = db_info.artifact_categories;
            info.total_artifacts = db_info.total_artifacts;
            info.total_files = db_info.total_files;
            info.total_data_sources = db_info.total_data_sources;
            info.tags = db_info.tags;

            // Fill in blanks from DB
            if info.case_name.is_empty() {
                info.case_name = db_info.case_name;
            }
            info.case_number = info.case_number.or(db_info.case_number);
            info.examiner = info.examiner.or(db_info.examiner);
        }
    }

    // 3. Default case name from folder
    if info.case_name.is_empty() {
        info.case_name = case_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Autopsy Case")
            .to_string();
    }

    Ok(info)
}

/// Get artifact categories from Autopsy database
pub fn get_autopsy_categories(
    path: &Path,
) -> Result<Vec<AutopsyArtifactCategory>, ContainerError> {
    let case_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    };

    let db_path = case_dir.join("autopsy.db");
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    query_artifact_types(&conn)
}

// =============================================================================
// .aut file parsing
// =============================================================================

/// Partial info from .aut file
#[derive(Default)]
struct AutFileInfo {
    case_name: String,
    case_number: Option<String>,
    examiner: Option<String>,
    case_type: Option<String>,
    autopsy_version: Option<String>,
    created_date: Option<String>,
    schema_version: Option<String>,
}

/// Find .aut file in case directory
fn find_aut_file(dir: &Path) -> Option<std::path::PathBuf> {
    // Check for autopsy.aut first (common name)
    let autopsy_aut = dir.join("autopsy.aut");
    if autopsy_aut.exists() {
        return Some(autopsy_aut);
    }

    // Check for any .aut file
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "aut").unwrap_or(false) {
                return Some(path);
            }
        }
    }

    None
}

/// Parse .aut file (XML or properties format)
///
/// Autopsy .aut files can be XML or Java properties format.
/// XML format:
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <AutopsyCase>
///   <CaseName>My Case</CaseName>
///   <CaseNumber>2024-001</CaseNumber>
///   <Examiner>John Doe</Examiner>
///   <CaseType>SingleUser</CaseType>
///   <CreatedDate>2024-01-15</CreatedDate>
///   <AutopsyVersion>4.21.0</AutopsyVersion>
///   <SchemaVersion>9.1</SchemaVersion>
/// </AutopsyCase>
/// ```
///
/// Properties format (older versions):
/// ```text
/// CaseName=My Case
/// CaseNumber=2024-001
/// Examiner=John Doe
/// ```
fn parse_aut_file(path: &Path) -> Result<AutFileInfo, ContainerError> {
    let content = fs::read_to_string(path)?;
    let trimmed = content.trim();

    if trimmed.starts_with("<?xml") || trimmed.starts_with('<') {
        parse_aut_xml(&content)
    } else {
        parse_aut_properties(&content)
    }
}

/// Parse .aut as XML
fn parse_aut_xml(content: &str) -> Result<AutFileInfo, ContainerError> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut info = AutFileInfo::default();
    let mut current_element = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_element =
                    String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Ok(Event::End(_)) => {
                current_element.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }

                match current_element.as_str() {
                    "CaseName" | "caseName" | "case_name" => {
                        info.case_name = text;
                    }
                    "CaseNumber" | "caseNumber" | "case_number" => {
                        info.case_number = Some(text);
                    }
                    "Examiner" | "examiner" => {
                        info.examiner = Some(text);
                    }
                    "CaseType" | "caseType" | "case_type" => {
                        info.case_type = Some(text);
                    }
                    "AutopsyVersion" | "autopsyVersion" | "version" => {
                        info.autopsy_version = Some(text);
                    }
                    "CreatedDate" | "createdDate" | "created" => {
                        info.created_date = Some(text);
                    }
                    "SchemaVersion" | "schemaVersion" | "schema_version" => {
                        info.schema_version = Some(text);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing .aut XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(info)
}

/// Parse .aut as Java properties file
fn parse_aut_properties(content: &str) -> Result<AutFileInfo, ContainerError> {
    let mut info = AutFileInfo::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "CaseName" | "caseName" | "case_name" => {
                    info.case_name = value.to_string();
                }
                "CaseNumber" | "caseNumber" | "case_number" => {
                    info.case_number = Some(value.to_string());
                }
                "Examiner" | "examiner" => {
                    info.examiner = Some(value.to_string());
                }
                "CaseType" | "caseType" | "case_type" => {
                    info.case_type = Some(value.to_string());
                }
                "AutopsyVersion" | "autopsyVersion" | "version" => {
                    info.autopsy_version = Some(value.to_string());
                }
                "CreatedDate" | "createdDate" | "created" => {
                    info.created_date = Some(value.to_string());
                }
                "SchemaVersion" | "schemaVersion" | "schema_version" => {
                    info.schema_version = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    Ok(info)
}

// =============================================================================
// SQLite database parsing
// =============================================================================

/// Partial info from autopsy.db
#[derive(Default)]
struct DbInfo {
    case_name: String,
    case_number: Option<String>,
    examiner: Option<String>,
    data_sources: Vec<AutopsyDataSource>,
    ingest_modules: Vec<AutopsyIngestModule>,
    artifact_categories: Vec<AutopsyArtifactCategory>,
    total_artifacts: u64,
    total_files: u64,
    total_data_sources: u64,
    tags: Vec<AutopsyTag>,
}

/// Parse autopsy.db SQLite database
fn parse_autopsy_db(path: &Path) -> Result<DbInfo, ContainerError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut info = DbInfo::default();

    // 1. Case metadata from tsk_db_info or cases table
    query_case_metadata(&conn, &mut info);

    // 2. Data sources
    info.data_sources = query_data_sources(&conn);
    info.total_data_sources = info.data_sources.len() as u64;

    // 3. Ingest modules
    info.ingest_modules = query_ingest_modules(&conn);

    // 4. Artifact categories
    info.artifact_categories = query_artifact_types(&conn).unwrap_or_default();
    info.total_artifacts = info.artifact_categories.iter().map(|c| c.count).sum();

    // 5. Total files
    info.total_files = query_file_count(&conn);

    // 6. Tags
    info.tags = query_tags(&conn);

    Ok(info)
}

/// Query case metadata from tsk_db_info or similar tables
fn query_case_metadata(conn: &Connection, info: &mut DbInfo) {
    // Autopsy stores case info in tsk_db_info as key-value pairs
    if let Ok(mut stmt) = conn.prepare("SELECT name, value FROM tsk_db_info") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        }) {
            for row in rows.flatten() {
                match row.0.as_str() {
                    "CASE_NAME" | "case_name" | "caseName" => {
                        info.case_name = row.1;
                    }
                    "CASE_NUMBER" | "case_number" | "caseNumber" => {
                        info.case_number = Some(row.1);
                    }
                    "EXAMINER" | "examiner" => {
                        info.examiner = Some(row.1);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Query data sources from tsk_image_info / data_source_info
fn query_data_sources(conn: &Connection) -> Vec<AutopsyDataSource> {
    let mut sources = Vec::new();

    // Try data_source_info table (Autopsy 4.x)
    if let Ok(mut stmt) = conn.prepare(
        "SELECT obj_id, display_name, device_id, time_zone FROM data_source_info",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(AutopsyDataSource {
                id: row.get::<_, i64>(0).unwrap_or(0),
                name: row.get::<_, String>(1).unwrap_or_default(),
                device_id: row.get::<_, String>(2).ok(),
                timezone: row.get::<_, String>(3).ok(),
                size: None,
            })
        }) {
            for row in rows.flatten() {
                sources.push(row);
            }
        }
    }

    // Fall back to tsk_image_names if data_source_info doesn't exist
    if sources.is_empty() {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT obj_id, name FROM tsk_image_names ORDER BY obj_id, sequence",
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(AutopsyDataSource {
                    id: row.get::<_, i64>(0).unwrap_or(0),
                    name: row.get::<_, String>(1).unwrap_or_default(),
                    device_id: None,
                    timezone: None,
                    size: None,
                })
            }) {
                for row in rows.flatten() {
                    // Deduplicate by id
                    if !sources.iter().any(|s| s.id == row.id) {
                        sources.push(row);
                    }
                }
            }
        }
    }

    sources
}

/// Query ingest modules
fn query_ingest_modules(conn: &Connection) -> Vec<AutopsyIngestModule> {
    let mut modules = Vec::new();

    // Autopsy stores ingest module info in ingest_modules table
    if let Ok(mut stmt) = conn.prepare(
        "SELECT display_name, version FROM ingest_modules ORDER BY display_name",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(AutopsyIngestModule {
                name: row.get::<_, String>(0).unwrap_or_default(),
                version: row.get::<_, String>(1).ok(),
            })
        }) {
            for row in rows.flatten() {
                if !row.name.is_empty() {
                    modules.push(row);
                }
            }
        }
    }

    modules
}

/// Query artifact type counts from blackboard_artifact_types + blackboard_artifacts
fn query_artifact_types(
    conn: &Connection,
) -> Result<Vec<AutopsyArtifactCategory>, ContainerError> {
    let mut categories = Vec::new();

    // Join artifact types with artifact counts
    let sql = "SELECT bat.type_name, bat.artifact_type_id, COUNT(ba.artifact_id) as cnt \
               FROM blackboard_artifact_types bat \
               INNER JOIN blackboard_artifacts ba ON bat.artifact_type_id = ba.artifact_type_id \
               GROUP BY bat.artifact_type_id \
               ORDER BY cnt DESC";

    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(AutopsyArtifactCategory {
                name: row.get::<_, String>(0).unwrap_or_default(),
                type_id: row.get::<_, i32>(1).unwrap_or(0),
                count: row.get::<_, i64>(2).unwrap_or(0) as u64,
            })
        }) {
            for row in rows.flatten() {
                if !row.name.is_empty() && row.count > 0 {
                    categories.push(row);
                }
            }
        }
    }

    Ok(categories)
}

/// Query total file count
fn query_file_count(conn: &Connection) -> u64 {
    conn.query_row("SELECT COUNT(*) FROM tsk_files", [], |row| {
        row.get::<_, i64>(0)
    })
    .unwrap_or(0) as u64
}

/// Query tags used in the case
fn query_tags(conn: &Connection) -> Vec<AutopsyTag> {
    let mut tags = Vec::new();

    // content_tags + blackboard_artifact_tags reference tag_names
    let sql = "SELECT tn.display_name, \
               (SELECT COUNT(*) FROM content_tags ct WHERE ct.tag_name_id = tn.tag_name_id) + \
               (SELECT COUNT(*) FROM blackboard_artifact_tags bat WHERE bat.tag_name_id = tn.tag_name_id) as cnt \
               FROM tag_names tn \
               ORDER BY cnt DESC";

    if let Ok(mut stmt) = conn.prepare(sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(AutopsyTag {
                name: row.get::<_, String>(0).unwrap_or_default(),
                count: row.get::<_, i64>(1).unwrap_or(0) as u64,
            })
        }) {
            for row in rows.flatten() {
                if !row.name.is_empty() {
                    tags.push(row);
                }
            }
        }
    }

    tags
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_autopsy_case_info_default() {
        let info = AutopsyCaseInfo::default();
        assert!(info.case_name.is_empty());
        assert_eq!(info.total_artifacts, 0);
        assert_eq!(info.total_files, 0);
        assert!(!info.has_database);
        assert!(!info.has_aut_file);
    }

    #[test]
    fn test_autopsy_case_info_serialization() {
        let info = AutopsyCaseInfo {
            case_name: "Phone Analysis".to_string(),
            case_number: Some("AP-2024-001".to_string()),
            examiner: Some("Jane Smith".to_string()),
            autopsy_version: Some("4.21.0".to_string()),
            total_artifacts: 12345,
            total_files: 67890,
            ..Default::default()
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("caseName"));
        assert!(json.contains("caseNumber"));
        assert!(json.contains("autopsyVersion"));
        assert!(json.contains("totalArtifacts"));
        assert!(json.contains("totalFiles"));

        let deser: AutopsyCaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.case_name, "Phone Analysis");
        assert_eq!(deser.total_artifacts, 12345);
    }

    #[test]
    fn test_parse_aut_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutopsyCase>
  <CaseName>Disk Image Analysis</CaseName>
  <CaseNumber>2024-042</CaseNumber>
  <Examiner>John Doe</Examiner>
  <CaseType>SingleUser</CaseType>
  <AutopsyVersion>4.21.0</AutopsyVersion>
  <CreatedDate>2024-03-15</CreatedDate>
  <SchemaVersion>9.1</SchemaVersion>
</AutopsyCase>"#;

        let info = parse_aut_xml(xml).unwrap();
        assert_eq!(info.case_name, "Disk Image Analysis");
        assert_eq!(info.case_number, Some("2024-042".to_string()));
        assert_eq!(info.examiner, Some("John Doe".to_string()));
        assert_eq!(info.case_type, Some("SingleUser".to_string()));
        assert_eq!(info.autopsy_version, Some("4.21.0".to_string()));
        assert_eq!(info.created_date, Some("2024-03-15".to_string()));
        assert_eq!(info.schema_version, Some("9.1".to_string()));
    }

    #[test]
    fn test_parse_aut_properties() {
        let props = "CaseName=USB Analysis\n\
                     CaseNumber=2024-099\n\
                     Examiner=Bob Smith\n\
                     CaseType=SingleUser\n\
                     AutopsyVersion=4.20.0\n";

        let info = parse_aut_properties(props).unwrap();
        assert_eq!(info.case_name, "USB Analysis");
        assert_eq!(info.case_number, Some("2024-099".to_string()));
        assert_eq!(info.examiner, Some("Bob Smith".to_string()));
        assert_eq!(info.case_type, Some("SingleUser".to_string()));
        assert_eq!(info.autopsy_version, Some("4.20.0".to_string()));
    }

    #[test]
    fn test_parse_aut_properties_with_comments() {
        let props = "# Autopsy case file\n\
                     \n\
                     CaseName=Test Case\n\
                     # Case number\n\
                     CaseNumber=TC-001\n";

        let info = parse_aut_properties(props).unwrap();
        assert_eq!(info.case_name, "Test Case");
        assert_eq!(info.case_number, Some("TC-001".to_string()));
    }

    #[test]
    fn test_parse_aut_file_xml_format() {
        let dir = TempDir::new().unwrap();
        let aut_path = dir.path().join("test.aut");
        let xml = "<?xml version=\"1.0\"?>\n<AutopsyCase>\n\
                   <CaseName>XML Case</CaseName>\n</AutopsyCase>";
        fs::write(&aut_path, xml).unwrap();

        let info = parse_aut_file(&aut_path).unwrap();
        assert_eq!(info.case_name, "XML Case");
    }

    #[test]
    fn test_parse_aut_file_properties_format() {
        let dir = TempDir::new().unwrap();
        let aut_path = dir.path().join("test.aut");
        fs::write(&aut_path, "CaseName=Props Case\n").unwrap();

        let info = parse_aut_file(&aut_path).unwrap();
        assert_eq!(info.case_name, "Props Case");
    }

    #[test]
    fn test_find_aut_file_by_extension() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("mycase.aut"), "CaseName=Test\n").unwrap();

        let found = find_aut_file(dir.path());
        assert!(found.is_some());
        let path = found.unwrap();
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("aut"));
    }

    #[test]
    fn test_find_aut_file_none() {
        let dir = TempDir::new().unwrap();
        let found = find_aut_file(dir.path());
        assert!(found.is_none());
    }

    #[test]
    fn test_parse_autopsy_case_folder_with_aut() {
        let dir = TempDir::new().unwrap();
        let xml = "<?xml version=\"1.0\"?>\n<AutopsyCase>\n\
                   <CaseName>Folder Case</CaseName>\n\
                   <CaseNumber>FC-001</CaseNumber>\n\
                   </AutopsyCase>";
        fs::write(dir.path().join("test.aut"), xml).unwrap();

        let result = parse_autopsy_case(dir.path()).unwrap();
        assert_eq!(result.case_name, "Folder Case");
        assert_eq!(result.case_number, Some("FC-001".to_string()));
        assert!(result.has_aut_file);
        assert!(!result.has_database);
    }

    #[test]
    fn test_parse_autopsy_case_no_files_uses_folder_name() {
        let dir = TempDir::new().unwrap();
        let result = parse_autopsy_case(dir.path()).unwrap();
        assert!(!result.case_name.is_empty());
        assert!(!result.has_aut_file);
        assert!(!result.has_database);
    }

    #[test]
    fn test_parse_autopsy_db_with_schema() {
        // Create in-memory database with Autopsy-like schema
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("autopsy.db");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch(
            "CREATE TABLE tsk_db_info (name TEXT, value TEXT);
             INSERT INTO tsk_db_info VALUES ('CASE_NAME', 'DB Case');
             INSERT INTO tsk_db_info VALUES ('CASE_NUMBER', 'DB-001');
             INSERT INTO tsk_db_info VALUES ('EXAMINER', 'Alice');

             CREATE TABLE data_source_info (obj_id INTEGER, display_name TEXT, device_id TEXT, time_zone TEXT);
             INSERT INTO data_source_info VALUES (1, 'disk.E01', 'dev-001', 'UTC');

             CREATE TABLE ingest_modules (display_name TEXT, version TEXT);
             INSERT INTO ingest_modules VALUES ('Hash Lookup', '4.21.0');
             INSERT INTO ingest_modules VALUES ('Keyword Search', '4.21.0');

             CREATE TABLE blackboard_artifact_types (artifact_type_id INTEGER, type_name TEXT);
             INSERT INTO blackboard_artifact_types VALUES (1, 'TSK_WEB_HISTORY');
             INSERT INTO blackboard_artifact_types VALUES (2, 'TSK_WEB_BOOKMARK');

             CREATE TABLE blackboard_artifacts (artifact_id INTEGER, artifact_type_id INTEGER);
             INSERT INTO blackboard_artifacts VALUES (100, 1);
             INSERT INTO blackboard_artifacts VALUES (101, 1);
             INSERT INTO blackboard_artifacts VALUES (102, 1);
             INSERT INTO blackboard_artifacts VALUES (200, 2);

             CREATE TABLE tsk_files (obj_id INTEGER, name TEXT);
             INSERT INTO tsk_files VALUES (1, 'file1.txt');
             INSERT INTO tsk_files VALUES (2, 'file2.jpg');
             INSERT INTO tsk_files VALUES (3, 'file3.doc');

             CREATE TABLE tag_names (tag_name_id INTEGER, display_name TEXT);
             INSERT INTO tag_names VALUES (1, 'Notable');
             INSERT INTO tag_names VALUES (2, 'Follow Up');

             CREATE TABLE content_tags (tag_id INTEGER, tag_name_id INTEGER);
             INSERT INTO content_tags VALUES (1, 1);
             INSERT INTO content_tags VALUES (2, 1);

             CREATE TABLE blackboard_artifact_tags (tag_id INTEGER, tag_name_id INTEGER);
             INSERT INTO blackboard_artifact_tags VALUES (1, 2);"
        ).unwrap();

        drop(conn);

        let result = parse_autopsy_db(&db_path).unwrap();
        assert_eq!(result.case_name, "DB Case");
        assert_eq!(result.case_number, Some("DB-001".to_string()));
        assert_eq!(result.examiner, Some("Alice".to_string()));
        assert_eq!(result.data_sources.len(), 1);
        assert_eq!(result.data_sources[0].name, "disk.E01");
        assert_eq!(result.ingest_modules.len(), 2);
        assert_eq!(result.total_artifacts, 4);
        assert_eq!(result.total_files, 3);
        assert_eq!(result.artifact_categories.len(), 2);
        assert_eq!(result.tags.len(), 2);
    }

    #[test]
    fn test_query_artifact_types_empty_db() {
        let conn = Connection::open_in_memory().unwrap();
        let result = query_artifact_types(&conn).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_autopsy_data_source_serialization() {
        let src = AutopsyDataSource {
            id: 1,
            name: "evidence.E01".to_string(),
            device_id: Some("dev-abc".to_string()),
            timezone: Some("America/New_York".to_string()),
            size: Some(1_000_000_000),
        };

        let json = serde_json::to_string(&src).unwrap();
        let deser: AutopsyDataSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "evidence.E01");
        assert_eq!(deser.timezone, Some("America/New_York".to_string()));
    }

    #[test]
    fn test_autopsy_ingest_module_serialization() {
        let module = AutopsyIngestModule {
            name: "Keyword Search".to_string(),
            version: Some("4.21.0".to_string()),
        };

        let json = serde_json::to_string(&module).unwrap();
        let deser: AutopsyIngestModule = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Keyword Search");
    }

    #[test]
    fn test_autopsy_tag_serialization() {
        let tag = AutopsyTag {
            name: "Notable".to_string(),
            count: 42,
        };

        let json = serde_json::to_string(&tag).unwrap();
        let deser: AutopsyTag = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Notable");
        assert_eq!(deser.count, 42);
    }
}
