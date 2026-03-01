// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for document operations
//!
//! This module exposes document functionality to the frontend via Tauri commands.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

use super::types::{DocumentContent, DocumentMetadata};
use super::DocumentService;

/// Serializable document content for frontend
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub success: bool,
    pub content: Option<DocumentContentDto>,
    pub error: Option<String>,
}

impl DocumentResponse {
    /// Create a successful response with content
    #[inline]
    pub fn success(content: DocumentContentDto) -> Self {
        Self {
            success: true,
            content: Some(content),
            error: None,
        }
    }

    /// Create a failed response with error
    #[inline]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            content: None,
            error: Some(error.into()),
        }
    }
}

/// Document content DTO for frontend serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContentDto {
    pub format: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: usize,
    pub file_size: u64,
    pub text: String,
    pub html: String,
}

impl From<DocumentContent> for DocumentContentDto {
    fn from(content: DocumentContent) -> Self {
        Self {
            format: format!("{:?}", content.metadata.format),
            title: content.metadata.title.clone(),
            author: content.metadata.author.clone(),
            page_count: content.metadata.page_count.unwrap_or(0),
            file_size: content.metadata.file_size,
            text: content.to_plain_text(),
            html: content.to_html(),
        }
    }
}

/// Metadata response
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MetadataResponse {
    pub success: bool,
    pub metadata: Option<DocumentMetadataDto>,
    pub error: Option<String>,
}

impl MetadataResponse {
    /// Create a successful response with metadata
    #[inline]
    pub fn success(metadata: DocumentMetadataDto) -> Self {
        Self {
            success: true,
            metadata: Some(metadata),
            error: None,
        }
    }

    /// Create a failed response with error
    #[inline]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            metadata: None,
            error: Some(error.into()),
        }
    }
}

/// Document metadata DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadataDto {
    pub format: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub page_count: usize,
    pub file_size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub producer: Option<String>,
    pub creator: Option<String>,
    pub encrypted: bool,
    pub word_count: Option<usize>,
}

impl From<DocumentMetadata> for DocumentMetadataDto {
    fn from(m: DocumentMetadata) -> Self {
        Self {
            format: format!("{:?}", m.format),
            title: m.title,
            author: m.author,
            subject: m.subject,
            keywords: m.keywords,
            page_count: m.page_count.unwrap_or(0),
            file_size: m.file_size,
            created: m
                .creation_date
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            modified: m
                .modification_date
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            producer: m.producer,
            creator: m.creator,
            encrypted: false, // DocumentMetadata doesn't track encryption
            word_count: m.word_count,
        }
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Read a document file and return its content
#[command]
pub async fn document_read(path: String) -> Result<DocumentResponse, String> {
    let service = DocumentService::new();
    let path = PathBuf::from(&path);

    match service.read(&path) {
        Ok(content) => Ok(DocumentResponse {
            success: true,
            content: Some(content.into()),
            error: None,
        }),
        Err(e) => Ok(DocumentResponse {
            success: false,
            content: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get document metadata
#[command]
pub async fn document_get_metadata(path: String) -> Result<MetadataResponse, String> {
    let service = DocumentService::new();
    let path = PathBuf::from(&path);

    match service.get_metadata(&path) {
        Ok(metadata) => Ok(MetadataResponse {
            success: true,
            metadata: Some(metadata.into()),
            error: None,
        }),
        Err(e) => Ok(MetadataResponse {
            success: false,
            metadata: None,
            error: Some(e.to_string()),
        }),
    }
}

// =============================================================================
// UNIVERSAL VIEWER OPERATIONS (Read-Only)
// =============================================================================

use super::universal::UniversalFormat;

/// Content-based format detection response
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetectResponse {
    /// Detected format name (e.g. "Pdf", "RegistryHive", "Sqlite")
    pub format: String,
    /// Recommended viewer type (e.g. "Hex", "Binary", "Image")
    pub viewer_type: String,
    /// Human-readable format description
    pub description: String,
    /// MIME type
    pub mime_type: String,
    /// Detection method used ("magic" or "extension")
    pub method: String,
}

/// Detect file format by reading magic bytes from the file header.
///
/// Uses a combined strategy:
/// 1. Magic-byte detection (reads first 32 bytes)
/// 2. For ambiguous magic results (e.g., ZIP-based containers), refine with extension
/// 3. Falls back to extension-based detection if magic bytes are inconclusive
/// Returns format info with recommended viewer type.
#[command]
pub async fn detect_content_format(path: String) -> Result<ContentDetectResponse, String> {
    let path_ref = std::path::Path::new(&path);

    // Try magic-byte detection first
    if let Some(magic_format) = UniversalFormat::detect_by_magic(path_ref) {
        // For ambiguous container formats, refine using the file extension.
        // ZIP magic bytes (PK\x03\x04) also match DOCX, XLSX, PPTX, ODS, ODT, etc.
        // OLE magic bytes (D0 CF 11 E0) match DOC, XLS, PPT, MSG, etc.
        let format = match magic_format {
            UniversalFormat::Zip | UniversalFormat::Doc => {
                UniversalFormat::from_path(path_ref).unwrap_or(magic_format)
            }
            _ => magic_format,
        };

        return Ok(ContentDetectResponse {
            format: format!("{:?}", format),
            viewer_type: format!("{:?}", format.viewer_type()),
            description: format.description().to_string(),
            mime_type: format.mime_type().to_string(),
            method: "magic".to_string(),
        });
    }

    // Fall back to extension-based detection
    if let Some(format) = UniversalFormat::from_path(path_ref) {
        return Ok(ContentDetectResponse {
            format: format!("{:?}", format),
            viewer_type: format!("{:?}", format.viewer_type()),
            description: format.description().to_string(),
            mime_type: format.mime_type().to_string(),
            method: "extension".to_string(),
        });
    }

    // Absolute fallback
    Ok(ContentDetectResponse {
        format: "Binary".to_string(),
        viewer_type: "Hex".to_string(),
        description: "Unknown binary data".to_string(),
        mime_type: "application/octet-stream".to_string(),
        method: "fallback".to_string(),
    })
}

// =============================================================================
// Spreadsheet Commands
// =============================================================================

use super::spreadsheet::{read_sheet, read_spreadsheet_info, CellValue, SpreadsheetInfo};

/// Get spreadsheet metadata (sheets, format, etc.)
#[command]
pub async fn spreadsheet_info(path: String) -> Result<SpreadsheetInfo, String> {
    read_spreadsheet_info(&path).map_err(|e| e.to_string())
}

/// Read a sheet from a spreadsheet file
#[command]
pub async fn spreadsheet_read_sheet(
    path: String,
    sheet_name: String,
    start_row: Option<usize>,
    max_rows: Option<usize>,
) -> Result<Vec<Vec<CellValue>>, String> {
    let start = start_row.unwrap_or(0);
    let max = max_rows.unwrap_or(500);
    read_sheet(&path, &sheet_name, start, max).map_err(|e| e.to_string())
}

// =============================================================================
// Email Commands
// =============================================================================

use super::email::{parse_eml, parse_mbox, parse_msg, EmailInfo};

/// Parse an EML email file and return structured email info
#[command]
pub async fn email_parse_eml(path: String) -> Result<EmailInfo, String> {
    parse_eml(&path).map_err(|e| e.to_string())
}

/// Parse an MBOX file and return multiple email messages
#[command]
pub async fn email_parse_mbox(
    path: String,
    max_messages: Option<usize>,
) -> Result<Vec<EmailInfo>, String> {
    parse_mbox(&path, max_messages).map_err(|e| e.to_string())
}

/// Parse an Outlook .msg file and return structured email info
#[command]
pub async fn email_parse_msg(path: String) -> Result<EmailInfo, String> {
    parse_msg(&path).map_err(|e| e.to_string())
}

// =============================================================================
// PST/OST Commands
// =============================================================================

use super::pst::{
    pst_get_message, pst_list_folders, pst_list_messages, PstInfo, PstMessageDetail,
    PstMessageSummary,
};

/// List all folders in a PST/OST file
#[command]
pub async fn pst_get_folders(path: String) -> Result<PstInfo, String> {
    // UnicodePstFile is !Send — must run on a blocking thread
    tokio::task::spawn_blocking(move || pst_list_folders(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// List message summaries in a PST folder
#[command]
pub async fn pst_get_messages(
    path: String,
    folder_node_id: u32,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<PstMessageSummary>, String> {
    tokio::task::spawn_blocking(move || {
        pst_list_messages(&path, folder_node_id, offset, limit).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get full message detail from a PST file
#[command]
pub async fn pst_get_message_detail(
    path: String,
    message_node_id: u32,
) -> Result<PstMessageDetail, String> {
    tokio::task::spawn_blocking(move || {
        pst_get_message(&path, message_node_id).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Plist Commands
// =============================================================================

use super::plist_viewer::{read_plist, PlistInfo};

/// Read and parse a plist file, returning flattened entries
#[command]
pub async fn plist_read(path: String) -> Result<PlistInfo, String> {
    read_plist(&path).map_err(|e| e.to_string())
}

// =============================================================================
// EXIF Metadata Commands
// =============================================================================

use super::exif::{extract_exif, ExifMetadata};

/// Extract EXIF metadata from an image file
#[command]
pub async fn exif_extract(path: String) -> Result<ExifMetadata, String> {
    extract_exif(&path).map_err(|e| e.to_string())
}

// =============================================================================
// Binary Analysis Commands
// =============================================================================

use super::binary::{analyze_binary, BinaryInfo};

/// Analyze a binary executable (PE/ELF/Mach-O)
#[command]
pub async fn binary_analyze(path: String) -> Result<BinaryInfo, String> {
    analyze_binary(&path).map_err(|e| e.to_string())
}

// =============================================================================
// Registry Hive Commands
// =============================================================================

use super::registry_viewer::{
    get_hive_info, get_key_info, get_subkeys, RegistryHiveInfo, RegistryKeyInfo,
    RegistrySubkeysResponse,
};

/// Get overview information about a Windows Registry hive file
#[command]
pub async fn registry_get_info(path: String) -> Result<RegistryHiveInfo, String> {
    get_hive_info(&path).map_err(|e| e.to_string())
}

/// Get immediate subkeys of a registry key
#[command]
pub async fn registry_get_subkeys(
    hive_path: String,
    key_path: String,
) -> Result<RegistrySubkeysResponse, String> {
    get_subkeys(&hive_path, &key_path).map_err(|e| e.to_string())
}

/// Get detailed key information including subkeys and values
#[command]
pub async fn registry_get_key_info(
    hive_path: String,
    key_path: String,
) -> Result<RegistryKeyInfo, String> {
    get_key_info(&hive_path, &key_path).map_err(|e| e.to_string())
}

// =============================================================================
// Database Viewer Commands
// =============================================================================

use super::database_viewer::{
    get_database_info, get_table_schema, query_table_rows, DatabaseInfo, TableRows, TableSchema,
};

/// Get overview information about a SQLite database
#[command]
pub async fn database_get_info(path: String) -> Result<DatabaseInfo, String> {
    get_database_info(&path).map_err(|e| e.to_string())
}

/// Get schema for a specific table
#[command]
pub async fn database_get_table_schema(
    db_path: String,
    table_name: String,
) -> Result<TableSchema, String> {
    get_table_schema(&db_path, &table_name).map_err(|e| e.to_string())
}

/// Query paginated rows from a table
#[command]
pub async fn database_query_table(
    db_path: String,
    table_name: String,
    page: usize,
    page_size: usize,
) -> Result<TableRows, String> {
    query_table_rows(&db_path, &table_name, page, page_size).map_err(|e| e.to_string())
}

// =============================================================================
// Office Document Commands
// =============================================================================

use super::office::{read_office_document, OfficeDocumentInfo};

/// Read an office document and extract text + metadata
///
/// Supports: DOCX, DOC, PPTX, PPT, ODT, ODP, RTF
#[command]
pub async fn office_read_document(path: String) -> Result<OfficeDocumentInfo, String> {
    read_office_document(&path).map_err(|e| e.to_string())
}
