// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Types for processed forensic databases
//!
//! These represent parsed examination results, not raw evidence.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The type/source of processed database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessedDbType {
    /// Magnet AXIOM (.mfdb, Case.mcfc)
    MagnetAxiom,
    /// Cellebrite Physical Analyzer (extracted UFDR contents)
    CellebritePA,
    /// X-Ways Forensics (.ctx case container)
    XWays,
    /// Autopsy (.aut case file)
    Autopsy,
    /// EnCase (.case, .LEF)
    EnCase,
    /// FTK (AccessData/Exterro)
    FTK,
    /// Generic SQLite forensic database
    GenericSqlite,
    /// Unknown processed database type
    Unknown,
}

impl ProcessedDbType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessedDbType::MagnetAxiom => "Magnet AXIOM",
            ProcessedDbType::CellebritePA => "Cellebrite PA",
            ProcessedDbType::XWays => "X-Ways",
            ProcessedDbType::Autopsy => "Autopsy",
            ProcessedDbType::EnCase => "EnCase",
            ProcessedDbType::FTK => "FTK",
            ProcessedDbType::GenericSqlite => "SQLite Database",
            ProcessedDbType::Unknown => "Unknown",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            ProcessedDbType::MagnetAxiom => "🧲",
            ProcessedDbType::CellebritePA => "📱",
            ProcessedDbType::XWays => "🔬",
            ProcessedDbType::Autopsy => "🔍",
            ProcessedDbType::EnCase => "📦",
            ProcessedDbType::FTK => "🗃️",
            ProcessedDbType::GenericSqlite => "🗄️",
            ProcessedDbType::Unknown => "❓",
        }
    }
}

/// Information about a processed database folder/file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedDbInfo {
    /// Type of processed database
    pub db_type: ProcessedDbType,
    /// Root path to the database folder or file
    pub path: PathBuf,
    /// Display name (case name, folder name, etc.)
    pub name: String,
    /// Case number if detected
    pub case_number: Option<String>,
    /// Examiner name if detected
    pub examiner: Option<String>,
    /// Date created/processed
    pub created_date: Option<String>,
    /// Total size on disk
    pub total_size: u64,
    /// Number of artifacts found (if scanned)
    pub artifact_count: Option<u32>,
    /// Database files within this processed DB
    pub database_files: Vec<DatabaseFile>,
    /// Notes or description
    pub notes: Option<String>,
}

/// Individual database file within a processed database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFile {
    /// File path relative to root
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// File size
    pub size: u64,
    /// What this database contains
    pub contents: DatabaseContents,
}

/// What type of data a database file contains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseContents {
    /// Main case database
    CaseInfo,
    /// Parsed artifacts (browser, chat, etc.)
    Artifacts,
    /// File system metadata
    FileSystem,
    /// Keyword search results
    Keywords,
    /// Hash values and sets
    Hashes,
    /// Media (thumbnails, carved files)
    Media,
    /// Timeline data
    Timeline,
    /// Bookmarks and tags
    Bookmarks,
    /// Reports
    Reports,
    /// Configuration
    Config,
    /// Unknown
    Unknown,
}

impl DatabaseContents {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseContents::CaseInfo => "Case Information",
            DatabaseContents::Artifacts => "Artifacts",
            DatabaseContents::FileSystem => "File System",
            DatabaseContents::Keywords => "Keywords",
            DatabaseContents::Hashes => "Hashes",
            DatabaseContents::Media => "Media",
            DatabaseContents::Timeline => "Timeline",
            DatabaseContents::Bookmarks => "Bookmarks",
            DatabaseContents::Reports => "Reports",
            DatabaseContents::Config => "Configuration",
            DatabaseContents::Unknown => "Unknown",
        }
    }
}

/// Summary of a processed database scan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessedDbSummary {
    /// Total processed databases found
    pub total_count: usize,
    /// Count by type
    pub by_type: std::collections::HashMap<String, usize>,
    /// Total size of all databases
    pub total_size: u64,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processed_db_type_as_str() {
        assert_eq!(ProcessedDbType::MagnetAxiom.as_str(), "Magnet AXIOM");
        assert_eq!(ProcessedDbType::CellebritePA.as_str(), "Cellebrite PA");
        assert_eq!(ProcessedDbType::XWays.as_str(), "X-Ways");
        assert_eq!(ProcessedDbType::Autopsy.as_str(), "Autopsy");
        assert_eq!(ProcessedDbType::EnCase.as_str(), "EnCase");
        assert_eq!(ProcessedDbType::FTK.as_str(), "FTK");
        assert_eq!(ProcessedDbType::GenericSqlite.as_str(), "SQLite Database");
        assert_eq!(ProcessedDbType::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_processed_db_type_icon() {
        assert_eq!(ProcessedDbType::MagnetAxiom.icon(), "🧲");
        assert_eq!(ProcessedDbType::CellebritePA.icon(), "📱");
        assert_eq!(ProcessedDbType::Unknown.icon(), "❓");
    }

    #[test]
    fn test_database_contents_as_str() {
        assert_eq!(DatabaseContents::CaseInfo.as_str(), "Case Information");
        assert_eq!(DatabaseContents::Artifacts.as_str(), "Artifacts");
        assert_eq!(DatabaseContents::FileSystem.as_str(), "File System");
        assert_eq!(DatabaseContents::Keywords.as_str(), "Keywords");
        assert_eq!(DatabaseContents::Hashes.as_str(), "Hashes");
        assert_eq!(DatabaseContents::Media.as_str(), "Media");
        assert_eq!(DatabaseContents::Timeline.as_str(), "Timeline");
        assert_eq!(DatabaseContents::Bookmarks.as_str(), "Bookmarks");
        assert_eq!(DatabaseContents::Reports.as_str(), "Reports");
        assert_eq!(DatabaseContents::Config.as_str(), "Configuration");
        assert_eq!(DatabaseContents::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_processed_db_info_serialization() {
        let info = ProcessedDbInfo {
            db_type: ProcessedDbType::MagnetAxiom,
            path: PathBuf::from("/test/case"),
            name: "Test Case".to_string(),
            case_number: Some("2024-001".to_string()),
            examiner: Some("John Doe".to_string()),
            created_date: Some("2024-01-15".to_string()),
            total_size: 1_000_000,
            artifact_count: Some(5000),
            database_files: vec![
                DatabaseFile {
                    path: PathBuf::from("Case.mfdb"),
                    name: "Case.mfdb".to_string(),
                    size: 500_000,
                    contents: DatabaseContents::CaseInfo,
                },
            ],
            notes: Some("Test notes".to_string()),
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProcessedDbInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.db_type, ProcessedDbType::MagnetAxiom);
        assert_eq!(deserialized.name, "Test Case");
        assert_eq!(deserialized.case_number, Some("2024-001".to_string()));
        assert_eq!(deserialized.total_size, 1_000_000);
        assert_eq!(deserialized.database_files.len(), 1);
    }

    #[test]
    fn test_processed_db_summary_default() {
        let summary = ProcessedDbSummary::default();
        assert_eq!(summary.total_count, 0);
        assert!(summary.by_type.is_empty());
        assert_eq!(summary.total_size, 0);
    }

    #[test]
    fn test_processed_db_type_equality() {
        assert_eq!(ProcessedDbType::MagnetAxiom, ProcessedDbType::MagnetAxiom);
        assert_ne!(ProcessedDbType::MagnetAxiom, ProcessedDbType::Autopsy);
    }

    #[test]
    fn test_database_contents_equality() {
        assert_eq!(DatabaseContents::Artifacts, DatabaseContents::Artifacts);
        assert_ne!(DatabaseContents::Artifacts, DatabaseContents::FileSystem);
    }
}
