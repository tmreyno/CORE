// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for document operations
//!
//! This module exposes document functionality to the frontend via Tauri commands.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use tauri::command;

use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{
    read_all_with_limit, read_range_fully, EvidenceByteSource, EvidenceSourceReader,
    EvidenceSourceRef,
};

use super::types::{DocumentContent, DocumentMetadata};
use super::{DocumentFormat, DocumentService};

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

const MATERIALIZED_SOURCE_COPY_CHUNK_BYTES: usize = 1024 * 1024;
const DOCUMENT_SOURCE_MAX_BYTES: u64 = 100 * 1024 * 1024;

fn copy_evidence_source_to_writer(
    byte_source: &dyn EvidenceByteSource,
    expected_size: u64,
    label: &str,
    writer: &mut impl Write,
) -> Result<(), String> {
    let source_id = byte_source.source_ref().display_id();
    let mut offset = 0u64;

    while offset < expected_size {
        let Some((remaining, read_size)) =
            checked_materialized_copy_read_size(expected_size, offset, label, &source_id)?
        else {
            break;
        };
        let chunk = byte_source.read_range(offset, read_size).map_err(|e| {
            format!("Failed to read {label} source {source_id} at offset {offset}: {e}")
        })?;

        if chunk.is_empty() {
            return Err(format!(
                "Short read materializing {label} source {source_id}: expected {expected_size} bytes but read {offset} bytes"
            ));
        }
        if chunk.len() as u64 > remaining {
            return Err(format!(
                "Invalid oversized read materializing {label} source {source_id}: {} bytes returned with {remaining} bytes remaining",
                chunk.len()
            ));
        }

        writer.write_all(&chunk).map_err(|e| {
            format!("Failed to write {label} source {source_id} at offset {offset}: {e}")
        })?;
        offset = checked_materialized_copy_advance(offset, chunk.len(), label, &source_id)?;
    }

    Ok(())
}

fn checked_materialized_copy_read_size(
    expected_size: u64,
    offset: u64,
    label: &str,
    source_id: &str,
) -> Result<Option<(u64, usize)>, String> {
    let remaining = expected_size.checked_sub(offset).ok_or_else(|| {
        format!(
            "{label} materialization byte counter exceeded source size for {source_id}: copied {offset} bytes > expected {expected_size} bytes"
        )
    })?;
    if remaining == 0 {
        return Ok(None);
    }

    let chunk_limit = u64::try_from(MATERIALIZED_SOURCE_COPY_CHUNK_BYTES).map_err(|_| {
        format!("{label} materialization chunk size does not fit in u64 for {source_id}")
    })?;
    let read_size = remaining.min(chunk_limit);
    let read_size = usize::try_from(read_size).map_err(|_| {
        format!("{label} materialization read size does not fit in usize for {source_id}")
    })?;
    Ok(Some((remaining, read_size)))
}

fn checked_materialized_copy_advance(
    offset: u64,
    bytes_read: usize,
    label: &str,
    source_id: &str,
) -> Result<u64, String> {
    let bytes_read = u64::try_from(bytes_read).map_err(|_| {
        format!("{label} source {source_id} returned a chunk length that does not fit in u64")
    })?;
    offset.checked_add(bytes_read).ok_or_else(|| {
        format!(
            "{label} materialization byte counter overflow for {source_id}: offset {offset} + {bytes_read} bytes"
        )
    })
}

fn document_format_from_source_id(source_id: &str) -> DocumentFormat {
    DocumentFormat::from_extension(source_id).unwrap_or(DocumentFormat::Text)
}

fn document_content_from_source(source: HashSourceInput) -> Result<DocumentContent, String> {
    let byte_source = open_hash_source(&source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    let data = read_all_with_limit(byte_source.as_ref(), DOCUMENT_SOURCE_MAX_BYTES)
        .map_err(|e| e.to_string())?;

    let service = DocumentService::new();
    let format = document_format_from_source_id(&source_id);
    let mut content = service
        .read_bytes(&data, format)
        .map_err(|e| e.to_string())?;

    content.metadata.file_size = size;
    content.metadata.format = format;
    if content.metadata.title.is_none() {
        content.metadata.title = Path::new(&source_id)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .or(Some(source_id));
    }
    content.normalize_in_place();

    Ok(content)
}

/// Read a document file and return its content
#[command]
pub async fn document_read(path: String) -> Result<DocumentResponse, String> {
    tokio::task::spawn_blocking(move || {
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
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Read a document from a local file or supported container entry.
#[command]
pub async fn document_read_source(source: HashSourceInput) -> Result<DocumentResponse, String> {
    tokio::task::spawn_blocking(move || match document_content_from_source(source) {
        Ok(content) => Ok(DocumentResponse {
            success: true,
            content: Some(content.into()),
            error: None,
        }),
        Err(e) => Ok(DocumentResponse {
            success: false,
            content: None,
            error: Some(e),
        }),
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get document metadata
#[command]
pub async fn document_get_metadata(path: String) -> Result<MetadataResponse, String> {
    tokio::task::spawn_blocking(move || {
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
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get document metadata from a local file or supported container entry.
#[command]
pub async fn document_get_metadata_source(
    source: HashSourceInput,
) -> Result<MetadataResponse, String> {
    tokio::task::spawn_blocking(move || match document_content_from_source(source) {
        Ok(content) => Ok(MetadataResponse {
            success: true,
            metadata: Some(content.metadata.into()),
            error: None,
        }),
        Err(e) => Ok(MetadataResponse {
            success: false,
            metadata: None,
            error: Some(e),
        }),
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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

fn content_detect_response(format: UniversalFormat, method: &str) -> ContentDetectResponse {
    ContentDetectResponse {
        format: format!("{:?}", format),
        viewer_type: format!("{:?}", format.viewer_type()),
        description: format.description().to_string(),
        mime_type: format.mime_type().to_string(),
        method: method.to_string(),
    }
}

fn is_registry_log_extension(extension: &str) -> bool {
    extension == "log"
        || (extension.starts_with("log")
            && extension.len() > 3
            && extension[3..].chars().all(|c| c.is_ascii_digit()))
}

fn refine_magic_format(magic_format: UniversalFormat, path_ref: &Path) -> UniversalFormat {
    match magic_format {
        UniversalFormat::Zip | UniversalFormat::Doc => {
            UniversalFormat::from_path(path_ref).unwrap_or(magic_format)
        }
        UniversalFormat::Exe => match UniversalFormat::from_path(path_ref) {
            Some(format @ (UniversalFormat::Sys | UniversalFormat::Dll | UniversalFormat::Exe)) => {
                format
            }
            _ => magic_format,
        },
        UniversalFormat::RegistryHive => {
            // Registry transaction logs (.log, .log1, .log2, etc.) share the
            // "regf" magic signature with actual hives but have a different
            // internal structure that notatin cannot parse. Route them to hex.
            let ext = path_ref
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if is_registry_log_extension(&ext) {
                UniversalFormat::Binary
            } else {
                magic_format
            }
        }
        _ => magic_format,
    }
}

fn detect_content_format_from_header(path_ref: &Path, header: &[u8]) -> ContentDetectResponse {
    if let Some(magic_format) = UniversalFormat::detect_by_magic_bytes(header) {
        return content_detect_response(refine_magic_format(magic_format, path_ref), "magic");
    }

    if let Some(format) = UniversalFormat::from_path(path_ref) {
        return content_detect_response(format, "extension");
    }

    ContentDetectResponse {
        format: "Binary".to_string(),
        viewer_type: "Hex".to_string(),
        description: "Unknown binary data".to_string(),
        mime_type: "application/octet-stream".to_string(),
        method: "fallback".to_string(),
    }
}

fn source_detection_name(source: &HashSourceInput) -> String {
    source
        .entry_path
        .as_deref()
        .or(source.path.as_deref())
        .or(source.nested_archive_path.as_deref())
        .or(source.container_path.as_deref())
        .unwrap_or("source")
        .to_string()
}

fn detect_content_format_for_source(
    source: &dyn EvidenceByteSource,
    source_name: &str,
) -> Result<ContentDetectResponse, String> {
    let total_size = source.len().map_err(|e| e.to_string())?;
    let read_size = total_size.min(265) as usize;
    let header = if read_size == 0 {
        Vec::new()
    } else {
        read_range_fully(source, 0, read_size).map_err(|e| e.to_string())?
    };
    Ok(detect_content_format_from_header(
        std::path::Path::new(source_name),
        &header,
    ))
}

fn read_local_header(path_ref: &Path, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path_ref)?;
    let total_size = file.metadata()?.len();
    let to_read = total_size.min(max_bytes as u64) as usize;
    let mut buf = vec![0u8; to_read];
    file.read_exact(&mut buf)?;
    Ok(buf)
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
    tokio::task::spawn_blocking(move || {
        let path_ref = std::path::Path::new(&path);
        let header = read_local_header(path_ref, 265).map_err(|e| {
            format!(
                "Failed to read local header for {}: {}",
                path_ref.display(),
                e
            )
        })?;

        Ok(detect_content_format_from_header(path_ref, &header))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Detect file format from a local file or supported container entry.
#[command]
pub async fn detect_content_format_source(
    source: HashSourceInput,
) -> Result<ContentDetectResponse, String> {
    let source_name = source_detection_name(&source);

    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        detect_content_format_for_source(byte_source.as_ref(), &source_name)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Spreadsheet Commands
// =============================================================================

use super::spreadsheet::{
    read_sheet, read_sheet_bytes, read_spreadsheet_info, read_spreadsheet_info_bytes, CellValue,
    SpreadsheetInfo,
};

const SPREADSHEET_SOURCE_MAX_BYTES: u64 = 100 * 1024 * 1024;

/// Get spreadsheet metadata (sheets, format, etc.)
#[command]
pub async fn spreadsheet_info(path: String) -> Result<SpreadsheetInfo, String> {
    tokio::task::spawn_blocking(move || read_spreadsheet_info(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Get spreadsheet metadata from a local file or supported container entry.
#[command]
pub async fn spreadsheet_info_source(source: HashSourceInput) -> Result<SpreadsheetInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), SPREADSHEET_SOURCE_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        read_spreadsheet_info_bytes(source_id, &data).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
    tokio::task::spawn_blocking(move || {
        read_sheet(&path, &sheet_name, start, max).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Read a sheet from a local file or supported container entry.
#[command]
pub async fn spreadsheet_read_sheet_source(
    source: HashSourceInput,
    sheet_name: String,
    start_row: Option<usize>,
    max_rows: Option<usize>,
) -> Result<Vec<Vec<CellValue>>, String> {
    let start = start_row.unwrap_or(0);
    let max = max_rows.unwrap_or(500);
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), SPREADSHEET_SOURCE_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        read_sheet_bytes(source_id, &data, &sheet_name, start, max).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Email Commands
// =============================================================================

use super::email::{
    parse_eml, parse_eml_bytes, parse_mbox, parse_mbox_bytes, parse_msg, EmailInfo,
};

const EMAIL_SOURCE_MAX_BYTES: u64 = 50 * 1024 * 1024;

/// Parse an EML email file and return structured email info
#[command]
pub async fn email_parse_eml(path: String) -> Result<EmailInfo, String> {
    tokio::task::spawn_blocking(move || parse_eml(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Parse an EML email from a local file or supported container entry.
#[command]
pub async fn email_parse_eml_source(source: HashSourceInput) -> Result<EmailInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), EMAIL_SOURCE_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        parse_eml_bytes(source_id, &data).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Parse an MBOX file and return multiple email messages
#[command]
pub async fn email_parse_mbox(
    path: String,
    max_messages: Option<usize>,
) -> Result<Vec<EmailInfo>, String> {
    tokio::task::spawn_blocking(move || parse_mbox(&path, max_messages).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Parse an MBOX mailbox from a local file or supported container entry.
#[command]
pub async fn email_parse_mbox_source(
    source: HashSourceInput,
    max_messages: Option<usize>,
) -> Result<Vec<EmailInfo>, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), EMAIL_SOURCE_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        parse_mbox_bytes(source_id, &data, max_messages).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Parse an Outlook .msg file and return structured email info
#[command]
pub async fn email_parse_msg(path: String) -> Result<EmailInfo, String> {
    tokio::task::spawn_blocking(move || parse_msg(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Parse an Outlook .msg file from a local file or supported container entry.
#[command]
pub async fn email_parse_msg_source(source: HashSourceInput) -> Result<EmailInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let size = byte_source.len().map_err(|e| e.to_string())?;
        if size > EMAIL_SOURCE_MAX_BYTES {
            return Err(format!(
                "MSG file too large ({:.1} MB, max 50 MB)",
                size as f64 / (1024.0 * 1024.0)
            ));
        }

        let mut temp = tempfile::Builder::new()
            .prefix("core-ffx-msg-")
            .suffix(".msg")
            .tempfile()
            .map_err(|e| format!("Failed to create temporary MSG copy: {}", e))?;
        copy_evidence_source_to_writer(byte_source.as_ref(), size, "MSG", &mut temp)?;
        temp.flush()
            .map_err(|e| format!("Failed to flush temporary MSG copy: {}", e))?;
        temp.as_file()
            .sync_all()
            .map_err(|e| format!("Failed to sync temporary MSG copy: {}", e))?;

        let mut info = parse_msg(temp.path()).map_err(|e| e.to_string())?;
        info.path = source_id;
        info.size = size;
        Ok(info)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// PST/OST Commands
// =============================================================================

use super::pst::{
    pst_get_message, pst_list_folders, pst_list_messages, PstInfo, PstMessageDetail,
    PstMessageSummary,
};

const PST_SOURCE_MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn with_pst_source<T>(
    source: HashSourceInput,
    operation: impl FnOnce(&Path, String) -> Result<T, String>,
) -> Result<T, String> {
    let byte_source = open_hash_source(&source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > PST_SOURCE_MAX_BYTES {
        return Err(format!(
            "PST/OST source is too large for preview: {} bytes > {} bytes",
            size, PST_SOURCE_MAX_BYTES
        ));
    }

    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-pst-")
        .suffix(".pst")
        .tempfile()
        .map_err(|e| format!("Failed to create temporary PST copy: {}", e))?;
    copy_evidence_source_to_writer(byte_source.as_ref(), size, "PST", &mut temp)?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary PST copy: {}", e))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary PST copy: {}", e))?;

    operation(temp.path(), source_id)
}

/// List all folders in a PST/OST file
#[command]
pub async fn pst_get_folders(path: String) -> Result<PstInfo, String> {
    // UnicodePstFile is !Send — must run on a blocking thread
    tokio::task::spawn_blocking(move || pst_list_folders(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// List all folders in a PST/OST file inside an evidence source.
#[command]
pub async fn pst_get_folders_source(source: HashSourceInput) -> Result<PstInfo, String> {
    tokio::task::spawn_blocking(move || {
        with_pst_source(source, |path, source_id| {
            let mut info = pst_list_folders(&path.to_string_lossy()).map_err(|e| e.to_string())?;
            info.path = source_id;
            Ok(info)
        })
    })
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

/// List message summaries from a PST/OST evidence source.
#[command]
pub async fn pst_get_messages_source(
    source: HashSourceInput,
    folder_node_id: u32,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<PstMessageSummary>, String> {
    tokio::task::spawn_blocking(move || {
        with_pst_source(source, |path, _source_id| {
            pst_list_messages(&path.to_string_lossy(), folder_node_id, offset, limit)
                .map_err(|e| e.to_string())
        })
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

/// Get full message detail from a PST/OST evidence source.
#[command]
pub async fn pst_get_message_detail_source(
    source: HashSourceInput,
    message_node_id: u32,
) -> Result<PstMessageDetail, String> {
    tokio::task::spawn_blocking(move || {
        with_pst_source(source, |path, _source_id| {
            pst_get_message(&path.to_string_lossy(), message_node_id).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Plist Commands
// =============================================================================

use super::plist_viewer::{
    ensure_plist_preview_size_allowed, read_plist, read_plist_from_reader, PlistInfo,
};

/// Read and parse a plist file, returning flattened entries
#[command]
pub async fn plist_read(path: String) -> Result<PlistInfo, String> {
    tokio::task::spawn_blocking(move || read_plist(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Read and parse a plist from a local file or supported container entry.
#[command]
pub async fn plist_read_source(source: HashSourceInput) -> Result<PlistInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        ensure_plist_preview_size_allowed(byte_source.len().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let source_id = byte_source.source_ref().display_id();
        let reader = EvidenceSourceReader::new(byte_source.as_ref());
        read_plist_from_reader(source_id, reader).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// EXIF Metadata Commands
// =============================================================================

use super::exif::{ensure_exif_size_allowed, extract_exif, extract_exif_from_reader, ExifMetadata};

/// Extract EXIF metadata from an image file
#[command]
pub async fn exif_extract(path: String) -> Result<ExifMetadata, String> {
    tokio::task::spawn_blocking(move || extract_exif(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Extract EXIF metadata from a local file or supported container entry.
#[command]
pub async fn exif_extract_source(source: HashSourceInput) -> Result<ExifMetadata, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        ensure_exif_size_allowed(byte_source.len().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let source_id = byte_source.source_ref().display_id();
        let reader = EvidenceSourceReader::new(byte_source.as_ref());
        extract_exif_from_reader(source_id, reader).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Binary Analysis Commands
// =============================================================================

use super::binary::{analyze_binary, analyze_binary_bytes, BinaryInfo};

const BINARY_ANALYSIS_MAX_BYTES: u64 = 512 * 1024 * 1024;

/// Analyze a binary executable (PE/ELF/Mach-O)
#[command]
pub async fn binary_analyze(path: String) -> Result<BinaryInfo, String> {
    tokio::task::spawn_blocking(move || analyze_binary(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Analyze a binary executable from a local file or supported container entry.
#[command]
pub async fn binary_analyze_source(source: HashSourceInput) -> Result<BinaryInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), BINARY_ANALYSIS_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        analyze_binary_bytes(source_id, &data).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Registry Hive Commands
// =============================================================================

use super::registry_viewer::{
    get_hive_info, get_key_info, get_subkeys, RegistryHiveInfo, RegistryKeyInfo,
    RegistrySubkeysResponse,
};

const REGISTRY_SOURCE_MAX_BYTES: u64 = 256 * 1024 * 1024;
const REGISTRY_SOURCE_CACHE_MAX_ENTRIES: usize = 8;

static REGISTRY_SOURCE_CACHE: LazyLock<Mutex<HashMap<String, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn with_registry_source<T>(
    source: HashSourceInput,
    operation: impl FnOnce(&Path, String) -> Result<T, String>,
) -> Result<T, String> {
    let byte_source = open_hash_source(&source)?;
    let source_ref = byte_source.source_ref();
    let source_id = source_ref.display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > REGISTRY_SOURCE_MAX_BYTES {
        return Err(format!(
            "Registry source is too large for preview: {} bytes > {} bytes",
            size, REGISTRY_SOURCE_MAX_BYTES
        ));
    }

    if let EvidenceSourceRef::LocalFile { path } = &source_ref {
        return operation(Path::new(path), source_id);
    }

    let cached_path =
        cached_registry_source_path(byte_source.as_ref(), &source_ref, &source_id, size)?;
    operation(&cached_path, source_id)
}

fn cached_registry_source_path(
    byte_source: &dyn EvidenceByteSource,
    source_ref: &EvidenceSourceRef,
    source_id: &str,
    size: u64,
) -> Result<PathBuf, String> {
    let cache_key = registry_source_cache_key(source_ref, size);
    if let Some(path) = registry_source_cache_get(&cache_key) {
        return Ok(path);
    }

    let cache_dir = std::env::temp_dir().join("core-ffx-registry-cache");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create registry source cache directory: {e}"))?;
    let cache_path = cache_dir.join(format!("{cache_key}.hive"));

    if cache_path.exists() {
        registry_source_cache_insert(cache_key, cache_path.clone());
        return Ok(cache_path);
    }

    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-registry-")
        .suffix(".hive")
        .tempfile_in(&cache_dir)
        .map_err(|e| format!("Failed to create temporary registry copy: {e}"))?;
    copy_evidence_source_to_writer(byte_source, size, "registry", &mut temp)?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary registry copy: {e}"))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary registry copy: {e}"))?;
    temp.persist(&cache_path)
        .map_err(|e| format!("Failed to persist registry source cache copy: {}", e.error))?;

    registry_source_cache_insert(cache_key, cache_path.clone());
    tracing::debug!(
        source_id,
        path = %cache_path.display(),
        "Cached registry evidence source for viewer navigation"
    );
    Ok(cache_path)
}

fn registry_source_cache_get(cache_key: &str) -> Option<PathBuf> {
    let mut cache = REGISTRY_SOURCE_CACHE.lock().ok()?;
    let path = cache.get(cache_key).cloned()?;
    if path.exists() {
        return Some(path);
    }
    cache.remove(cache_key);
    None
}

fn registry_source_cache_insert(cache_key: String, cache_path: PathBuf) {
    let Ok(mut cache) = REGISTRY_SOURCE_CACHE.lock() else {
        return;
    };
    if cache.len() >= REGISTRY_SOURCE_CACHE_MAX_ENTRIES && !cache.contains_key(&cache_key) {
        let remove_count = cache.len() - REGISTRY_SOURCE_CACHE_MAX_ENTRIES + 1;
        let keys: Vec<String> = cache.keys().take(remove_count).cloned().collect();
        for key in keys {
            if let Some(path) = cache.remove(&key) {
                let _ = fs::remove_file(path);
            }
        }
    }
    cache.insert(cache_key, cache_path);
}

fn registry_source_cache_key(source_ref: &EvidenceSourceRef, size: u64) -> String {
    let mut hasher = DefaultHasher::new();
    source_ref.display_id().hash(&mut hasher);
    size.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get overview information about a Windows Registry hive file
#[command]
pub async fn registry_get_info(path: String) -> Result<RegistryHiveInfo, String> {
    tokio::task::spawn_blocking(move || get_hive_info(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Get overview information about a Windows Registry hive inside an evidence source.
#[command]
pub async fn registry_get_info_source(source: HashSourceInput) -> Result<RegistryHiveInfo, String> {
    tokio::task::spawn_blocking(move || {
        with_registry_source(source, |path, source_id| {
            let mut info = get_hive_info(&path.to_string_lossy()).map_err(|e| e.to_string())?;
            info.path = source_id;
            Ok(info)
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get immediate subkeys of a registry key
#[command]
pub async fn registry_get_subkeys(
    hive_path: String,
    key_path: String,
) -> Result<RegistrySubkeysResponse, String> {
    tokio::task::spawn_blocking(move || {
        get_subkeys(&hive_path, &key_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get immediate subkeys from a registry hive inside an evidence source.
#[command]
pub async fn registry_get_subkeys_source(
    source: HashSourceInput,
    key_path: String,
) -> Result<RegistrySubkeysResponse, String> {
    tokio::task::spawn_blocking(move || {
        with_registry_source(source, |path, _source_id| {
            get_subkeys(&path.to_string_lossy(), &key_path).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get detailed key information including subkeys and values
#[command]
pub async fn registry_get_key_info(
    hive_path: String,
    key_path: String,
) -> Result<RegistryKeyInfo, String> {
    tokio::task::spawn_blocking(move || {
        get_key_info(&hive_path, &key_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get detailed key information from a registry hive inside an evidence source.
#[command]
pub async fn registry_get_key_info_source(
    source: HashSourceInput,
    key_path: String,
) -> Result<RegistryKeyInfo, String> {
    tokio::task::spawn_blocking(move || {
        with_registry_source(source, |path, _source_id| {
            get_key_info(&path.to_string_lossy(), &key_path).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Database Viewer Commands
// =============================================================================

use super::database_viewer::{
    get_database_info, get_table_schema, query_table_rows, DatabaseInfo, TableRows, TableSchema,
};

const DATABASE_SOURCE_MAX_BYTES: u64 = 512 * 1024 * 1024;

fn with_database_source<T>(
    source: HashSourceInput,
    operation: impl FnOnce(&Path, String) -> Result<T, String>,
) -> Result<T, String> {
    let byte_source = open_hash_source(&source)?;
    let source_id = byte_source.source_ref().display_id();
    let size = byte_source.len().map_err(|e| e.to_string())?;
    if size > DATABASE_SOURCE_MAX_BYTES {
        return Err(format!(
            "Database source is too large for preview: {} bytes > {} bytes",
            size, DATABASE_SOURCE_MAX_BYTES
        ));
    }

    let mut temp = tempfile::Builder::new()
        .prefix("core-ffx-db-")
        .suffix(".sqlite")
        .tempfile()
        .map_err(|e| format!("Failed to create temporary database copy: {}", e))?;
    copy_evidence_source_to_writer(byte_source.as_ref(), size, "database", &mut temp)?;
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary database copy: {}", e))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| format!("Failed to sync temporary database copy: {}", e))?;

    operation(temp.path(), source_id)
}

/// Get overview information about a SQLite database
#[command]
pub async fn database_get_info(path: String) -> Result<DatabaseInfo, String> {
    tokio::task::spawn_blocking(move || get_database_info(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Get overview information about a SQLite database inside an evidence source.
#[command]
pub async fn database_get_info_source(source: HashSourceInput) -> Result<DatabaseInfo, String> {
    tokio::task::spawn_blocking(move || {
        with_database_source(source, |path, source_id| {
            let mut info = get_database_info(path).map_err(|e| e.to_string())?;
            info.path = source_id;
            Ok(info)
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get schema for a specific table
#[command]
pub async fn database_get_table_schema(
    db_path: String,
    table_name: String,
) -> Result<TableSchema, String> {
    tokio::task::spawn_blocking(move || {
        get_table_schema(&db_path, &table_name).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get schema for a table in a SQLite database inside an evidence source.
#[command]
pub async fn database_get_table_schema_source(
    source: HashSourceInput,
    table_name: String,
) -> Result<TableSchema, String> {
    tokio::task::spawn_blocking(move || {
        with_database_source(source, |path, _source_id| {
            get_table_schema(path, &table_name).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Query paginated rows from a table
#[command]
pub async fn database_query_table(
    db_path: String,
    table_name: String,
    page: usize,
    page_size: usize,
) -> Result<TableRows, String> {
    tokio::task::spawn_blocking(move || {
        query_table_rows(&db_path, &table_name, page, page_size).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Query paginated rows from a table in a SQLite evidence source.
#[command]
pub async fn database_query_table_source(
    source: HashSourceInput,
    table_name: String,
    page: usize,
    page_size: usize,
) -> Result<TableRows, String> {
    tokio::task::spawn_blocking(move || {
        with_database_source(source, |path, _source_id| {
            query_table_rows(path, &table_name, page, page_size).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

// =============================================================================
// Office Document Commands
// =============================================================================

use super::office::{read_office_document, read_office_document_bytes, OfficeDocumentInfo};

const OFFICE_SOURCE_MAX_BYTES: u64 = 100 * 1024 * 1024;

/// Read an office document and extract text + metadata
///
/// Supports: DOCX, DOC, PPTX, PPT, ODT, ODP, RTF
#[command]
pub async fn office_read_document(path: String) -> Result<OfficeDocumentInfo, String> {
    tokio::task::spawn_blocking(move || read_office_document(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Read an office document from a local file or supported container entry.
#[command]
pub async fn office_read_document_source(
    source: HashSourceInput,
) -> Result<OfficeDocumentInfo, String> {
    tokio::task::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        let source_id = byte_source.source_ref().display_id();
        let data = read_all_with_limit(byte_source.as_ref(), OFFICE_SOURCE_MAX_BYTES)
            .map_err(|e| e.to_string())?;
        read_office_document_bytes(source_id, &data).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EvidenceSourceError, EvidenceSourceRef, EvidenceSourceResult};
    use rusqlite::Connection;

    struct ChunkedByteSource {
        source_ref: EvidenceSourceRef,
        data: Vec<u8>,
        max_chunk: usize,
    }

    impl ChunkedByteSource {
        fn new(path: &str, data: &[u8], max_chunk: usize) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                data: data.to_vec(),
                max_chunk,
            }
        }
    }

    impl EvidenceByteSource for ChunkedByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.data.len() as u64)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let requested = size.min(self.max_chunk);
            let end = start.saturating_add(requested).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    struct ShortReadByteSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
        data: Vec<u8>,
    }

    impl ShortReadByteSource {
        fn new(path: &str, declared_len: u64, data: &[u8]) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                declared_len,
                data: data.to_vec(),
            }
        }
    }

    impl EvidenceByteSource for ShortReadByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, offset: u64, size: usize) -> EvidenceSourceResult<Vec<u8>> {
            if offset > self.declared_len {
                return Err(EvidenceSourceError::InvalidRange {
                    source_id: self.source_ref.display_id(),
                    offset,
                    size: self.declared_len,
                });
            }

            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let end = start.saturating_add(size).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    fn create_sqlite_source() -> (tempfile::NamedTempFile, HashSourceInput) {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE contacts (id INTEGER PRIMARY KEY, name TEXT);
             INSERT INTO contacts VALUES (1, 'Alice');
             INSERT INTO contacts VALUES (2, 'Bob');",
        )
        .unwrap();
        drop(conn);

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: None,
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        (tmp, source)
    }

    fn minimal_elf64_header() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
        data.extend_from_slice(&[2, 1, 1, 0]);
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&0x3eu16.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&0x400000u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&64u16.to_le_bytes());
        data.extend_from_slice(&56u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&64u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data
    }

    #[test]
    fn copy_evidence_source_to_writer_accepts_chunked_reads() {
        let source = ChunkedByteSource::new("chunked.sqlite", b"0123456789", 3);
        let mut output = Vec::new();

        copy_evidence_source_to_writer(&source, source.len().unwrap(), "database", &mut output)
            .unwrap();

        assert_eq!(output, b"0123456789");
    }

    #[test]
    fn copy_evidence_source_to_writer_rejects_short_reads() {
        let source = ShortReadByteSource::new("short.sqlite", 8, b"abc");
        let mut output = Vec::new();

        let err =
            copy_evidence_source_to_writer(&source, source.len().unwrap(), "database", &mut output)
                .unwrap_err();

        assert!(err.contains("Short read materializing database source"));
        assert!(err.contains("expected 8 bytes but read 3 bytes"));
        assert_eq!(output, b"abc");
    }

    #[test]
    fn checked_materialized_copy_read_size_returns_none_at_expected_size() {
        assert_eq!(
            checked_materialized_copy_read_size(10, 10, "database", "source.db").unwrap(),
            None
        );
    }

    #[test]
    fn checked_materialized_copy_read_size_clamps_to_tail() {
        assert_eq!(
            checked_materialized_copy_read_size(10, 8, "database", "source.db").unwrap(),
            Some((2, 2))
        );
    }

    #[test]
    fn checked_materialized_copy_read_size_rejects_counter_past_expected_size() {
        let err = checked_materialized_copy_read_size(10, 11, "database", "source.db").unwrap_err();

        assert!(err.contains("byte counter exceeded source size"));
    }

    #[test]
    fn registry_source_cache_key_changes_with_size() {
        let source_ref = EvidenceSourceRef::VfsEntry {
            container_path: "/cases/windows.E01".to_string(),
            entry_path: "/Windows/System32/config/SYSTEM".to_string(),
            container_type: Some("e01".to_string()),
        };

        assert_eq!(
            registry_source_cache_key(&source_ref, 4096),
            registry_source_cache_key(&source_ref, 4096)
        );
        assert_ne!(
            registry_source_cache_key(&source_ref, 4096),
            registry_source_cache_key(&source_ref, 8192)
        );
    }

    #[test]
    fn registry_source_uses_local_file_without_materializing_copy() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"regf").unwrap();
        tmp.flush().unwrap();
        let local_path = tmp.path().to_string_lossy().to_string();
        let source = HashSourceInput {
            path: Some(local_path.clone()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("disk".to_string()),
            size: Some(4),
            data_addr: None,
            item_addr: None,
        };

        with_registry_source(source, |path, source_id| {
            assert_eq!(path, Path::new(&local_path));
            assert_eq!(source_id, local_path);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn checked_materialized_copy_advance_rejects_overflow() {
        let err =
            checked_materialized_copy_advance(u64::MAX, 1, "database", "source.db").unwrap_err();

        assert!(err.contains("byte counter overflow"));
    }

    #[test]
    fn content_detection_refines_zip_magic_with_source_name() {
        let detected =
            detect_content_format_from_header(Path::new("case-export.xlsx"), b"PK\x03\x04");
        assert_eq!(detected.format, "Xlsx");
        assert_eq!(detected.viewer_type, "Spreadsheet");
        assert_eq!(detected.method, "magic");
    }

    #[test]
    fn content_detection_routes_registry_logs_to_hex() {
        let detected = detect_content_format_from_header(Path::new("transaction.log1"), b"regf");
        assert_eq!(detected.format, "Binary");
        assert_eq!(detected.viewer_type, "Hex");
        assert_eq!(detected.method, "magic");
    }

    #[test]
    fn content_detection_refines_mz_magic_for_windows_drivers() {
        let detected = detect_content_format_from_header(Path::new("netadapter.sys"), b"MZ\x90\0");
        assert_eq!(detected.format, "Sys");
        assert_eq!(detected.viewer_type, "Binary");
        assert_eq!(detected.description, "Windows Driver");
        assert_eq!(detected.mime_type, "application/x-windows-driver");
        assert_eq!(detected.method, "magic");
    }

    #[test]
    fn content_detection_source_assembles_chunked_header() {
        let source = ChunkedByteSource::new("case-export.xlsx", b"PK\x03\x04chunked office zip", 2);

        let detected = detect_content_format_for_source(&source, "case-export.xlsx").unwrap();

        assert_eq!(detected.format, "Xlsx");
        assert_eq!(detected.viewer_type, "Spreadsheet");
        assert_eq!(detected.method, "magic");
    }

    #[tokio::test]
    async fn detect_content_format_source_reads_local_source() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"%PDF-1.7\n").unwrap();
        tmp.flush().unwrap();

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: Some("disk".to_string()),
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let detected = detect_content_format_source(source).await.unwrap();
        assert_eq!(detected.format, "Pdf");
        assert_eq!(detected.viewer_type, "Pdf");
        assert_eq!(detected.method, "magic");
    }

    #[tokio::test]
    async fn detect_content_format_reads_exact_local_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("case-export.xlsx");
        std::fs::write(&path, b"PK\x03\x04chunked office zip").unwrap();

        let detected = detect_content_format(path.to_string_lossy().to_string())
            .await
            .unwrap();

        assert_eq!(detected.format, "Xlsx");
        assert_eq!(detected.viewer_type, "Spreadsheet");
        assert_eq!(detected.method, "magic");
    }

    #[tokio::test]
    async fn detect_content_format_rejects_missing_local_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.pdf");

        let err = detect_content_format(path.to_string_lossy().to_string())
            .await
            .unwrap_err();

        assert!(err.contains("Failed to read local header"));
        assert!(err.contains("missing.pdf"));
    }

    #[tokio::test]
    async fn database_source_commands_read_local_source() {
        let (_tmp, source) = create_sqlite_source();

        let info = database_get_info_source(source.clone()).await.unwrap();
        assert_eq!(info.path, source.path.as_deref().unwrap());
        assert_eq!(info.tables.len(), 1);
        assert_eq!(info.tables[0].name, "contacts");
        assert_eq!(info.tables[0].row_count, 2);

        let schema = database_get_table_schema_source(source.clone(), "contacts".to_string())
            .await
            .unwrap();
        assert_eq!(schema.name, "contacts");
        assert_eq!(schema.columns.len(), 2);

        let rows = database_query_table_source(source, "contacts".to_string(), 0, 100)
            .await
            .unwrap();
        assert_eq!(rows.rows.len(), 2);
        assert_eq!(rows.rows[0][1], "Alice");
        assert_eq!(rows.rows[1][1], "Bob");
    }

    #[tokio::test]
    async fn binary_analyze_reads_local_binary_on_blocking_worker() {
        let mut tmp = tempfile::Builder::new().suffix(".elf").tempfile().unwrap();
        tmp.write_all(&minimal_elf64_header()).unwrap();
        tmp.flush().unwrap();

        let info = binary_analyze(tmp.path().to_string_lossy().to_string())
            .await
            .unwrap();

        assert!(matches!(
            info.format,
            super::super::binary::BinaryFormat::ELF64
        ));
        assert_eq!(info.architecture, "x86_64");
        assert_eq!(info.entry_point, Some(0x400000));
    }

    #[tokio::test]
    async fn registry_source_command_reports_parse_error_for_non_hive_source() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"not a registry hive").unwrap();
        tmp.flush().unwrap();

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: None,
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let error = registry_get_info_source(source).await.unwrap_err();
        assert!(
            error.contains("Failed to open registry hive"),
            "unexpected error: {}",
            error
        );
    }

    #[tokio::test]
    async fn document_source_commands_read_local_text_source() {
        let mut tmp = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
        tmp.write_all(b"Alpha\n\nBeta").unwrap();
        tmp.flush().unwrap();

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: None,
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let response = document_read_source(source.clone()).await.unwrap();
        assert!(response.success);
        let content = response.content.unwrap();
        assert_eq!(content.format, "Text");
        assert!(content.text.contains("Alpha"));
        assert!(content.html.contains("Beta"));

        let metadata = document_get_metadata_source(source).await.unwrap();
        assert!(metadata.success);
        let metadata = metadata.metadata.unwrap();
        assert_eq!(metadata.format, "Text");
        assert_eq!(metadata.file_size, 11);
    }

    #[tokio::test]
    async fn pst_source_command_reports_parse_error_for_non_pst_source() {
        let mut tmp = tempfile::Builder::new().suffix(".pst").tempfile().unwrap();
        tmp.write_all(b"not a pst file").unwrap();
        tmp.flush().unwrap();

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: None,
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let error = pst_get_folders_source(source).await.unwrap_err();
        assert!(
            error.contains("Failed to open PST"),
            "unexpected error: {}",
            error
        );
    }

    #[tokio::test]
    async fn msg_source_command_reports_parse_error_for_non_msg_source() {
        let mut tmp = tempfile::Builder::new().suffix(".msg").tempfile().unwrap();
        tmp.write_all(b"not a msg file").unwrap();
        tmp.flush().unwrap();

        let source = HashSourceInput {
            path: Some(tmp.path().to_string_lossy().to_string()),
            container_path: None,
            entry_path: None,
            nested_archive_path: None,
            container_type: None,
            size: Some(std::fs::metadata(tmp.path()).unwrap().len()),
            data_addr: None,
            item_addr: None,
        };

        let error = email_parse_msg_source(source).await.unwrap_err();
        assert!(
            error.contains("Failed to parse MSG file"),
            "unexpected error: {}",
            error
        );
    }
}
