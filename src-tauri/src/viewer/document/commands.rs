// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for document operations
//!
//! This module exposes document functionality to the frontend via Tauri commands.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::command;

use super::{DocumentFormat, DocumentService};
use super::types::{DocumentContent, DocumentMetadata};

/// Serializable document content for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub success: bool,
    pub content: Option<DocumentContentDto>,
    pub error: Option<String>,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataResponse {
    pub success: bool,
    pub metadata: Option<DocumentMetadataDto>,
    pub error: Option<String>,
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
            created: m.creation_date.map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            modified: m.modification_date.map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            producer: m.producer,
            creator: m.creator,
            encrypted: false, // DocumentMetadata doesn't track encryption
            word_count: m.word_count,
        }
    }
}

/// HTML render response
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlResponse {
    pub success: bool,
    pub html: Option<String>,
    pub error: Option<String>,
}

/// Text extraction response
#[derive(Debug, Serialize, Deserialize)]
pub struct TextResponse {
    pub success: bool,
    pub text: Option<String>,
    pub error: Option<String>,
}

/// Write operation response
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteResponse {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
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

/// Read a document from raw bytes
#[command]
pub async fn document_read_bytes(data: Vec<u8>, format: String) -> Result<DocumentResponse, String> {
    let service = DocumentService::new();
    
    let doc_format = match format.to_lowercase().as_str() {
        "pdf" => DocumentFormat::Pdf,
        "docx" => DocumentFormat::Docx,
        "html" | "htm" => DocumentFormat::Html,
        "md" | "markdown" => DocumentFormat::Markdown,
        _ => return Ok(DocumentResponse {
            success: false,
            content: None,
            error: Some(format!("Unsupported format: {}", format)),
        }),
    };

    match service.read_bytes(&data, doc_format) {
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

/// Render a document to HTML
#[command]
pub async fn document_render_html(path: String) -> Result<HtmlResponse, String> {
    let service = DocumentService::new();
    let path = PathBuf::from(&path);

    match service.render_html(&path) {
        Ok(html) => Ok(HtmlResponse {
            success: true,
            html: Some(html),
            error: None,
        }),
        Err(e) => Ok(HtmlResponse {
            success: false,
            html: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Extract plain text from a document
#[command]
pub async fn document_extract_text(path: String) -> Result<TextResponse, String> {
    let service = DocumentService::new();
    let path = PathBuf::from(&path);

    match service.extract_text(&path) {
        Ok(text) => Ok(TextResponse {
            success: true,
            text: Some(text),
            error: None,
        }),
        Err(e) => Ok(TextResponse {
            success: false,
            text: None,
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

/// Detect document format from file extension
#[command]
pub async fn document_detect_format(path: String) -> Result<Option<String>, String> {
    let format = DocumentService::detect_format(&path);
    Ok(format.map(|f| format!("{:?}", f)))
}

/// Check if a file is a supported document format
#[command]
pub async fn document_is_supported(path: String) -> Result<bool, String> {
    Ok(DocumentService::is_supported(&path))
}

/// Get list of supported document extensions
#[command]
pub async fn document_supported_extensions() -> Result<Vec<String>, String> {
    Ok(DocumentService::supported_extensions()
        .iter()
        .map(|s| s.to_string())
        .collect())
}

// =============================================================================
// BULK OPERATIONS
// =============================================================================

/// Batch read multiple documents
#[command]
pub async fn document_read_batch(paths: Vec<String>) -> Result<Vec<DocumentResponse>, String> {
    let service = DocumentService::new();
    let mut results = Vec::with_capacity(paths.len());

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        let response = match service.read(&path) {
            Ok(content) => DocumentResponse {
                success: true,
                content: Some(content.into()),
                error: None,
            },
            Err(e) => DocumentResponse {
                success: false,
                content: None,
                error: Some(e.to_string()),
            },
        };
        results.push(response);
    }

    Ok(results)
}

/// Search text across multiple documents
#[command]
pub async fn document_search_text(
    paths: Vec<String>,
    query: String,
    case_sensitive: bool,
) -> Result<Vec<SearchResult>, String> {
    let service = DocumentService::new();
    let mut results = Vec::new();

    let query_lower = if case_sensitive {
        query.clone()
    } else {
        query.to_lowercase()
    };

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if let Ok(text) = service.extract_text(&path) {
            let search_text = if case_sensitive {
                text.clone()
            } else {
                text.to_lowercase()
            };

            if search_text.contains(&query_lower) {
                // Find match positions
                let mut matches = Vec::new();
                let mut start = 0;
                while let Some(pos) = search_text[start..].find(&query_lower) {
                    let abs_pos = start + pos;
                    // Get surrounding context
                    let context_start = abs_pos.saturating_sub(50);
                    let context_end = (abs_pos + query.len() + 50).min(text.len());
                    let context = text[context_start..context_end].to_string();
                    
                    matches.push(SearchMatch {
                        position: abs_pos,
                        context,
                    });
                    start = abs_pos + 1;
                    
                    // Limit matches per file
                    if matches.len() >= 10 {
                        break;
                    }
                }

                results.push(SearchResult {
                    path: path_str,
                    matches,
                });
            }
        }
    }

    Ok(results)
}

/// Search result for a single document
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub matches: Vec<SearchMatch>,
}

/// A single match within a document
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatch {
    pub position: usize,
    pub context: String,
}

// =============================================================================
// CONVERSION OPERATIONS
// =============================================================================

/// Convert document to another format
#[command]
pub async fn document_convert(
    input_path: String,
    output_path: String,
    output_format: String,
) -> Result<WriteResponse, String> {
    let service = DocumentService::new();
    let input = PathBuf::from(&input_path);
    let output = PathBuf::from(&output_path);

    // Read source document
    let content = match service.read(&input) {
        Ok(c) => c,
        Err(e) => return Ok(WriteResponse {
            success: false,
            output_path: None,
            error: Some(format!("Failed to read source: {}", e)),
        }),
    };

    // Convert based on output format
    let result = match output_format.to_lowercase().as_str() {
        "html" | "htm" => {
            let html = content.to_html();
            std::fs::write(&output, html)
        }
        "txt" | "text" => {
            let text = content.to_plain_text();
            std::fs::write(&output, text)
        }
        "md" | "markdown" => {
            // Convert to markdown-like text
            let mut md = String::new();
            for page in &content.pages {
                for element in &page.elements {
                    match element {
                        super::types::DocumentElement::Heading(h) => {
                            let prefix = "#".repeat(h.level as usize);
                            md.push_str(&format!("{} {}\n\n", prefix, h.text));
                        }
                        super::types::DocumentElement::Paragraph(p) => {
                            md.push_str(&format!("{}\n\n", p.text));
                        }
                        super::types::DocumentElement::List(l) => {
                            for (i, item) in l.items.iter().enumerate() {
                                if l.ordered {
                                    md.push_str(&format!("{}. {}\n", i + 1, item.text));
                                } else {
                                    md.push_str(&format!("- {}\n", item.text));
                                }
                            }
                            md.push('\n');
                        }
                        super::types::DocumentElement::Table(t) => {
                            if !t.rows.is_empty() {
                                // Header row
                                let row = &t.rows[0];
                                md.push('|');
                                for cell in &row.cells {
                                    md.push_str(&format!(" {} |", cell.text));
                                }
                                md.push('\n');
                                
                                // Separator
                                md.push('|');
                                for _ in &row.cells {
                                    md.push_str(" --- |");
                                }
                                md.push('\n');
                                
                                // Data rows
                                for row in t.rows.iter().skip(if t.has_header { 1 } else { 0 }) {
                                    md.push('|');
                                    for cell in &row.cells {
                                        md.push_str(&format!(" {} |", cell.text));
                                    }
                                    md.push('\n');
                                }
                            }
                            md.push('\n');
                        }
                        super::types::DocumentElement::Break => {
                            md.push_str("---\n\n");
                        }
                        _ => {}
                    }
                }
            }
            std::fs::write(&output, md)
        }
        _ => {
            return Ok(WriteResponse {
                success: false,
                output_path: None,
                error: Some(format!("Unsupported output format: {}", output_format)),
            });
        }
    };

    match result {
        Ok(_) => Ok(WriteResponse {
            success: true,
            output_path: Some(output.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(WriteResponse {
            success: false,
            output_path: None,
            error: Some(e.to_string()),
        }),
    }
}

// =============================================================================
// UNIVERSAL VIEWER OPERATIONS (Read-Only)
// =============================================================================

use super::universal::{
    UniversalFormat, FileInfo, ViewerHint,
    read_as_data_url, read_as_text, read_bytes,
    get_image_dimensions, create_thumbnail_data_url, get_viewer_hint,
};

/// Universal file info response
#[derive(Debug, Serialize, Deserialize)]
pub struct UniversalInfoResponse {
    pub success: bool,
    pub info: Option<FileInfoDto>,
    pub error: Option<String>,
}

/// File info DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoDto {
    pub path: String,
    pub name: String,
    pub format: String,
    pub viewer_type: String,
    pub mime_type: String,
    pub description: String,
    pub size: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub is_readable: bool,
    pub is_binary: bool,
}

impl From<FileInfo> for FileInfoDto {
    fn from(info: FileInfo) -> Self {
        Self {
            path: info.path,
            name: info.name,
            format: format!("{:?}", info.format),
            viewer_type: format!("{:?}", info.viewer_type),
            mime_type: info.mime_type,
            description: info.description,
            size: info.size,
            created: info.created,
            modified: info.modified,
            is_readable: info.is_readable,
            is_binary: info.is_binary,
        }
    }
}

/// Viewer hint response
#[derive(Debug, Serialize, Deserialize)]
pub struct ViewerHintResponse {
    pub success: bool,
    pub hint: Option<ViewerHintDto>,
    pub error: Option<String>,
}

/// Viewer hint DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerHintDto {
    pub viewer: String,
    pub format: String,
    pub mime_type: String,
    pub can_render: bool,
    pub can_search: bool,
    pub can_copy: bool,
    pub display_mode: String,
    pub dimensions: Option<(u32, u32)>,
    pub language: Option<String>,
    pub line_count: Option<usize>,
}

impl From<ViewerHint> for ViewerHintDto {
    fn from(hint: ViewerHint) -> Self {
        Self {
            viewer: format!("{:?}", hint.viewer),
            format: format!("{:?}", hint.format),
            mime_type: hint.mime_type,
            can_render: hint.can_render,
            can_search: hint.can_search,
            can_copy: hint.can_copy,
            display_mode: format!("{:?}", hint.display_mode),
            dimensions: hint.config.dimensions.map(|d| (d.width, d.height)),
            language: hint.config.language,
            line_count: hint.config.line_count,
        }
    }
}

/// Get file info (read-only metadata)
#[command]
pub async fn universal_get_info(path: String) -> Result<UniversalInfoResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    match FileInfo::from_path(&path_buf) {
        Ok(info) => Ok(UniversalInfoResponse {
            success: true,
            info: Some(info.into()),
            error: None,
        }),
        Err(e) => Ok(UniversalInfoResponse {
            success: false,
            info: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get viewer hint (what viewer to use)
#[command]
pub async fn universal_get_viewer_hint(path: String) -> Result<ViewerHintResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    match get_viewer_hint(&path_buf) {
        Ok(hint) => Ok(ViewerHintResponse {
            success: true,
            hint: Some(hint.into()),
            error: None,
        }),
        Err(e) => Ok(ViewerHintResponse {
            success: false,
            hint: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Detect format from file path
#[command]
pub async fn universal_detect_format(path: String) -> Result<Option<String>, String> {
    Ok(UniversalFormat::from_path(&path).map(|f| format!("{:?}", f)))
}

/// Check if file is supported
#[command]
pub async fn universal_is_supported(path: String) -> Result<bool, String> {
    Ok(UniversalFormat::from_path(&path).is_some())
}

/// Get all supported extensions
#[command]
pub async fn universal_supported_extensions() -> Result<Vec<String>, String> {
    Ok(UniversalFormat::all_extensions()
        .iter()
        .map(|s| s.to_string())
        .collect())
}

/// Read file as base64 data URL (for images)
#[command]
pub async fn universal_read_data_url(path: String) -> Result<DataUrlResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    match read_as_data_url(&path_buf) {
        Ok(data_url) => Ok(DataUrlResponse {
            success: true,
            data_url: Some(data_url),
            error: None,
        }),
        Err(e) => Ok(DataUrlResponse {
            success: false,
            data_url: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Data URL response
#[derive(Debug, Serialize, Deserialize)]
pub struct DataUrlResponse {
    pub success: bool,
    pub data_url: Option<String>,
    pub error: Option<String>,
}

/// Read file as text (with size limit)
#[command]
pub async fn universal_read_text(path: String, max_bytes: Option<usize>) -> Result<TextReadResponse, String> {
    let path_buf = PathBuf::from(&path);
    let limit = max_bytes.unwrap_or(10 * 1024 * 1024); // 10MB default
    
    match read_as_text(&path_buf, limit) {
        Ok((text, truncated)) => Ok(TextReadResponse {
            success: true,
            text: Some(text),
            truncated,
            error: None,
        }),
        Err(e) => Ok(TextReadResponse {
            success: false,
            text: None,
            truncated: false,
            error: Some(e.to_string()),
        }),
    }
}

/// Text read response
#[derive(Debug, Serialize, Deserialize)]
pub struct TextReadResponse {
    pub success: bool,
    pub text: Option<String>,
    pub truncated: bool,
    pub error: Option<String>,
}

/// Get image dimensions without loading full image
#[command]
pub async fn universal_get_image_dimensions(path: String) -> Result<ImageDimensionsResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    match get_image_dimensions(&path_buf) {
        Ok(dims) => Ok(ImageDimensionsResponse {
            success: true,
            width: Some(dims.width),
            height: Some(dims.height),
            error: None,
        }),
        Err(e) => Ok(ImageDimensionsResponse {
            success: false,
            width: None,
            height: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Image dimensions response
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDimensionsResponse {
    pub success: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub error: Option<String>,
}

/// Create thumbnail as data URL (in memory, no file created)
#[command]
pub async fn universal_create_thumbnail(path: String, max_size: u32) -> Result<DataUrlResponse, String> {
    let path_buf = PathBuf::from(&path);
    
    match create_thumbnail_data_url(&path_buf, max_size) {
        Ok(data_url) => Ok(DataUrlResponse {
            success: true,
            data_url: Some(data_url),
            error: None,
        }),
        Err(e) => Ok(DataUrlResponse {
            success: false,
            data_url: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Read file bytes as base64 (with size limit)
#[command]
pub async fn universal_read_bytes(path: String, max_bytes: Option<usize>) -> Result<BytesReadResponse, String> {
    let path_buf = PathBuf::from(&path);
    let limit = max_bytes.unwrap_or(50 * 1024 * 1024); // 50MB default
    
    match read_bytes(&path_buf, limit) {
        Ok((bytes, truncated)) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let base64 = STANDARD.encode(&bytes);
            
            Ok(BytesReadResponse {
                success: true,
                data: Some(base64),
                size: bytes.len(),
                truncated,
                error: None,
            })
        }
        Err(e) => Ok(BytesReadResponse {
            success: false,
            data: None,
            size: 0,
            truncated: false,
            error: Some(e.to_string()),
        }),
    }
}

/// Bytes read response
#[derive(Debug, Serialize, Deserialize)]
pub struct BytesReadResponse {
    pub success: bool,
    pub data: Option<String>, // base64 encoded
    pub size: usize,
    pub truncated: bool,
    pub error: Option<String>,
}

// =============================================================================
// Spreadsheet Commands
// =============================================================================

use super::spreadsheet::{SpreadsheetInfo, SheetInfo, CellValue, read_spreadsheet_info, read_sheet};

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
