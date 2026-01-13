// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Magnet AXIOM specific parsing
//!
//! Handles AXIOM .mfdb databases and case structure.
//! AXIOM stores artifacts in SQLite databases with a well-defined schema.
//! 
//! Case files:
//! - `.mfdb` - SQLite database with artifacts and some metadata
//! - `.mcfc` - XML configuration file with case details (examiner, agency, etc.)
//! - `Case Information.xml` - Detailed XML summary with search results
//! - `Case Information.txt` - Human-readable summary (not parsed, XML has same data)

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use rusqlite::{Connection, OpenFlags};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use tracing::{debug, warn};
use crate::containers::ContainerError;

/// AXIOM specific case information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomCaseInfo {
    /// Case name
    pub case_name: String,
    /// Case number
    pub case_number: Option<String>,
    /// Case type (e.g., "Other", "Child Exploitation", etc.)
    pub case_type: Option<String>,
    /// Case description
    pub description: Option<String>,
    /// Examiner name
    pub examiner: Option<String>,
    /// Examiner agency/organization
    pub agency: Option<String>,
    /// Windows user who ran AXIOM
    pub user: Option<String>,
    /// Host machine name
    pub host_name: Option<String>,
    /// Operating system
    pub operating_system: Option<String>,
    /// Created date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
    /// AXIOM version used
    pub axiom_version: Option<String>,
    /// Search start time
    pub search_start: Option<String>,
    /// Search end time
    pub search_end: Option<String>,
    /// Search duration
    pub search_duration: Option<String>,
    /// Search outcome (Completed, Cancelled, etc.)
    pub search_outcome: Option<String>,
    /// Output folder where processed data is stored
    pub output_folder: Option<String>,
    /// Evidence sources added
    pub evidence_sources: Vec<AxiomEvidenceSource>,
    /// Search result counts by artifact type
    pub search_results: Vec<AxiomSearchResult>,
    /// Total artifact count
    pub total_artifacts: u64,
    /// Case folder path
    pub case_path: Option<String>,
    /// Keyword search information
    pub keyword_info: Option<AxiomKeywordInfo>,
}

/// AXIOM search result entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomSearchResult {
    /// Artifact type name
    pub artifact_type: String,
    /// Number of hits/items found
    pub hit_count: u64,
}

/// AXIOM keyword entry
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomKeyword {
    /// The keyword/search term value
    pub value: String,
    /// Whether this is a regular expression
    pub is_regex: bool,
    /// Whether the search is case sensitive
    pub is_case_sensitive: bool,
    /// Encoding types (e.g., UTF-8, UTF-16)
    pub encoding_types: Vec<String>,
    /// Whether this keyword came from a file
    pub from_file: bool,
    /// Source file name if from_file is true
    pub file_name: Option<String>,
}

/// AXIOM keyword file
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomKeywordFile {
    /// Display name of the file
    pub file_name: String,
    /// Full path to the keyword file
    pub file_path: String,
    /// Date the file was added
    pub date_added: Option<String>,
    /// Number of keywords/records in the file
    pub record_count: u64,
    /// Whether the file is enabled for searching
    pub enabled: bool,
    /// Whether searches from this file are case sensitive
    pub is_case_sensitive: bool,
}

/// AXIOM keyword search information
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomKeywordInfo {
    /// Number of keywords entered
    pub keywords_entered: u64,
    /// Number of regular expressions
    pub regex_count: u64,
    /// Individual keywords
    pub keywords: Vec<AxiomKeyword>,
    /// Keyword files loaded
    pub keyword_files: Vec<AxiomKeywordFile>,
    /// Privileged content keywords (attorney-client, etc.)
    pub privileged_content_keywords: Vec<AxiomKeyword>,
    /// Privileged content mode (Off, Tagging, Filtering)
    pub privileged_content_mode: Option<String>,
}

/// AXIOM evidence source
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AxiomEvidenceSource {
    /// Source name/path
    pub name: String,
    /// Evidence number
    pub evidence_number: Option<String>,
    /// Source type (image, mobile, cloud, etc.)
    pub source_type: String,
    /// Search types applied
    pub search_types: Vec<String>,
    /// Path to source
    pub path: Option<String>,
    /// Hash if available
    pub hash: Option<String>,
    /// Size in bytes
    pub size: Option<u64>,
    /// Acquisition date
    pub acquired: Option<String>,
}

/// Generic AXIOM artifact
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AxiomArtifact {
    /// Artifact ID
    pub id: i64,
    /// Artifact type/category
    pub artifact_type: String,
    /// Artifact name
    pub name: String,
    /// Evidence source
    pub source: String,
    /// Timestamp if available
    pub timestamp: Option<String>,
    /// Key-value data
    pub data: HashMap<String, String>,
}

/// Artifact category summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactCategorySummary {
    pub category: String,
    pub artifact_type: String,
    pub count: u64,
}

/// Open an AXIOM .mfdb database (read-only)
fn open_axiom_db(path: &Path) -> Result<Connection, ContainerError> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| ContainerError::from(format!("Failed to open AXIOM database: {}", e)))
}

/// Parse AXIOM case information from multiple sources
/// 
/// Priority:
/// 1. Parse Case Information.xml for complete search results and metadata
/// 2. Parse Case.mcfc XML for additional case details
/// 3. Query the .mfdb database for supplemental info
/// 4. Merge all sources into comprehensive case info
pub fn parse_axiom_case(path: &Path) -> Result<AxiomCaseInfo, ContainerError> {
    let case_dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| path.to_path_buf())
    };
    
    // Primary source: Case Information.xml (most complete)
    let case_info_xml = case_dir.join("Case Information.xml");
    let mut case_info = if case_info_xml.exists() {
        debug!("Parsing AXIOM Case Information.xml: {}", case_info_xml.display());
        parse_case_information_xml(&case_info_xml).unwrap_or_default()
    } else {
        AxiomCaseInfo::default()
    };
    
    // Secondary source: .mcfc file (may have additional metadata)
    let mcfc_path = find_mcfc_file(&case_dir);
    if let Some(mcfc) = mcfc_path {
        debug!("Parsing AXIOM case config: {}", mcfc.display());
        if let Ok(mcfc_info) = parse_mcfc_file(&mcfc) {
            // Merge mcfc data (only fill in missing fields)
            if case_info.case_name.is_empty() {
                case_info.case_name = mcfc_info.case_name;
            }
            if case_info.case_number.is_none() {
                case_info.case_number = mcfc_info.case_number;
            }
            if case_info.description.is_none() {
                case_info.description = mcfc_info.description;
            }
            if case_info.examiner.is_none() {
                case_info.examiner = mcfc_info.examiner;
            }
            if case_info.agency.is_none() {
                case_info.agency = mcfc_info.agency;
            }
            if case_info.created.is_none() {
                case_info.created = mcfc_info.created;
            }
            if case_info.modified.is_none() {
                case_info.modified = mcfc_info.modified;
            }
            if case_info.axiom_version.is_none() {
                case_info.axiom_version = mcfc_info.axiom_version;
            }
            // Merge evidence sources if none from XML
            if case_info.evidence_sources.is_empty() {
                case_info.evidence_sources = mcfc_info.evidence_sources;
            }
        }
    }
    
    // Tertiary source: database for additional info and artifact count
    let mfdb_path = if path.is_dir() {
        find_main_mfdb(path).ok()
    } else if path.extension().map(|e| e == "mfdb").unwrap_or(false) {
        Some(path.to_path_buf())
    } else {
        find_main_mfdb(&case_dir).ok()
    };
    
    if let Some(mfdb) = mfdb_path {
        if let Ok(conn) = open_axiom_db(&mfdb) {
            // Get case info from DB if not already set
            if case_info.case_name.is_empty() {
                if let Ok(db_info) = query_case_info(&conn) {
                    case_info.case_name = db_info.0;
                    case_info.case_number = case_info.case_number.or(db_info.1);
                    case_info.examiner = case_info.examiner.or(db_info.2);
                    case_info.created = case_info.created.or(db_info.3);
                    case_info.axiom_version = case_info.axiom_version.or(db_info.4);
                }
            }
            
            // Get evidence sources from DB if we don't have them
            if case_info.evidence_sources.is_empty() {
                case_info.evidence_sources = query_evidence_sources(&conn).unwrap_or_default();
            }
            
            // Get artifact count from database (may be more accurate than XML)
            let db_count = count_total_artifacts(&conn).unwrap_or(0);
            if case_info.total_artifacts == 0 {
                case_info.total_artifacts = db_count;
            }
            
            // Get keyword search information from database
            if let Ok(kw_info) = query_keyword_info(&conn) {
                // Only set if there's actual keyword data
                if !kw_info.keywords.is_empty() || !kw_info.keyword_files.is_empty() || !kw_info.privileged_content_keywords.is_empty() {
                    case_info.keyword_info = Some(kw_info);
                }
            }
        }
    }
    
    // Calculate total from search results if we have them
    if case_info.total_artifacts == 0 && !case_info.search_results.is_empty() {
        case_info.total_artifacts = case_info.search_results.iter()
            .map(|r| r.hit_count)
            .sum();
    }
    
    // Set case path
    case_info.case_path = Some(case_dir.to_string_lossy().to_string());
    
    // Default case name if still empty
    if case_info.case_name.is_empty() {
        case_info.case_name = case_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("AXIOM Case")
            .to_string();
    }
    
    Ok(case_info)
}

/// Find the Case.mcfc file in an AXIOM case folder
fn find_mcfc_file(dir: &Path) -> Option<std::path::PathBuf> {
    // Common locations for .mcfc file
    let case_mcfc = dir.join("Case.mcfc");
    if case_mcfc.exists() {
        return Some(case_mcfc);
    }
    
    // Search for any .mcfc file
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "mcfc").unwrap_or(false) {
                return Some(path);
            }
        }
    }
    
    None
}

/// Parse AXIOM Case.mcfc XML file
/// 
/// The .mcfc file is XML with structure like:
/// ```xml
/// <Case>
///   <CaseName>My Case</CaseName>
///   <CaseNumber>2024-001</CaseNumber>
///   <Description>Case description</Description>
///   <Examiner>John Doe</Examiner>
///   <Agency>ACME Forensics</Agency>
///   <Created>2024-01-15T10:30:00</Created>
///   <Modified>2024-01-16T14:00:00</Modified>
///   <AxiomVersion>7.5.0</AxiomVersion>
///   <EvidenceSources>
///     <Source>
///       <Name>iPhone_backup</Name>
///       <Type>Mobile</Type>
///       <Path>/path/to/backup</Path>
///     </Source>
///   </EvidenceSources>
/// </Case>
/// ```
fn parse_mcfc_file(path: &Path) -> Result<AxiomCaseInfo, ContainerError> {
    let file = fs::File::open(path)
        .map_err(|e| ContainerError::from(format!("Failed to open .mcfc file: {}", e)))?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);
    
    let mut case_info = AxiomCaseInfo::default();
    let mut buf = Vec::new();
    let mut current_element = String::new();
    let mut in_evidence_sources = false;
    let mut current_source = AxiomEvidenceSource::default();
    let mut in_source = false;
    
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                current_element = name.clone();
                
                if name == "evidencesources" || name == "evidence_sources" || name == "sources" {
                    in_evidence_sources = true;
                } else if in_evidence_sources && (name == "source" || name == "evidencesource") {
                    in_source = true;
                    current_source = AxiomEvidenceSource::default();
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                
                if name == "evidencesources" || name == "evidence_sources" || name == "sources" {
                    in_evidence_sources = false;
                } else if in_source && (name == "source" || name == "evidencesource") {
                    if !current_source.name.is_empty() {
                        case_info.evidence_sources.push(current_source.clone());
                    }
                    in_source = false;
                }
                
                current_element.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                
                if in_source {
                    // Inside an evidence source element
                    match current_element.as_str() {
                        "name" | "sourcename" => current_source.name = text,
                        "type" | "sourcetype" => current_source.source_type = text,
                        "path" | "sourcepath" | "filepath" => current_source.path = Some(text),
                        "hash" | "md5" | "sha1" | "sha256" => current_source.hash = Some(text),
                        "size" | "filesize" => current_source.size = text.parse().ok(),
                        "acquired" | "acquisitiondate" | "date" => current_source.acquired = Some(text),
                        _ => {}
                    }
                } else {
                    // Top-level case elements
                    match current_element.as_str() {
                        "casename" | "case_name" | "name" => case_info.case_name = text,
                        "casenumber" | "case_number" | "number" => case_info.case_number = Some(text),
                        "description" | "casedescription" => case_info.description = Some(text),
                        "examiner" | "examinername" => case_info.examiner = Some(text),
                        "agency" | "organization" | "examineragency" => case_info.agency = Some(text),
                        "created" | "createdate" | "creationdate" => case_info.created = Some(text),
                        "modified" | "modifieddate" | "lastmodified" => case_info.modified = Some(text),
                        "axiomversion" | "version" | "appversion" => case_info.axiom_version = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing .mcfc XML: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    
    Ok(case_info)
}

/// Parse AXIOM Case Information.xml file
/// 
/// This file contains the most complete search information including:
/// - Product version
/// - Search settings (examiner, case number, times, etc.)
/// - Evidence sources with evidence numbers
/// - Search results with artifact counts
/// 
/// Structure:
/// ```xml
/// <CaseSummary>
///   <Scan id="1">
///     <ProductInfo>
///       <ProductName>AXIOM</ProductName>
///       <Version>9.2.0.44134</Version>
///     </ProductInfo>
///     <SearchInfo>
///       <CaseNumber>24-042</CaseNumber>
///       <CaseType>Other</CaseType>
///       <Examiner>SA Reynolds</Examiner>
///       <StartTime>2025-05-20T11:41:22</StartTime>
///       <EndTime>2025-05-20T11:45:12</EndTime>
///       <SearchDuration>00:03:20</SearchDuration>
///       <Outcome>Completed</Outcome>
///       ...
///     </SearchInfo>
///     <Sources>
///       <Source>
///         <Name>path/to/evidence</Name>
///         <EvidenceNumber>E001</EvidenceNumber>
///         <SearchTypes><SearchType>Files and Folders</SearchType></SearchTypes>
///       </Source>
///     </Sources>
///     <SearchResults>
///       <Result name="Cloud MBOX Emails" hitCount="47078" />
///     </SearchResults>
///   </Scan>
/// </CaseSummary>
/// ```
fn parse_case_information_xml(path: &Path) -> Result<AxiomCaseInfo, ContainerError> {
    let file = fs::File::open(path)
        .map_err(|e| ContainerError::from(format!("Failed to open Case Information.xml: {}", e)))?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);
    
    let mut case_info = AxiomCaseInfo::default();
    let mut buf = Vec::new();
    let mut current_element = String::new();
    
    // State tracking
    let mut in_search_info = false;
    let mut in_sources = false;
    let mut in_source = false;
    let mut in_search_types = false;
    let mut in_search_results = false;
    let mut in_product_info = false;
    
    let mut current_source = AxiomEvidenceSource::default();
    
    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let name_lower = name.to_lowercase();
                current_element = name_lower.clone();
                
                match name_lower.as_str() {
                    "productinfo" => in_product_info = true,
                    "searchinfo" => in_search_info = true,
                    "sources" => in_sources = true,
                    "source" if in_sources => {
                        in_source = true;
                        current_source = AxiomEvidenceSource::default();
                    }
                    "searchtypes" => in_search_types = true,
                    "searchresults" => in_search_results = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing <Result name="..." hitCount="..." />
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                
                if name == "result" && in_search_results {
                    let mut result = AxiomSearchResult::default();
                    
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        
                        match key.as_str() {
                            "name" => result.artifact_type = value,
                            "hitcount" => result.hit_count = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                    
                    if !result.artifact_type.is_empty() && result.hit_count > 0 {
                        case_info.search_results.push(result);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                
                match name.as_str() {
                    "productinfo" => in_product_info = false,
                    "searchinfo" => in_search_info = false,
                    "sources" => in_sources = false,
                    "source" if in_sources => {
                        if !current_source.name.is_empty() {
                            case_info.evidence_sources.push(current_source.clone());
                        }
                        in_source = false;
                    }
                    "searchtypes" => in_search_types = false,
                    "searchresults" => in_search_results = false,
                    _ => {}
                }
                
                current_element.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }
                
                if in_product_info {
                    if current_element == "version" {
                        case_info.axiom_version = Some(text);
                    }
                } else if in_source {
                    match current_element.as_str() {
                        "name" => current_source.name = text,
                        "evidencenumber" => current_source.evidence_number = Some(text),
                        "searchtype" if in_search_types => {
                            current_source.search_types.push(text);
                        }
                        _ => {}
                    }
                } else if in_search_info {
                    match current_element.as_str() {
                        "casenumber" => case_info.case_number = Some(text),
                        "casetype" => case_info.case_type = Some(text),
                        "examiner" => case_info.examiner = Some(text),
                        "user" => case_info.user = Some(text),
                        "hostname" => case_info.host_name = Some(text),
                        "operatingsystem" => case_info.operating_system = Some(text),
                        "description" => case_info.description = Some(text),
                        "starttime" => case_info.search_start = Some(text),
                        "endtime" => case_info.search_end = Some(text),
                        "searchduration" => case_info.search_duration = Some(text),
                        "outcome" => case_info.search_outcome = Some(text),
                        "outputfolder" => case_info.output_folder = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("Error parsing Case Information.xml: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    
    // Use case number as case name if no explicit name
    if case_info.case_name.is_empty() {
        if let Some(ref num) = case_info.case_number {
            case_info.case_name = num.clone();
        }
    }
    
    Ok(case_info)
}

/// Find the main .mfdb file in an AXIOM case folder
fn find_main_mfdb(dir: &Path) -> Result<std::path::PathBuf, ContainerError> {
    // Look for Case.mfdb or any .mfdb file
    let case_mfdb = dir.join("Case.mfdb");
    if case_mfdb.exists() {
        return Ok(case_mfdb);
    }
    
    // Find any .mfdb file
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "mfdb").unwrap_or(false) {
                return Ok(path);
            }
        }
    }
    
    // Check subdirectories (Artifacts folder)
    let artifacts_dir = dir.join("Artifacts");
    if artifacts_dir.exists() {
        if let Ok(entries) = fs::read_dir(&artifacts_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "mfdb").unwrap_or(false) {
                    return Ok(path);
                }
            }
        }
    }
    
    Err(ContainerError::FileNotFound("No .mfdb file found in AXIOM case folder".to_string()))
}

/// Basic case info tuple: (case_name, case_number, examiner, created, version)
type BasicCaseInfo = (String, Option<String>, Option<String>, Option<String>, Option<String>);

/// Query case information from AXIOM database
fn query_case_info(conn: &Connection) -> Result<BasicCaseInfo, ContainerError> {
    // AXIOM stores case info in various tables - try common ones
    // Tables might include: CaseInfo, Case, Metadata, Properties
    
    // Try CaseInfo table
    if let Ok(mut stmt) = conn.prepare("SELECT name, value FROM CaseInfo") {
        let mut case_name = String::from("Unknown Case");
        let mut case_number = None;
        let mut examiner = None;
        let mut created = None;
        let mut version = None;
        
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                let (key, value) = row;
                match key.to_lowercase().as_str() {
                    "casename" | "case_name" | "name" => case_name = value,
                    "casenumber" | "case_number" => case_number = Some(value),
                    "examiner" | "investigator" => examiner = Some(value),
                    "created" | "createdate" | "creation_date" => created = Some(value),
                    "version" | "axiomversion" => version = Some(value),
                    _ => {}
                }
            }
        }
        
        return Ok((case_name, case_number, examiner, created, version));
    }
    
    // Try Properties or Metadata table
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM Properties") {
        let mut case_name = String::from("Unknown Case");
        let mut case_number = None;
        let mut examiner = None;
        let mut created = None;
        let mut version = None;
        
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                let (key, value) = row;
                let key_lower = key.to_lowercase();
                if key_lower.contains("case") && key_lower.contains("name") {
                    case_name = value.clone();
                }
                if key_lower.contains("number") {
                    case_number = Some(value.clone());
                }
                if key_lower.contains("examiner") {
                    examiner = Some(value.clone());
                }
                if key_lower.contains("created") || key_lower.contains("date") {
                    created = Some(value.clone());
                }
                if key_lower.contains("version") {
                    version = Some(value);
                }
            }
        }
        
        return Ok((case_name, case_number, examiner, created, version));
    }
    
    // Fallback: use database filename
    Ok(("AXIOM Case".to_string(), None, None, None, None))
}

/// Query evidence sources from AXIOM database  
fn query_evidence_sources(conn: &Connection) -> Result<Vec<AxiomEvidenceSource>, ContainerError> {
    let mut sources = Vec::new();
    
    // Try EvidenceSources or Sources table
    let tables = ["EvidenceSources", "Sources", "Evidence", "DataSources"];
    
    for table in tables {
        let query = format!("SELECT * FROM {} LIMIT 100", table);
        if let Ok(mut stmt) = conn.prepare(&query) {
            // Get column names
            let col_count = stmt.column_count();
            let col_names: Vec<String> = (0..col_count)
                .map(|i| stmt.column_name(i).unwrap_or("").to_lowercase())
                .collect();
            
            if let Ok(rows) = stmt.query_map([], |row| {
                let mut source = AxiomEvidenceSource {
                    name: String::new(),
                    evidence_number: None,
                    source_type: String::new(),
                    search_types: Vec::new(),
                    path: None,
                    hash: None,
                    size: None,
                    acquired: None,
                };
                
                for (i, col_name) in col_names.iter().enumerate() {
                    if col_name.contains("name") || col_name == "source" {
                        source.name = row.get::<_, String>(i).unwrap_or_default();
                    }
                    if col_name.contains("type") {
                        source.source_type = row.get::<_, String>(i).unwrap_or_default();
                    }
                    if col_name.contains("path") {
                        source.path = row.get::<_, String>(i).ok();
                    }
                    if col_name.contains("hash") || col_name.contains("md5") || col_name.contains("sha") {
                        source.hash = row.get::<_, String>(i).ok();
                    }
                    if col_name.contains("size") {
                        source.size = row.get::<_, i64>(i).ok().map(|s| s as u64);
                    }
                    if col_name.contains("evidence") && col_name.contains("number") {
                        source.evidence_number = row.get::<_, String>(i).ok();
                    }
                }
                
                Ok(source)
            }) {
                for row in rows.flatten() {
                    if !row.name.is_empty() {
                        sources.push(row);
                    }
                }
            }
            
            if !sources.is_empty() {
                break;
            }
        }
    }
    
    Ok(sources)
}

/// Query keyword search information from AXIOM database
/// 
/// AXIOM stores keyword configuration in the scan_attribute table as JSON:
/// - attribute_name = 'ScanDef' contains JSON with Keywords, KeywordFiles, PrivilegedContentKeywords
fn query_keyword_info(conn: &Connection) -> Result<AxiomKeywordInfo, ContainerError> {
    let mut keyword_info = AxiomKeywordInfo::default();
    
    // Query the ScanDef JSON from scan_attribute table
    let scan_def_query = "SELECT attribute_value FROM scan_attribute WHERE attribute_name = 'ScanDef'";
    
    if let Ok(json_str) = conn.query_row(scan_def_query, [], |row| row.get::<_, String>(0)) {
        // Parse the JSON
        if let Ok(scan_def) = serde_json::from_str::<serde_json::Value>(&json_str) {
            // Parse Keywords array
            if let Some(keywords) = scan_def.get("Keywords").and_then(|k| k.as_array()) {
                for kw in keywords {
                    let keyword = AxiomKeyword {
                        value: kw.get("Value").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                        is_regex: kw.get("Regex").and_then(|v| v.as_bool()).unwrap_or(false),
                        is_case_sensitive: kw.get("IsCaseSensitive").and_then(|v| v.as_bool()).unwrap_or(false),
                        encoding_types: kw.get("EncodingTypes")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|e| e.as_i64().map(encoding_type_to_string))
                                .collect())
                            .unwrap_or_default(),
                        from_file: kw.get("FromFile").and_then(|v| v.as_bool()).unwrap_or(false),
                        file_name: kw.get("FileName").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    };
                    
                    if keyword.is_regex {
                        keyword_info.regex_count += 1;
                    }
                    
                    if !keyword.value.is_empty() {
                        keyword_info.keywords.push(keyword);
                    }
                }
            }
            
            // Parse KeywordFiles array
            if let Some(files) = scan_def.get("KeywordFiles").and_then(|k| k.as_array()) {
                for file in files {
                    let kw_file = AxiomKeywordFile {
                        file_name: file.get("FileName").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                        file_path: file.get("FilePath").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                        date_added: file.get("DateAdded").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        record_count: file.get("RecordCount").and_then(|v| v.as_u64()).unwrap_or(0),
                        enabled: file.get("Enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                        is_case_sensitive: file.get("IsCaseSensitive").and_then(|v| v.as_bool()).unwrap_or(false),
                    };
                    
                    if !kw_file.file_name.is_empty() {
                        keyword_info.keyword_files.push(kw_file);
                    }
                }
            }
            
            // Parse PrivilegedContentKeywords array
            if let Some(priv_keywords) = scan_def.get("PrivilegedContentKeywords").and_then(|k| k.as_array()) {
                for kw in priv_keywords {
                    let keyword = AxiomKeyword {
                        value: kw.get("Value").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                        is_regex: kw.get("Regex").and_then(|v| v.as_bool()).unwrap_or(false),
                        is_case_sensitive: kw.get("IsCaseSensitive").and_then(|v| v.as_bool()).unwrap_or(false),
                        encoding_types: kw.get("EncodingTypes")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|e| e.as_i64().map(encoding_type_to_string))
                                .collect())
                            .unwrap_or_default(),
                        from_file: false,
                        file_name: kw.get("TagName").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    };
                    
                    if !keyword.value.is_empty() {
                        keyword_info.privileged_content_keywords.push(keyword);
                    }
                }
            }
            
            // Get privileged content mode
            keyword_info.privileged_content_mode = scan_def.get("PrivilegedContentMode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }
    
    // Calculate total keywords entered
    keyword_info.keywords_entered = keyword_info.keywords.len() as u64;
    
    debug!(
        "Parsed {} keywords, {} files, {} privileged keywords", 
        keyword_info.keywords.len(),
        keyword_info.keyword_files.len(),
        keyword_info.privileged_content_keywords.len()
    );
    
    Ok(keyword_info)
}

/// Convert AXIOM encoding type integer to human-readable string
fn encoding_type_to_string(encoding: i64) -> String {
    match encoding {
        1 => "UTF-8".to_string(),
        2 => "UTF-16 LE".to_string(),
        3 => "UTF-16 BE".to_string(),
        4 => "ASCII".to_string(),
        5 => "Latin-1".to_string(),
        _ => format!("Encoding-{}", encoding),
    }
}

/// Count total artifacts in AXIOM database
fn count_total_artifacts(conn: &Connection) -> Result<u64, ContainerError> {
    // AXIOM stores hits in scan_artifact_hit table
    if let Ok(count) = conn.query_row(
        "SELECT COUNT(*) FROM scan_artifact_hit",
        [],
        |row| row.get::<_, i64>(0)
    ) {
        if count > 0 {
            return Ok(count as u64);
        }
    }
    
    // Fallback: try legacy table names
    let artifact_tables = [
        "Artifacts", "HitArtifacts", "Hits", "Results",
    ];
    
    let mut total = 0u64;
    for table in artifact_tables {
        let query = format!("SELECT COUNT(*) FROM {}", table);
        if let Ok(count) = conn.query_row(&query, [], |row| row.get::<_, i64>(0)) {
            total += count as u64;
        }
    }
    
    Ok(total)
}

/// Get artifact category summaries from AXIOM database
/// 
/// AXIOM schema uses:
/// - `scan_artifact_hit` - the actual artifact hits with artifact_version_id
/// - `artifact_version` - links artifact_version_id to artifact names
/// - `artifact_group` - category groupings
/// - `artifact_version_group` - links versions to groups
pub fn get_artifact_categories(path: &Path) -> Result<Vec<ArtifactCategorySummary>, ContainerError> {
    let mfdb_path = if path.is_dir() {
        find_main_mfdb(path)?
    } else {
        path.to_path_buf()
    };
    
    let conn = open_axiom_db(&mfdb_path)?;
    let mut categories = Vec::new();
    
    // Try the actual AXIOM schema first:
    // Join scan_artifact_hit -> artifact_version to get artifact names with counts
    let axiom_query = r#"
        SELECT 
            av.artifact_name,
            COUNT(sah.hit_id) as hit_count
        FROM scan_artifact_hit sah
        JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
        GROUP BY av.artifact_name
        ORDER BY hit_count DESC
    "#;
    
    if let Ok(mut stmt) = conn.prepare(axiom_query) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ArtifactCategorySummary {
                category: categorize_artifact_name(&row.get::<_, String>(0).unwrap_or_default()),
                artifact_type: row.get(0)?,
                count: row.get::<_, i64>(1)? as u64,
            })
        }) {
            categories.extend(rows.flatten());
        }
    }
    
    // If we got results, return them
    if !categories.is_empty() {
        debug!("Found {} artifact categories from AXIOM schema", categories.len());
        return Ok(categories);
    }
    
    // Fallback: try to get from artifact_group with counts
    let group_query = r#"
        SELECT 
            ag.group_name,
            av.artifact_name,
            COUNT(sah.hit_id) as hit_count
        FROM scan_artifact_hit sah
        JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
        JOIN artifact_version_group avg ON av.artifact_version_id = avg.artifact_version_id
        JOIN artifact_group ag ON avg.artifact_group_id = ag.artifact_group_id
        GROUP BY ag.group_name, av.artifact_name
        ORDER BY hit_count DESC
    "#;
    
    if let Ok(mut stmt) = conn.prepare(group_query) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ArtifactCategorySummary {
                category: row.get(0)?,
                artifact_type: row.get(1)?,
                count: row.get::<_, i64>(2)? as u64,
            })
        }) {
            categories.extend(rows.flatten());
        }
    }
    
    if !categories.is_empty() {
        debug!("Found {} artifact categories from artifact_group", categories.len());
        return Ok(categories);
    }
    
    // Legacy fallback: try old-style table names
    debug!("No AXIOM tables found, trying legacy schema");
    if let Ok(mut stmt) = conn.prepare(
        "SELECT category, artifact_type, COUNT(*) as count 
         FROM Artifacts 
         GROUP BY category, artifact_type 
         ORDER BY count DESC"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ArtifactCategorySummary {
                category: row.get(0)?,
                artifact_type: row.get(1)?,
                count: row.get::<_, i64>(2)? as u64,
            })
        }) {
            categories.extend(rows.flatten());
        }
    }
    
    Ok(categories)
}

/// Categorize an artifact name into a general category
fn categorize_artifact_name(name: &str) -> String {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("email") || name_lower.contains("mbox") || name_lower.contains("mail") {
        "Email & Calendar".to_string()
    } else if name_lower.contains("chat") || name_lower.contains("message") || name_lower.contains("sms") 
        || name_lower.contains("call") || name_lower.contains("contact") {
        "Communication".to_string()
    } else if name_lower.contains("cloud") || name_lower.contains("google") || name_lower.contains("dropbox")
        || name_lower.contains("onedrive") || name_lower.contains("icloud") {
        "Cloud".to_string()
    } else if name_lower.contains("web") || name_lower.contains("browser") || name_lower.contains("history")
        || name_lower.contains("bookmark") || name_lower.contains("cookie") || name_lower.contains("download") {
        "Web".to_string()
    } else if name_lower.contains("document") || name_lower.contains("pdf") || name_lower.contains("csv")
        || name_lower.contains("text") || name_lower.contains("office") {
        "Documents".to_string()
    } else if name_lower.contains("picture") || name_lower.contains("photo") || name_lower.contains("image")
        || name_lower.contains("video") || name_lower.contains("media") || name_lower.contains("audio") {
        "Media".to_string()
    } else if name_lower.contains("user") || name_lower.contains("account") || name_lower.contains("login") {
        "User Accounts".to_string()
    } else if name_lower.contains("identifier") || name_lower.contains("device") {
        "Identifiers".to_string()
    } else if name_lower.contains("file system") || name_lower.contains("filesystem") {
        "File System".to_string()
    } else if name_lower.contains("gps") || name_lower.contains("location") || name_lower.contains("geo") {
        "Location".to_string()
    } else if name_lower.contains("app") || name_lower.contains("install") || name_lower.contains("program") {
        "Applications".to_string()
    } else if name_lower.contains("registry") || name_lower.contains("system") || name_lower.contains("log") {
        "System".to_string()
    } else {
        "Other".to_string()
    }
}

/// Query AXIOM artifacts with pagination
pub fn query_axiom_artifacts(
    path: &Path, 
    artifact_type: Option<&str>,
    limit: usize,
    offset: usize
) -> Result<Vec<AxiomArtifact>, ContainerError> {
    let mfdb_path = if path.is_dir() {
        find_main_mfdb(path)?
    } else {
        path.to_path_buf()
    };
    
    let conn = open_axiom_db(&mfdb_path)?;
    let mut artifacts = Vec::new();
    
    // Build query based on artifact type
    let query = if let Some(atype) = artifact_type {
        format!(
            "SELECT * FROM \"{}\" LIMIT {} OFFSET {}", 
            atype, limit, offset
        )
    } else {
        // Try main Artifacts table
        format!("SELECT * FROM Artifacts LIMIT {} OFFSET {}", limit, offset)
    };
    
    if let Ok(mut stmt) = conn.prepare(&query) {
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count)
            .map(|i| stmt.column_name(i).unwrap_or("").to_string())
            .collect();
        
        if let Ok(rows) = stmt.query_map([], |row| {
            let mut artifact = AxiomArtifact {
                id: 0,
                artifact_type: artifact_type.unwrap_or("Unknown").to_string(),
                name: String::new(),
                source: String::new(),
                timestamp: None,
                data: HashMap::new(),
            };
            
            for (i, col_name) in col_names.iter().enumerate() {
                let col_lower = col_name.to_lowercase();
                
                // Try to get value as string
                let value: String = row.get::<_, String>(i)
                    .or_else(|_| row.get::<_, i64>(i).map(|v| v.to_string()))
                    .or_else(|_| row.get::<_, f64>(i).map(|v| v.to_string()))
                    .unwrap_or_default();
                
                if col_lower == "id" || col_lower == "rowid" {
                    artifact.id = value.parse().unwrap_or(0);
                } else if col_lower.contains("name") || col_lower.contains("title") {
                    artifact.name = value.clone();
                } else if col_lower.contains("source") {
                    artifact.source = value.clone();
                } else if col_lower.contains("time") || col_lower.contains("date") {
                    artifact.timestamp = Some(value.clone());
                }
                
                // Store all columns in data map
                if !value.is_empty() {
                    artifact.data.insert(col_name.clone(), value);
                }
            }
            
            Ok(artifact)
        }) {
            artifacts.extend(rows.flatten());
        }
    }
    
    Ok(artifacts)
}

/// List all tables in an AXIOM database (for exploration)
pub fn list_axiom_tables(path: &Path) -> Result<Vec<(String, u64)>, String> {
    let mfdb_path = if path.is_dir() {
        find_main_mfdb(path)?
    } else {
        path.to_path_buf()
    };
    
    let conn = open_axiom_db(&mfdb_path)?;
    let mut tables = Vec::new();
    
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for table_name in rows.flatten() {
                let query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
                let count = conn.query_row(&query, [], |row| row.get::<_, i64>(0))
                    .unwrap_or(0) as u64;
                tables.push((table_name, count));
            }
        }
    }
    
    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // encoding_type_to_string tests
    // =========================================================================

    #[test]
    fn test_encoding_type_utf8() {
        assert_eq!(encoding_type_to_string(1), "UTF-8");
    }

    #[test]
    fn test_encoding_type_utf16_le() {
        assert_eq!(encoding_type_to_string(2), "UTF-16 LE");
    }

    #[test]
    fn test_encoding_type_utf16_be() {
        assert_eq!(encoding_type_to_string(3), "UTF-16 BE");
    }

    #[test]
    fn test_encoding_type_ascii() {
        assert_eq!(encoding_type_to_string(4), "ASCII");
    }

    #[test]
    fn test_encoding_type_latin1() {
        assert_eq!(encoding_type_to_string(5), "Latin-1");
    }

    #[test]
    fn test_encoding_type_unknown() {
        assert_eq!(encoding_type_to_string(99), "Encoding-99");
        assert_eq!(encoding_type_to_string(0), "Encoding-0");
        assert_eq!(encoding_type_to_string(-1), "Encoding--1");
    }

    // =========================================================================
    // categorize_artifact_name tests
    // =========================================================================

    #[test]
    fn test_categorize_email() {
        assert_eq!(categorize_artifact_name("Email Messages"), "Email & Calendar");
        assert_eq!(categorize_artifact_name("MBOX Files"), "Email & Calendar");
        assert_eq!(categorize_artifact_name("Mail Attachments"), "Email & Calendar");
    }

    #[test]
    fn test_categorize_communication() {
        assert_eq!(categorize_artifact_name("Chat Messages"), "Communication");
        assert_eq!(categorize_artifact_name("SMS Texts"), "Communication");
        assert_eq!(categorize_artifact_name("Call Logs"), "Communication");
        assert_eq!(categorize_artifact_name("Contact List"), "Communication");
    }

    #[test]
    fn test_categorize_cloud() {
        assert_eq!(categorize_artifact_name("Google Drive Files"), "Cloud");
        assert_eq!(categorize_artifact_name("Dropbox Sync"), "Cloud");
        assert_eq!(categorize_artifact_name("OneDrive Data"), "Cloud");
        assert_eq!(categorize_artifact_name("iCloud Photos"), "Cloud");
        assert_eq!(categorize_artifact_name("Cloud Storage"), "Cloud");
    }

    #[test]
    fn test_categorize_web() {
        assert_eq!(categorize_artifact_name("Web History"), "Web");
        assert_eq!(categorize_artifact_name("Browser Cache"), "Web");
        assert_eq!(categorize_artifact_name("Bookmarks"), "Web");
        assert_eq!(categorize_artifact_name("Cookies"), "Web");
        assert_eq!(categorize_artifact_name("Downloads"), "Web");
    }

    #[test]
    fn test_categorize_documents() {
        assert_eq!(categorize_artifact_name("PDF Documents"), "Documents");
        assert_eq!(categorize_artifact_name("CSV Files"), "Documents");
        assert_eq!(categorize_artifact_name("Text Files"), "Documents");
        assert_eq!(categorize_artifact_name("Office Documents"), "Documents");
    }

    #[test]
    fn test_categorize_media() {
        assert_eq!(categorize_artifact_name("Pictures"), "Media");
        assert_eq!(categorize_artifact_name("Photos"), "Media");
        assert_eq!(categorize_artifact_name("Image Files"), "Media");
        assert_eq!(categorize_artifact_name("Video Files"), "Media");
        assert_eq!(categorize_artifact_name("Audio Files"), "Media");
        assert_eq!(categorize_artifact_name("Media Gallery"), "Media");
    }

    #[test]
    fn test_categorize_user_accounts() {
        assert_eq!(categorize_artifact_name("User Profiles"), "User Accounts");
        assert_eq!(categorize_artifact_name("Account Information"), "User Accounts");
        assert_eq!(categorize_artifact_name("Login Sessions"), "User Accounts");
    }

    #[test]
    fn test_categorize_identifiers() {
        assert_eq!(categorize_artifact_name("Device Identifiers"), "Identifiers");
        assert_eq!(categorize_artifact_name("Hardware Identifier"), "Identifiers");
    }

    #[test]
    fn test_categorize_filesystem() {
        assert_eq!(categorize_artifact_name("File System Activity"), "File System");
        assert_eq!(categorize_artifact_name("Filesystem Metadata"), "File System");
    }

    #[test]
    fn test_categorize_location() {
        assert_eq!(categorize_artifact_name("GPS Coordinates"), "Location");
        // "Location History" matches "history" first, so it's Web category
        assert_eq!(categorize_artifact_name("Location History"), "Web");
        assert_eq!(categorize_artifact_name("Geolocation Data"), "Location");
        // Pure location terms without "history"
        assert_eq!(categorize_artifact_name("GPS Waypoints"), "Location");
        assert_eq!(categorize_artifact_name("Location Data"), "Location");
    }

    #[test]
    fn test_categorize_applications() {
        assert_eq!(categorize_artifact_name("Installed Apps"), "Applications");
        assert_eq!(categorize_artifact_name("Application Data"), "Applications");
        assert_eq!(categorize_artifact_name("Program Files"), "Applications");
    }

    #[test]
    fn test_categorize_system() {
        assert_eq!(categorize_artifact_name("Registry Keys"), "System");
        assert_eq!(categorize_artifact_name("System Events"), "System");
        assert_eq!(categorize_artifact_name("Event Logs"), "System");
    }

    #[test]
    fn test_categorize_other() {
        assert_eq!(categorize_artifact_name("Unknown Artifact"), "Other");
        assert_eq!(categorize_artifact_name("Misc Data"), "Other");
        assert_eq!(categorize_artifact_name(""), "Other");
    }

    #[test]
    fn test_categorize_case_insensitive() {
        assert_eq!(categorize_artifact_name("EMAIL MESSAGES"), "Email & Calendar");
        assert_eq!(categorize_artifact_name("email messages"), "Email & Calendar");
        assert_eq!(categorize_artifact_name("Email Messages"), "Email & Calendar");
    }

    // =========================================================================
    // Struct default/construction tests
    // =========================================================================

    #[test]
    fn test_axiom_case_info_default() {
        let info = AxiomCaseInfo::default();
        assert!(info.case_name.is_empty());
        assert!(info.case_number.is_none());
        assert!(info.examiner.is_none());
        assert!(info.evidence_sources.is_empty());
        assert!(info.search_results.is_empty());
        assert_eq!(info.total_artifacts, 0);
    }

    #[test]
    fn test_axiom_search_result_default() {
        let result = AxiomSearchResult::default();
        assert!(result.artifact_type.is_empty());
        assert_eq!(result.hit_count, 0);
    }

    #[test]
    fn test_axiom_keyword_default() {
        let kw = AxiomKeyword::default();
        assert!(kw.value.is_empty());
        assert!(!kw.is_regex);
        assert!(!kw.is_case_sensitive);
        assert!(kw.encoding_types.is_empty());
        assert!(!kw.from_file);
        assert!(kw.file_name.is_none());
    }

    #[test]
    fn test_axiom_keyword_file_default() {
        let kwf = AxiomKeywordFile::default();
        assert!(kwf.file_name.is_empty());
        assert!(kwf.file_path.is_empty());
        assert!(kwf.date_added.is_none());
        assert_eq!(kwf.record_count, 0);
        assert!(!kwf.enabled);
        assert!(!kwf.is_case_sensitive);
    }

    #[test]
    fn test_axiom_evidence_source_default() {
        let source = AxiomEvidenceSource::default();
        assert!(source.name.is_empty());
        assert!(source.evidence_number.is_none());
        assert!(source.source_type.is_empty());
        assert!(source.search_types.is_empty());
    }

    #[test]
    fn test_axiom_artifact_construction() {
        let artifact = AxiomArtifact {
            id: 42,
            artifact_type: "Test".to_string(),
            name: "Test Artifact".to_string(),
            source: "source.mfdb".to_string(),
            timestamp: Some("2024-01-01".to_string()),
            data: HashMap::new(),
        };
        assert_eq!(artifact.id, 42);
        assert_eq!(artifact.artifact_type, "Test");
        assert_eq!(artifact.name, "Test Artifact");
    }

    #[test]
    fn test_artifact_category_summary_construction() {
        let summary = ArtifactCategorySummary {
            category: "Communication".to_string(),
            artifact_type: "SMS Messages".to_string(),
            count: 150,
        };
        assert_eq!(summary.category, "Communication");
        assert_eq!(summary.artifact_type, "SMS Messages");
        assert_eq!(summary.count, 150);
    }

    // =========================================================================
    // Path handling tests (non-existent paths return defaults)
    // =========================================================================

    #[test]
    fn test_parse_axiom_case_nonexistent_path_returns_default() {
        // parse_axiom_case returns AxiomCaseInfo for non-existent paths
        // It uses the parent directory name as the case_name fallback for files
        let result = parse_axiom_case(std::path::Path::new("/nonexistent/mycase/test.mfdb"));
        // It succeeds but returns mostly empty info with directory name as case_name
        assert!(result.is_ok());
        let info = result.unwrap();
        // case_name will be the parent directory basename "mycase"
        assert_eq!(info.case_name, "mycase");
        assert!(info.case_number.is_none());
        assert!(info.examiner.is_none());
        assert!(info.evidence_sources.is_empty());
        assert_eq!(info.total_artifacts, 0);
    }

    #[test]
    fn test_get_artifact_categories_nonexistent_path() {
        let result = get_artifact_categories(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_axiom_tables_nonexistent_path() {
        let result = list_axiom_tables(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_query_axiom_artifacts_nonexistent_path() {
        let result = query_axiom_artifacts(
            std::path::Path::new("/nonexistent/path"),
            None,
            10,
            0
        );
        assert!(result.is_err());
    }
}
