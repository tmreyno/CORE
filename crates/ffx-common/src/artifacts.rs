// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Normalized artifact records extracted from evidence byte sources.
//!
//! This module is intentionally source-oriented: callers provide an
//! [`EvidenceByteSource`] and receive a stable record shape that can be shared by
//! viewers, search indexing, reporting, and project-database workflows.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

use chrono::{TimeZone, Utc};
use plist::Value as PlistValue;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::evidence_source::{
    read_range_fully, EvidenceByteSource, EvidenceSourceRef, EvidenceSourceResult,
};
use crate::magic::{detect_file_type, FileCategory};
use crate::source_analysis::{extract_source_indicators, SourceIndicator};

const DEFAULT_HEADER_BYTES: usize = 4096;
const DEFAULT_PREVIEW_BYTES: usize = 8192;
const DEFAULT_IMAGE_METADATA_BYTES: usize = 64 * 1024;
const DEFAULT_STRUCTURED_METADATA_BYTES: usize = 128 * 1024;
const MAX_HEADER_BYTES: usize = 256 * 1024;
const MAX_PREVIEW_BYTES: usize = 64 * 1024;
const MAX_TIFF_IFD_ENTRIES: usize = 1024;
const MAX_TIFF_RATIONAL_VALUES: usize = 16;
const MAX_IMAGE_METADATA_PIXELS: u64 = 100_000_000;
const MAX_METADATA_VALUE_CHARS: usize = 4096;
const TRUNCATED_METADATA_SUFFIX: &str = "... [truncated]";

/// Options for bounded artifact extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactExtractionOptions {
    /// Header bytes to read for magic/type detection.
    pub header_bytes: usize,
    /// Maximum bytes to read for text preview extraction.
    pub preview_bytes: usize,
}

impl Default for ArtifactExtractionOptions {
    fn default() -> Self {
        Self {
            header_bytes: DEFAULT_HEADER_BYTES,
            preview_bytes: DEFAULT_PREVIEW_BYTES,
        }
    }
}

/// Normalized artifact extracted from a byte source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedArtifact {
    /// Deterministic ID derived from source reference and source length.
    pub id: String,
    /// Stable source identity.
    pub source_ref: EvidenceSourceRef,
    /// Human-readable source ID for logs, search docs, and reports.
    pub source_id: String,
    /// Best-effort display name.
    pub name: String,
    /// Lowercase file extension, if present.
    pub extension: Option<String>,
    /// Byte length of the source.
    pub size: u64,
    /// MIME type from magic detection, if recognized.
    pub mime_type: Option<String>,
    /// Human-readable type description.
    pub type_description: String,
    /// Coarse category used by search/report pipelines.
    pub category: String,
    /// Detection confidence string from the magic detector.
    pub confidence: String,
    /// Whether the header/extension suggests text content.
    pub is_text: bool,
    /// Bounded lossy UTF-8 preview for text-like content.
    pub content_preview: Option<String>,
    /// Additional normalized metadata facts.
    pub metadata: BTreeMap<String, String>,
}

/// Extract a normalized artifact record from an evidence byte source.
pub fn extract_normalized_artifact(
    source: &dyn EvidenceByteSource,
    options: ArtifactExtractionOptions,
) -> EvidenceSourceResult<NormalizedArtifact> {
    let source_ref = source.source_ref();
    let source_id = source_ref.display_id();
    let size = source.len()?;
    let name = source_name(&source_ref);
    let extension = extension_for_name(&name);

    let requested_header_len = options.header_bytes.min(MAX_HEADER_BYTES);
    let requested_preview_len = options.preview_bytes.min(MAX_PREVIEW_BYTES);

    let initial_header_len = if is_image_extension(extension.as_deref()) {
        requested_header_len.max(DEFAULT_IMAGE_METADATA_BYTES)
    } else if is_structured_metadata_extension(extension.as_deref()) {
        requested_header_len.max(DEFAULT_STRUCTURED_METADATA_BYTES)
    } else {
        requested_header_len
    };
    let header_len = bounded_read_len(size, initial_header_len);
    let header = if header_len > 0 {
        read_range_fully(source, 0, header_len)?
    } else {
        Vec::new()
    };

    let detected = detect_file_type(&header);
    let mut mime_type = detected.as_ref().map(|ft| ft.mime.clone());
    let mut type_description = detected
        .as_ref()
        .map(|ft| ft.description.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let mut category = detected
        .as_ref()
        .map(|ft| category_name(ft.category).to_string())
        .or_else(|| extension.as_deref().map(category_from_extension))
        .unwrap_or_else(|| "unknown".to_string());
    let mut confidence = detected
        .as_ref()
        .map(|ft| format!("{:?}", ft.confidence).to_lowercase())
        .unwrap_or_else(|| "low".to_string());
    refine_type_from_extension(
        extension.as_deref(),
        &mut mime_type,
        &mut type_description,
        &mut category,
        &mut confidence,
    );
    let is_text = is_text_artifact(&category, extension.as_deref(), &header);
    let mut preview_bytes_read = None;
    let content_preview = if is_text {
        let preview_len = bounded_read_len(size, requested_preview_len);
        if preview_len > 0 {
            let preview = read_range_fully(source, 0, preview_len)?;
            preview_bytes_read = Some(preview.len());
            Some(String::from_utf8_lossy(&preview).to_string())
        } else {
            preview_bytes_read = Some(0);
            Some(String::new())
        }
    } else {
        None
    };

    let mut metadata = BTreeMap::new();
    metadata.insert("sourceId".to_string(), source_id.clone());
    metadata.insert("sizeBytes".to_string(), size.to_string());
    metadata.insert("header.bytesRead".to_string(), header.len().to_string());
    metadata.insert(
        "header.truncated".to_string(),
        (size > header.len() as u64).to_string(),
    );
    if let Some(extension) = &extension {
        metadata.insert("extension".to_string(), extension.clone());
    }
    if let Some(mime) = &mime_type {
        metadata.insert("mimeType".to_string(), mime.clone());
    }
    metadata.extend(header_metadata(&header, extension.as_deref(), &category));
    metadata.extend(source_indicator_metadata(&header));
    if let Some(preview) = &content_preview {
        let preview_bytes_read = preview_bytes_read.unwrap_or(preview.len());
        metadata.insert(
            "preview.bytesRead".to_string(),
            preview_bytes_read.to_string(),
        );
        metadata.insert(
            "preview.truncated".to_string(),
            (size > preview_bytes_read as u64).to_string(),
        );
    }
    truncate_metadata_values(&mut metadata);

    Ok(NormalizedArtifact {
        id: artifact_id(&source_ref, size),
        source_ref,
        source_id,
        name,
        extension,
        size,
        mime_type,
        type_description,
        category,
        confidence,
        is_text,
        content_preview,
        metadata,
    })
}

fn artifact_id(source_ref: &EvidenceSourceRef, size: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(artifact_id_source_bytes(source_ref));
    hasher.update(size.to_le_bytes());
    format!("{:x}", hasher.finalize())
}

fn artifact_id_source_bytes(source_ref: &EvidenceSourceRef) -> Vec<u8> {
    serde_json::to_vec(source_ref).unwrap_or_else(|_| source_ref.display_id().into_bytes())
}

fn bounded_read_len(source_size: u64, requested: usize) -> usize {
    source_size.min(requested as u64) as usize
}

fn truncate_metadata_values(metadata: &mut BTreeMap<String, String>) {
    for value in metadata.values_mut() {
        *value = truncate_chars(value, MAX_METADATA_VALUE_CHARS);
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = TRUNCATED_METADATA_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + TRUNCATED_METADATA_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(TRUNCATED_METADATA_SUFFIX);
    truncated
}

fn source_name(source_ref: &EvidenceSourceRef) -> String {
    let raw = match source_ref {
        EvidenceSourceRef::LocalFile { path } => path.as_str(),
        EvidenceSourceRef::ContainerEntry { entry_path, .. }
        | EvidenceSourceRef::NestedContainerEntry { entry_path, .. }
        | EvidenceSourceRef::VfsEntry { entry_path, .. } => entry_path.as_str(),
    };

    source_leaf_name(raw)
}

fn source_leaf_name(raw: &str) -> String {
    raw.rsplit(['/', '\\'])
        .find(|component| !component.is_empty())
        .unwrap_or(raw)
        .to_string()
}

fn extension_for_name(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim_start_matches('.').to_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn category_name(category: FileCategory) -> &'static str {
    match category {
        FileCategory::Image => "image",
        FileCategory::Document => "document",
        FileCategory::Archive => "archive",
        FileCategory::Executable => "executable",
        FileCategory::Audio => "audio",
        FileCategory::Video => "video",
        FileCategory::Database => "database",
        FileCategory::Forensic => "forensic",
        FileCategory::System => "system",
        FileCategory::Text => "text",
        FileCategory::Unknown => "unknown",
    }
}

fn category_from_extension(extension: &str) -> String {
    match extension {
        "txt" | "log" | "md" | "csv" | "tsv" => "text",
        "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "plist" | "reg" => "config",
        "pdf" | "doc" | "docx" | "rtf" | "odt" => "document",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tif" | "tiff" | "webp" => "image",
        "zip" | "7z" | "rar" | "tar" | "gz" | "bz2" | "xz" => "archive",
        "db" | "sqlite" | "sqlite3" => "database",
        "e01" | "l01" | "ad1" | "ufdr" | "ufdx" | "dd" | "raw" | "img" => "forensic",
        _ => "unknown",
    }
    .to_string()
}

fn refine_type_from_extension(
    extension: Option<&str>,
    mime_type: &mut Option<String>,
    type_description: &mut String,
    category: &mut String,
    confidence: &mut String,
) {
    let Some(extension) = extension else {
        return;
    };

    let refined = match extension {
        "docx" => Some((
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "Microsoft Word Document (OOXML)",
            "document",
        )),
        "xlsx" => Some((
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "Microsoft Excel Workbook (OOXML)",
            "spreadsheet",
        )),
        "pptx" => Some((
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "Microsoft PowerPoint Presentation (OOXML)",
            "presentation",
        )),
        "odt" => Some((
            "application/vnd.oasis.opendocument.text",
            "OpenDocument Text",
            "document",
        )),
        "ods" => Some((
            "application/vnd.oasis.opendocument.spreadsheet",
            "OpenDocument Spreadsheet",
            "spreadsheet",
        )),
        "odp" => Some((
            "application/vnd.oasis.opendocument.presentation",
            "OpenDocument Presentation",
            "presentation",
        )),
        "doc" => Some(("application/msword", "Microsoft Word Document", "document")),
        "xls" => Some((
            "application/vnd.ms-excel",
            "Microsoft Excel Workbook",
            "spreadsheet",
        )),
        "ppt" => Some((
            "application/vnd.ms-powerpoint",
            "Microsoft PowerPoint Presentation",
            "presentation",
        )),
        "msg" => Some(("application/vnd.ms-outlook", "Outlook Message", "email")),
        "eml" => Some(("message/rfc822", "Email Message", "email")),
        "pst" => Some((
            "application/vnd.ms-outlook-pst",
            "Outlook PST Mailbox",
            "email",
        )),
        "ost" => Some((
            "application/vnd.ms-outlook-ost",
            "Outlook OST Mailbox",
            "email",
        )),
        "txt" => Some(("text/plain", "Plain Text", "text")),
        "log" => Some(("text/plain", "Log File", "text")),
        "md" => Some(("text/markdown", "Markdown Document", "text")),
        "csv" => Some(("text/csv", "CSV Data", "text")),
        "tsv" => Some(("text/tab-separated-values", "TSV Data", "text")),
        "json" => Some(("application/json", "JSON Data", "config")),
        "xml" => Some(("application/xml", "XML Data", "config")),
        "plist" => Some(("application/x-plist", "Apple Property List", "config")),
        "yaml" | "yml" => Some(("application/yaml", "YAML Data", "config")),
        "toml" => Some(("application/toml", "TOML Data", "config")),
        "ini" | "cfg" | "conf" | "env" => Some(("text/plain", "Configuration File", "config")),
        "html" | "htm" => Some(("text/html", "HTML Document", "text")),
        "css" => Some(("text/css", "CSS Stylesheet", "text")),
        "js" => Some(("text/javascript", "JavaScript Source", "text")),
        "ts" => Some(("text/typescript", "TypeScript Source", "text")),
        "py" => Some(("text/x-python", "Python Source", "text")),
        "rs" => Some(("text/rust", "Rust Source", "text")),
        "sql" => Some(("application/sql", "SQL Script", "text")),
        "jpg" | "jpeg" => Some(("image/jpeg", "JPEG Image", "image")),
        "png" => Some(("image/png", "PNG Image", "image")),
        "gif" => Some(("image/gif", "GIF Image", "image")),
        "bmp" => Some(("image/bmp", "Bitmap Image", "image")),
        "tif" | "tiff" => Some(("image/tiff", "TIFF Image", "image")),
        "webp" => Some(("image/webp", "WebP Image", "image")),
        _ => None,
    };

    if let Some((mime, description, refined_category)) = refined {
        *mime_type = Some(mime.to_string());
        *type_description = description.to_string();
        *category = refined_category.to_string();
        if confidence == "low" {
            *confidence = "medium".to_string();
        }
    }
}

fn header_metadata(
    header: &[u8],
    extension: Option<&str>,
    category: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    if let Some(version) = pdf_version(header) {
        metadata.insert("pdf.version".to_string(), version);
    }

    metadata.extend(sqlite_metadata(header));
    metadata.extend(registry_hive_metadata(header));
    metadata.extend(email_metadata(header, extension, category));
    metadata.extend(plist_metadata(header, extension));

    if category == "image" || matches!(extension, Some("jpg" | "jpeg" | "png" | "gif" | "bmp")) {
        if let Some(dimensions) = image_dimensions(header) {
            metadata.insert("image.width".to_string(), dimensions.width.to_string());
            metadata.insert("image.height".to_string(), dimensions.height.to_string());
            metadata.insert(
                "image.dimensions".to_string(),
                format!("{}x{}", dimensions.width, dimensions.height),
            );
            metadata.insert("image.format".to_string(), dimensions.format.to_string());
        }

        for (key, value) in exif_metadata(header) {
            if matches!(key.as_str(), "image.width" | "image.height") && metadata.contains_key(&key)
            {
                continue;
            }
            metadata.insert(key, value);
        }
    }

    metadata
}

fn is_image_extension(extension: Option<&str>) -> bool {
    matches!(
        extension,
        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "tif" | "tiff" | "webp")
    )
}

fn is_structured_metadata_extension(extension: Option<&str>) -> bool {
    matches!(extension, Some("eml" | "mbox" | "plist"))
}

fn pdf_version(header: &[u8]) -> Option<String> {
    let prefix = b"%PDF-";
    if header.len() < prefix.len() + 3 || !header.starts_with(prefix) {
        return None;
    }

    let version = &header[prefix.len()..prefix.len() + 3];
    std::str::from_utf8(version)
        .ok()
        .filter(|value| {
            let bytes = value.as_bytes();
            bytes[0].is_ascii_digit() && bytes[1] == b'.' && bytes[2].is_ascii_digit()
        })
        .map(ToString::to_string)
}

fn plist_metadata(header: &[u8], extension: Option<&str>) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, extension) {
        return metadata;
    }

    let Some(format) = plist_format(header) else {
        return metadata;
    };
    metadata.insert("plist.format".to_string(), format.to_string());

    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };

    metadata.insert(
        "plist.rootType".to_string(),
        plist_value_type(&value).to_string(),
    );
    match &value {
        PlistValue::Dictionary(dictionary) => {
            metadata.insert(
                "plist.topLevelKeys".to_string(),
                dictionary.len().to_string(),
            );
            insert_plist_string_keys(
                &mut metadata,
                dictionary,
                &[
                    "CFBundleIdentifier",
                    "CFBundleName",
                    "CFBundleDisplayName",
                    "CFBundleVersion",
                    "CFBundleShortVersionString",
                    "Label",
                    "Program",
                    "UserName",
                ],
            );
            if let Some(arguments) = dictionary
                .get("ProgramArguments")
                .and_then(plist_string_array)
            {
                metadata.insert("plist.ProgramArguments".to_string(), arguments);
            }
            if let Some(value) = dictionary.get("RunAtLoad").and_then(plist_scalar_string) {
                metadata.insert("plist.RunAtLoad".to_string(), value);
            }
            if let Some(value) = dictionary.get("KeepAlive").and_then(plist_scalar_string) {
                metadata.insert("plist.KeepAlive".to_string(), value);
            }
        }
        PlistValue::Array(items) => {
            metadata.insert("plist.topLevelItems".to_string(), items.len().to_string());
        }
        _ => {}
    }

    metadata
}

fn looks_like_plist(header: &[u8], extension: Option<&str>) -> bool {
    if matches!(extension, Some("plist")) {
        return true;
    }
    plist_format(header).is_some()
}

fn plist_format(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(b"bplist") {
        return Some("binary");
    }

    let prefix = String::from_utf8_lossy(&header[..header.len().min(256)]).to_ascii_lowercase();
    if prefix.contains("<plist") || prefix.contains("property list") {
        Some("xml")
    } else {
        None
    }
}

fn plist_value_type(value: &PlistValue) -> &'static str {
    match value {
        PlistValue::Array(_) => "array",
        PlistValue::Dictionary(_) => "dictionary",
        PlistValue::Boolean(_) => "boolean",
        PlistValue::Data(_) => "data",
        PlistValue::Date(_) => "date",
        PlistValue::Real(_) => "real",
        PlistValue::Integer(_) => "integer",
        PlistValue::String(_) => "string",
        PlistValue::Uid(_) => "uid",
        _ => "unknown",
    }
}

fn insert_plist_string_keys(
    metadata: &mut BTreeMap<String, String>,
    dictionary: &plist::Dictionary,
    keys: &[&str],
) {
    for key in keys {
        if let Some(value) = dictionary.get(key).and_then(plist_scalar_string) {
            metadata.insert(format!("plist.{key}"), value);
        }
    }
}

fn plist_scalar_string(value: &PlistValue) -> Option<String> {
    match value {
        PlistValue::String(value) => Some(value.clone()),
        PlistValue::Boolean(value) => Some(value.to_string()),
        PlistValue::Integer(value) => Some(value.to_string()),
        PlistValue::Real(value) => Some(value.to_string()),
        _ => None,
    }
    .filter(|value| !value.trim().is_empty())
}

fn plist_string_array(value: &PlistValue) -> Option<String> {
    let PlistValue::Array(items) = value else {
        return None;
    };

    let values: Vec<String> = items
        .iter()
        .filter_map(plist_scalar_string)
        .take(8)
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values.join(" "))
    }
}

fn sqlite_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if header.len() < 100 || !header.starts_with(b"SQLite format 3\0") {
        return metadata;
    }

    let page_size_raw = u16::from_be_bytes([header[16], header[17]]);
    let page_size = if page_size_raw == 1 {
        65_536
    } else {
        u32::from(page_size_raw)
    };
    if page_size > 0 {
        metadata.insert("sqlite.pageSize".to_string(), page_size.to_string());
    }

    metadata.insert("sqlite.writeVersion".to_string(), header[18].to_string());
    metadata.insert("sqlite.readVersion".to_string(), header[19].to_string());
    metadata.insert(
        "sqlite.pageCount".to_string(),
        read_be_u32(header, 28).to_string(),
    );
    metadata.insert(
        "sqlite.schemaCookie".to_string(),
        read_be_u32(header, 40).to_string(),
    );
    metadata.insert(
        "sqlite.schemaFormat".to_string(),
        read_be_u32(header, 44).to_string(),
    );
    metadata.insert(
        "sqlite.textEncoding".to_string(),
        sqlite_text_encoding(read_be_u32(header, 56)),
    );

    metadata
}

fn read_be_u32(bytes: &[u8], offset: usize) -> u32 {
    checked_slice(bytes, offset, 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_be_bytes)
        .unwrap_or(0)
}

fn sqlite_text_encoding(value: u32) -> String {
    match value {
        1 => "UTF-8",
        2 => "UTF-16le",
        3 => "UTF-16be",
        _ => "unknown",
    }
    .to_string()
}

fn source_indicator_metadata(bytes: &[u8]) -> BTreeMap<String, String> {
    let indicators = extract_source_indicators(bytes, 0);
    let mut metadata = BTreeMap::new();
    if indicators.is_empty() {
        return metadata;
    }

    metadata.insert("indicators.count".to_string(), indicators.len().to_string());
    insert_indicator_group(&mut metadata, &indicators, "email", "emails", "emailCount");
    insert_indicator_group(&mut metadata, &indicators, "ipv4", "ipv4", "ipv4Count");
    insert_indicator_group(&mut metadata, &indicators, "url", "urls", "urlCount");
    insert_indicator_group(
        &mut metadata,
        &indicators,
        "windows_path",
        "windowsPaths",
        "windowsPathCount",
    );
    insert_indicator_group(
        &mut metadata,
        &indicators,
        "unc_path",
        "uncPaths",
        "uncPathCount",
    );

    metadata
}

fn insert_indicator_group(
    metadata: &mut BTreeMap<String, String>,
    indicators: &[SourceIndicator],
    indicator_type: &str,
    values_key: &str,
    count_key: &str,
) {
    let matching: Vec<&SourceIndicator> = indicators
        .iter()
        .filter(|indicator| indicator.indicator_type == indicator_type)
        .collect();
    if matching.is_empty() {
        return;
    }

    metadata.insert(
        format!("indicators.{count_key}"),
        matching.len().to_string(),
    );
    let mut values = Vec::new();
    for indicator in &matching {
        let value = indicator.value.as_str();
        if values.iter().any(|existing| existing == &value) {
            continue;
        }
        values.push(value);
        if values.len() >= 8 {
            break;
        }
    }
    metadata.insert(format!("indicators.{values_key}"), values.join(", "));
}

fn registry_hive_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if header.len() < 0x30 || !header.starts_with(b"regf") {
        return metadata;
    }

    let primary_sequence = read_le_u32(header, 0x04);
    let secondary_sequence = read_le_u32(header, 0x08);
    let major_version = read_le_u32(header, 0x14);
    let minor_version = read_le_u32(header, 0x18);
    let file_type = read_le_u32(header, 0x1c);
    let format = read_le_u32(header, 0x20);
    let root_cell_offset = read_le_u32(header, 0x24);
    let hive_bins_data_size = read_le_u32(header, 0x28);
    let clustering_factor = read_le_u32(header, 0x2c);

    metadata.insert(
        "registry.sequencePrimary".to_string(),
        primary_sequence.to_string(),
    );
    metadata.insert(
        "registry.sequenceSecondary".to_string(),
        secondary_sequence.to_string(),
    );
    metadata.insert(
        "registry.dirty".to_string(),
        (primary_sequence != secondary_sequence).to_string(),
    );
    metadata.insert(
        "registry.version".to_string(),
        format!("{major_version}.{minor_version}"),
    );
    metadata.insert("registry.fileType".to_string(), file_type.to_string());
    metadata.insert("registry.format".to_string(), format.to_string());
    metadata.insert(
        "registry.rootCellOffset".to_string(),
        root_cell_offset.to_string(),
    );
    metadata.insert(
        "registry.hiveBinsDataSize".to_string(),
        hive_bins_data_size.to_string(),
    );
    metadata.insert(
        "registry.clusteringFactor".to_string(),
        clustering_factor.to_string(),
    );

    if let Some(timestamp) = filetime_to_rfc3339(read_le_u64(header, 0x0c)) {
        metadata.insert("registry.lastWriteTime".to_string(), timestamp);
    }
    if let Some(path) = registry_header_path(header) {
        metadata.insert("registry.path".to_string(), path);
    }

    metadata
}

fn read_le_u32(bytes: &[u8], offset: usize) -> u32 {
    checked_slice(bytes, offset, 4)
        .and_then(|value| value.try_into().ok())
        .map(u32::from_le_bytes)
        .unwrap_or(0)
}

fn read_le_u64(bytes: &[u8], offset: usize) -> u64 {
    checked_slice(bytes, offset, 8)
        .and_then(|value| value.try_into().ok())
        .map(u64::from_le_bytes)
        .unwrap_or(0)
}

fn filetime_to_rfc3339(filetime: u64) -> Option<String> {
    if filetime == 0 {
        return None;
    }

    const FILETIME_UNIX_EPOCH_SECONDS: i64 = 11_644_473_600;
    let seconds = (filetime / 10_000_000) as i64 - FILETIME_UNIX_EPOCH_SECONDS;
    let nanos = ((filetime % 10_000_000) * 100) as u32;
    Utc.timestamp_opt(seconds, nanos)
        .single()
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

fn registry_header_path(header: &[u8]) -> Option<String> {
    let bytes = header.get(0x30..header.len().min(0x230))?;
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    if units.is_empty() {
        return None;
    }

    String::from_utf16(&units)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn email_metadata(
    header: &[u8],
    extension: Option<&str>,
    category: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if category != "email" && !matches!(extension, Some("eml" | "mbox")) {
        return metadata;
    }

    let header_text = String::from_utf8_lossy(header);
    let headers = parse_email_headers(&header_text);
    for (field, key) in [
        ("subject", "email.subject"),
        ("from", "email.from"),
        ("to", "email.to"),
        ("cc", "email.cc"),
        ("date", "email.date"),
        ("message-id", "email.messageId"),
    ] {
        if let Some(value) = headers.get(field) {
            metadata.insert(key.to_string(), value.clone());
        }
    }

    if !metadata.is_empty() {
        metadata.insert("email.headerCount".to_string(), headers.len().to_string());
    }
    metadata.extend(email_mime_metadata(&header_text));
    metadata
}

fn parse_email_headers(header_text: &str) -> BTreeMap<String, String> {
    let mut headers: BTreeMap<String, String> = BTreeMap::new();
    let mut current_key: Option<String> = None;

    for line in header_text.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        if trimmed_end.is_empty() {
            break;
        }

        if trimmed_end.starts_with(' ') || trimmed_end.starts_with('\t') {
            if let Some(key) = current_key.as_ref() {
                if let Some(value) = headers.get_mut(key) {
                    value.push(' ');
                    value.push_str(trimmed_end.trim());
                }
            }
            continue;
        }

        let Some((key, value)) = trimmed_end.split_once(':') else {
            current_key = None;
            continue;
        };
        let normalized_key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if !normalized_key.is_empty() && !value.is_empty() {
            headers.insert(normalized_key.clone(), value.to_string());
            current_key = Some(normalized_key);
        }
    }

    headers
}

fn email_mime_metadata(header_text: &str) -> BTreeMap<String, String> {
    let blocks = parse_mime_header_blocks(header_text);
    let mut metadata = BTreeMap::new();
    if blocks.is_empty() {
        return metadata;
    }

    if let Some(content_type) = blocks
        .first()
        .and_then(|headers| headers.get("content-type"))
        .map(|value| header_value_main(value))
    {
        metadata.insert("email.mimeType".to_string(), content_type);
    }

    let mut content_types = Vec::new();
    let mut attachment_names = Vec::new();
    let mut attachment_count = 0usize;
    let mut inline_attachment_count = 0usize;
    let mut has_html = false;
    let mut has_text = false;

    for headers in &blocks {
        let content_type = headers
            .get("content-type")
            .map(|value| header_value_main(value));
        if let Some(content_type) = &content_type {
            push_unique_limited(&mut content_types, content_type.clone(), 8);
            if content_type.eq_ignore_ascii_case("text/html") {
                has_html = true;
            }
            if content_type.eq_ignore_ascii_case("text/plain") {
                has_text = true;
            }
        }

        let disposition = headers.get("content-disposition");
        let is_attachment = disposition
            .map(|value| value.to_ascii_lowercase().contains("attachment"))
            .unwrap_or(false)
            || disposition
                .and_then(|value| header_param(value, "filename"))
                .is_some();
        let is_inline = disposition
            .map(|value| value.to_ascii_lowercase().contains("inline"))
            .unwrap_or(false);

        if is_attachment {
            attachment_count += 1;
        } else if is_inline {
            inline_attachment_count += 1;
        }

        if is_attachment || is_inline {
            if let Some(name) = disposition
                .and_then(|value| header_param(value, "filename"))
                .or_else(|| {
                    headers
                        .get("content-type")
                        .and_then(|value| header_param(value, "name"))
                })
            {
                push_unique_limited(&mut attachment_names, name, 8);
            }
        }
    }

    let part_count = blocks.len().saturating_sub(1);
    if part_count > 0 {
        metadata.insert("email.mimePartCount".to_string(), part_count.to_string());
    }
    if attachment_count > 0 {
        metadata.insert(
            "email.attachmentCount".to_string(),
            attachment_count.to_string(),
        );
    }
    if inline_attachment_count > 0 {
        metadata.insert(
            "email.inlineAttachmentCount".to_string(),
            inline_attachment_count.to_string(),
        );
    }
    if !attachment_names.is_empty() {
        metadata.insert(
            "email.attachmentNames".to_string(),
            attachment_names.join(", "),
        );
    }
    if !content_types.is_empty() {
        metadata.insert("email.contentTypes".to_string(), content_types.join(", "));
    }
    if has_html {
        metadata.insert("email.hasHtml".to_string(), "true".to_string());
    }
    if has_text {
        metadata.insert("email.hasText".to_string(), "true".to_string());
    }

    metadata
}

fn parse_mime_header_blocks(header_text: &str) -> Vec<BTreeMap<String, String>> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut collecting_headers = true;

    for line in header_text.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        let trimmed = trimmed_end.trim();

        if trimmed.is_empty() {
            push_mime_block(&mut blocks, &mut current);
            collecting_headers = false;
            continue;
        }

        if trimmed.starts_with("--") {
            current.clear();
            collecting_headers = true;
            continue;
        }

        if trimmed_end.starts_with(' ') || trimmed_end.starts_with('\t') {
            if collecting_headers && !current.is_empty() {
                current.push(trimmed_end.to_string());
            }
            continue;
        }

        if trimmed_end.contains(':') {
            collecting_headers = true;
            current.push(trimmed_end.to_string());
        } else if collecting_headers {
            current.clear();
            collecting_headers = false;
        }
    }

    push_mime_block(&mut blocks, &mut current);
    blocks
}

fn push_mime_block(blocks: &mut Vec<BTreeMap<String, String>>, current: &mut Vec<String>) {
    if current.is_empty() {
        return;
    }

    let joined = current.join("\n");
    let headers = parse_email_headers(&joined);
    if !headers.is_empty() {
        blocks.push(headers);
    }
    current.clear();
}

fn header_value_main(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or(value)
        .trim()
        .to_ascii_lowercase()
}

fn header_param(value: &str, key: &str) -> Option<String> {
    let target = key.to_ascii_lowercase();
    let target_star = format!("{target}*");
    for part in value.split(';').skip(1) {
        let Some((raw_key, raw_value)) = part.split_once('=') else {
            continue;
        };
        let normalized_key = raw_key.trim().to_ascii_lowercase();
        if normalized_key != target && normalized_key != target_star {
            continue;
        }

        let mut value = raw_value.trim().trim_matches('"').to_string();
        if normalized_key == target_star {
            if let Some((_, encoded)) = value.rsplit_once("''") {
                value = encoded.replace("%20", " ");
            }
        }
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn push_unique_limited(values: &mut Vec<String>, value: String, limit: usize) {
    if values.len() >= limit || values.iter().any(|existing| existing == &value) {
        return;
    }
    values.push(value);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageDimensions {
    width: u32,
    height: u32,
    format: &'static str,
}

impl ImageDimensions {
    fn new(width: u32, height: u32, format: &'static str) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let pixels = u64::from(width).checked_mul(u64::from(height))?;
        if pixels > MAX_IMAGE_METADATA_PIXELS {
            return None;
        }

        Some(Self {
            width,
            height,
            format,
        })
    }
}

fn image_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    png_dimensions(header)
        .or_else(|| gif_dimensions(header))
        .or_else(|| bmp_dimensions(header))
        .or_else(|| webp_dimensions(header))
        .or_else(|| tiff_dimensions(header))
        .or_else(|| jpeg_dimensions(header))
}

fn png_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    if header.len() < 24 || !header.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }

    ImageDimensions::new(
        u32::from_be_bytes(header[16..20].try_into().ok()?),
        u32::from_be_bytes(header[20..24].try_into().ok()?),
        "png",
    )
}

fn gif_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    if header.len() < 10 || !(header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a")) {
        return None;
    }

    ImageDimensions::new(
        u16::from_le_bytes(header[6..8].try_into().ok()?) as u32,
        u16::from_le_bytes(header[8..10].try_into().ok()?) as u32,
        "gif",
    )
}

fn bmp_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    if header.len() < 26 || !header.starts_with(b"BM") {
        return None;
    }

    let width = i32::from_le_bytes(header[18..22].try_into().ok()?);
    let height = i32::from_le_bytes(header[22..26].try_into().ok()?);
    if width == i32::MIN || height == i32::MIN {
        return None;
    }

    ImageDimensions::new(width.unsigned_abs(), height.unsigned_abs(), "bmp")
}

fn webp_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    if header.len() < 20 || !header.starts_with(b"RIFF") || header.get(8..12)? != b"WEBP" {
        return None;
    }

    match header.get(12..16)? {
        b"VP8X" if header.len() >= 30 => {
            let width = 1 + read_u24_le(header, 24)?;
            let height = 1 + read_u24_le(header, 27)?;
            ImageDimensions::new(width, height, "webp")
        }
        b"VP8L" if header.len() >= 25 && header[20] == 0x2f => {
            let b0 = header[21] as u32;
            let b1 = header[22] as u32;
            let b2 = header[23] as u32;
            let b3 = header[24] as u32;
            let width = 1 + (((b1 & 0x3f) << 8) | b0);
            let height = 1 + (((b3 & 0x0f) << 10) | (b2 << 2) | ((b1 & 0xc0) >> 6));
            ImageDimensions::new(width, height, "webp")
        }
        b"VP8 " if header.len() >= 30 && header.get(23..26)? == b"\x9d\x01\x2a" => {
            let width = u16::from_le_bytes(header[26..28].try_into().ok()?) as u32 & 0x3fff;
            let height = u16::from_le_bytes(header[28..30].try_into().ok()?) as u32 & 0x3fff;
            ImageDimensions::new(width, height, "webp")
        }
        _ => None,
    }
}

fn read_u24_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let value = checked_slice(bytes, offset, 3)?;
    Some((value[0] as u32) | ((value[1] as u32) << 8) | ((value[2] as u32) << 16))
}

fn tiff_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    let tiff = standalone_tiff(header)?;
    let byte_order = TiffByteOrder::from_tiff(tiff)?;
    if tiff.len() < 8 || byte_order.read_u16(tiff, 2) != Some(42) {
        return None;
    }

    let ifd0_offset = byte_order.read_u32(tiff, 4)? as usize;
    let ifd0 = read_tiff_ifd(tiff, ifd0_offset, byte_order)?;
    let width = tag_u32(tiff, byte_order, &ifd0, 0x0100)?;
    let height = tag_u32(tiff, byte_order, &ifd0, 0x0101)?;
    ImageDimensions::new(width, height, "tiff")
}

fn jpeg_dimensions(header: &[u8]) -> Option<ImageDimensions> {
    if header.len() < 4 || !header.starts_with(&[0xff, 0xd8]) {
        return None;
    }

    let mut index = 2;
    while checked_slice(header, index, 4).is_some() {
        while index < header.len() && header[index] != 0xff {
            index += 1;
        }
        while index < header.len() && header[index] == 0xff {
            index += 1;
        }
        if index >= header.len() {
            break;
        }

        let marker = header[index];
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if checked_slice(header, index, 2).is_none() {
            break;
        }

        let segment_len =
            u16::from_be_bytes(checked_slice(header, index, 2)?.try_into().ok()?) as usize;
        if segment_len < 2 {
            break;
        }

        if is_jpeg_sof_marker(marker) && checked_slice(header, index, 7).is_some() {
            let height = u16::from_be_bytes(
                checked_slice(header, index.checked_add(3)?, 2)?
                    .try_into()
                    .ok()?,
            ) as u32;
            let width = u16::from_be_bytes(
                checked_slice(header, index.checked_add(5)?, 2)?
                    .try_into()
                    .ok()?,
            ) as u32;
            return ImageDimensions::new(width, height, "jpeg");
        }

        index = index.checked_add(segment_len)?;
    }

    None
}

fn is_jpeg_sof_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn exif_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Some(tiff) = jpeg_exif_tiff(header).or_else(|| standalone_tiff(header)) else {
        return metadata;
    };
    let Some(byte_order) = TiffByteOrder::from_tiff(tiff) else {
        return metadata;
    };
    if tiff.len() < 8 || byte_order.read_u16(tiff, 2) != Some(42) {
        return metadata;
    }

    let Some(ifd0_offset) = byte_order.read_u32(tiff, 4).map(|value| value as usize) else {
        return metadata;
    };
    let Some(ifd0) = read_tiff_ifd(tiff, ifd0_offset, byte_order) else {
        return metadata;
    };

    insert_ascii_tag(&mut metadata, "exif.make", tiff, byte_order, &ifd0, 0x010f);
    insert_ascii_tag(&mut metadata, "exif.model", tiff, byte_order, &ifd0, 0x0110);
    insert_ascii_tag(
        &mut metadata,
        "exif.software",
        tiff,
        byte_order,
        &ifd0,
        0x0131,
    );
    insert_ascii_tag(
        &mut metadata,
        "exif.dateTime",
        tiff,
        byte_order,
        &ifd0,
        0x0132,
    );
    insert_short_tag(
        &mut metadata,
        "exif.orientation",
        tiff,
        byte_order,
        &ifd0,
        0x0112,
    );

    if let Some(exif_offset) = tag_u32(tiff, byte_order, &ifd0, 0x8769) {
        if let Some(exif_ifd) = read_tiff_ifd(tiff, exif_offset as usize, byte_order) {
            insert_ascii_tag(
                &mut metadata,
                "exif.dateTimeOriginal",
                tiff,
                byte_order,
                &exif_ifd,
                0x9003,
            );
            insert_ascii_tag(
                &mut metadata,
                "exif.dateTimeDigitized",
                tiff,
                byte_order,
                &exif_ifd,
                0x9004,
            );
            insert_ascii_tag(
                &mut metadata,
                "exif.lensModel",
                tiff,
                byte_order,
                &exif_ifd,
                0xa434,
            );
            insert_ascii_tag(
                &mut metadata,
                "exif.bodySerialNumber",
                tiff,
                byte_order,
                &exif_ifd,
                0xa431,
            );
            insert_exif_dimensions(&mut metadata, tiff, byte_order, &exif_ifd);
        }
    }

    if let Some(gps_offset) = tag_u32(tiff, byte_order, &ifd0, 0x8825) {
        if let Some(gps_ifd) = read_tiff_ifd(tiff, gps_offset as usize, byte_order) {
            insert_gps_metadata(&mut metadata, tiff, byte_order, &gps_ifd);
        }
    }

    metadata
}

fn jpeg_exif_tiff(header: &[u8]) -> Option<&[u8]> {
    if header.len() < 4 || !header.starts_with(&[0xff, 0xd8]) {
        return None;
    }

    let mut index = 2;
    while checked_slice(header, index, 4).is_some() {
        while index < header.len() && header[index] == 0xff {
            index += 1;
        }
        if index >= header.len() {
            break;
        }

        let marker = header[index];
        index += 1;
        if marker == 0xda || marker == 0xd9 {
            break;
        }
        if checked_slice(header, index, 2).is_none() {
            break;
        }
        let segment_len =
            u16::from_be_bytes(checked_slice(header, index, 2)?.try_into().ok()?) as usize;
        let Some(segment_end) = index.checked_add(segment_len) else {
            break;
        };
        if segment_len < 2 || segment_end > header.len() {
            break;
        }

        let segment = checked_slice(header, index.checked_add(2)?, segment_len - 2)?;
        if marker == 0xe1 && segment.starts_with(b"Exif\0\0") {
            return Some(&segment[6..]);
        }

        index = segment_end;
    }

    None
}

fn standalone_tiff(header: &[u8]) -> Option<&[u8]> {
    if header.starts_with(b"II*\0") || header.starts_with(b"MM\0*") {
        Some(header)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum TiffByteOrder {
    Little,
    Big,
}

impl TiffByteOrder {
    fn from_tiff(tiff: &[u8]) -> Option<Self> {
        match tiff.get(0..2)? {
            b"II" => Some(Self::Little),
            b"MM" => Some(Self::Big),
            _ => None,
        }
    }

    fn read_u16(self, bytes: &[u8], offset: usize) -> Option<u16> {
        let value = checked_slice(bytes, offset, 2)?;
        Some(match self {
            Self::Little => u16::from_le_bytes(value.try_into().ok()?),
            Self::Big => u16::from_be_bytes(value.try_into().ok()?),
        })
    }

    fn read_u32(self, bytes: &[u8], offset: usize) -> Option<u32> {
        let value = checked_slice(bytes, offset, 4)?;
        Some(match self {
            Self::Little => u32::from_le_bytes(value.try_into().ok()?),
            Self::Big => u32::from_be_bytes(value.try_into().ok()?),
        })
    }
}

fn checked_slice(bytes: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    let end = offset.checked_add(len)?;
    bytes.get(offset..end)
}

#[derive(Debug, Clone, Copy)]
struct TiffEntry {
    tag: u16,
    field_type: u16,
    count: u32,
    value_offset: usize,
    inline_value_offset: usize,
}

fn read_tiff_ifd(tiff: &[u8], offset: usize, byte_order: TiffByteOrder) -> Option<Vec<TiffEntry>> {
    let entry_count = byte_order.read_u16(tiff, offset)? as usize;
    if entry_count > MAX_TIFF_IFD_ENTRIES {
        return None;
    }
    let mut entries = Vec::with_capacity(entry_count);
    let mut entry_offset = offset.checked_add(2)?;

    for _ in 0..entry_count {
        let entry_end = entry_offset.checked_add(12)?;
        if entry_end > tiff.len() {
            return None;
        }
        let tag = byte_order.read_u16(tiff, entry_offset)?;
        let field_type = byte_order.read_u16(tiff, entry_offset + 2)?;
        let count = byte_order.read_u32(tiff, entry_offset + 4)?;
        let value_or_offset = byte_order.read_u32(tiff, entry_offset + 8)? as usize;
        let byte_len = tiff_value_byte_len(field_type, count).unwrap_or(0);
        let inline_value_offset = entry_offset.checked_add(8)?;
        let value_offset = if byte_len <= 4 {
            inline_value_offset
        } else {
            value_or_offset
        };

        entries.push(TiffEntry {
            tag,
            field_type,
            count,
            value_offset,
            inline_value_offset,
        });
        entry_offset = entry_end;
    }

    Some(entries)
}

fn tiff_value_byte_len(field_type: u16, count: u32) -> Option<usize> {
    let count = usize::try_from(count).ok()?;
    let unit_size = match field_type {
        1 | 2 | 7 => 1,
        3 => 2,
        4 | 9 => 4,
        5 | 10 => 8,
        _ => return None,
    };
    count.checked_mul(unit_size)
}

fn find_tag(entries: &[TiffEntry], tag: u16) -> Option<TiffEntry> {
    entries.iter().copied().find(|entry| entry.tag == tag)
}

fn tag_ascii(tiff: &[u8], entries: &[TiffEntry], tag: u16) -> Option<String> {
    let entry = find_tag(entries, tag)?;
    if entry.field_type != 2 || entry.count == 0 {
        return None;
    }
    let byte_len = usize::try_from(entry.count).ok()?;
    let value_end = entry.value_offset.checked_add(byte_len)?;
    let bytes = tiff.get(entry.value_offset..value_end)?;
    let value = bytes
        .split(|byte| *byte == 0)
        .next()
        .and_then(|text| std::str::from_utf8(text).ok())?
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn tag_u32(tiff: &[u8], byte_order: TiffByteOrder, entries: &[TiffEntry], tag: u16) -> Option<u32> {
    let entry = find_tag(entries, tag)?;
    if entry.count != 1 {
        return None;
    }
    match entry.field_type {
        3 => byte_order
            .read_u16(tiff, entry.inline_value_offset)
            .map(u32::from),
        4 => byte_order.read_u32(tiff, entry.value_offset),
        _ => None,
    }
}

fn tag_rational(
    tiff: &[u8],
    byte_order: TiffByteOrder,
    entries: &[TiffEntry],
    tag: u16,
) -> Option<Vec<f64>> {
    let entry = find_tag(entries, tag)?;
    if entry.field_type != 5 {
        return None;
    }

    let count = usize::try_from(entry.count).ok()?;
    if count > MAX_TIFF_RATIONAL_VALUES {
        return None;
    }

    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        let offset = entry.value_offset.checked_add(index.checked_mul(8)?)?;
        let numerator = byte_order.read_u32(tiff, offset)? as f64;
        let denominator = byte_order.read_u32(tiff, offset.checked_add(4)?)? as f64;
        if denominator == 0.0 {
            return None;
        }
        values.push(numerator / denominator);
    }
    Some(values)
}

fn insert_ascii_tag(
    metadata: &mut BTreeMap<String, String>,
    key: &str,
    tiff: &[u8],
    byte_order: TiffByteOrder,
    entries: &[TiffEntry],
    tag: u16,
) {
    let _ = byte_order;
    if let Some(value) = tag_ascii(tiff, entries, tag) {
        metadata.insert(key.to_string(), value);
    }
}

fn insert_short_tag(
    metadata: &mut BTreeMap<String, String>,
    key: &str,
    tiff: &[u8],
    byte_order: TiffByteOrder,
    entries: &[TiffEntry],
    tag: u16,
) {
    if let Some(value) = tag_u32(tiff, byte_order, entries, tag) {
        metadata.insert(key.to_string(), value.to_string());
    }
}

fn insert_exif_dimensions(
    metadata: &mut BTreeMap<String, String>,
    tiff: &[u8],
    byte_order: TiffByteOrder,
    entries: &[TiffEntry],
) {
    let Some(width) = tag_u32(tiff, byte_order, entries, 0xa002) else {
        return;
    };
    let Some(height) = tag_u32(tiff, byte_order, entries, 0xa003) else {
        return;
    };
    if ImageDimensions::new(width, height, "exif").is_some() {
        metadata.insert("image.width".to_string(), width.to_string());
        metadata.insert("image.height".to_string(), height.to_string());
    }
}

fn insert_gps_metadata(
    metadata: &mut BTreeMap<String, String>,
    tiff: &[u8],
    byte_order: TiffByteOrder,
    entries: &[TiffEntry],
) {
    let lat_ref = tag_ascii(tiff, entries, 0x0001);
    let lon_ref = tag_ascii(tiff, entries, 0x0003);
    let latitude =
        tag_rational(tiff, byte_order, entries, 0x0002).and_then(|parts| gps_decimal(&parts));
    let longitude =
        tag_rational(tiff, byte_order, entries, 0x0004).and_then(|parts| gps_decimal(&parts));

    if let (Some(mut latitude), Some(mut longitude)) = (latitude, longitude) {
        if !gps_ref_is_valid(lat_ref.as_deref(), "N", "S")
            || !gps_ref_is_valid(lon_ref.as_deref(), "E", "W")
        {
            return;
        }
        if lat_ref.as_deref() == Some("S") {
            latitude = -latitude;
        }
        if lon_ref.as_deref() == Some("W") {
            longitude = -longitude;
        }
        if !gps_coordinate_is_valid(latitude, longitude) {
            return;
        }
        metadata.insert("gps.latitude".to_string(), format!("{latitude:.6}"));
        metadata.insert("gps.longitude".to_string(), format!("{longitude:.6}"));
        if let Some(lat_ref) = lat_ref {
            metadata.insert("gps.latitudeRef".to_string(), lat_ref);
        }
        if let Some(lon_ref) = lon_ref {
            metadata.insert("gps.longitudeRef".to_string(), lon_ref);
        }
    }

    if let Some(altitude) =
        tag_rational(tiff, byte_order, entries, 0x0006).and_then(|values| values.first().copied())
    {
        metadata.insert("gps.altitude".to_string(), format!("{altitude:.3}"));
    }
}

fn gps_decimal(parts: &[f64]) -> Option<f64> {
    if parts.len() < 3 {
        return None;
    }
    let value = parts[0] + parts[1] / 60.0 + parts[2] / 3600.0;
    value.is_finite().then_some(value)
}

fn gps_ref_is_valid(value: Option<&str>, positive: &str, negative: &str) -> bool {
    value
        .map(|value| value == positive || value == negative)
        .unwrap_or(true)
}

fn gps_coordinate_is_valid(latitude: f64, longitude: f64) -> bool {
    latitude.is_finite()
        && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

fn is_text_artifact(category: &str, extension: Option<&str>, header: &[u8]) -> bool {
    matches!(category, "text" | "config")
        || matches!(
            extension,
            Some(
                "txt"
                    | "log"
                    | "md"
                    | "csv"
                    | "tsv"
                    | "json"
                    | "xml"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "ini"
                    | "html"
                    | "htm"
                    | "css"
                    | "js"
                    | "ts"
                    | "py"
                    | "rs"
                    | "sql"
            )
        )
        || (category == "unknown" && looks_like_text(header))
}

fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let printable = bytes
        .iter()
        .filter(|&&b| (0x20..=0x7e).contains(&b) || matches!(b, b'\n' | b'\r' | b'\t'))
        .count();
    printable * 100 / bytes.len() >= 85
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_source::{
        EvidenceByteSource, EvidenceSourceRef, EvidenceSourceResult, LocalFileByteSource,
    };
    use std::io::Write;

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

    fn write_temp_file(suffix: &str, bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::Builder::new().suffix(suffix).tempfile().unwrap();
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn source_leaf_name_accepts_windows_separators() {
        assert_eq!(
            source_leaf_name(r"Users\Alice\Documents\notes.txt"),
            "notes.txt"
        );
        assert_eq!(
            source_leaf_name("/Users/Alice/Documents/notes.txt"),
            "notes.txt"
        );
    }

    #[test]
    fn extract_artifact_uses_windows_leaf_name_for_container_entry() {
        let source = ChunkedByteSource {
            source_ref: EvidenceSourceRef::ContainerEntry {
                container_path: "/cases/logical.L01".to_string(),
                entry_path: r"Users\Alice\Documents\notes.txt".to_string(),
                container_type: "l01".to_string(),
            },
            data: b"case notes".to_vec(),
            max_chunk: 4,
        };

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.name, "notes.txt");
        assert_eq!(artifact.extension.as_deref(), Some("txt"));
        assert_eq!(artifact.category, "text");
        assert_eq!(
            artifact.metadata.get("extension").map(String::as_str),
            Some("txt")
        );
    }

    #[derive(Clone, Copy)]
    struct TestTiffEntry {
        tag: u16,
        field_type: u16,
        count: u32,
        value: u32,
    }

    fn push_u16_le(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32_le(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn append_ascii(bytes: &mut Vec<u8>, value: &str) -> (u32, u32) {
        let offset = bytes.len() as u32;
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
        (value.len() as u32 + 1, offset)
    }

    fn append_rationals(bytes: &mut Vec<u8>, values: &[(u32, u32)]) -> (u32, u32) {
        let offset = bytes.len() as u32;
        for (numerator, denominator) in values {
            push_u32_le(bytes, *numerator);
            push_u32_le(bytes, *denominator);
        }
        (values.len() as u32, offset)
    }

    fn write_ifd_at(bytes: &mut [u8], offset: usize, entries: &[TestTiffEntry]) {
        bytes[offset..offset + 2].copy_from_slice(&(entries.len() as u16).to_le_bytes());
        for (index, entry) in entries.iter().enumerate() {
            let entry_offset = offset + 2 + index * 12;
            bytes[entry_offset..entry_offset + 2].copy_from_slice(&entry.tag.to_le_bytes());
            bytes[entry_offset + 2..entry_offset + 4]
                .copy_from_slice(&entry.field_type.to_le_bytes());
            bytes[entry_offset + 4..entry_offset + 8].copy_from_slice(&entry.count.to_le_bytes());
            bytes[entry_offset + 8..entry_offset + 12].copy_from_slice(&entry.value.to_le_bytes());
        }
        let next_offset = offset + 2 + entries.len() * 12;
        bytes[next_offset..next_offset + 4].copy_from_slice(&0u32.to_le_bytes());
    }

    fn append_ifd(bytes: &mut Vec<u8>, entries: &[TestTiffEntry]) -> u32 {
        let offset = bytes.len();
        bytes.resize(offset + 2 + entries.len() * 12 + 4, 0);
        write_ifd_at(bytes, offset, entries);
        offset as u32
    }

    fn short_entry_value(value: u16) -> u32 {
        u32::from(value)
    }

    fn inline_ascii_value(value: &[u8]) -> u32 {
        let mut bytes = [0u8; 4];
        for (index, byte) in value.iter().take(4).enumerate() {
            bytes[index] = *byte;
        }
        u32::from_le_bytes(bytes)
    }

    fn make_jpeg_with_exif() -> Vec<u8> {
        let ifd0_offset = 8usize;
        let ifd0_entries = 6usize;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        push_u16_le(&mut tiff, 42);
        push_u32_le(&mut tiff, ifd0_offset as u32);
        tiff.resize(ifd0_offset + 2 + ifd0_entries * 12 + 4, 0);

        let (make_count, make_offset) = append_ascii(&mut tiff, "CORE");
        let (model_count, model_offset) = append_ascii(&mut tiff, "Camera 1");
        let (date_count, date_offset) = append_ascii(&mut tiff, "2026:02:16 10:03:00");

        let (original_count, original_offset) = append_ascii(&mut tiff, "2026:02:16 10:01:00");
        let (lens_count, lens_offset) = append_ascii(&mut tiff, "CORE Lens");
        let (serial_count, serial_offset) = append_ascii(&mut tiff, "SN123");
        let exif_offset = append_ifd(
            &mut tiff,
            &[
                TestTiffEntry {
                    tag: 0x9003,
                    field_type: 2,
                    count: original_count,
                    value: original_offset,
                },
                TestTiffEntry {
                    tag: 0xa434,
                    field_type: 2,
                    count: lens_count,
                    value: lens_offset,
                },
                TestTiffEntry {
                    tag: 0xa431,
                    field_type: 2,
                    count: serial_count,
                    value: serial_offset,
                },
                TestTiffEntry {
                    tag: 0xa002,
                    field_type: 4,
                    count: 1,
                    value: 4032,
                },
                TestTiffEntry {
                    tag: 0xa003,
                    field_type: 4,
                    count: 1,
                    value: 3024,
                },
            ],
        );

        let (lat_count, lat_offset) = append_rationals(&mut tiff, &[(37, 1), (46, 1), (2964, 100)]);
        let (lon_count, lon_offset) = append_rationals(&mut tiff, &[(122, 1), (25, 1), (984, 100)]);
        let gps_offset = append_ifd(
            &mut tiff,
            &[
                TestTiffEntry {
                    tag: 0x0001,
                    field_type: 2,
                    count: 2,
                    value: inline_ascii_value(b"N\0"),
                },
                TestTiffEntry {
                    tag: 0x0002,
                    field_type: 5,
                    count: lat_count,
                    value: lat_offset,
                },
                TestTiffEntry {
                    tag: 0x0003,
                    field_type: 2,
                    count: 2,
                    value: inline_ascii_value(b"W\0"),
                },
                TestTiffEntry {
                    tag: 0x0004,
                    field_type: 5,
                    count: lon_count,
                    value: lon_offset,
                },
            ],
        );

        write_ifd_at(
            &mut tiff,
            ifd0_offset,
            &[
                TestTiffEntry {
                    tag: 0x010f,
                    field_type: 2,
                    count: make_count,
                    value: make_offset,
                },
                TestTiffEntry {
                    tag: 0x0110,
                    field_type: 2,
                    count: model_count,
                    value: model_offset,
                },
                TestTiffEntry {
                    tag: 0x0132,
                    field_type: 2,
                    count: date_count,
                    value: date_offset,
                },
                TestTiffEntry {
                    tag: 0x0112,
                    field_type: 3,
                    count: 1,
                    value: short_entry_value(1),
                },
                TestTiffEntry {
                    tag: 0x8769,
                    field_type: 4,
                    count: 1,
                    value: exif_offset,
                },
                TestTiffEntry {
                    tag: 0x8825,
                    field_type: 4,
                    count: 1,
                    value: gps_offset,
                },
            ],
        );

        let mut app1 = Vec::from(&b"Exif\0\0"[..]);
        app1.extend_from_slice(&tiff);
        let segment_len = (app1.len() + 2) as u16;

        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend_from_slice(&segment_len.to_be_bytes());
        jpeg.extend_from_slice(&app1);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        jpeg
    }

    fn make_jpeg_with_sof_and_exif_dimensions(
        sof_width: u16,
        sof_height: u16,
        exif_width: u32,
        exif_height: u32,
    ) -> Vec<u8> {
        let ifd0_offset = 8usize;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        push_u16_le(&mut tiff, 42);
        push_u32_le(&mut tiff, ifd0_offset as u32);
        tiff.resize(ifd0_offset + 2 + 12 + 4, 0);
        let exif_offset = append_ifd(
            &mut tiff,
            &[
                TestTiffEntry {
                    tag: 0xa002,
                    field_type: 4,
                    count: 1,
                    value: exif_width,
                },
                TestTiffEntry {
                    tag: 0xa003,
                    field_type: 4,
                    count: 1,
                    value: exif_height,
                },
            ],
        );
        write_ifd_at(
            &mut tiff,
            ifd0_offset,
            &[TestTiffEntry {
                tag: 0x8769,
                field_type: 4,
                count: 1,
                value: exif_offset,
            }],
        );

        let mut app1 = Vec::from(&b"Exif\0\0"[..]);
        app1.extend_from_slice(&tiff);
        let segment_len = (app1.len() + 2) as u16;

        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend_from_slice(&segment_len.to_be_bytes());
        jpeg.extend_from_slice(&app1);
        jpeg.extend_from_slice(&[
            0xff, 0xc0, // SOF0
            0x00, 0x11, // segment length
            0x08, // precision
        ]);
        jpeg.extend_from_slice(&sof_height.to_be_bytes());
        jpeg.extend_from_slice(&sof_width.to_be_bytes());
        jpeg.extend_from_slice(&[0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00]);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        jpeg
    }

    #[test]
    fn extracts_text_artifact_preview() {
        let file = write_temp_file(".txt", b"hello artifact\nline 2");
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.extension.as_deref(), Some("txt"));
        assert_eq!(artifact.category, "text");
        assert!(artifact.is_text);
        assert_eq!(
            artifact.content_preview.as_deref(),
            Some("hello artifact\nline 2")
        );
        assert_eq!(artifact.size, 21);
    }

    #[test]
    fn extracts_text_artifact_preview_from_chunked_source() {
        let bytes = b"hello artifact\nline 2";
        let source = ChunkedByteSource::new("chunked.txt", bytes, 4);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.extension.as_deref(), Some("txt"));
        assert_eq!(artifact.category, "text");
        assert_eq!(
            artifact.content_preview.as_deref(),
            Some("hello artifact\nline 2")
        );
        assert_eq!(
            artifact.metadata.get("sizeBytes").map(String::as_str),
            Some("21")
        );
    }

    #[test]
    fn clamps_oversized_artifact_extraction_limits() {
        let bytes = vec![b'a'; MAX_HEADER_BYTES + 4096];
        let source = ChunkedByteSource::new("large.txt", &bytes, usize::MAX);

        let artifact = extract_normalized_artifact(
            &source,
            ArtifactExtractionOptions {
                header_bytes: usize::MAX,
                preview_bytes: usize::MAX,
            },
        )
        .unwrap();

        let expected_header_bytes = MAX_HEADER_BYTES.to_string();
        let expected_preview_bytes = MAX_PREVIEW_BYTES.to_string();
        assert_eq!(
            artifact
                .metadata
                .get("header.bytesRead")
                .map(String::as_str),
            Some(expected_header_bytes.as_str())
        );
        assert_eq!(
            artifact
                .metadata
                .get("header.truncated")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact
                .metadata
                .get("preview.bytesRead")
                .map(String::as_str),
            Some(expected_preview_bytes.as_str())
        );
        assert_eq!(
            artifact
                .metadata
                .get("preview.truncated")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(artifact.content_preview.unwrap().len(), MAX_PREVIEW_BYTES);
    }

    #[test]
    fn honors_smaller_artifact_preview_limit() {
        let bytes = b"abcdefghijklmnopqrstuvwxyz";
        let source = ChunkedByteSource::new("small.txt", bytes, usize::MAX);

        let artifact = extract_normalized_artifact(
            &source,
            ArtifactExtractionOptions {
                header_bytes: 4,
                preview_bytes: 8,
            },
        )
        .unwrap();

        assert_eq!(artifact.content_preview.as_deref(), Some("abcdefgh"));
        assert_eq!(
            artifact
                .metadata
                .get("preview.bytesRead")
                .map(String::as_str),
            Some("8")
        );
        assert_eq!(
            artifact
                .metadata
                .get("preview.truncated")
                .map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn extracts_source_indicators_into_artifact_metadata() {
        let bytes = b"Contact admin@example.com from 192.168.1.10 and visit https://example.com/login or C:\\Users\\Alice\\NTUSER.DAT";
        let file = write_temp_file(".txt", bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(
            artifact
                .metadata
                .get("indicators.emailCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("indicators.emails")
                .map(String::as_str),
            Some("admin@example.com")
        );
        assert_eq!(
            artifact.metadata.get("indicators.ipv4").map(String::as_str),
            Some("192.168.1.10")
        );
        assert_eq!(
            artifact.metadata.get("indicators.urls").map(String::as_str),
            Some("https://example.com/login")
        );
        assert_eq!(
            artifact
                .metadata
                .get("indicators.windowsPaths")
                .map(String::as_str),
            Some(r"C:\Users\Alice\NTUSER.DAT")
        );
    }

    #[test]
    fn artifact_indicator_metadata_counts_repeated_occurrences_but_dedupes_samples() {
        let bytes =
            b"admin@example.com admin@example.com https://example.test https://example.test";
        let source = ChunkedByteSource::new("repeated.txt", bytes, 11);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(
            artifact
                .metadata
                .get("indicators.emailCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("indicators.emails")
                .map(String::as_str),
            Some("admin@example.com")
        );
        assert_eq!(
            artifact
                .metadata
                .get("indicators.urlCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact.metadata.get("indicators.urls").map(String::as_str),
            Some("https://example.test")
        );
    }

    #[test]
    fn detects_pdf_magic_as_document() {
        let file = write_temp_file(".bin", b"%PDF-1.7\n%test");
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "document");
        assert_eq!(artifact.mime_type.as_deref(), Some("application/pdf"));
        assert_eq!(
            artifact.metadata.get("pdf.version").map(String::as_str),
            Some("1.7")
        );
        assert!(!artifact.is_text);
    }

    #[test]
    fn refines_ooxml_zip_artifact_from_extension() {
        let file = write_temp_file(".docx", b"PK\x03\x04\x14\x00\x00\x00");
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "document");
        assert_eq!(
            artifact.mime_type.as_deref(),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        );
        assert_eq!(artifact.type_description, "Microsoft Word Document (OOXML)");
    }

    #[test]
    fn refines_image_artifact_type_from_extension_when_magic_is_short() {
        let file = write_temp_file(".webp", b"RIFF");
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert_eq!(artifact.mime_type.as_deref(), Some("image/webp"));
        assert_eq!(artifact.type_description, "WebP Image");
        assert_eq!(artifact.confidence, "medium");
        assert!(!artifact.metadata.contains_key("image.width"));
        assert!(!artifact.metadata.contains_key("image.height"));
    }

    #[test]
    fn extracts_png_image_dimensions() {
        let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&640u32.to_be_bytes());
        bytes.extend_from_slice(&480u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        let file = write_temp_file(".png", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert_eq!(
            artifact.metadata.get("image.width").map(String::as_str),
            Some("640")
        );
        assert_eq!(
            artifact.metadata.get("image.height").map(String::as_str),
            Some("480")
        );
        assert_eq!(
            artifact
                .metadata
                .get("image.dimensions")
                .map(String::as_str),
            Some("640x480")
        );
    }

    #[test]
    fn image_dimension_parsers_reject_zero_dimensions() {
        let mut png = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(&480u32.to_be_bytes());
        png.extend_from_slice(&[8, 2, 0, 0, 0]);
        assert!(png_dimensions(&png).is_none());

        let mut gif = Vec::from(&b"GIF89a"[..]);
        gif.extend_from_slice(&0u16.to_le_bytes());
        gif.extend_from_slice(&10u16.to_le_bytes());
        assert!(gif_dimensions(&gif).is_none());

        let mut bmp = vec![0u8; 26];
        bmp[0..2].copy_from_slice(b"BM");
        bmp[18..22].copy_from_slice(&100i32.to_le_bytes());
        bmp[22..26].copy_from_slice(&0i32.to_le_bytes());
        assert!(bmp_dimensions(&bmp).is_none());

        let mut webp = vec![0u8; 30];
        webp[0..4].copy_from_slice(b"RIFF");
        webp[4..8].copy_from_slice(&22u32.to_le_bytes());
        webp[8..12].copy_from_slice(b"WEBP");
        webp[12..16].copy_from_slice(b"VP8 ");
        webp[23..26].copy_from_slice(b"\x9d\x01\x2a");
        webp[26..28].copy_from_slice(&0u16.to_le_bytes());
        webp[28..30].copy_from_slice(&10u16.to_le_bytes());
        assert!(webp_dimensions(&webp).is_none());

        let jpeg = [
            0xff, 0xd8, // SOI
            0xff, 0xc0, // SOF0
            0x00, 0x11, // segment length
            0x08, // precision
            0x00, 0x00, // height
            0x00, 0x10, // width
            0x03, 0x01, 0x11, 0x00,
        ];
        assert!(jpeg_dimensions(&jpeg).is_none());

        let ifd0_offset = 8usize;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        push_u16_le(&mut tiff, 42);
        push_u32_le(&mut tiff, ifd0_offset as u32);
        tiff.resize(ifd0_offset + 2 + 2 * 12 + 4, 0);
        write_ifd_at(
            &mut tiff,
            ifd0_offset,
            &[
                TestTiffEntry {
                    tag: 0x0100,
                    field_type: 4,
                    count: 1,
                    value: 1024,
                },
                TestTiffEntry {
                    tag: 0x0101,
                    field_type: 4,
                    count: 1,
                    value: 0,
                },
            ],
        );
        assert!(tiff_dimensions(&tiff).is_none());
    }

    #[test]
    fn image_dimension_parsers_reject_excessive_dimensions() {
        let mut png = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&100_001u32.to_be_bytes());
        png.extend_from_slice(&1_000u32.to_be_bytes());
        png.extend_from_slice(&[8, 2, 0, 0, 0]);
        assert!(png_dimensions(&png).is_none());

        let jpeg = [
            0xff, 0xd8, // SOI
            0xff, 0xc0, // SOF0
            0x00, 0x11, // segment length
            0x08, // precision
            0xff, 0xff, // height
            0xff, 0xff, // width
            0x03, 0x01, 0x11, 0x00,
        ];
        assert!(jpeg_dimensions(&jpeg).is_none());
    }

    #[test]
    fn artifact_extraction_omits_invalid_zero_image_dimensions() {
        let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&640u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        let file = write_temp_file(".png", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert!(!artifact.metadata.contains_key("image.width"));
        assert!(!artifact.metadata.contains_key("image.height"));
        assert!(!artifact.metadata.contains_key("image.dimensions"));
        assert!(!artifact.metadata.contains_key("image.format"));
    }

    #[test]
    fn extracts_tiff_image_dimensions() {
        let ifd0_offset = 8usize;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"II");
        push_u16_le(&mut bytes, 42);
        push_u32_le(&mut bytes, ifd0_offset as u32);
        bytes.resize(ifd0_offset + 2 + 2 * 12 + 4, 0);
        write_ifd_at(
            &mut bytes,
            ifd0_offset,
            &[
                TestTiffEntry {
                    tag: 0x0100,
                    field_type: 4,
                    count: 1,
                    value: 1024,
                },
                TestTiffEntry {
                    tag: 0x0101,
                    field_type: 4,
                    count: 1,
                    value: 768,
                },
            ],
        );
        let file = write_temp_file(".tiff", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert_eq!(
            artifact
                .metadata
                .get("image.dimensions")
                .map(String::as_str),
            Some("1024x768")
        );
        assert_eq!(
            artifact.metadata.get("image.format").map(String::as_str),
            Some("tiff")
        );
    }

    #[test]
    fn tiff_ifd_rejects_oversized_entry_count() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(MAX_TIFF_IFD_ENTRIES as u16 + 1).to_le_bytes());

        assert!(read_tiff_ifd(&bytes, 0, TiffByteOrder::Little).is_none());
    }

    #[test]
    fn primitive_metadata_readers_reject_overflowing_offsets() {
        let bytes = [0u8; 16];

        assert_eq!(read_be_u32(&bytes, usize::MAX), 0);
        assert_eq!(read_le_u32(&bytes, usize::MAX), 0);
        assert_eq!(read_le_u64(&bytes, usize::MAX), 0);
        assert!(read_u24_le(&bytes, usize::MAX).is_none());
        assert!(TiffByteOrder::Little.read_u16(&bytes, usize::MAX).is_none());
        assert!(TiffByteOrder::Little.read_u32(&bytes, usize::MAX).is_none());
    }

    #[test]
    fn jpeg_parsers_reject_truncated_segment_lengths() {
        let bytes = [
            0xff, 0xd8, // SOI
            0xff, 0xe1, // APP1
            0xff, 0xff, // impossible segment length for this buffer
            b'E', b'x', b'i', b'f', 0, 0,
        ];

        assert!(jpeg_dimensions(&bytes).is_none());
        assert!(jpeg_exif_tiff(&bytes).is_none());
    }

    #[test]
    fn tiff_tag_helpers_reject_oversized_or_overflowing_values() {
        let tiff = [0u8; 16];
        let huge_ascii = [TiffEntry {
            tag: 0x010f,
            field_type: 2,
            count: 8,
            value_offset: usize::MAX - 2,
            inline_value_offset: 0,
        }];
        assert!(tag_ascii(&tiff, &huge_ascii, 0x010f).is_none());

        let huge_rational = [TiffEntry {
            tag: 0x0002,
            field_type: 5,
            count: MAX_TIFF_RATIONAL_VALUES as u32 + 1,
            value_offset: 0,
            inline_value_offset: 0,
        }];
        assert!(tag_rational(&tiff, TiffByteOrder::Little, &huge_rational, 0x0002).is_none());

        let invalid_scalar_counts = [
            TiffEntry {
                tag: 0x0100,
                field_type: 4,
                count: 0,
                value_offset: 0,
                inline_value_offset: 0,
            },
            TiffEntry {
                tag: 0x0101,
                field_type: 4,
                count: 2,
                value_offset: 0,
                inline_value_offset: 0,
            },
        ];
        assert!(tag_u32(&tiff, TiffByteOrder::Little, &invalid_scalar_counts, 0x0100).is_none());
        assert!(tag_u32(&tiff, TiffByteOrder::Little, &invalid_scalar_counts, 0x0101).is_none());
    }

    #[test]
    fn tiff_dimensions_reject_malformed_scalar_counts() {
        let ifd0_offset = 8usize;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        push_u16_le(&mut tiff, 42);
        push_u32_le(&mut tiff, ifd0_offset as u32);
        tiff.resize(ifd0_offset + 2 + 2 * 12 + 4, 0);
        write_ifd_at(
            &mut tiff,
            ifd0_offset,
            &[
                TestTiffEntry {
                    tag: 0x0100,
                    field_type: 4,
                    count: 2,
                    value: 1024,
                },
                TestTiffEntry {
                    tag: 0x0101,
                    field_type: 4,
                    count: 1,
                    value: 768,
                },
            ],
        );

        assert!(tiff_dimensions(&tiff).is_none());
    }

    #[test]
    fn exif_dimensions_do_not_override_valid_header_dimensions() {
        let bytes = make_jpeg_with_sof_and_exif_dimensions(640, 480, 4032, 3024);
        let file = write_temp_file(".jpg", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(
            artifact.metadata.get("image.width").map(String::as_str),
            Some("640")
        );
        assert_eq!(
            artifact.metadata.get("image.height").map(String::as_str),
            Some("480")
        );
        assert_eq!(
            artifact
                .metadata
                .get("image.dimensions")
                .map(String::as_str),
            Some("640x480")
        );
    }

    #[test]
    fn exif_dimensions_reject_invalid_pairs() {
        let bytes = make_jpeg_with_sof_and_exif_dimensions(640, 480, 100_001, 1_000);
        let metadata = exif_metadata(&bytes);

        assert!(!metadata.contains_key("image.width"));
        assert!(!metadata.contains_key("image.height"));
    }

    #[test]
    fn extracts_webp_image_dimensions() {
        let mut bytes = vec![0u8; 30];
        bytes[0..4].copy_from_slice(b"RIFF");
        bytes[4..8].copy_from_slice(&22u32.to_le_bytes());
        bytes[8..12].copy_from_slice(b"WEBP");
        bytes[12..16].copy_from_slice(b"VP8X");
        bytes[16..20].copy_from_slice(&10u32.to_le_bytes());
        bytes[24..27].copy_from_slice(&319u32.to_le_bytes()[..3]);
        bytes[27..30].copy_from_slice(&239u32.to_le_bytes()[..3]);
        let file = write_temp_file(".webp", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert_eq!(
            artifact
                .metadata
                .get("image.dimensions")
                .map(String::as_str),
            Some("320x240")
        );
        assert_eq!(
            artifact.metadata.get("image.format").map(String::as_str),
            Some("webp")
        );
    }

    #[test]
    fn extracts_sqlite_header_metadata() {
        let mut bytes = vec![0u8; 100];
        bytes[..16].copy_from_slice(b"SQLite format 3\0");
        bytes[16..18].copy_from_slice(&4096u16.to_be_bytes());
        bytes[18] = 1;
        bytes[19] = 1;
        bytes[28..32].copy_from_slice(&12u32.to_be_bytes());
        bytes[40..44].copy_from_slice(&7u32.to_be_bytes());
        bytes[44..48].copy_from_slice(&4u32.to_be_bytes());
        bytes[56..60].copy_from_slice(&1u32.to_be_bytes());
        let file = write_temp_file(".sqlite", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "database");
        assert_eq!(
            artifact.metadata.get("sqlite.pageSize").map(String::as_str),
            Some("4096")
        );
        assert_eq!(
            artifact
                .metadata
                .get("sqlite.pageCount")
                .map(String::as_str),
            Some("12")
        );
        assert_eq!(
            artifact
                .metadata
                .get("sqlite.textEncoding")
                .map(String::as_str),
            Some("UTF-8")
        );
    }

    #[test]
    fn extracts_registry_hive_header_metadata() {
        let mut bytes = vec![0u8; 0x230];
        bytes[..4].copy_from_slice(b"regf");
        bytes[0x04..0x08].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x08..0x0c].copy_from_slice(&6u32.to_le_bytes());
        bytes[0x0c..0x14].copy_from_slice(&116_444_736_000_000_000u64.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x18..0x1c].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x1c..0x20].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x24..0x28].copy_from_slice(&32u32.to_le_bytes());
        bytes[0x28..0x2c].copy_from_slice(&4096u32.to_le_bytes());
        bytes[0x2c..0x30].copy_from_slice(&1u32.to_le_bytes());
        let path = r"\SystemRoot\System32\Config\SAM";
        for (index, unit) in path.encode_utf16().enumerate() {
            let offset = 0x30 + index * 2;
            bytes[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
        }
        let file = write_temp_file(".dat", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "system");
        assert_eq!(artifact.type_description, "Windows Registry Hive");
        assert_eq!(
            artifact.mime_type.as_deref(),
            Some("application/x-ms-registry")
        );
        assert_eq!(
            artifact.metadata.get("registry.dirty").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact
                .metadata
                .get("registry.version")
                .map(String::as_str),
            Some("1.5")
        );
        assert_eq!(
            artifact
                .metadata
                .get("registry.lastWriteTime")
                .map(String::as_str),
            Some("1970-01-01T00:00:00Z")
        );
        assert_eq!(
            artifact
                .metadata
                .get("registry.hiveBinsDataSize")
                .map(String::as_str),
            Some("4096")
        );
        assert_eq!(
            artifact.metadata.get("registry.path").map(String::as_str),
            Some(path)
        );
    }

    #[test]
    fn extracts_email_header_metadata() {
        let bytes = b"From: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\nSubject: Quarterly\r\n update\r\nDate: Mon, 16 Feb 2026 10:01:00 +0000\r\nMessage-ID: <msg-1@example.com>\r\n\r\nBody text";
        let file = write_temp_file(".eml", bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "email");
        assert_eq!(
            artifact.metadata.get("email.from").map(String::as_str),
            Some("Alice <alice@example.com>")
        );
        assert_eq!(
            artifact.metadata.get("email.subject").map(String::as_str),
            Some("Quarterly update")
        );
        assert_eq!(
            artifact.metadata.get("email.messageId").map(String::as_str),
            Some("<msg-1@example.com>")
        );
    }

    #[test]
    fn artifact_metadata_values_are_bounded() {
        let subject = "é".repeat(MAX_METADATA_VALUE_CHARS + 128);
        let bytes =
            format!("From: Alice <alice@example.com>\r\nSubject: {subject}\r\n\r\nBody text");
        let file = write_temp_file(".eml", bytes.as_bytes());
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();
        let subject = artifact
            .metadata
            .get("email.subject")
            .expect("subject metadata should exist");

        assert_eq!(subject.chars().count(), MAX_METADATA_VALUE_CHARS);
        assert!(subject.ends_with(TRUNCATED_METADATA_SUFFIX));
    }

    #[test]
    fn extracts_email_mime_attachment_metadata() {
        let bytes = br#"From: Alice <alice@example.com>
To: Bob <bob@example.com>
Subject: Attachment review
Content-Type: multipart/mixed; boundary="case-boundary"

--case-boundary
Content-Type: text/plain; charset=utf-8

Body text
--case-boundary
Content-Type: text/html; charset=utf-8

<p>Body text</p>
--case-boundary
Content-Type: application/pdf; name="invoice.pdf"
Content-Disposition: attachment; filename="invoice.pdf"
Content-Transfer-Encoding: base64

JVBERi0xLjQ=
--case-boundary--
"#;
        let file = write_temp_file(".eml", bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "email");
        assert_eq!(
            artifact.metadata.get("email.mimeType").map(String::as_str),
            Some("multipart/mixed")
        );
        assert_eq!(
            artifact
                .metadata
                .get("email.mimePartCount")
                .map(String::as_str),
            Some("3")
        );
        assert_eq!(
            artifact
                .metadata
                .get("email.attachmentCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("email.attachmentNames")
                .map(String::as_str),
            Some("invoice.pdf")
        );
        assert_eq!(
            artifact.metadata.get("email.hasHtml").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact
                .metadata
                .get("email.contentTypes")
                .map(String::as_str),
            Some("multipart/mixed, text/plain, text/html, application/pdf")
        );
    }

    #[test]
    fn extracts_plist_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.core.ffx.agent</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/core-ffx</string>
    <string>scan</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
</dict>
</plist>"#;
        let file = write_temp_file(".plist", bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "config");
        assert_eq!(artifact.type_description, "Apple Property List");
        assert_eq!(
            artifact.metadata.get("plist.format").map(String::as_str),
            Some("xml")
        );
        assert_eq!(
            artifact.metadata.get("plist.rootType").map(String::as_str),
            Some("dictionary")
        );
        assert_eq!(
            artifact
                .metadata
                .get("plist.topLevelKeys")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            artifact.metadata.get("plist.Label").map(String::as_str),
            Some("com.core.ffx.agent")
        );
        assert_eq!(
            artifact
                .metadata
                .get("plist.ProgramArguments")
                .map(String::as_str),
            Some("/usr/bin/core-ffx scan")
        );
        assert_eq!(
            artifact.metadata.get("plist.RunAtLoad").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn extracts_jpeg_exif_metadata() {
        let file = write_temp_file(".jpg", &make_jpeg_with_exif());
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "image");
        assert_eq!(
            artifact.metadata.get("exif.make").map(String::as_str),
            Some("CORE")
        );
        assert_eq!(
            artifact.metadata.get("exif.model").map(String::as_str),
            Some("Camera 1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("exif.dateTimeOriginal")
                .map(String::as_str),
            Some("2026:02:16 10:01:00")
        );
        assert_eq!(
            artifact.metadata.get("exif.lensModel").map(String::as_str),
            Some("CORE Lens")
        );
        assert_eq!(
            artifact.metadata.get("image.width").map(String::as_str),
            Some("4032")
        );
        assert_eq!(
            artifact.metadata.get("gps.latitude").map(String::as_str),
            Some("37.774900")
        );
        assert_eq!(
            artifact.metadata.get("gps.longitude").map(String::as_str),
            Some("-122.419400")
        );
    }

    #[test]
    fn gps_metadata_rejects_out_of_range_coordinates_and_refs() {
        let mut tiff = Vec::new();
        let (lat_ref_count, lat_ref_offset) = append_ascii(&mut tiff, "N");
        let (lat_count, lat_offset) = append_rationals(&mut tiff, &[(181, 1), (0, 1), (0, 1)]);
        let (lon_ref_count, lon_ref_offset) = append_ascii(&mut tiff, "W");
        let (lon_count, lon_offset) = append_rationals(&mut tiff, &[(1, 1), (0, 1), (0, 1)]);
        let entries = [
            TiffEntry {
                tag: 0x0001,
                field_type: 2,
                count: lat_ref_count,
                value_offset: lat_ref_offset as usize,
                inline_value_offset: 0,
            },
            TiffEntry {
                tag: 0x0002,
                field_type: 5,
                count: lat_count,
                value_offset: lat_offset as usize,
                inline_value_offset: 0,
            },
            TiffEntry {
                tag: 0x0003,
                field_type: 2,
                count: lon_ref_count,
                value_offset: lon_ref_offset as usize,
                inline_value_offset: 0,
            },
            TiffEntry {
                tag: 0x0004,
                field_type: 5,
                count: lon_count,
                value_offset: lon_offset as usize,
                inline_value_offset: 0,
            },
        ];
        let mut metadata = BTreeMap::new();
        insert_gps_metadata(&mut metadata, &tiff, TiffByteOrder::Little, &entries);
        assert!(!metadata.contains_key("gps.latitude"));

        assert!(!gps_ref_is_valid(Some("X"), "N", "S"));
    }

    #[test]
    fn artifact_id_is_stable_for_same_source_and_size() {
        let file = write_temp_file(".log", b"same");
        let source = LocalFileByteSource::new(file.path());

        let first =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();
        let second =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(first.id, second.id);
    }

    #[test]
    fn artifact_id_differs_for_different_sources_with_same_size() {
        let first = EvidenceSourceRef::LocalFile {
            path: "/case/source-a.log".to_string(),
        };
        let second = EvidenceSourceRef::LocalFile {
            path: "/case/source-b.log".to_string(),
        };

        assert_ne!(artifact_id(&first, 128), artifact_id(&second, 128));
    }

    #[test]
    fn artifact_id_source_bytes_use_serialized_source_ref() {
        let source_ref = EvidenceSourceRef::ContainerEntry {
            container_path: "/case/image.E01".to_string(),
            entry_path: "/Users/alice/file.txt".to_string(),
            container_type: "ewf".to_string(),
        };
        let expected = serde_json::to_vec(&source_ref).unwrap();

        assert_eq!(artifact_id_source_bytes(&source_ref), expected);
    }
}
