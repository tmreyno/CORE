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
use quick_xml::events::Event;
use quick_xml::Reader;
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
const MAX_PE_VERSION_INFO_FIELDS: usize = 32;
const MAX_PE_VERSION_INFO_VALUE_CHARS: usize = 512;
const MAX_PE_DRIVER_STRING_CHARS: usize = 240;
const MAX_REGISTRY_IDENTITY_STRINGS: usize = 512;
const UNIX_REGULAR_USER_MIN_UID: u32 = 1000;
const UNIX_REGULAR_USER_MAX_UID: u32 = 60000;
const TRUNCATED_METADATA_SUFFIX: &str = "... [truncated]";

const PE_VERSION_INFO_KEYS: &[&str] = &[
    "CompanyName",
    "FileDescription",
    "FileVersion",
    "InternalName",
    "OriginalFilename",
    "ProductName",
    "ProductVersion",
    "LegalCopyright",
    "LegalTrademarks",
    "PrivateBuild",
    "SpecialBuild",
    "Comments",
];

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

    let normalized_source_id = normalize_artifact_path(&source_id);
    let normalized_name = name.to_ascii_lowercase();
    let initial_header_len = if is_image_extension(extension.as_deref()) {
        requested_header_len.max(DEFAULT_IMAGE_METADATA_BYTES)
    } else if is_structured_metadata_extension(extension.as_deref())
        || is_windows_registry_hive_path(&normalized_source_id, &normalized_name)
    {
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
    let system_info = system_info_metadata(&source_id, &name, extension.as_deref(), &header);
    if !system_info.is_empty() {
        category = "systeminfo".to_string();
        type_description = system_info_type_description(&system_info);
        if confidence == "low" {
            confidence = "medium".to_string();
        }
        metadata.extend(system_info);
    }
    let activity = activity_metadata(&source_id, &header);
    if !activity.is_empty() {
        category = "activity".to_string();
        type_description = activity_type_description(&activity);
        if confidence == "low" {
            confidence = "medium".to_string();
        }
        metadata.extend(activity);
    }
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
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tif" | "tiff" | "webp" | "heic" | "heif"
        | "avif" => "image",
        "zip" | "7z" | "rar" | "tar" | "gz" | "bz2" | "xz" => "archive",
        "db" | "sqlite" | "sqlite3" => "database",
        "e01" | "l01" | "ad1" | "ufdr" | "ufdx" | "dd" | "raw" | "img" => "forensic",
        "sys" | "drv" | "ko" | "kext" => "system",
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
        "sys" => Some((
            "application/vnd.microsoft.portable-executable",
            "Windows System Driver",
            "system",
        )),
        "drv" => Some((
            "application/vnd.microsoft.portable-executable",
            "Windows Driver",
            "system",
        )),
        "ko" => Some((
            "application/x-linux-kernel-module",
            "Linux Kernel Module",
            "system",
        )),
        "kext" => Some((
            "application/x-macos-kernel-extension",
            "macOS Kernel Extension",
            "system",
        )),
        "jpg" | "jpeg" => Some(("image/jpeg", "JPEG Image", "image")),
        "png" => Some(("image/png", "PNG Image", "image")),
        "gif" => Some(("image/gif", "GIF Image", "image")),
        "bmp" => Some(("image/bmp", "Bitmap Image", "image")),
        "tif" | "tiff" => Some(("image/tiff", "TIFF Image", "image")),
        "webp" => Some(("image/webp", "WebP Image", "image")),
        "heic" => Some(("image/heic", "HEIC Image", "image")),
        "heif" => Some(("image/heif", "HEIF Image", "image")),
        "avif" => Some(("image/avif", "AVIF Image", "image")),
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
    metadata.extend(pe_driver_metadata(header, extension));

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
        Some(
            "jpg"
                | "jpeg"
                | "png"
                | "gif"
                | "bmp"
                | "tif"
                | "tiff"
                | "webp"
                | "heic"
                | "heif"
                | "avif",
        )
    )
}

fn is_structured_metadata_extension(extension: Option<&str>) -> bool {
    matches!(extension, Some("eml" | "mbox" | "plist" | "sys" | "drv"))
}

fn pe_driver_metadata(header: &[u8], extension: Option<&str>) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_pe(header) {
        return metadata;
    }

    metadata.insert("pe.format".to_string(), "portable-executable".to_string());
    let is_driver_extension = matches!(extension, Some("sys" | "drv"));
    let driver_indicators = pe_driver_indicators(header, is_driver_extension);
    if !driver_indicators.is_empty() {
        metadata.insert("pe.isDriver".to_string(), "true".to_string());
        metadata.insert(
            "pe.driverIndicators".to_string(),
            driver_indicators.join("; "),
        );
        metadata.insert(
            "pe.driverType".to_string(),
            pe_driver_type_from_indicators(&driver_indicators).to_string(),
        );
        insert_pe_driver_string_metadata(&mut metadata, header);
    }

    for (key, value) in pe_version_info_strings(header) {
        metadata.insert(format!("pe.version.{key}"), value);
    }

    metadata
}

fn looks_like_pe(header: &[u8]) -> bool {
    if header.len() < 0x40 || header.get(0..2) != Some(&b"MZ"[..]) {
        return false;
    }
    let Ok(pe_offset) = usize::try_from(read_le_u32(header, 0x3c)) else {
        return false;
    };
    if pe_offset == 0 {
        return false;
    }
    header.get(pe_offset..pe_offset.saturating_add(4)) == Some(&b"PE\0\0"[..])
}

fn pe_driver_indicators(header: &[u8], has_driver_extension: bool) -> Vec<String> {
    let text = String::from_utf8_lossy(header).to_ascii_lowercase();
    let mut indicators = Vec::new();
    if has_driver_extension {
        indicators.push("driver file extension".to_string());
    }

    for (needle, label) in [
        ("fltmgr.sys", "file-system filter driver APIs"),
        ("fltregisterfilter", "file-system filter driver APIs"),
        ("ntoskrnl.exe", "Windows kernel import library"),
        ("driverentry", "kernel DriverEntry export/string"),
        ("storport.sys", "storage driver APIs"),
        ("ndis.sys", "network driver APIs"),
        ("wdfldr.sys", "KMDF driver framework APIs"),
        ("wdfdrivercreate", "KMDF driver framework APIs"),
        ("usbport.sys", "USB driver APIs"),
        ("hidparse.sys", "HID driver APIs"),
        ("dxgkrnl.sys", "display driver APIs"),
    ] {
        if text.contains(needle) && !indicators.iter().any(|existing| existing == label) {
            indicators.push(label.to_string());
        }
    }

    indicators
}

fn pe_driver_type_from_indicators(indicators: &[String]) -> &'static str {
    if indicators
        .iter()
        .any(|indicator| indicator == "file-system filter driver APIs")
    {
        "File system minifilter driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "storage driver APIs")
    {
        "Storage driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "network driver APIs")
    {
        "Network driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "USB driver APIs")
    {
        "USB driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "HID driver APIs")
    {
        "HID driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "display driver APIs")
    {
        "Display driver"
    } else if indicators
        .iter()
        .any(|indicator| indicator == "KMDF driver framework APIs")
    {
        "Kernel-Mode Driver Framework driver"
    } else {
        "Windows kernel driver"
    }
}

fn insert_pe_driver_string_metadata(metadata: &mut BTreeMap<String, String>, header: &[u8]) {
    let strings = pe_embedded_strings(header);
    if strings.is_empty() {
        return;
    }

    let mut service_names = Vec::new();
    let mut device_names = Vec::new();
    let mut dos_device_names = Vec::new();
    let mut registry_paths = Vec::new();
    let mut pdb_paths = Vec::new();
    let mut urls = Vec::new();
    let mut guids = Vec::new();

    for value in strings {
        if let Some(service_name) = extract_windows_driver_service_name(&value) {
            push_limited_system_value(&mut service_names, &service_name);
        }
        if let Some(device_name) = extract_windows_object_name(&value, "\\device\\") {
            push_limited_system_value(&mut device_names, &device_name);
        }
        if let Some(dos_device_name) = extract_windows_object_name(&value, "\\dosdevices\\") {
            push_limited_system_value(&mut dos_device_names, &dos_device_name);
        }
        if let Some(registry_path) = extract_windows_driver_registry_path(&value) {
            push_limited_system_value(&mut registry_paths, &registry_path);
        }
        if let Some(pdb_path) = extract_windows_driver_pdb_path(&value) {
            push_limited_system_value(&mut pdb_paths, &pdb_path);
        }
        if let Some(url) = extract_embedded_url(&value) {
            push_limited_system_value(&mut urls, &url);
        }
        if let Some(guid) = extract_braced_guid(&value) {
            push_limited_system_value(&mut guids, &guid);
        }
    }

    insert_limited_system_values(metadata, "pe.driverServiceNames", &service_names);
    insert_limited_system_values(metadata, "pe.driverDeviceNames", &device_names);
    insert_limited_system_values(metadata, "pe.driverDosDeviceNames", &dos_device_names);
    insert_limited_system_values(metadata, "pe.driverRegistryPaths", &registry_paths);
    insert_limited_system_values(metadata, "pe.driverPdbPaths", &pdb_paths);
    insert_limited_system_values(metadata, "pe.driverUrls", &urls);
    insert_limited_system_values(metadata, "pe.driverGuids", &guids);
}

fn pe_embedded_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    collect_ascii_embedded_strings(data, &mut strings);
    collect_utf16le_embedded_strings(data, &mut strings);
    strings
}

fn collect_ascii_embedded_strings(data: &[u8], strings: &mut Vec<String>) {
    let mut start = None;
    for (index, byte) in data.iter().copied().enumerate() {
        if byte.is_ascii_graphic() || byte == b' ' {
            start.get_or_insert(index);
            continue;
        }
        if let Some(segment_start) = start.take() {
            push_embedded_string(strings, &data[segment_start..index]);
        }
    }
    if let Some(segment_start) = start {
        push_embedded_string(strings, &data[segment_start..]);
    }
}

fn push_embedded_string(strings: &mut Vec<String>, bytes: &[u8]) {
    if bytes.len() < 4 {
        return;
    }
    let value = String::from_utf8_lossy(bytes);
    let value = value.trim();
    if !looks_like_embedded_driver_string(value) {
        return;
    }
    push_limited_system_value(strings, &truncate_chars(value, MAX_PE_DRIVER_STRING_CHARS));
}

fn collect_utf16le_embedded_strings(data: &[u8], strings: &mut Vec<String>) {
    let mut units = Vec::new();
    let mut cursor = 0usize;
    while cursor + 1 < data.len() {
        let unit = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
        let ch = char::from_u32(unit as u32);
        if ch.is_some_and(|ch| ch.is_ascii_graphic() || ch == ' ') {
            units.push(unit);
        } else {
            push_utf16_embedded_string(strings, &mut units);
        }
        cursor += 2;
    }
    push_utf16_embedded_string(strings, &mut units);
}

fn push_utf16_embedded_string(strings: &mut Vec<String>, units: &mut Vec<u16>) {
    if units.len() < 4 {
        units.clear();
        return;
    }
    if let Ok(value) = String::from_utf16(units) {
        let value = value.trim();
        if looks_like_embedded_driver_string(value) {
            push_limited_system_value(strings, &truncate_chars(value, MAX_PE_DRIVER_STRING_CHARS));
        }
    }
    units.clear();
}

fn looks_like_embedded_driver_string(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("\\registry\\machine\\")
        || lower.contains("system\\currentcontrolset\\services\\")
        || lower.contains("system\\controlset001\\services\\")
        || lower.contains("\\device\\")
        || lower.contains("\\dosdevices\\")
        || lower.contains(".pdb")
        || lower.contains("http://")
        || lower.contains("https://")
        || extract_braced_guid(value).is_some()
}

fn extract_windows_driver_service_name(value: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    for marker in [
        "\\currentcontrolset\\services\\",
        "\\controlset001\\services\\",
        "\\controlset002\\services\\",
        "\\controlset003\\services\\",
    ] {
        if let Some(name) = extract_after_marker(&normalized, marker) {
            return Some(name);
        }
    }
    None
}

fn extract_windows_object_name(value: &str, marker: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    extract_after_marker(&normalized, marker)
}

fn extract_windows_driver_registry_path(value: &str) -> Option<String> {
    extract_segment_starting_with(value, "\\registry\\machine\\")
        .or_else(|| extract_segment_starting_with(value, "system\\currentcontrolset\\services\\"))
        .or_else(|| extract_segment_starting_with(value, "system\\controlset001\\services\\"))
        .map(|value| value.replace('/', "\\"))
}

fn extract_windows_driver_pdb_path(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let end = lower.find(".pdb")?.checked_add(4)?;
    let prefix = value.get(..end)?;
    let start = prefix
        .rfind(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\''))
        .map(|index| index + 1)
        .unwrap_or(0);
    let candidate = prefix.get(start..)?.trim_matches(['\0', '"', '\'']);
    if candidate.len() < 5 || !(candidate.contains('\\') || candidate.contains('/')) {
        return None;
    }
    Some(truncate_chars(candidate, MAX_PE_DRIVER_STRING_CHARS))
}

fn extract_embedded_url(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find("https://").or_else(|| lower.find("http://"))?;
    let raw = value.get(start..)?;
    let end = raw
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '<' | '>' | ')' | ']'))
        .unwrap_or(raw.len());
    let candidate = raw.get(..end)?.trim_end_matches(['.', ',', ';']);
    Some(truncate_chars(candidate, MAX_PE_DRIVER_STRING_CHARS))
        .filter(|value| value.contains("://"))
}

fn extract_braced_guid(value: &str) -> Option<String> {
    let start = value.find('{')?;
    let end = value.get(start..)?.find('}')?.checked_add(start + 1)?;
    let candidate = value.get(start..end)?;
    if is_braced_guid(candidate) {
        Some(candidate.to_ascii_uppercase())
    } else {
        None
    }
}

fn is_braced_guid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 38 || bytes.first() != Some(&b'{') || bytes.last() != Some(&b'}') {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate().skip(1).take(36) {
        match index {
            9 | 14 | 19 | 24 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

fn extract_segment_starting_with(value: &str, marker: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    let start = lower.find(marker)?;
    let raw = normalized.get(start..)?;
    let end = raw
        .find(|ch: char| ch == ';' || ch == '"' || ch == '\'' || ch.is_whitespace())
        .unwrap_or(raw.len());
    let candidate = raw.get(..end)?.trim_matches([':', '.', '\\']);
    if candidate.is_empty() {
        return None;
    }
    Some(truncate_chars(candidate, MAX_PE_DRIVER_STRING_CHARS))
}

fn extract_after_marker(value: &str, marker: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find(marker)?.checked_add(marker.len())?;
    let raw = value.get(start..)?;
    let end = raw
        .find(|ch: char| {
            ch == '\\' || ch == '/' || ch == ';' || ch == '"' || ch == '\'' || ch.is_whitespace()
        })
        .unwrap_or(raw.len());
    let candidate = raw.get(..end)?.trim_matches([':', '.']);
    if candidate.is_empty()
        || !candidate.chars().any(|ch| ch.is_ascii_alphanumeric())
        || !candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return None;
    }
    Some(truncate_chars(candidate, 120))
}

fn pe_version_info_strings(header: &[u8]) -> BTreeMap<String, String> {
    let mut version_info = BTreeMap::new();
    for key in PE_VERSION_INFO_KEYS {
        if version_info.len() >= MAX_PE_VERSION_INFO_FIELDS {
            break;
        }
        if let Some(value) = find_utf16le_version_value(header, key) {
            version_info.insert((*key).to_string(), value);
        }
    }
    version_info
}

fn find_utf16le_version_value(data: &[u8], key: &str) -> Option<String> {
    let key_utf16: Vec<u8> = key.encode_utf16().flat_map(u16::to_le_bytes).collect();
    data.windows(key_utf16.len())
        .position(|window| window == key_utf16.as_slice())
        .and_then(|index| {
            let value_start = index + key_utf16.len();
            let search_end = data.len().min(value_start.saturating_add(1024));
            let mut cursor = value_start;
            while cursor + 1 < search_end {
                if data[cursor] != 0 || data[cursor + 1] != 0 {
                    break;
                }
                cursor += 2;
            }
            read_utf16le_string(data, cursor, search_end)
        })
        .filter(|value| looks_like_version_resource_value(value))
}

fn read_utf16le_string(data: &[u8], start: usize, end: usize) -> Option<String> {
    let mut units = Vec::new();
    let mut cursor = start;
    while cursor + 1 < end {
        let unit = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        if units.len() >= MAX_PE_VERSION_INFO_VALUE_CHARS {
            break;
        }
        cursor += 2;
    }

    if units.is_empty() {
        return None;
    }
    String::from_utf16(&units)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn looks_like_version_resource_value(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.chars().any(|ch| ch.is_ascii_alphanumeric())
        && !trimmed.chars().any(char::is_control)
}

fn system_info_metadata(
    source_id: &str,
    name: &str,
    extension: Option<&str>,
    header: &[u8],
) -> BTreeMap<String, String> {
    let normalized_path = normalize_artifact_path(source_id);
    let normalized_name = name.to_ascii_lowercase();

    if is_linux_os_release_path(&normalized_path, &normalized_name) {
        return linux_os_release_metadata(header);
    }
    if is_linux_hostname_path(&normalized_path, &normalized_name) {
        return single_line_system_metadata(header, "system.hostname", "linux", "hostname");
    }
    if is_linux_machine_id_path(&normalized_path, &normalized_name) {
        return single_line_system_metadata(header, "system.machineId", "linux", "machine-id");
    }
    if is_linux_machine_info_path(&normalized_path, &normalized_name) {
        return linux_machine_info_metadata(header);
    }
    if is_linux_locale_path(&normalized_path) {
        return linux_locale_metadata(header);
    }
    if is_unix_timezone_path(&normalized_path) {
        return unix_timezone_metadata(&normalized_path, header);
    }
    if is_unix_mount_table_path(&normalized_path) {
        return unix_mount_table_metadata(header);
    }
    if is_linux_cpuinfo_path(&normalized_path, &normalized_name) {
        return linux_cpuinfo_metadata(header);
    }
    if is_linux_meminfo_path(&normalized_path, &normalized_name) {
        return linux_meminfo_metadata(header);
    }
    if is_linux_network_config_path(&normalized_path) {
        return linux_network_config_metadata(&normalized_path, header);
    }
    if is_unix_account_path(&normalized_path) {
        return unix_account_metadata(&normalized_path, header);
    }
    if let Some(metadata) = linux_dmi_metadata(&normalized_path, header) {
        return metadata;
    }
    if is_macos_hardware_identity_path(&normalized_path, &normalized_name, extension) {
        return macos_hardware_identity_metadata(header);
    }
    if is_macos_system_version_path(&normalized_path, &normalized_name, extension) {
        return macos_system_version_metadata(header);
    }
    if is_macos_preferences_identity_path(&normalized_path, &normalized_name, extension) {
        return macos_preferences_identity_metadata(header);
    }
    if is_macos_network_interfaces_path(&normalized_path, &normalized_name, extension) {
        return macos_network_interfaces_metadata(header);
    }
    if is_macos_wifi_preferences_path(&normalized_path, &normalized_name, extension) {
        return macos_wifi_preferences_metadata(header);
    }
    if is_macos_disk_management_path(&normalized_path, &normalized_name, extension) {
        return macos_disk_management_metadata(header);
    }
    if is_macos_firewall_preferences_path(&normalized_path, &normalized_name, extension) {
        return macos_firewall_preferences_metadata(header);
    }
    if is_windows_wifi_profile_path(&normalized_path, extension) {
        return windows_wifi_profile_metadata(header);
    }
    if let Some(metadata) = firewall_metadata(&normalized_path, header) {
        return metadata;
    }
    if let Some(metadata) = windows_setup_log_metadata(&normalized_path, header) {
        return metadata;
    }
    if is_macos_install_history_path(&normalized_path, &normalized_name, extension) {
        return macos_install_history_metadata(header);
    }
    if let Some(metadata) = windows_registry_system_info_metadata(&normalized_path, header) {
        return metadata;
    }

    BTreeMap::new()
}

fn normalize_artifact_path(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_matches('/')
        .to_ascii_lowercase()
}

fn is_linux_os_release_path(path: &str, name: &str) -> bool {
    name == "os-release"
        && (path.ends_with("etc/os-release") || path.ends_with("usr/lib/os-release"))
}

fn is_linux_hostname_path(path: &str, name: &str) -> bool {
    name == "hostname"
        && (path.ends_with("etc/hostname") || path.ends_with("proc/sys/kernel/hostname"))
}

fn is_linux_machine_id_path(path: &str, name: &str) -> bool {
    name == "machine-id"
        && (path.ends_with("etc/machine-id") || path.ends_with("var/lib/dbus/machine-id"))
}

fn is_linux_machine_info_path(path: &str, name: &str) -> bool {
    name == "machine-info" && path.ends_with("etc/machine-info")
}

fn is_linux_locale_path(path: &str) -> bool {
    path.ends_with("etc/default/locale")
}

fn is_unix_timezone_path(path: &str) -> bool {
    path.ends_with("etc/timezone")
        || path.ends_with("etc/localtime")
        || path.ends_with("private/etc/localtime")
        || path.ends_with("var/db/timezone/localtime")
}

fn is_unix_mount_table_path(path: &str) -> bool {
    path.ends_with("etc/fstab") || path.ends_with("etc/mtab")
}

fn is_linux_cpuinfo_path(path: &str, name: &str) -> bool {
    name == "cpuinfo" && path.ends_with("proc/cpuinfo")
}

fn is_linux_meminfo_path(path: &str, name: &str) -> bool {
    name == "meminfo" && path.ends_with("proc/meminfo")
}

fn is_linux_network_config_path(path: &str) -> bool {
    path.ends_with("etc/network/interfaces")
        || path.ends_with("etc/resolv.conf")
        || path.ends_with("etc/hosts")
        || path.contains("etc/sysconfig/network-scripts/ifcfg-")
        || path.contains("etc/networkmanager/system-connections/")
        || (path.contains("etc/netplan/") && (path.ends_with(".yaml") || path.ends_with(".yml")))
}

fn is_unix_account_path(path: &str) -> bool {
    path.ends_with("etc/passwd")
        || path.ends_with("private/etc/passwd")
        || path.ends_with("etc/group")
        || path.ends_with("private/etc/group")
        || path.ends_with("etc/shadow")
        || path.ends_with("private/etc/shadow")
        || path.ends_with("etc/gshadow")
        || path.ends_with("private/etc/gshadow")
}

fn is_macos_system_version_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    name.eq_ignore_ascii_case("systemversion.plist")
        && matches!(extension, Some("plist"))
        && (path.ends_with("system/library/coreservices/systemversion.plist")
            || path.ends_with("library/coreservices/systemversion.plist"))
}

fn is_macos_hardware_identity_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    matches!(extension, Some("plist"))
        && (name.eq_ignore_ascii_case("sphardwaredatatype.plist")
            || name.eq_ignore_ascii_case("ioplatformexpertdevice.plist")
            || path.ends_with("systemprofiler/sphardwaredatatype.plist")
            || path.ends_with("ioregistry/ioplatformexpertdevice.plist"))
}

fn is_macos_preferences_identity_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    name.eq_ignore_ascii_case("preferences.plist")
        && matches!(extension, Some("plist"))
        && (path.ends_with("library/preferences/systemconfiguration/preferences.plist")
            || path.ends_with("private/var/db/systemconfiguration/preferences.plist"))
}

fn is_macos_network_interfaces_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    name.eq_ignore_ascii_case("networkinterfaces.plist")
        && matches!(extension, Some("plist"))
        && (path.ends_with("library/preferences/systemconfiguration/networkinterfaces.plist")
            || path.ends_with("private/var/db/systemconfiguration/networkinterfaces.plist"))
}

fn is_macos_wifi_preferences_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    matches!(extension, Some("plist"))
        && (name.eq_ignore_ascii_case("com.apple.airport.preferences.plist")
            || name.eq_ignore_ascii_case("com.apple.wifi.known-networks.plist")
            || path.ends_with(
                "library/preferences/systemconfiguration/com.apple.airport.preferences.plist",
            )
            || path.ends_with("library/preferences/com.apple.wifi.known-networks.plist"))
}

fn is_macos_disk_management_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    matches!(extension, Some("plist"))
        && (name.eq_ignore_ascii_case("diskmanagement.plist")
            || path.ends_with("var/db/diskmanagement.plist")
            || path.ends_with("library/preferences/systemconfiguration/diskmanagement.plist"))
}

fn is_macos_firewall_preferences_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    matches!(extension, Some("plist"))
        && (name.eq_ignore_ascii_case("com.apple.alf.plist")
            || path.ends_with("library/preferences/com.apple.alf.plist"))
}

fn is_windows_wifi_profile_path(path: &str, extension: Option<&str>) -> bool {
    matches!(extension, Some("xml"))
        && path.contains("programdata/microsoft/wlansvc/profiles/interfaces/")
}

fn is_macos_install_history_path(path: &str, name: &str, extension: Option<&str>) -> bool {
    name.eq_ignore_ascii_case("installhistory.plist")
        && matches!(extension, Some("plist"))
        && path.ends_with("library/receipts/installhistory.plist")
}

fn is_windows_registry_hive_path(path: &str, name: &str) -> bool {
    matches!(name, "system" | "software" | "sam")
        && (path.ends_with("windows/system32/config/system")
            || path.ends_with("windows/system32/config/software")
            || path.ends_with("windows/system32/config/sam"))
}

fn windows_registry_system_info_metadata(
    path: &str,
    header: &[u8],
) -> Option<BTreeMap<String, String>> {
    if !header.starts_with(b"regf") {
        return None;
    }
    let hive = if path.ends_with("windows/system32/config/system") {
        "system"
    } else if path.ends_with("windows/system32/config/software") {
        "software"
    } else if path.ends_with("windows/system32/config/sam") {
        "sam"
    } else {
        return None;
    };

    let mut metadata = BTreeMap::new();
    metadata.insert("system.osFamily".to_string(), "windows".to_string());
    metadata.insert("system.infoType".to_string(), "registry-hive".to_string());
    metadata.insert("windows.registryHive".to_string(), hive.to_string());
    metadata.extend(windows_registry_identity_metadata(header, hive));
    Some(metadata)
}

fn windows_registry_identity_metadata(header: &[u8], hive: &str) -> BTreeMap<String, String> {
    let strings = registry_candidate_strings(header);
    let mut metadata = BTreeMap::new();
    if strings.is_empty() {
        return metadata;
    }

    metadata.insert(
        "windows.registryScannedStrings".to_string(),
        strings.len().to_string(),
    );

    match hive {
        "system" => {
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &[
                    "ComputerName",
                    "ActiveComputerName",
                    "NV Hostname",
                    "Hostname",
                ],
                "system.computerName",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["SystemManufacturer", "BaseBoardManufacturer"],
                "system.manufacturer",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["SystemProductName", "BaseBoardProduct"],
                "system.model",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["SystemVersion"],
                "system.productVersion",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &[
                    "SystemSerialNumber",
                    "BaseBoardSerialNumber",
                    "SerialNumber",
                ],
                "system.serialNumber",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["BIOSVendor"],
                "system.biosVendor",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["BIOSVersion"],
                "system.biosVersion",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["BIOSReleaseDate"],
                "system.biosDate",
            );
        }
        "software" => {
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["ProductName"],
                "os.release.name",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["DisplayVersion", "ReleaseId"],
                "os.release.version",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["CurrentBuildNumber", "CurrentBuild"],
                "os.release.buildId",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["EditionID"],
                "os.release.edition",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["InstallationType"],
                "os.release.installationType",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["ProductId"],
                "system.windowsProductId",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["MachineGuid"],
                "system.machineGuid",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["RegisteredOwner"],
                "system.registeredOwner",
            );
            insert_registry_value_after_labels(
                &mut metadata,
                &strings,
                &["RegisteredOrganization"],
                "system.registeredOrganization",
            );
        }
        _ => {}
    }

    metadata
}

fn insert_registry_value_after_labels(
    metadata: &mut BTreeMap<String, String>,
    strings: &[String],
    labels: &[&str],
    metadata_key: &str,
) {
    if metadata.contains_key(metadata_key) {
        return;
    }
    let Some(value) = registry_value_after_labels(strings, labels) else {
        return;
    };
    metadata.insert(metadata_key.to_string(), value);
}

fn registry_value_after_labels(strings: &[String], labels: &[&str]) -> Option<String> {
    for (index, value) in strings.iter().enumerate() {
        if !labels.iter().any(|label| value.eq_ignore_ascii_case(label)) {
            continue;
        }
        for candidate in strings.iter().skip(index + 1).take(8) {
            if labels
                .iter()
                .any(|label| candidate.eq_ignore_ascii_case(label))
            {
                continue;
            }
            if let Some(value) = clean_registry_identity_value(candidate) {
                return Some(value);
            }
        }
    }
    None
}

fn clean_registry_identity_value(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_matches('\0')
        .trim_matches(['"', '\'', '[', ']'])
        .trim();
    if value.len() < 2 || value.len() > 240 {
        return None;
    }
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "default" | "none" | "unknown" | "to be filled by o.e.m." | "system" | "software"
    ) || lower.starts_with("\\registry\\")
        || lower.starts_with("hkey_")
    {
        return None;
    }
    if !value.chars().any(|ch| ch.is_ascii_alphanumeric()) || value.chars().any(char::is_control) {
        return None;
    }
    Some(truncate_chars(value, 180))
}

fn registry_candidate_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    collect_utf16le_registry_strings(data, &mut strings);
    collect_ascii_registry_strings(data, &mut strings);
    strings
}

fn collect_utf16le_registry_strings(data: &[u8], strings: &mut Vec<String>) {
    for alignment in 0..=1 {
        let mut units = Vec::new();
        let mut cursor = alignment;
        while cursor + 1 < data.len() {
            let unit = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
            if registry_utf16_unit_is_printable(unit) {
                units.push(unit);
            } else {
                push_registry_utf16_string(strings, &units);
                units.clear();
            }
            if strings.len() >= MAX_REGISTRY_IDENTITY_STRINGS {
                return;
            }
            cursor += 2;
        }
        push_registry_utf16_string(strings, &units);
        if strings.len() >= MAX_REGISTRY_IDENTITY_STRINGS {
            return;
        }
    }
}

fn registry_utf16_unit_is_printable(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x20..=0x7e) || (0xa0..=0xd7ff).contains(&unit)
}

fn push_registry_utf16_string(strings: &mut Vec<String>, units: &[u16]) {
    if units.len() < 3 || strings.len() >= MAX_REGISTRY_IDENTITY_STRINGS {
        return;
    }
    let Ok(value) = String::from_utf16(units) else {
        return;
    };
    push_registry_candidate_string(strings, &value);
}

fn collect_ascii_registry_strings(data: &[u8], strings: &mut Vec<String>) {
    let mut start = None;
    for (index, byte) in data.iter().copied().enumerate() {
        if (0x20..=0x7e).contains(&byte) || byte == b'\t' {
            start.get_or_insert(index);
            continue;
        }
        if let Some(offset) = start.take() {
            push_registry_ascii_string(strings, &data[offset..index]);
            if strings.len() >= MAX_REGISTRY_IDENTITY_STRINGS {
                return;
            }
        }
    }
    if let Some(offset) = start {
        push_registry_ascii_string(strings, &data[offset..]);
    }
}

fn push_registry_ascii_string(strings: &mut Vec<String>, bytes: &[u8]) {
    if bytes.len() < 3 || strings.len() >= MAX_REGISTRY_IDENTITY_STRINGS {
        return;
    }
    let Ok(value) = std::str::from_utf8(bytes) else {
        return;
    };
    push_registry_candidate_string(strings, value);
}

fn push_registry_candidate_string(strings: &mut Vec<String>, value: &str) {
    let value = value.trim().trim_matches('\0').trim();
    if value.len() < 3
        || value.len() > 240
        || !value.chars().any(|ch| ch.is_ascii_alphanumeric())
        || value.chars().any(char::is_control)
    {
        return;
    }
    let normalized = truncate_chars(value, 180);
    if !strings.iter().any(|existing| existing == &normalized) {
        strings.push(normalized);
    }
}

fn windows_wifi_profile_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut reader = Reader::from_str(&text);
    reader.config_mut().trim_text(true);

    let mut metadata = BTreeMap::new();
    let mut element_stack: Vec<String> = Vec::new();
    let mut profile_names = Vec::new();
    let mut ssids = Vec::new();
    let mut auth_types = Vec::new();
    let mut encryption_types = Vec::new();
    let mut connection_modes = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                element_stack
                    .push(String::from_utf8_lossy(element.local_name().as_ref()).to_string());
            }
            Ok(Event::End(_)) => {
                element_stack.pop();
            }
            Ok(Event::Text(text_event)) => {
                let Ok(text) = text_event.unescape() else {
                    continue;
                };
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                let current = element_stack.last().map(String::as_str).unwrap_or("");
                match current {
                    "name" if element_stack.iter().any(|element| element == "SSID") => {
                        push_limited_system_value(&mut ssids, text);
                    }
                    "name" if element_stack.iter().any(|element| element == "WLANProfile") => {
                        push_limited_system_value(&mut profile_names, text);
                    }
                    "authentication" => push_limited_system_value(&mut auth_types, text),
                    "encryption" => push_limited_system_value(&mut encryption_types, text),
                    "connectionMode" => push_limited_system_value(&mut connection_modes, text),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    if profile_names.is_empty()
        && ssids.is_empty()
        && auth_types.is_empty()
        && encryption_types.is_empty()
        && connection_modes.is_empty()
    {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "windows".to_string());
    metadata.insert("system.infoType".to_string(), "wifi-profile".to_string());
    metadata.insert(
        "system.networkConfigType".to_string(),
        "windows-wlan-profile".to_string(),
    );
    insert_limited_system_values(&mut metadata, "system.connectionIds", &profile_names);
    insert_limited_system_values(&mut metadata, "system.wifiSsids", &ssids);
    insert_limited_system_values(&mut metadata, "system.wifiAuthTypes", &auth_types);
    insert_limited_system_values(
        &mut metadata,
        "system.wifiEncryptionTypes",
        &encryption_types,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.networkConnectionModes",
        &connection_modes,
    );
    metadata
}

fn firewall_metadata(path: &str, header: &[u8]) -> Option<BTreeMap<String, String>> {
    if path.ends_with("etc/sysconfig/iptables") || path.contains("etc/iptables/") {
        return Some(iptables_metadata(header));
    }
    if path.ends_with("windows/system32/logfiles/firewall/pfirewall.log") {
        return Some(windows_firewall_log_metadata(header));
    }
    None
}

fn iptables_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut metadata = BTreeMap::new();
    let mut tables = Vec::new();
    let mut chains = Vec::new();
    let mut policies = Vec::new();
    let mut rule_count = 0usize;

    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(table) = line.strip_prefix('*') {
            push_limited_system_value(&mut tables, table);
            continue;
        }
        if let Some(chain) = line.strip_prefix(':') {
            let mut parts = chain.split_whitespace();
            let Some(name) = parts.next() else {
                continue;
            };
            push_limited_system_value(&mut chains, name);
            if let Some(policy) = parts.next().filter(|policy| *policy != "-") {
                push_limited_system_value(&mut policies, &format!("{name}:{policy}"));
            }
            continue;
        }
        if line.starts_with("-A ") || line.starts_with("-I ") {
            rule_count = rule_count.saturating_add(1);
        }
    }

    metadata.insert("system.osFamily".to_string(), "linux".to_string());
    metadata.insert("system.infoType".to_string(), "firewall".to_string());
    metadata.insert(
        "system.firewallConfigType".to_string(),
        "iptables".to_string(),
    );
    if rule_count > 0 {
        metadata.insert(
            "system.firewallRuleCount".to_string(),
            rule_count.to_string(),
        );
    }
    insert_limited_system_values(&mut metadata, "system.firewallTables", &tables);
    insert_limited_system_values(&mut metadata, "system.firewallChains", &chains);
    insert_limited_system_values(&mut metadata, "system.firewallPolicies", &policies);
    metadata
}

fn windows_firewall_log_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut metadata = BTreeMap::new();
    let mut entries = 0usize;
    let mut allowed = 0usize;
    let mut dropped = 0usize;
    let mut protocols = Vec::new();

    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 5 {
            continue;
        }
        entries = entries.saturating_add(1);
        match fields.get(2).copied().unwrap_or_default() {
            "ALLOW" => allowed = allowed.saturating_add(1),
            "DROP" => dropped = dropped.saturating_add(1),
            _ => {}
        }
        if let Some(protocol) = fields.get(3) {
            push_limited_system_value(&mut protocols, protocol);
        }
    }

    metadata.insert("system.osFamily".to_string(), "windows".to_string());
    metadata.insert("system.infoType".to_string(), "firewall".to_string());
    metadata.insert(
        "system.firewallConfigType".to_string(),
        "windows-firewall-log".to_string(),
    );
    if entries > 0 {
        metadata.insert(
            "system.firewallLogEntryCount".to_string(),
            entries.to_string(),
        );
    }
    if allowed > 0 {
        metadata.insert(
            "system.firewallAllowedCount".to_string(),
            allowed.to_string(),
        );
    }
    if dropped > 0 {
        metadata.insert(
            "system.firewallDroppedCount".to_string(),
            dropped.to_string(),
        );
    }
    insert_limited_system_values(&mut metadata, "system.firewallProtocols", &protocols);
    metadata
}

#[derive(Default)]
struct WindowsSetupLogSummary {
    line_count: usize,
    device_install_count: usize,
    computer_names: Vec<String>,
    host_os_versions: Vec<String>,
    setup_build_versions: Vec<String>,
    manufacturers: Vec<String>,
    models: Vec<String>,
    bios_versions: Vec<String>,
    architectures: Vec<String>,
    device_hardware_ids: Vec<String>,
    device_descriptions: Vec<String>,
    driver_providers: Vec<String>,
    driver_versions: Vec<String>,
    inf_names: Vec<String>,
}

fn windows_setup_log_metadata(path: &str, header: &[u8]) -> Option<BTreeMap<String, String>> {
    let setup_type = if path.ends_with("windows/inf/setupapi.dev.log") {
        "setupapi-dev"
    } else if path.ends_with("windows/inf/setupapi.app.log") {
        "setupapi-app"
    } else if path.ends_with("windows/panther/setuperr.log")
        || path.contains("windows/system32/sysprep/panther/setuperr.log")
    {
        "setup-error"
    } else if path.ends_with("windows/panther/setupact.log")
        || path.contains("windows/system32/sysprep/panther/setupact.log")
    {
        "setup-action"
    } else {
        return None;
    };

    let text = String::from_utf8_lossy(header);
    let mut summary = WindowsSetupLogSummary::default();

    for line in text.lines().take(4096) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        summary.line_count = summary.line_count.saturating_add(1);
        collect_windows_setup_log_line(line, &mut summary);
    }

    Some(windows_setup_log_metadata_to_map(summary, setup_type))
}

fn collect_windows_setup_log_line(line: &str, summary: &mut WindowsSetupLogSummary) {
    collect_setup_marker_value(
        line,
        &[
            "computername",
            "computer name",
            "machine name",
            "target computer name",
        ],
        &mut summary.computer_names,
    );
    collect_setup_marker_value(
        line,
        &[
            "host os version",
            "source os version",
            "detected os version",
        ],
        &mut summary.host_os_versions,
    );
    collect_setup_marker_value(
        line,
        &["setup build version", "setup version", "build version"],
        &mut summary.setup_build_versions,
    );
    collect_setup_marker_value(
        line,
        &["system manufacturer", "manufacturer"],
        &mut summary.manufacturers,
    );
    collect_setup_marker_value(
        line,
        &[
            "system product name",
            "product name",
            "system model",
            "model",
        ],
        &mut summary.models,
    );
    collect_setup_marker_value(
        line,
        &["bios version", "firmware version"],
        &mut summary.bios_versions,
    );
    collect_setup_marker_value(
        line,
        &["architecture", "processor architecture"],
        &mut summary.architectures,
    );

    if let Some(hardware_id) = extract_setupapi_device_hardware_id(line) {
        summary.device_install_count = summary.device_install_count.saturating_add(1);
        push_limited_system_value(&mut summary.device_hardware_ids, &hardware_id);
    }
    collect_setupapi_value(line, "device description", &mut summary.device_descriptions);
    collect_setupapi_value(line, "provider", &mut summary.driver_providers);
    collect_setupapi_value(line, "driver version", &mut summary.driver_versions);
    collect_setupapi_value(line, "original inf name", &mut summary.inf_names);
    collect_setupapi_value(line, "inf name", &mut summary.inf_names);
}

fn windows_setup_log_metadata_to_map(
    summary: WindowsSetupLogSummary,
    setup_type: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("system.osFamily".to_string(), "windows".to_string());
    metadata.insert("system.infoType".to_string(), "setup-log".to_string());
    metadata.insert("system.setupLogType".to_string(), setup_type.to_string());
    if summary.line_count > 0 {
        metadata.insert(
            "system.setupLogLineCount".to_string(),
            summary.line_count.to_string(),
        );
    }
    if summary.device_install_count > 0 {
        metadata.insert(
            "system.setupDeviceInstallCount".to_string(),
            summary.device_install_count.to_string(),
        );
    }
    insert_limited_system_values(
        &mut metadata,
        "system.setupComputerNames",
        &summary.computer_names,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupHostOsVersions",
        &summary.host_os_versions,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupBuildVersions",
        &summary.setup_build_versions,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupManufacturers",
        &summary.manufacturers,
    );
    insert_limited_system_values(&mut metadata, "system.setupModels", &summary.models);
    insert_limited_system_values(
        &mut metadata,
        "system.setupBiosVersions",
        &summary.bios_versions,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupArchitectures",
        &summary.architectures,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupDeviceHardwareIds",
        &summary.device_hardware_ids,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupDeviceDescriptions",
        &summary.device_descriptions,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupDriverProviders",
        &summary.driver_providers,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.setupDriverVersions",
        &summary.driver_versions,
    );
    insert_limited_system_values(&mut metadata, "system.setupInfNames", &summary.inf_names);
    metadata
}

fn collect_setup_marker_value(line: &str, markers: &[&str], values: &mut Vec<String>) {
    for marker in markers {
        if let Some(value) = setup_log_value_after_marker(line, marker) {
            push_limited_system_value(values, &value);
            return;
        }
    }
}

fn collect_setupapi_value(line: &str, marker: &str, values: &mut Vec<String>) {
    if let Some(value) = setup_log_value_after_marker(line, marker) {
        push_limited_system_value(values, &value);
    }
}

fn setup_log_value_after_marker(line: &str, marker: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let marker_start = lower.find(marker)?;
    let after_marker = line.get(marker_start + marker.len()..)?;
    let separator_index = after_marker.find([':', '=', '-'])?;
    let value = after_marker
        .get(separator_index + 1..)?
        .trim()
        .trim_matches(['"', '\'', '[', ']']);
    if setup_log_value_is_useful(value) {
        Some(value.to_string())
    } else {
        None
    }
}

fn setup_log_value_is_useful(value: &str) -> bool {
    !value.is_empty()
        && value.chars().any(|ch| ch.is_ascii_alphanumeric())
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "none" | "unknown" | "n/a"
        )
}

fn extract_setupapi_device_hardware_id(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("device install") {
        return None;
    }
    let (_, raw) = line.split_once(" - ")?;
    let candidate = raw.trim().trim_end_matches(']').trim();
    if candidate.contains('\\') || candidate.contains("VEN_") || candidate.contains("VID_") {
        Some(truncate_chars(candidate, 180))
    } else {
        None
    }
}

fn linux_os_release_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let text = String::from_utf8_lossy(header);
    let values = parse_shell_key_values(&text);
    if values.is_empty() {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "linux".to_string());
    metadata.insert("system.infoType".to_string(), "os-release".to_string());
    insert_key_value_alias(&mut metadata, &values, "PRETTY_NAME", "os.release.name");
    insert_key_value_alias(&mut metadata, &values, "NAME", "os.release.distribution");
    insert_key_value_alias(&mut metadata, &values, "ID", "os.release.id");
    insert_key_value_alias(&mut metadata, &values, "VERSION", "os.release.version");
    insert_key_value_alias(&mut metadata, &values, "VERSION_ID", "os.release.versionId");
    insert_key_value_alias(&mut metadata, &values, "VARIANT", "os.release.variant");
    insert_key_value_alias(&mut metadata, &values, "BUILD_ID", "os.release.buildId");
    insert_key_value_alias(&mut metadata, &values, "HOME_URL", "os.release.homeUrl");
    metadata
}

fn linux_machine_info_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let values = parse_shell_key_values(&text);
    let mut metadata = BTreeMap::new();

    insert_key_value_alias(
        &mut metadata,
        &values,
        "PRETTY_HOSTNAME",
        "system.prettyHostname",
    );
    insert_key_value_alias(&mut metadata, &values, "ICON_NAME", "system.iconName");
    insert_key_value_alias(&mut metadata, &values, "CHASSIS", "system.chassis");
    insert_key_value_alias(&mut metadata, &values, "DEPLOYMENT", "system.deployment");
    insert_key_value_alias(&mut metadata, &values, "LOCATION", "system.location");

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "linux".to_string());
        metadata.insert("system.infoType".to_string(), "machine-info".to_string());
    }
    metadata
}

fn linux_locale_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let values = parse_shell_key_values(&text);
    let mut metadata = BTreeMap::new();

    insert_key_value_alias(&mut metadata, &values, "LANG", "system.locale");
    insert_key_value_alias(&mut metadata, &values, "LANGUAGE", "system.language");
    insert_key_value_alias(&mut metadata, &values, "LC_TIME", "system.localeTime");
    insert_key_value_alias(&mut metadata, &values, "LC_NUMERIC", "system.localeNumeric");

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "linux".to_string());
        metadata.insert("system.infoType".to_string(), "locale".to_string());
    }
    metadata
}

fn unix_timezone_metadata(path: &str, header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();

    if header.starts_with(b"TZif") {
        metadata.insert("system.timeZoneFormat".to_string(), "TZif".to_string());
        if let Some(version) = header.get(4).copied().filter(|byte| *byte != 0) {
            metadata.insert(
                "system.timeZoneFileVersion".to_string(),
                (version as char).to_string(),
            );
        }
        if let Some(rule) = tzif_posix_rule(header) {
            metadata.insert("system.timeZoneRule".to_string(), rule);
        }
    } else {
        let text = String::from_utf8_lossy(header);
        let value = text.trim();
        let zone = if path.ends_with("etc/timezone") {
            (!value.is_empty()).then(|| value.to_string())
        } else {
            value
                .split("/zoneinfo/")
                .nth(1)
                .or_else(|| value.strip_prefix("zoneinfo/"))
                .map(|value| value.trim_matches('/').to_string())
        };
        if let Some(zone) = zone.filter(|value| !value.is_empty()) {
            metadata.insert("system.timeZone".to_string(), truncate_chars(&zone, 180));
        }
    }

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "unix".to_string());
        metadata.insert("system.infoType".to_string(), "timezone".to_string());
    }
    metadata
}

fn tzif_posix_rule(data: &[u8]) -> Option<String> {
    data.get(4)
        .copied()
        .filter(|byte| matches!(*byte, b'2' | b'3' | b'4'))?;
    let last_newline = data.iter().rposition(|byte| *byte == b'\n')?;
    let previous_newline = data[..last_newline]
        .iter()
        .rposition(|byte| *byte == b'\n')?;
    let value = std::str::from_utf8(&data[previous_newline + 1..last_newline])
        .ok()?
        .trim();
    (!value.is_empty() && value.chars().all(|ch| ch.is_ascii_graphic()))
        .then(|| truncate_chars(value, 180))
}

fn unix_mount_table_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut descriptions = Vec::new();
    let mut root_device = None;

    for raw_line in text.lines().take(2048) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        let device = decode_mount_field(fields[0]);
        let mount_point = decode_mount_field(fields[1]);
        let fs_type = fields[2];
        let options = fields.get(3).copied().unwrap_or("-");
        if mount_point == "/" && root_device.is_none() {
            root_device = Some(device.clone());
        }
        push_limited_system_value(
            &mut descriptions,
            &format!("{device} on {mount_point} ({fs_type}, {options})"),
        );
    }

    let mut metadata = BTreeMap::new();
    if let Some(root_device) = root_device {
        metadata.insert("system.rootDevice".to_string(), root_device);
    }
    if !descriptions.is_empty() {
        metadata.insert(
            "system.mountCount".to_string(),
            descriptions.len().to_string(),
        );
        insert_limited_system_values(&mut metadata, "system.mounts", &descriptions);
    }
    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "unix".to_string());
        metadata.insert("system.infoType".to_string(), "mount-table".to_string());
    }
    metadata
}

fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[derive(Default)]
struct LinuxCpuInfoSummary {
    logical_processors: usize,
    model_names: Vec<String>,
    vendors: Vec<String>,
    core_counts: Vec<String>,
    architectures: Vec<String>,
    hardware: Vec<String>,
    features: Vec<String>,
}

fn linux_cpuinfo_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut summary = LinuxCpuInfoSummary::default();

    for line in text.lines().take(2048) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            continue;
        }

        match key {
            "processor" => {
                summary.logical_processors = summary.logical_processors.saturating_add(1)
            }
            "model name" | "Processor" => {
                push_limited_system_value(&mut summary.model_names, value)
            }
            "vendor_id" | "CPU implementer" => {
                push_limited_system_value(&mut summary.vendors, value)
            }
            "cpu cores" => push_limited_system_value(&mut summary.core_counts, value),
            "Architecture" | "CPU architecture" => {
                push_limited_system_value(&mut summary.architectures, value)
            }
            "Hardware" => push_limited_system_value(&mut summary.hardware, value),
            "flags" | "Features" => {
                for feature in value.split_whitespace() {
                    push_limited_system_value(&mut summary.features, feature);
                }
            }
            _ => {}
        }
    }

    let mut metadata = BTreeMap::new();
    if summary.logical_processors > 0 {
        metadata.insert(
            "system.cpuLogicalProcessorCount".to_string(),
            summary.logical_processors.to_string(),
        );
    }
    insert_limited_system_values(&mut metadata, "system.cpuModels", &summary.model_names);
    insert_limited_system_values(&mut metadata, "system.cpuVendors", &summary.vendors);
    insert_limited_system_values(&mut metadata, "system.cpuCoreCounts", &summary.core_counts);
    insert_limited_system_values(
        &mut metadata,
        "system.cpuArchitectures",
        &summary.architectures,
    );
    insert_limited_system_values(&mut metadata, "system.cpuHardware", &summary.hardware);
    insert_limited_system_values(&mut metadata, "system.cpuFeatures", &summary.features);

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "linux".to_string());
        metadata.insert("system.infoType".to_string(), "cpuinfo".to_string());
    }
    metadata
}

fn linux_meminfo_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut metadata = BTreeMap::new();

    for line in text.lines().take(256) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim() != "MemTotal" {
            continue;
        }
        if let Some(kib) = value
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<u64>().ok())
        {
            metadata.insert("system.memoryTotalKiB".to_string(), kib.to_string());
            metadata.insert(
                "system.memoryTotalBytes".to_string(),
                kib.saturating_mul(1024).to_string(),
            );
        }
        break;
    }

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "linux".to_string());
        metadata.insert("system.infoType".to_string(), "meminfo".to_string());
    }
    metadata
}

#[derive(Default)]
struct LinuxNetworkSummary {
    interfaces: Vec<String>,
    addresses: Vec<String>,
    gateways: Vec<String>,
    dns_servers: Vec<String>,
    methods: Vec<String>,
    search_domains: Vec<String>,
    host_aliases: Vec<String>,
    connection_ids: Vec<String>,
    connection_uuids: Vec<String>,
    mac_addresses: Vec<String>,
    wifi_ssids: Vec<String>,
}

fn linux_network_config_metadata(path: &str, header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut summary = LinuxNetworkSummary::default();
    let config_type = if path.ends_with("etc/network/interfaces") {
        parse_debian_interfaces_metadata(&text, &mut summary);
        "debian-interfaces"
    } else if path.ends_with("etc/resolv.conf") {
        parse_resolv_conf_metadata(&text, &mut summary);
        "resolver"
    } else if path.ends_with("etc/hosts") {
        parse_hosts_metadata(&text, &mut summary);
        "hosts"
    } else if path.contains("etc/sysconfig/network-scripts/ifcfg-") {
        parse_ifcfg_metadata(path, &text, &mut summary);
        "ifcfg"
    } else if path.contains("etc/networkmanager/system-connections/") {
        parse_network_manager_metadata(&text, &mut summary);
        "networkmanager"
    } else {
        parse_netplan_metadata(&text, &mut summary);
        "netplan"
    };

    let mut metadata = BTreeMap::new();
    metadata.insert("system.osFamily".to_string(), "linux".to_string());
    metadata.insert("system.infoType".to_string(), "network-config".to_string());
    metadata.insert(
        "system.networkConfigType".to_string(),
        config_type.to_string(),
    );
    insert_limited_system_values(
        &mut metadata,
        "system.networkInterfaces",
        &summary.interfaces,
    );
    insert_limited_system_values(&mut metadata, "system.ipv4Addresses", &summary.addresses);
    insert_limited_system_values(&mut metadata, "system.gateways", &summary.gateways);
    insert_limited_system_values(&mut metadata, "system.dnsServers", &summary.dns_servers);
    insert_limited_system_values(&mut metadata, "system.networkMethods", &summary.methods);
    insert_limited_system_values(
        &mut metadata,
        "system.dnsSearchDomains",
        &summary.search_domains,
    );
    insert_limited_system_values(&mut metadata, "system.hostAliases", &summary.host_aliases);
    insert_limited_system_values(
        &mut metadata,
        "system.connectionIds",
        &summary.connection_ids,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.connectionUuids",
        &summary.connection_uuids,
    );
    insert_limited_system_values(&mut metadata, "system.macAddresses", &summary.mac_addresses);
    insert_limited_system_values(&mut metadata, "system.wifiSsids", &summary.wifi_ssids);
    metadata
}

#[derive(Default)]
struct UnixAccountSummary {
    user_count: usize,
    regular_user_count: usize,
    login_user_count: usize,
    group_count: usize,
    shadow_entry_count: usize,
    password_hash_user_count: usize,
    password_locked_user_count: usize,
    password_disabled_user_count: usize,
    password_empty_user_count: usize,
    users: Vec<String>,
    regular_users: Vec<String>,
    login_users: Vec<String>,
    password_hash_users: Vec<String>,
    password_locked_users: Vec<String>,
    password_disabled_users: Vec<String>,
    password_empty_users: Vec<String>,
    password_hash_algorithms: Vec<String>,
    home_directories: Vec<String>,
    login_shells: Vec<String>,
    groups: Vec<String>,
    admin_groups: Vec<String>,
    group_members: Vec<String>,
    min_uid: Option<u32>,
    max_uid: Option<u32>,
    root_present: bool,
}

fn unix_account_metadata(path: &str, header: &[u8]) -> BTreeMap<String, String> {
    let text = String::from_utf8_lossy(header);
    let mut summary = UnixAccountSummary::default();
    let config_type = if path.ends_with("etc/group") || path.ends_with("private/etc/group") {
        parse_unix_group_metadata(&text, &mut summary);
        "unix-group"
    } else if path.ends_with("etc/shadow") || path.ends_with("private/etc/shadow") {
        parse_unix_shadow_metadata(&text, &mut summary);
        "unix-shadow"
    } else if path.ends_with("etc/gshadow") || path.ends_with("private/etc/gshadow") {
        parse_unix_gshadow_metadata(&text, &mut summary);
        "unix-gshadow"
    } else {
        parse_unix_passwd_metadata(&text, &mut summary);
        "unix-passwd"
    };

    unix_account_metadata_to_map(summary, config_type)
}

fn parse_unix_passwd_metadata(text: &str, summary: &mut UnixAccountSummary) {
    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 7 {
            continue;
        }
        let name = fields[0].trim();
        let uid = fields[2].trim().parse::<u32>().ok();
        let gid = fields[3].trim();
        let gecos = fields[4].trim();
        let home = fields[5].trim();
        let shell = fields[6].trim();
        if name.is_empty() {
            continue;
        }

        summary.user_count = summary.user_count.saturating_add(1);
        if name == "root" {
            summary.root_present = true;
        }
        if let Some(uid) = uid {
            summary.min_uid = Some(summary.min_uid.map_or(uid, |current| current.min(uid)));
            summary.max_uid = Some(summary.max_uid.map_or(uid, |current| current.max(uid)));
        }
        push_limited_system_value(
            &mut summary.users,
            &format!("{name}:uid={}", uid.unwrap_or(0)),
        );

        if is_unix_regular_user(uid) {
            summary.regular_user_count = summary.regular_user_count.saturating_add(1);
            let display_name = if gecos.is_empty() {
                name.to_string()
            } else {
                format!("{name} ({gecos})")
            };
            push_limited_system_value(&mut summary.regular_users, &display_name);
        }
        if is_unix_login_shell(shell) {
            summary.login_user_count = summary.login_user_count.saturating_add(1);
            push_limited_system_value(
                &mut summary.login_users,
                &format!("{name}:uid={}:gid={gid}", uid.unwrap_or(0)),
            );
            push_limited_system_value(&mut summary.login_shells, shell);
        }
        if !home.is_empty() && home != "/" && home != "/nonexistent" {
            push_limited_system_value(&mut summary.home_directories, home);
        }
    }
}

fn parse_unix_group_metadata(text: &str, summary: &mut UnixAccountSummary) {
    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields[0].trim();
        let gid = fields[2].trim();
        let members = fields[3].trim();
        if name.is_empty() {
            continue;
        }

        summary.group_count = summary.group_count.saturating_add(1);
        push_limited_system_value(&mut summary.groups, &format!("{name}:gid={gid}"));
        if is_unix_admin_group(name) {
            push_limited_system_value(
                &mut summary.admin_groups,
                &format!("{name}:members={members}"),
            );
        }
        if !members.is_empty() {
            push_limited_system_value(&mut summary.group_members, &format!("{name}={members}"));
        }
    }
}

fn parse_unix_shadow_metadata(text: &str, summary: &mut UnixAccountSummary) {
    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 2 {
            continue;
        }
        let name = fields[0].trim();
        let credential = fields[1].trim();
        if name.is_empty() {
            continue;
        }

        summary.shadow_entry_count = summary.shadow_entry_count.saturating_add(1);
        match unix_shadow_password_status(credential) {
            UnixShadowPasswordStatus::HasHash(algorithm) => {
                summary.password_hash_user_count =
                    summary.password_hash_user_count.saturating_add(1);
                push_limited_system_value(&mut summary.password_hash_users, name);
                push_limited_system_value(&mut summary.password_hash_algorithms, algorithm);
            }
            UnixShadowPasswordStatus::Locked => {
                summary.password_locked_user_count =
                    summary.password_locked_user_count.saturating_add(1);
                push_limited_system_value(&mut summary.password_locked_users, name);
            }
            UnixShadowPasswordStatus::Disabled => {
                summary.password_disabled_user_count =
                    summary.password_disabled_user_count.saturating_add(1);
                push_limited_system_value(&mut summary.password_disabled_users, name);
            }
            UnixShadowPasswordStatus::Empty => {
                summary.password_empty_user_count =
                    summary.password_empty_user_count.saturating_add(1);
                push_limited_system_value(&mut summary.password_empty_users, name);
            }
        }
    }
}

fn parse_unix_gshadow_metadata(text: &str, summary: &mut UnixAccountSummary) {
    for raw_line in text.lines().take(4096) {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields[0].trim();
        let admins = fields[2].trim();
        let members = fields[3].trim();
        if name.is_empty() {
            continue;
        }

        summary.group_count = summary.group_count.saturating_add(1);
        push_limited_system_value(&mut summary.groups, name);
        if !admins.is_empty() {
            push_limited_system_value(
                &mut summary.admin_groups,
                &format!("{name}:admins={admins}"),
            );
        }
        if !members.is_empty() {
            push_limited_system_value(&mut summary.group_members, &format!("{name}={members}"));
        }
    }
}

enum UnixShadowPasswordStatus {
    HasHash(&'static str),
    Locked,
    Disabled,
    Empty,
}

fn unix_shadow_password_status(credential: &str) -> UnixShadowPasswordStatus {
    let credential = credential.trim();
    if credential.is_empty() {
        return UnixShadowPasswordStatus::Empty;
    }
    if credential.starts_with('!') {
        return UnixShadowPasswordStatus::Locked;
    }
    if credential.starts_with('*') {
        return UnixShadowPasswordStatus::Disabled;
    }
    UnixShadowPasswordStatus::HasHash(unix_shadow_hash_algorithm(credential))
}

fn unix_shadow_hash_algorithm(credential: &str) -> &'static str {
    if credential.starts_with("$y$") {
        "yescrypt"
    } else if credential.starts_with("$6$") {
        "sha512-crypt"
    } else if credential.starts_with("$5$") {
        "sha256-crypt"
    } else if credential.starts_with("$2a$")
        || credential.starts_with("$2b$")
        || credential.starts_with("$2y$")
    {
        "bcrypt"
    } else if credential.starts_with("$1$") {
        "md5-crypt"
    } else {
        "traditional-crypt"
    }
}

fn is_unix_regular_user(uid: Option<u32>) -> bool {
    uid.is_some_and(|uid| (UNIX_REGULAR_USER_MIN_UID..UNIX_REGULAR_USER_MAX_UID).contains(&uid))
}

fn is_unix_login_shell(shell: &str) -> bool {
    let shell = shell.trim();
    !shell.is_empty()
        && !shell.ends_with("/nologin")
        && !shell.ends_with("/false")
        && shell != "nologin"
        && shell != "false"
}

fn is_unix_admin_group(name: &str) -> bool {
    matches!(name, "admin" | "sudo" | "wheel" | "staff")
}

fn unix_account_metadata_to_map(
    summary: UnixAccountSummary,
    config_type: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert("system.osFamily".to_string(), "unix".to_string());
    metadata.insert("system.infoType".to_string(), "account-config".to_string());
    metadata.insert(
        "system.accountConfigType".to_string(),
        config_type.to_string(),
    );
    if summary.user_count > 0 {
        metadata.insert(
            "system.localUserCount".to_string(),
            summary.user_count.to_string(),
        );
    }
    if summary.regular_user_count > 0 {
        metadata.insert(
            "system.regularUserCount".to_string(),
            summary.regular_user_count.to_string(),
        );
    }
    if summary.login_user_count > 0 {
        metadata.insert(
            "system.loginUserCount".to_string(),
            summary.login_user_count.to_string(),
        );
    }
    if summary.group_count > 0 {
        metadata.insert(
            "system.localGroupCount".to_string(),
            summary.group_count.to_string(),
        );
    }
    if summary.shadow_entry_count > 0 {
        metadata.insert(
            "system.shadowEntryCount".to_string(),
            summary.shadow_entry_count.to_string(),
        );
    }
    if summary.password_hash_user_count > 0 {
        metadata.insert(
            "system.passwordHashUserCount".to_string(),
            summary.password_hash_user_count.to_string(),
        );
    }
    if summary.password_locked_user_count > 0 {
        metadata.insert(
            "system.passwordLockedUserCount".to_string(),
            summary.password_locked_user_count.to_string(),
        );
    }
    if summary.password_disabled_user_count > 0 {
        metadata.insert(
            "system.passwordDisabledUserCount".to_string(),
            summary.password_disabled_user_count.to_string(),
        );
    }
    if summary.password_empty_user_count > 0 {
        metadata.insert(
            "system.passwordEmptyUserCount".to_string(),
            summary.password_empty_user_count.to_string(),
        );
    }
    if summary.root_present {
        metadata.insert("system.rootAccountPresent".to_string(), "true".to_string());
    }
    if let (Some(min_uid), Some(max_uid)) = (summary.min_uid, summary.max_uid) {
        metadata.insert(
            "system.userUidRange".to_string(),
            format!("{min_uid}-{max_uid}"),
        );
    }
    insert_limited_system_values(&mut metadata, "system.localUsers", &summary.users);
    insert_limited_system_values(&mut metadata, "system.regularUsers", &summary.regular_users);
    insert_limited_system_values(&mut metadata, "system.loginUsers", &summary.login_users);
    insert_limited_system_values(
        &mut metadata,
        "system.passwordHashUsers",
        &summary.password_hash_users,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.passwordLockedUsers",
        &summary.password_locked_users,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.passwordDisabledUsers",
        &summary.password_disabled_users,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.passwordEmptyUsers",
        &summary.password_empty_users,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.passwordHashAlgorithms",
        &summary.password_hash_algorithms,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.homeDirectories",
        &summary.home_directories,
    );
    insert_limited_system_values(&mut metadata, "system.loginShells", &summary.login_shells);
    insert_limited_system_values(&mut metadata, "system.localGroups", &summary.groups);
    insert_limited_system_values(&mut metadata, "system.adminGroups", &summary.admin_groups);
    insert_limited_system_values(&mut metadata, "system.groupMembers", &summary.group_members);
    metadata
}

fn parse_debian_interfaces_metadata(text: &str, summary: &mut LinuxNetworkSummary) {
    let mut current_interface: Option<String> = None;
    for raw_line in text.lines().take(1024) {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["auto" | "allow-hotplug", interfaces @ ..] => {
                for interface in interfaces {
                    push_limited_system_value(&mut summary.interfaces, interface);
                }
            }
            ["iface", interface, family, method, ..] => {
                current_interface = Some((*interface).to_string());
                push_limited_system_value(&mut summary.interfaces, interface);
                if *family == "inet" || *family == "inet6" {
                    push_limited_system_value(
                        &mut summary.methods,
                        &format!("{interface}:{family}:{method}"),
                    );
                }
            }
            ["address", address, ..] => {
                push_limited_system_value(&mut summary.addresses, address);
            }
            ["gateway", gateway, ..] => {
                push_limited_system_value(&mut summary.gateways, gateway);
            }
            ["dns-nameservers", servers @ ..] => {
                for server in servers {
                    push_limited_system_value(&mut summary.dns_servers, server);
                }
            }
            ["hwaddress", "ether", mac, ..] => {
                if let Some(interface) = &current_interface {
                    push_limited_system_value(
                        &mut summary.interfaces,
                        &format!("{interface} ({mac})"),
                    );
                    push_limited_system_value(&mut summary.mac_addresses, mac);
                }
            }
            _ => {}
        }
    }
}

fn parse_ifcfg_metadata(path: &str, text: &str, summary: &mut LinuxNetworkSummary) {
    let pairs = parse_shell_key_values(text);
    let interface = pairs
        .get("DEVICE")
        .or_else(|| pairs.get("NAME"))
        .cloned()
        .or_else(|| {
            path.rsplit('/')
                .next()
                .and_then(|name| name.strip_prefix("ifcfg-"))
                .map(ToString::to_string)
        });

    if let Some(interface) = &interface {
        push_limited_system_value(&mut summary.interfaces, interface);
    }
    if let Some(address) = pairs.get("IPADDR") {
        let address = pairs
            .get("PREFIX")
            .map(|prefix| format!("{address}/{prefix}"))
            .or_else(|| {
                pairs
                    .get("NETMASK")
                    .map(|netmask| format!("{address}/{netmask}"))
            })
            .unwrap_or_else(|| address.clone());
        push_limited_system_value(&mut summary.addresses, &address);
    }
    if let Some(gateway) = pairs.get("GATEWAY") {
        push_limited_system_value(&mut summary.gateways, gateway);
    }
    for key in ["DNS1", "DNS2", "DNS3"] {
        if let Some(server) = pairs.get(key) {
            push_limited_system_value(&mut summary.dns_servers, server);
        }
    }
    if let Some(mac) = pairs.get("HWADDR").or_else(|| pairs.get("MACADDR")) {
        push_limited_system_value(&mut summary.mac_addresses, mac);
    }
    if let (Some(interface), Some(method)) = (interface.as_deref(), pairs.get("BOOTPROTO")) {
        push_limited_system_value(&mut summary.methods, &format!("{interface}:inet:{method}"));
    }
}

fn parse_resolv_conf_metadata(text: &str, summary: &mut LinuxNetworkSummary) {
    for raw_line in text.lines().take(512) {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["nameserver", server, ..] => {
                push_limited_system_value(&mut summary.dns_servers, server);
            }
            ["domain", domain, ..] => {
                push_limited_system_value(&mut summary.search_domains, domain);
            }
            ["search", domains @ ..] => {
                for domain in domains {
                    push_limited_system_value(&mut summary.search_domains, domain);
                }
            }
            _ => {}
        }
    }
}

fn parse_hosts_metadata(text: &str, summary: &mut LinuxNetworkSummary) {
    for raw_line in text.lines().take(1024) {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        let Some((address, aliases)) = parts.split_first() else {
            continue;
        };
        if aliases.is_empty() {
            continue;
        }
        push_limited_system_value(
            &mut summary.host_aliases,
            &format!("{address}={}", aliases.join(",")),
        );
    }
}

fn parse_network_manager_metadata(text: &str, summary: &mut LinuxNetworkSummary) {
    let mut section = "";
    let mut connection_type: Option<String> = None;
    let mut interface: Option<String> = None;

    for raw_line in text.lines().take(2048) {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line.trim_start_matches('[').trim_end_matches(']');
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            continue;
        }

        match (section, key) {
            ("connection", "id") => push_limited_system_value(&mut summary.connection_ids, value),
            ("connection", "uuid") => {
                push_limited_system_value(&mut summary.connection_uuids, value)
            }
            ("connection", "type") => connection_type = Some(value.to_string()),
            ("connection", "interface-name") => {
                interface = Some(value.to_string());
                push_limited_system_value(&mut summary.interfaces, value);
            }
            ("wifi", "ssid") => push_limited_system_value(&mut summary.wifi_ssids, value),
            ("wifi", "mac-address") | ("ethernet", "mac-address") => {
                push_limited_system_value(&mut summary.mac_addresses, value)
            }
            ("ipv4" | "ipv6", "method") => {
                let interface = interface.as_deref().unwrap_or("unknown");
                let family = if section == "ipv6" { "inet6" } else { "inet" };
                push_limited_system_value(
                    &mut summary.methods,
                    &format!("{interface}:{family}:{value}"),
                );
            }
            ("ipv4" | "ipv6", "addresses") | ("ipv4" | "ipv6", "address1") => {
                collect_network_manager_addresses(value, summary);
            }
            ("ipv4" | "ipv6", "gateway") => {
                push_limited_system_value(&mut summary.gateways, value);
            }
            ("ipv4" | "ipv6", "dns") => {
                for server in split_network_config_values(value) {
                    push_limited_system_value(&mut summary.dns_servers, &server);
                }
            }
            ("ipv4" | "ipv6", "dns-search") => {
                for domain in split_network_config_values(value) {
                    push_limited_system_value(&mut summary.search_domains, &domain);
                }
            }
            _ => {}
        }
    }

    if summary.interfaces.is_empty() {
        if let Some(connection_type) = connection_type {
            push_limited_system_value(&mut summary.interfaces, &connection_type);
        }
    }
}

fn collect_network_manager_addresses(value: &str, summary: &mut LinuxNetworkSummary) {
    for address in value.split(';') {
        let address = address.trim();
        if address.is_empty() {
            continue;
        }
        let mut parts = address
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        if let Some(ip) = parts.next() {
            push_limited_system_value(&mut summary.addresses, ip);
        }
        if let Some(gateway) = parts.next() {
            push_limited_system_value(&mut summary.gateways, gateway);
        }
    }
}

fn parse_netplan_metadata(text: &str, summary: &mut LinuxNetworkSummary) {
    let mut current_interface: Option<String> = None;
    let mut in_nameservers = false;
    let mut pending_address_list = false;
    let mut pending_dns_list = false;

    for raw_line in text.lines().take(2048) {
        let line = trim_network_config_line(raw_line);
        if line.is_empty() {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.ends_with(':') {
            let key = trimmed.trim_end_matches(':').trim().trim_matches('"');
            if key == "nameservers" {
                in_nameservers = true;
            } else {
                in_nameservers = false;
                pending_dns_list = false;
            }
            if is_netplan_interface_key(key) {
                current_interface = Some(key.to_string());
                push_limited_system_value(&mut summary.interfaces, key);
            }
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- ") {
            if pending_dns_list || in_nameservers {
                for server in split_network_config_values(value) {
                    push_limited_system_value(&mut summary.dns_servers, &server);
                }
            } else if pending_address_list {
                for address in split_network_config_values(value) {
                    push_limited_system_value(&mut summary.addresses, &address);
                }
            }
            continue;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        pending_address_list = false;
        pending_dns_list = false;

        match key {
            "addresses" if in_nameservers => {
                if value.is_empty() {
                    pending_dns_list = true;
                } else {
                    for server in split_network_config_values(value) {
                        push_limited_system_value(&mut summary.dns_servers, &server);
                    }
                }
            }
            "addresses" => {
                if value.is_empty() {
                    pending_address_list = true;
                } else {
                    for address in split_network_config_values(value) {
                        push_limited_system_value(&mut summary.addresses, &address);
                    }
                }
            }
            "gateway4" | "gateway6" => {
                for gateway in split_network_config_values(value) {
                    push_limited_system_value(&mut summary.gateways, &gateway);
                }
            }
            "dhcp4" | "dhcp6" if value.eq_ignore_ascii_case("true") => {
                let interface = current_interface.as_deref().unwrap_or("unknown");
                let family = if key == "dhcp6" { "inet6" } else { "inet" };
                push_limited_system_value(
                    &mut summary.methods,
                    &format!("{interface}:{family}:dhcp"),
                );
            }
            _ => {}
        }
    }
}

fn trim_network_config_line(line: &str) -> &str {
    line.split_once('#')
        .map_or(line, |(before, _)| before)
        .trim()
}

fn is_netplan_interface_key(key: &str) -> bool {
    !matches!(
        key,
        "network"
            | "version"
            | "renderer"
            | "ethernets"
            | "wifis"
            | "bridges"
            | "bonds"
            | "vlans"
            | "addresses"
            | "nameservers"
            | "routes"
            | "gateway4"
            | "gateway6"
            | "dhcp4"
            | "dhcp6"
            | "optional"
            | "match"
            | "set-name"
    )
}

fn split_network_config_values(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_matches('[')
        .trim_matches(']')
        .split([',', ';', ' '])
        .filter_map(|part| {
            let part = part.trim().trim_matches('"').trim_matches('\'');
            (!part.is_empty()).then(|| truncate_chars(part, 180))
        })
        .collect()
}

fn push_limited_system_value(values: &mut Vec<String>, value: &str) {
    const MAX_SYSTEM_VALUES: usize = 32;
    const MAX_SYSTEM_VALUE_CHARS: usize = 180;
    if values.len() >= MAX_SYSTEM_VALUES {
        return;
    }
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    let normalized: String = value.chars().take(MAX_SYSTEM_VALUE_CHARS).collect();
    if !values.iter().any(|existing| existing == &normalized) {
        values.push(normalized);
    }
}

fn insert_limited_system_values(
    metadata: &mut BTreeMap<String, String>,
    key: &str,
    values: &[String],
) {
    if !values.is_empty() {
        metadata.insert(key.to_string(), values.join("; "));
    }
}

fn parse_shell_key_values(text: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in text.lines().take(256) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty()
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            continue;
        }
        let value = unquote_shell_value(value.trim());
        if !value.is_empty() {
            values.insert(key.to_string(), value);
        }
    }
    values
}

fn unquote_shell_value(value: &str) -> String {
    let unquoted = if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    };
    unquoted
        .replace("\\\"", "\"")
        .replace("\\'", "'")
        .replace("\\\\", "\\")
        .trim()
        .to_string()
}

fn insert_key_value_alias(
    metadata: &mut BTreeMap<String, String>,
    values: &BTreeMap<String, String>,
    source_key: &str,
    metadata_key: &str,
) {
    if let Some(value) = values
        .get(source_key)
        .filter(|value| !value.trim().is_empty())
    {
        metadata.insert(metadata_key.to_string(), value.clone());
    }
}

fn single_line_system_metadata(
    header: &[u8],
    metadata_key: &str,
    os_family: &str,
    info_type: &str,
) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    let Some(value) = first_text_line(header) else {
        return metadata;
    };

    metadata.insert("system.osFamily".to_string(), os_family.to_string());
    metadata.insert("system.infoType".to_string(), info_type.to_string());
    metadata.insert(metadata_key.to_string(), value);
    metadata
}

fn first_text_line(header: &[u8]) -> Option<String> {
    String::from_utf8_lossy(header)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToString::to_string)
}

fn linux_dmi_metadata(path: &str, header: &[u8]) -> Option<BTreeMap<String, String>> {
    let field = path
        .strip_suffix('/')
        .unwrap_or(path)
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let metadata_key = match field {
        "sys_vendor" => "system.manufacturer",
        "product_name" => "system.model",
        "product_version" => "system.productVersion",
        "product_family" => "system.productFamily",
        "product_serial" => "system.serialNumber",
        "product_uuid" => "system.uuid",
        "board_vendor" => "system.boardVendor",
        "board_name" => "system.boardName",
        "board_version" => "system.boardVersion",
        "board_serial" => "system.boardSerialNumber",
        "bios_vendor" => "system.biosVendor",
        "bios_version" => "system.biosVersion",
        "bios_date" => "system.biosDate",
        "chassis_vendor" => "system.chassisVendor",
        "chassis_type" => "system.chassisType",
        "chassis_serial" => "system.chassisSerialNumber",
        _ => return None,
    };
    if !(path.contains("sys/class/dmi/id/") || path.contains("sys/devices/virtual/dmi/id/")) {
        return None;
    }

    Some(single_line_system_metadata(
        header,
        metadata_key,
        "linux",
        "dmi",
    ))
}

fn macos_system_version_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(PlistValue::Dictionary(dictionary)) = PlistValue::from_reader(Cursor::new(header))
    else {
        return metadata;
    };

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert("system.infoType".to_string(), "system-version".to_string());
    for (source_key, metadata_key) in [
        ("ProductName", "os.release.name"),
        ("ProductVersion", "os.release.version"),
        ("ProductBuildVersion", "os.release.buildId"),
        ("ProductUserVisibleVersion", "os.release.userVisibleVersion"),
    ] {
        if let Some(value) = dictionary.get(source_key).and_then(plist_scalar_string) {
            metadata.insert(metadata_key.to_string(), value);
        }
    }
    metadata
}

fn macos_preferences_identity_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };

    for (source_key, metadata_key) in [
        ("ComputerName", "system.computerName"),
        ("HostName", "system.hostname"),
        ("LocalHostName", "system.localHostname"),
    ] {
        if let Some(value) = plist_find_scalar_string(&value, source_key) {
            metadata.insert(metadata_key.to_string(), value);
        }
    }

    if !metadata.is_empty() {
        metadata.insert("system.osFamily".to_string(), "macos".to_string());
        metadata.insert("system.infoType".to_string(), "system-identity".to_string());
    }
    metadata
}

fn macos_network_interfaces_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };
    let Some(interfaces) = plist_find_array(&value, "Interfaces") else {
        return metadata;
    };

    let mut descriptions = Vec::new();
    let mut mac_addresses = Vec::new();
    for interface in interfaces.iter().take(32) {
        let PlistValue::Dictionary(interface) = interface else {
            continue;
        };
        let bsd_name = plist_dict_string(interface, "BSD Name");
        let interface_type = plist_dict_string(interface, "SCNetworkInterfaceType")
            .or_else(|| plist_dict_string(interface, "SCNetworkInterfaceSubType"));
        let display_name = interface
            .get("SCNetworkInterfaceInfo")
            .and_then(|value| match value {
                PlistValue::Dictionary(info) => plist_dict_string(info, "UserDefinedName"),
                _ => None,
            });
        let mac_address = interface
            .get("IOMACAddress")
            .and_then(plist_data_mac_address);

        if let Some(mac_address) = &mac_address {
            push_limited_system_value(&mut mac_addresses, mac_address);
        }
        if let Some(description) = describe_macos_network_interface(
            bsd_name,
            display_name,
            interface_type,
            mac_address.as_deref(),
        ) {
            push_limited_system_value(&mut descriptions, &description);
        }
    }

    if descriptions.is_empty() && mac_addresses.is_empty() {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert(
        "system.infoType".to_string(),
        "network-interfaces".to_string(),
    );
    if !descriptions.is_empty() {
        metadata.insert(
            "system.networkInterfaceCount".to_string(),
            descriptions.len().to_string(),
        );
        insert_limited_system_values(&mut metadata, "system.networkInterfaces", &descriptions);
    }
    if let Some(primary) = mac_addresses.first() {
        metadata.insert("system.primaryMacAddress".to_string(), primary.clone());
    }
    insert_limited_system_values(&mut metadata, "system.macAddresses", &mac_addresses);
    metadata
}

#[derive(Default)]
struct MacosWifiSummary {
    ssids: Vec<String>,
    security_types: Vec<String>,
    auto_join_ssids: Vec<String>,
    last_connected: Vec<String>,
}

fn macos_wifi_preferences_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };
    let mut summary = MacosWifiSummary::default();
    collect_macos_wifi_metadata(&value, &mut summary);

    if summary.ssids.is_empty() {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert(
        "system.infoType".to_string(),
        "wifi-preferences".to_string(),
    );
    metadata.insert(
        "system.wifiKnownNetworkCount".to_string(),
        summary.ssids.len().to_string(),
    );
    insert_limited_system_values(&mut metadata, "system.wifiSsids", &summary.ssids);
    insert_limited_system_values(
        &mut metadata,
        "system.wifiSecurityTypes",
        &summary.security_types,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.wifiAutoJoinSsids",
        &summary.auto_join_ssids,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.wifiLastConnected",
        &summary.last_connected,
    );
    metadata
}

fn collect_macos_wifi_metadata(value: &PlistValue, summary: &mut MacosWifiSummary) {
    match value {
        PlistValue::Dictionary(dictionary) => {
            collect_macos_wifi_network_dictionary(dictionary, summary);
            for child in dictionary.values() {
                collect_macos_wifi_metadata(child, summary);
            }
        }
        PlistValue::Array(items) => {
            for child in items {
                collect_macos_wifi_metadata(child, summary);
            }
        }
        _ => {}
    }
}

fn collect_macos_wifi_network_dictionary(
    dictionary: &plist::Dictionary,
    summary: &mut MacosWifiSummary,
) {
    let Some(ssid) = macos_wifi_ssid(dictionary) else {
        return;
    };
    push_limited_system_value(&mut summary.ssids, &ssid);

    for key in [
        "SecurityType",
        "Security",
        "AuthType",
        "EncryptionType",
        "SupportedSecurityTypes",
    ] {
        if let Some(value) = plist_dict_string(dictionary, key) {
            push_limited_system_value(&mut summary.security_types, value);
        } else if let Some(values) = plist_dict_string_array(dictionary, key) {
            for value in values {
                push_limited_system_value(&mut summary.security_types, &value);
            }
        }
    }

    if plist_dict_bool(dictionary, "AutoJoin").or_else(|| plist_dict_bool(dictionary, "AutoLogin"))
        == Some(true)
    {
        push_limited_system_value(&mut summary.auto_join_ssids, &ssid);
    }

    for key in ["LastConnected", "LastAutoJoined", "LastJoined"] {
        if let Some(date) = plist_dict_date(dictionary, key) {
            push_limited_system_value(&mut summary.last_connected, &format!("{ssid}={date}"));
            break;
        }
    }
}

fn macos_wifi_ssid(dictionary: &plist::Dictionary) -> Option<String> {
    for key in ["SSIDString", "SSID_STR", "SSID", "name"] {
        if let Some(value) = plist_dict_string(dictionary, key) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(truncate_chars(value, 180));
            }
        }
    }
    for key in ["SSID", "SSIDData"] {
        if let Some(value) = dictionary.get(key).and_then(plist_data_utf8_string) {
            return Some(truncate_chars(&value, 180));
        }
    }
    None
}

#[derive(Default)]
struct MacosDiskManagementSummary {
    volume_names: Vec<String>,
    volume_uuids: Vec<String>,
    disk_identifiers: Vec<String>,
    filesystems: Vec<String>,
    mount_points: Vec<String>,
    descriptions: Vec<String>,
    total_size_bytes: u64,
}

fn macos_disk_management_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };
    let mut summary = MacosDiskManagementSummary::default();
    collect_macos_disk_management_metadata(&value, &mut summary);

    if summary.descriptions.is_empty() {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert("system.infoType".to_string(), "disk-management".to_string());
    metadata.insert(
        "system.volumeCount".to_string(),
        summary.descriptions.len().to_string(),
    );
    if summary.total_size_bytes > 0 {
        metadata.insert(
            "system.totalVolumeBytes".to_string(),
            summary.total_size_bytes.to_string(),
        );
    }
    insert_limited_system_values(&mut metadata, "system.volumeNames", &summary.volume_names);
    insert_limited_system_values(&mut metadata, "system.volumeUuids", &summary.volume_uuids);
    insert_limited_system_values(
        &mut metadata,
        "system.diskIdentifiers",
        &summary.disk_identifiers,
    );
    insert_limited_system_values(
        &mut metadata,
        "system.volumeFilesystems",
        &summary.filesystems,
    );
    insert_limited_system_values(&mut metadata, "system.volumeMounts", &summary.mount_points);
    insert_limited_system_values(&mut metadata, "system.volumes", &summary.descriptions);
    metadata
}

fn collect_macos_disk_management_metadata(
    value: &PlistValue,
    summary: &mut MacosDiskManagementSummary,
) {
    match value {
        PlistValue::Dictionary(dictionary) => {
            collect_macos_disk_management_dictionary(dictionary, summary);
            for child in dictionary.values() {
                collect_macos_disk_management_metadata(child, summary);
            }
        }
        PlistValue::Array(items) => {
            for child in items {
                collect_macos_disk_management_metadata(child, summary);
            }
        }
        _ => {}
    }
}

fn collect_macos_disk_management_dictionary(
    dictionary: &plist::Dictionary,
    summary: &mut MacosDiskManagementSummary,
) {
    let bsd_name = plist_dict_first_scalar_string(
        dictionary,
        &["BSD Name", "BSDName", "DeviceIdentifier", "DAMediaBSDName"],
    );
    let uuid = plist_dict_first_scalar_string(dictionary, &["VolumeUUID", "DAVolumeUUID"]);
    let filesystem = plist_dict_first_scalar_string(
        dictionary,
        &[
            "FilesystemName",
            "FilesystemType",
            "DAVolumeKind",
            "Content",
            "DAMediaContent",
        ],
    );
    let mount_point =
        plist_dict_first_scalar_string(dictionary, &["MountPoint", "DAVolumePath", "Path"]);
    let size = plist_dict_first_scalar_string(dictionary, &["Size", "VolumeSize", "DAMediaSize"]);

    if bsd_name.is_none()
        && uuid.is_none()
        && filesystem.is_none()
        && mount_point.is_none()
        && size.is_none()
    {
        return;
    }

    let name = plist_dict_first_scalar_string(dictionary, &["VolumeName", "DAVolumeName", "Name"]);
    if let Some(name) = &name {
        push_limited_system_value(&mut summary.volume_names, name);
    }
    if let Some(uuid) = &uuid {
        push_limited_system_value(&mut summary.volume_uuids, uuid);
    }
    if let Some(bsd_name) = &bsd_name {
        push_limited_system_value(&mut summary.disk_identifiers, bsd_name);
    }
    if let Some(filesystem) = &filesystem {
        push_limited_system_value(&mut summary.filesystems, filesystem);
    }
    if let Some(mount_point) = &mount_point {
        push_limited_system_value(&mut summary.mount_points, mount_point);
    }
    if let Some(size) = &size {
        if let Ok(size) = size.parse::<u64>() {
            summary.total_size_bytes = summary.total_size_bytes.saturating_add(size);
        }
    }

    if let Some(description) =
        macos_disk_description(name, bsd_name, uuid, filesystem, mount_point, size)
    {
        push_limited_system_value(&mut summary.descriptions, &description);
    }
}

fn macos_disk_description(
    name: Option<String>,
    bsd_name: Option<String>,
    uuid: Option<String>,
    filesystem: Option<String>,
    mount_point: Option<String>,
    size: Option<String>,
) -> Option<String> {
    let label = name.or_else(|| bsd_name.clone()).or_else(|| uuid.clone())?;
    let mut parts = Vec::new();
    if let Some(bsd_name) = bsd_name.filter(|value| value != &label) {
        parts.push(bsd_name);
    }
    if let Some(filesystem) = filesystem {
        parts.push(filesystem);
    }
    if let Some(mount_point) = mount_point {
        parts.push(format!("mounted={mount_point}"));
    }
    if let Some(uuid) = uuid.filter(|value| value != &label) {
        parts.push(format!("uuid={uuid}"));
    }
    if let Some(size) = size {
        parts.push(format!("size={size}"));
    }

    if parts.is_empty() {
        Some(label)
    } else {
        Some(format!("{} ({})", label, parts.join(", ")))
    }
}

fn macos_firewall_preferences_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(PlistValue::Dictionary(dictionary)) = PlistValue::from_reader(Cursor::new(header))
    else {
        return metadata;
    };

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert("system.infoType".to_string(), "firewall".to_string());
    metadata.insert(
        "system.firewallConfigType".to_string(),
        "macos-alf".to_string(),
    );
    insert_plist_integer_metadata(
        &mut metadata,
        &dictionary,
        "globalstate",
        "system.firewallGlobalState",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dictionary,
        "stealthenabled",
        "system.firewallStealthEnabled",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dictionary,
        "allowsignedenabled",
        "system.firewallAllowSignedEnabled",
    );
    insert_plist_bool_metadata(
        &mut metadata,
        &dictionary,
        "loggingenabled",
        "system.firewallLoggingEnabled",
    );
    if let Some(count) = plist_dict_array_len(&dictionary, "applications") {
        metadata.insert(
            "system.firewallApplicationRuleCount".to_string(),
            count.to_string(),
        );
    }
    metadata
}

fn macos_install_history_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(PlistValue::Array(entries)) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };

    let mut install_count = 0usize;
    let mut latest_install = None;
    for entry in entries.iter().take(32) {
        let PlistValue::Dictionary(dictionary) = entry else {
            continue;
        };
        install_count = install_count.saturating_add(1);
        latest_install = Some(dictionary);
    }

    if install_count == 0 {
        return metadata;
    }

    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert("system.infoType".to_string(), "install-history".to_string());
    metadata.insert(
        "system.installHistoryCount".to_string(),
        install_count.to_string(),
    );

    let Some(latest_install) = latest_install else {
        return metadata;
    };
    if let Some(value) = plist_dict_string(latest_install, "displayName") {
        metadata.insert("system.latestInstallName".to_string(), value.to_string());
    }
    if let Some(value) = plist_dict_string(latest_install, "displayVersion") {
        metadata.insert("system.latestInstallVersion".to_string(), value.to_string());
    }
    if let Some(value) = plist_dict_date(latest_install, "date") {
        metadata.insert("system.latestInstallDate".to_string(), value);
    }
    if let Some(values) = plist_dict_string_array(latest_install, "packageIdentifiers") {
        metadata.insert(
            "system.latestInstallPackages".to_string(),
            values.join(", "),
        );
    }
    metadata
}

fn macos_hardware_identity_metadata(header: &[u8]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !looks_like_plist(header, Some("plist")) {
        return metadata;
    }
    let Ok(value) = PlistValue::from_reader(Cursor::new(header)) else {
        return metadata;
    };

    for (source_key, metadata_key) in [
        ("IOPlatformSerialNumber", "system.serialNumber"),
        ("SerialNumber", "system.serialNumber"),
        ("serial_number", "system.serialNumber"),
        ("IOPlatformUUID", "system.hardwareUuid"),
        ("HardwareUUID", "system.hardwareUuid"),
        ("platform_UUID", "system.hardwareUuid"),
        ("machine_name", "system.model"),
        ("machine_model", "system.modelIdentifier"),
        ("boot_rom_version", "system.bootRomVersion"),
        ("smc_version_system", "system.smcVersion"),
        ("cpu_type", "system.cpuType"),
        ("current_processor_speed", "system.processorSpeed"),
    ] {
        if metadata.contains_key(metadata_key) {
            continue;
        }
        if let Some(value) = plist_find_scalar_string(&value, source_key) {
            metadata.insert(metadata_key.to_string(), value);
        }
    }

    if metadata.is_empty() {
        return metadata;
    }
    metadata.insert("system.osFamily".to_string(), "macos".to_string());
    metadata.insert(
        "system.infoType".to_string(),
        "hardware-identity".to_string(),
    );
    metadata
}

fn plist_find_scalar_string(value: &PlistValue, wanted_key: &str) -> Option<String> {
    match value {
        PlistValue::Dictionary(dictionary) => {
            if let Some(value) = dictionary.get(wanted_key).and_then(plist_scalar_string) {
                return Some(value);
            }
            dictionary
                .values()
                .find_map(|child| plist_find_scalar_string(child, wanted_key))
        }
        PlistValue::Array(items) => items
            .iter()
            .find_map(|child| plist_find_scalar_string(child, wanted_key)),
        _ => None,
    }
}

fn plist_find_array<'a>(value: &'a PlistValue, wanted_key: &str) -> Option<&'a Vec<PlistValue>> {
    match value {
        PlistValue::Dictionary(dictionary) => {
            if let Some(PlistValue::Array(value)) = dictionary.get(wanted_key) {
                return Some(value);
            }
            dictionary
                .values()
                .find_map(|child| plist_find_array(child, wanted_key))
        }
        PlistValue::Array(items) => items
            .iter()
            .find_map(|child| plist_find_array(child, wanted_key)),
        _ => None,
    }
}

fn plist_dict_string<'a>(dictionary: &'a plist::Dictionary, key: &str) -> Option<&'a str> {
    dictionary.get(key).and_then(PlistValue::as_string)
}

fn plist_dict_first_scalar_string(dictionary: &plist::Dictionary, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| dictionary.get(key).and_then(plist_scalar_string))
}

fn plist_dict_string_array(dictionary: &plist::Dictionary, key: &str) -> Option<Vec<String>> {
    let PlistValue::Array(values) = dictionary.get(key)? else {
        return None;
    };
    let strings: Vec<String> = values
        .iter()
        .filter_map(PlistValue::as_string)
        .map(ToString::to_string)
        .take(32)
        .collect();
    (!strings.is_empty()).then_some(strings)
}

fn plist_dict_bool(dictionary: &plist::Dictionary, key: &str) -> Option<bool> {
    match dictionary.get(key)? {
        PlistValue::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn plist_dict_date(dictionary: &plist::Dictionary, key: &str) -> Option<String> {
    let PlistValue::Date(value) = dictionary.get(key)? else {
        return None;
    };
    Some(value.to_xml_format())
}

fn plist_dict_integer(dictionary: &plist::Dictionary, key: &str) -> Option<i64> {
    match dictionary.get(key)? {
        PlistValue::Integer(value) => value.as_signed(),
        _ => None,
    }
}

fn plist_dict_array_len(dictionary: &plist::Dictionary, key: &str) -> Option<usize> {
    let PlistValue::Array(values) = dictionary.get(key)? else {
        return None;
    };
    Some(values.len())
}

fn insert_plist_bool_metadata(
    metadata: &mut BTreeMap<String, String>,
    dictionary: &plist::Dictionary,
    plist_key: &str,
    metadata_key: &str,
) {
    if let Some(value) = plist_dict_bool(dictionary, plist_key) {
        metadata.insert(metadata_key.to_string(), value.to_string());
    }
}

fn insert_plist_integer_metadata(
    metadata: &mut BTreeMap<String, String>,
    dictionary: &plist::Dictionary,
    plist_key: &str,
    metadata_key: &str,
) {
    if let Some(value) = plist_dict_integer(dictionary, plist_key) {
        metadata.insert(metadata_key.to_string(), value.to_string());
    }
}

fn plist_data_utf8_string(value: &PlistValue) -> Option<String> {
    let PlistValue::Data(data) = value else {
        return None;
    };
    String::from_utf8(data.clone())
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn plist_data_mac_address(value: &PlistValue) -> Option<String> {
    let PlistValue::Data(data) = value else {
        return None;
    };
    format_mac_address(data)
}

fn format_mac_address(data: &[u8]) -> Option<String> {
    if data.len() != 6 {
        return None;
    }
    Some(
        data.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}

fn describe_macos_network_interface(
    bsd_name: Option<&str>,
    display_name: Option<&str>,
    interface_type: Option<&str>,
    mac_address: Option<&str>,
) -> Option<String> {
    let name = bsd_name.or(display_name)?;
    let mut parts = Vec::new();
    if let Some(display_name) = display_name.filter(|value| Some(*value) != bsd_name) {
        parts.push(display_name.to_string());
    }
    if let Some(interface_type) = interface_type {
        parts.push(interface_type.to_string());
    }
    if let Some(mac_address) = mac_address {
        parts.push(mac_address.to_string());
    }

    if parts.is_empty() {
        Some(name.to_string())
    } else {
        Some(format!("{} ({})", name, parts.join(", ")))
    }
}

fn system_info_type_description(metadata: &BTreeMap<String, String>) -> String {
    match (
        metadata.get("system.osFamily").map(String::as_str),
        metadata.get("system.infoType").map(String::as_str),
    ) {
        (Some("linux"), Some("os-release")) => "Linux OS Release Info",
        (Some("linux"), Some("hostname")) => "Linux Hostname",
        (Some("linux"), Some("machine-id")) => "Linux Machine ID",
        (Some("linux"), Some("machine-info")) => "Linux Machine Information",
        (Some("linux"), Some("locale")) => "Linux Locale Configuration",
        (Some("linux"), Some("cpuinfo")) => "Linux CPU Information",
        (Some("linux"), Some("meminfo")) => "Linux Memory Information",
        (Some("linux"), Some("network-config")) => "Linux Network Configuration",
        (Some("linux"), Some("firewall")) => "Linux Firewall Configuration",
        (Some("linux"), Some("dmi")) => "Linux DMI System Information",
        (Some("unix"), Some("account-config")) => "Unix Account Configuration",
        (Some("unix"), Some("timezone")) => "Unix Time Zone Configuration",
        (Some("unix"), Some("mount-table")) => "Unix Mount Table",
        (Some("macos"), Some("hardware-identity")) => "macOS Hardware Identity",
        (Some("macos"), Some("system-version")) => "macOS System Version Info",
        (Some("macos"), Some("system-identity")) => "macOS System Identity",
        (Some("macos"), Some("network-interfaces")) => "macOS Network Interfaces",
        (Some("macos"), Some("wifi-preferences")) => "macOS Wi-Fi Preferences",
        (Some("macos"), Some("disk-management")) => "macOS Disk Management",
        (Some("macos"), Some("firewall")) => "macOS Firewall Preferences",
        (Some("macos"), Some("install-history")) => "macOS Install History",
        (Some("windows"), Some("registry-hive")) => "Windows Registry System Information",
        (Some("windows"), Some("wifi-profile")) => "Windows Wi-Fi Profile",
        (Some("windows"), Some("firewall")) => "Windows Firewall Log",
        (Some("windows"), Some("setup-log")) => "Windows Setup Log",
        _ => "System Information Artifact",
    }
    .to_string()
}

#[derive(Default)]
struct CommandHistorySummary {
    command_count: usize,
    command_names: Vec<String>,
    network_command_count: usize,
    privileged_command_count: usize,
    file_transfer_command_count: usize,
}

fn activity_metadata(source_id: &str, header: &[u8]) -> BTreeMap<String, String> {
    let normalized_path = normalize_artifact_path(source_id);
    if !is_command_history_path(&normalized_path) {
        return BTreeMap::new();
    }
    command_history_metadata(&normalized_path, header)
}

fn is_command_history_path(path: &str) -> bool {
    path.ends_with("/.bash_history")
        || path.ends_with("/.zsh_history")
        || path.ends_with("/consolehost_history.txt")
}

fn command_history_metadata(path: &str, header: &[u8]) -> BTreeMap<String, String> {
    let history_type = if path.ends_with("/consolehost_history.txt") {
        "powershell"
    } else if path.ends_with("/.zsh_history") {
        "zsh"
    } else {
        "bash"
    };
    let text = String::from_utf8_lossy(header);
    let mut summary = CommandHistorySummary::default();

    for line in text.lines().take(2048) {
        let Some(command) = normalized_history_command(line, history_type) else {
            continue;
        };
        summary.command_count = summary.command_count.saturating_add(1);
        collect_command_history_summary(command, &mut summary);
    }

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "activity.commandHistoryType".to_string(),
        history_type.to_string(),
    );
    if summary.command_count > 0 {
        metadata.insert(
            "activity.commandCount".to_string(),
            summary.command_count.to_string(),
        );
    }
    if summary.network_command_count > 0 {
        metadata.insert(
            "activity.networkCommandCount".to_string(),
            summary.network_command_count.to_string(),
        );
    }
    if summary.privileged_command_count > 0 {
        metadata.insert(
            "activity.privilegedCommandCount".to_string(),
            summary.privileged_command_count.to_string(),
        );
    }
    if summary.file_transfer_command_count > 0 {
        metadata.insert(
            "activity.fileTransferCommandCount".to_string(),
            summary.file_transfer_command_count.to_string(),
        );
    }
    insert_limited_system_values(
        &mut metadata,
        "activity.commandNames",
        &summary.command_names,
    );
    metadata
}

fn normalized_history_command<'a>(line: &'a str, history_type: &str) -> Option<&'a str> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if history_type == "zsh" && line.starts_with(": ") {
        return line.split_once(';').map(|(_, command)| command.trim());
    }
    Some(line)
}

fn collect_command_history_summary(command: &str, summary: &mut CommandHistorySummary) {
    let Some(command_name) = command_history_command_name(command) else {
        return;
    };
    let command_name_lower = command_name.to_ascii_lowercase();
    push_limited_system_value(&mut summary.command_names, &command_name);

    if matches!(command_name_lower.as_str(), "sudo" | "su" | "runas") {
        summary.privileged_command_count = summary.privileged_command_count.saturating_add(1);
    }
    if is_network_command(&command_name_lower) {
        summary.network_command_count = summary.network_command_count.saturating_add(1);
    }
    if is_file_transfer_command(&command_name_lower) {
        summary.file_transfer_command_count = summary.file_transfer_command_count.saturating_add(1);
    }
}

fn command_history_command_name(command: &str) -> Option<String> {
    let mut first = command
        .split_whitespace()
        .next()?
        .trim_matches(|ch| matches!(ch, '"' | '\'' | '&' | ';' | '(' | ')'));
    if first.is_empty() {
        return None;
    }
    if let Some(last) = first.rsplit(['/', '\\']).next() {
        first = last;
    }
    (!first.is_empty()).then(|| first.to_string())
}

fn is_network_command(command_name: &str) -> bool {
    matches!(
        command_name,
        "ssh"
            | "scp"
            | "sftp"
            | "curl"
            | "wget"
            | "nc"
            | "ncat"
            | "netcat"
            | "ftp"
            | "rsync"
            | "invoke-webrequest"
            | "iwr"
            | "invoke-restmethod"
            | "irm"
    )
}

fn is_file_transfer_command(command_name: &str) -> bool {
    matches!(
        command_name,
        "scp"
            | "sftp"
            | "curl"
            | "wget"
            | "ftp"
            | "rsync"
            | "invoke-webrequest"
            | "iwr"
            | "invoke-restmethod"
            | "irm"
    )
}

fn activity_type_description(metadata: &BTreeMap<String, String>) -> String {
    match metadata
        .get("activity.commandHistoryType")
        .map(String::as_str)
    {
        Some("powershell") => "PowerShell Command History",
        Some("zsh") => "Zsh Command History",
        Some("bash") => "Bash Command History",
        _ => "Activity Artifact",
    }
    .to_string()
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
        VfsEntryByteSource,
    };
    use crate::vfs::{DirEntry, FileAttr, VfsError, VirtualFileSystem};
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::Arc;

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

    struct InMemoryVfs {
        files: HashMap<String, Vec<u8>>,
    }

    impl InMemoryVfs {
        fn new(files: &[(&str, &[u8])]) -> Self {
            Self {
                files: files
                    .iter()
                    .map(|(path, bytes)| ((*path).to_string(), (*bytes).to_vec()))
                    .collect(),
            }
        }
    }

    impl VirtualFileSystem for InMemoryVfs {
        fn getattr(&self, path: &str) -> Result<FileAttr, VfsError> {
            if path == "/" {
                return Ok(FileAttr::directory());
            }

            self.files
                .get(path)
                .map(|bytes| FileAttr::file(bytes.len() as u64))
                .ok_or_else(|| VfsError::NotFound(path.to_string()))
        }

        fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, VfsError> {
            if path != "/" {
                return Err(VfsError::NotADirectory(path.to_string()));
            }

            Ok(self
                .files
                .keys()
                .filter_map(|file_path| file_path.trim_start_matches('/').split('/').next())
                .map(|name| DirEntry::new(name, false))
                .collect())
        }

        fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, VfsError> {
            let bytes = self
                .files
                .get(path)
                .ok_or_else(|| VfsError::NotFound(path.to_string()))?;
            let start = usize::try_from(offset).map_err(|_| VfsError::OutOfBounds {
                offset,
                size: bytes.len(),
            })?;
            if start >= bytes.len() {
                return Ok(Vec::new());
            }
            let end = start.saturating_add(size).min(bytes.len());
            Ok(bytes[start..end].to_vec())
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

    #[test]
    fn extracts_system_artifact_from_forensic_image_vfs_entry() {
        let passwd = br#"root:x:0:0:root:/root:/bin/bash
alice:x:1000:1000:Alice Analyst:/home/alice:/bin/bash
"#;
        let vfs = Arc::new(InMemoryVfs::new(&[("/etc/passwd", passwd)]));
        let source = VfsEntryByteSource::new(
            vfs,
            "/cases/workstation.E01",
            "/etc/passwd",
            Some("ewf".to_string()),
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Account Configuration");
        assert_eq!(artifact.name, "passwd");
        assert_eq!(
            artifact.source_ref,
            EvidenceSourceRef::VfsEntry {
                container_path: "/cases/workstation.E01".to_string(),
                entry_path: "/etc/passwd".to_string(),
                container_type: Some("ewf".to_string()),
            }
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.accountConfigType")
                .map(String::as_str),
            Some("unix-passwd")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.regularUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.loginUserCount")
                .map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn extracts_windows_registry_identity_from_forensic_image_vfs_entry() {
        let registry = windows_registry_hive_with_utf16_strings(&[
            "ComputerName",
            "CORE-LAB01",
            "SystemManufacturer",
            "Dell Inc.",
            "SystemProductName",
            "Latitude 7490",
            "SystemSerialNumber",
            "ABC12345",
        ]);
        let vfs = Arc::new(InMemoryVfs::new(&[(
            "/Windows/System32/config/SYSTEM",
            registry.as_slice(),
        )]));
        let source = VfsEntryByteSource::new(
            vfs,
            "/cases/windows.E01",
            "/Windows/System32/config/SYSTEM",
            Some("ewf".to_string()),
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(
            artifact.type_description,
            "Windows Registry System Information"
        );
        assert_eq!(
            artifact
                .metadata
                .get("windows.registryHive")
                .map(String::as_str),
            Some("system")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.computerName")
                .map(String::as_str),
            Some("CORE-LAB01")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.manufacturer")
                .map(String::as_str),
            Some("Dell Inc.")
        );
        assert_eq!(
            artifact.metadata.get("system.model").map(String::as_str),
            Some("Latitude 7490")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.serialNumber")
                .map(String::as_str),
            Some("ABC12345")
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

    fn windows_registry_hive_with_utf16_strings(strings: &[&str]) -> Vec<u8> {
        let mut bytes = vec![0u8; 8192];
        bytes[..4].copy_from_slice(b"regf");
        bytes[0x04..0x08].copy_from_slice(&8u32.to_le_bytes());
        bytes[0x08..0x0c].copy_from_slice(&8u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x18..0x1c].copy_from_slice(&5u32.to_le_bytes());
        bytes[0x28..0x2c].copy_from_slice(&4096u32.to_le_bytes());

        let mut offset = 6000;
        for value in strings {
            for unit in value.encode_utf16() {
                bytes[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
                offset += 2;
            }
            offset += 4;
        }
        bytes
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

    fn make_minimal_pe_driver_header() -> Vec<u8> {
        let pe_offset = 0x80usize;
        let mut bytes = vec![0u8; pe_offset + 0x100];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
        bytes[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
        bytes.extend_from_slice(b"ntoskrnl.exe\0FltRegisterFilter\0DriverEntry\0");
        bytes.extend_from_slice(br"\Registry\Machine\System\CurrentControlSet\Services\contosoflt");
        bytes.push(0);
        bytes.extend_from_slice(
            br"\Registry\Machine\System\ControlSet001\Services\legacyflt\Parameters",
        );
        bytes.push(0);
        bytes.extend_from_slice(br"\Device\ContosoFilter");
        bytes.push(0);
        bytes.extend_from_slice(br"\DosDevices\ContosoFilter");
        bytes.push(0);
        bytes.extend_from_slice(br"C:\agent\_work\drivers\contosoflt\objfre\amd64\contosoflt.pdb");
        bytes.push(0);
        bytes.extend_from_slice(b"https://drivers.example.test/support");
        bytes.push(0);
        bytes.extend_from_slice(b"{12345678-9abc-def0-1234-56789abcdef0}");
        bytes.push(0);
        append_utf16le_version_pair(&mut bytes, "CompanyName", "Contoso Driver Labs");
        append_utf16le_version_pair(&mut bytes, "FileDescription", "Contoso File Filter");
        append_utf16le_version_pair(&mut bytes, "FileVersion", "1.2.3.4");
        append_utf16le_version_pair(&mut bytes, "OriginalFilename", "contosoflt.sys");
        bytes
    }

    fn append_utf16le_version_pair(bytes: &mut Vec<u8>, key: &str, value: &str) {
        for unit in key.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
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
    fn extracts_linux_os_release_system_info_metadata() {
        let bytes = br#"NAME="Ubuntu"
VERSION_ID="24.04"
PRETTY_NAME="Ubuntu 24.04.2 LTS"
ID=ubuntu
BUILD_ID=20260201
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/os-release", bytes, 17);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux OS Release Info");
        assert_eq!(
            artifact.metadata.get("system.osFamily").map(String::as_str),
            Some("linux")
        );
        assert_eq!(
            artifact.metadata.get("os.release.name").map(String::as_str),
            Some("Ubuntu 24.04.2 LTS")
        );
        assert_eq!(
            artifact.metadata.get("os.release.id").map(String::as_str),
            Some("ubuntu")
        );
        assert_eq!(
            artifact
                .metadata
                .get("os.release.versionId")
                .map(String::as_str),
            Some("24.04")
        );
    }

    #[test]
    fn extracts_linux_hostname_and_machine_id_metadata() {
        let hostname_source = ChunkedByteSource::new("/mnt/image/etc/hostname", b"labhost\n", 3);
        let machine_id_source = ChunkedByteSource::new(
            "/mnt/image/var/lib/dbus/machine-id",
            b"0123456789abcdef0123456789abcdef\n",
            8,
        );

        let hostname_artifact =
            extract_normalized_artifact(&hostname_source, ArtifactExtractionOptions::default())
                .unwrap();
        let machine_id_artifact =
            extract_normalized_artifact(&machine_id_source, ArtifactExtractionOptions::default())
                .unwrap();

        assert_eq!(hostname_artifact.category, "systeminfo");
        assert_eq!(
            hostname_artifact
                .metadata
                .get("system.hostname")
                .map(String::as_str),
            Some("labhost")
        );
        assert_eq!(machine_id_artifact.category, "systeminfo");
        assert_eq!(
            machine_id_artifact
                .metadata
                .get("system.machineId")
                .map(String::as_str),
            Some("0123456789abcdef0123456789abcdef")
        );
    }

    #[test]
    fn extracts_linux_machine_info_metadata() {
        let bytes = br#"PRETTY_HOSTNAME="Case Workstation"
ICON_NAME=computer-desktop
CHASSIS=desktop
DEPLOYMENT=production
LOCATION="Lab 3"
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/machine-info", bytes, 19);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux Machine Information");
        assert_eq!(
            artifact
                .metadata
                .get("system.prettyHostname")
                .map(String::as_str),
            Some("Case Workstation")
        );
        assert_eq!(
            artifact.metadata.get("system.chassis").map(String::as_str),
            Some("desktop")
        );
        assert_eq!(
            artifact.metadata.get("system.location").map(String::as_str),
            Some("Lab 3")
        );
    }

    #[test]
    fn extracts_linux_locale_metadata() {
        let bytes = br#"LANG=en_US.UTF-8
LANGUAGE=en_US:en
LC_TIME=en_GB.UTF-8
LC_NUMERIC=C
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/default/locale", bytes, 128);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux Locale Configuration");
        assert_eq!(
            artifact.metadata.get("system.locale").map(String::as_str),
            Some("en_US.UTF-8")
        );
        assert_eq!(
            artifact.metadata.get("system.language").map(String::as_str),
            Some("en_US:en")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localeTime")
                .map(String::as_str),
            Some("en_GB.UTF-8")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localeNumeric")
                .map(String::as_str),
            Some("C")
        );
    }

    #[test]
    fn extracts_unix_timezone_metadata() {
        let timezone_source =
            ChunkedByteSource::new("/mnt/image/etc/timezone", b"America/Anchorage\n", 32);
        let localtime_source = ChunkedByteSource::new(
            "/mnt/image/etc/localtime",
            b"/usr/share/zoneinfo/America/Anchorage\n",
            64,
        );

        let timezone_artifact =
            extract_normalized_artifact(&timezone_source, ArtifactExtractionOptions::default())
                .unwrap();
        let localtime_artifact =
            extract_normalized_artifact(&localtime_source, ArtifactExtractionOptions::default())
                .unwrap();

        assert_eq!(timezone_artifact.category, "systeminfo");
        assert_eq!(
            timezone_artifact.type_description,
            "Unix Time Zone Configuration"
        );
        assert_eq!(
            timezone_artifact
                .metadata
                .get("system.timeZone")
                .map(String::as_str),
            Some("America/Anchorage")
        );
        assert_eq!(localtime_artifact.category, "systeminfo");
        assert_eq!(
            localtime_artifact
                .metadata
                .get("system.timeZone")
                .map(String::as_str),
            Some("America/Anchorage")
        );
    }

    #[test]
    fn extracts_unix_mount_table_metadata() {
        let bytes = br#"# /etc/fstab
UUID=root-uuid / ext4 defaults 0 1
/dev/disk/by-label/Case\040Data /mnt/case\040data xfs ro,nosuid 0 0
tmpfs /run tmpfs rw,nosuid,nodev 0 0
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/fstab", bytes, 512);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Mount Table");
        assert_eq!(
            artifact
                .metadata
                .get("system.rootDevice")
                .map(String::as_str),
            Some("UUID=root-uuid")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.mountCount")
                .map(String::as_str),
            Some("3")
        );
        let mounts = artifact
            .metadata
            .get("system.mounts")
            .expect("mount descriptions are captured");
        assert!(mounts.contains("UUID=root-uuid on / (ext4, defaults)"));
        assert!(mounts.contains("/dev/disk/by-label/Case Data on /mnt/case data (xfs, ro,nosuid)"));
    }

    #[test]
    fn extracts_linux_cpuinfo_metadata() {
        let bytes = br#"processor   : 0
vendor_id   : GenuineIntel
cpu cores   : 8
model name  : Intel(R) Core(TM) i7-1185G7
flags       : fpu vme de pse tsc

processor   : 1
vendor_id   : GenuineIntel
cpu cores   : 8
model name  : Intel(R) Core(TM) i7-1185G7
flags       : fpu vme de pse tsc
"#;
        let source = ChunkedByteSource::new("/mnt/image/proc/cpuinfo", bytes, 64);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux CPU Information");
        assert_eq!(
            artifact.metadata.get("system.osFamily").map(String::as_str),
            Some("linux")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.cpuLogicalProcessorCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.cpuModels")
                .map(String::as_str),
            Some("Intel(R) Core(TM) i7-1185G7")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.cpuVendors")
                .map(String::as_str),
            Some("GenuineIntel")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.cpuCoreCounts")
                .map(String::as_str),
            Some("8")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.cpuFeatures")
                .map(String::as_str),
            Some("fpu; vme; de; pse; tsc")
        );
    }

    #[test]
    fn extracts_linux_meminfo_metadata() {
        let bytes = br#"MemTotal:       32768000 kB
MemFree:         1024000 kB
"#;
        let source = ChunkedByteSource::new("/mnt/image/proc/meminfo", bytes, usize::MAX);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux Memory Information");
        assert_eq!(
            artifact
                .metadata
                .get("system.memoryTotalKiB")
                .map(String::as_str),
            Some("32768000")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.memoryTotalBytes")
                .map(String::as_str),
            Some("33554432000")
        );
    }

    #[test]
    fn extracts_linux_debian_network_interfaces_metadata() {
        let bytes = br#"# primary interface
auto lo eth0
iface lo inet loopback
iface eth0 inet static
    address 192.168.10.5/24
    gateway 192.168.10.1
    dns-nameservers 1.1.1.1 8.8.8.8
    hwaddress ether aa:bb:cc:dd:ee:ff
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/network/interfaces", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux Network Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("debian-interfaces")
        );
        assert!(artifact
            .metadata
            .get("system.networkInterfaces")
            .is_some_and(|value| value.contains("eth0")));
        assert_eq!(
            artifact
                .metadata
                .get("system.ipv4Addresses")
                .map(String::as_str),
            Some("192.168.10.5/24")
        );
        assert_eq!(
            artifact.metadata.get("system.gateways").map(String::as_str),
            Some("192.168.10.1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.dnsServers")
                .map(String::as_str),
            Some("1.1.1.1; 8.8.8.8")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.macAddresses")
                .map(String::as_str),
            Some("aa:bb:cc:dd:ee:ff")
        );
    }

    #[test]
    fn extracts_linux_resolver_and_hosts_metadata() {
        let resolv_source = ChunkedByteSource::new(
            "/mnt/image/etc/resolv.conf",
            br#"search corp.example.com lab.example.com
nameserver 10.10.0.2
nameserver 1.1.1.1
"#,
            128,
        );
        let hosts_source = ChunkedByteSource::new(
            "/mnt/image/etc/hosts",
            br#"127.0.0.1 localhost
10.10.0.25 workstation01 workstation01.corp.example.com
"#,
            128,
        );

        let resolv_artifact =
            extract_normalized_artifact(&resolv_source, ArtifactExtractionOptions::default())
                .unwrap();
        let hosts_artifact =
            extract_normalized_artifact(&hosts_source, ArtifactExtractionOptions::default())
                .unwrap();

        assert_eq!(
            resolv_artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("resolver")
        );
        assert_eq!(
            resolv_artifact
                .metadata
                .get("system.dnsServers")
                .map(String::as_str),
            Some("10.10.0.2; 1.1.1.1")
        );
        assert_eq!(
            resolv_artifact
                .metadata
                .get("system.dnsSearchDomains")
                .map(String::as_str),
            Some("corp.example.com; lab.example.com")
        );
        assert_eq!(
            hosts_artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("hosts")
        );
        let aliases = hosts_artifact
            .metadata
            .get("system.hostAliases")
            .expect("host aliases are captured");
        assert!(aliases.contains("127.0.0.1=localhost"));
        assert!(aliases.contains("10.10.0.25=workstation01,workstation01.corp.example.com"));
    }

    #[test]
    fn extracts_linux_network_manager_metadata() {
        let bytes = br#"[connection]
id=Corp WiFi
uuid=11111111-2222-3333-4444-555555555555
type=wifi
interface-name=wlp2s0

[wifi]
ssid=CorpNet
mac-address=aa:bb:cc:dd:ee:ff

[ipv4]
method=manual
address1=192.168.50.25/24,192.168.50.1
dns=10.10.0.2;1.1.1.1;
dns-search=corp.example.com;
"#;
        let source = ChunkedByteSource::new(
            "/mnt/image/etc/NetworkManager/system-connections/Corp WiFi.nmconnection",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(
            artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("networkmanager")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.connectionIds")
                .map(String::as_str),
            Some("Corp WiFi")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.connectionUuids")
                .map(String::as_str),
            Some("11111111-2222-3333-4444-555555555555")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.networkInterfaces")
                .map(String::as_str),
            Some("wlp2s0")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSsids")
                .map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.ipv4Addresses")
                .map(String::as_str),
            Some("192.168.50.25/24")
        );
        assert_eq!(
            artifact.metadata.get("system.gateways").map(String::as_str),
            Some("192.168.50.1")
        );
    }

    #[test]
    fn extracts_linux_netplan_metadata() {
        let bytes = br#"network:
  version: 2
  ethernets:
    ens18:
      dhcp4: false
      addresses: [172.16.1.5/24]
      gateway4: 172.16.1.1
      nameservers:
        addresses: [1.1.1.1, 8.8.4.4]
"#;
        let source = ChunkedByteSource::new("/mnt/image/etc/netplan/01-netcfg.yaml", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(
            artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("netplan")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.networkInterfaces")
                .map(String::as_str),
            Some("ens18")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.ipv4Addresses")
                .map(String::as_str),
            Some("172.16.1.5/24")
        );
        assert_eq!(
            artifact.metadata.get("system.gateways").map(String::as_str),
            Some("172.16.1.1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.dnsServers")
                .map(String::as_str),
            Some("1.1.1.1; 8.8.4.4")
        );
    }

    #[test]
    fn extracts_windows_wifi_profile_metadata() {
        let bytes = br#"<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
  <name>CorpNet Profile</name>
  <SSIDConfig>
    <SSID>
      <hex>436F72704E6574</hex>
      <name>CorpNet</name>
    </SSID>
  </SSIDConfig>
  <connectionType>ESS</connectionType>
  <connectionMode>auto</connectionMode>
  <MSM>
    <security>
      <authEncryption>
        <authentication>WPA2PSK</authentication>
        <encryption>AES</encryption>
        <useOneX>false</useOneX>
      </authEncryption>
    </security>
  </MSM>
</WLANProfile>
"#;
        let source = ChunkedByteSource::new(
            "/image/ProgramData/Microsoft/Wlansvc/Profiles/Interfaces/{iface}/{profile}.xml",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Windows Wi-Fi Profile");
        assert_eq!(
            artifact
                .metadata
                .get("system.networkConfigType")
                .map(String::as_str),
            Some("windows-wlan-profile")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.connectionIds")
                .map(String::as_str),
            Some("CorpNet Profile")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSsids")
                .map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiAuthTypes")
                .map(String::as_str),
            Some("WPA2PSK")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiEncryptionTypes")
                .map(String::as_str),
            Some("AES")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.networkConnectionModes")
                .map(String::as_str),
            Some("auto")
        );
    }

    #[test]
    fn extracts_unix_passwd_account_metadata() {
        let bytes = br#"root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
alice:x:1000:1000:Alice Analyst:/home/alice:/bin/bash
bob:x:1001:1001:Bob User:/home/bob:/bin/zsh
"#;
        let source = ChunkedByteSource::new("/image/etc/passwd", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Account Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.accountConfigType")
                .map(String::as_str),
            Some("unix-passwd")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localUserCount")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.regularUserCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.loginUserCount")
                .map(String::as_str),
            Some("3")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.rootAccountPresent")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.userUidRange")
                .map(String::as_str),
            Some("0-1001")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.regularUsers")
                .map(String::as_str),
            Some("alice (Alice Analyst); bob (Bob User)")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.homeDirectories")
                .map(String::as_str),
            Some("/root; /usr/sbin; /home/alice; /home/bob")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.loginShells")
                .map(String::as_str),
            Some("/bin/bash; /bin/zsh")
        );
    }

    #[test]
    fn extracts_unix_group_account_metadata() {
        let bytes = br#"root:x:0:
wheel:x:10:root,alice
sudo:x:27:alice,bob
users:x:100:alice,bob
"#;
        let source = ChunkedByteSource::new("/image/etc/group", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Account Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.accountConfigType")
                .map(String::as_str),
            Some("unix-group")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localGroupCount")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localGroups")
                .map(String::as_str),
            Some("root:gid=0; wheel:gid=10; sudo:gid=27; users:gid=100")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.adminGroups")
                .map(String::as_str),
            Some("wheel:members=root,alice; sudo:members=alice,bob")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.groupMembers")
                .map(String::as_str),
            Some("wheel=root,alice; sudo=alice,bob; users=alice,bob")
        );
    }

    #[test]
    fn extracts_unix_shadow_account_metadata_without_hash_values() {
        let bytes = br#"root:$6$salt$abcdef:19500:0:99999:7:::
alice:$y$j9T$salt$hash:19501:0:99999:7:::
bob:!:19502:0:99999:7:::
daemon:*:19503:0:99999:7:::
test::19504:0:99999:7:::
"#;
        let source = ChunkedByteSource::new("/image/etc/shadow", bytes, 512);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Account Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.accountConfigType")
                .map(String::as_str),
            Some("unix-shadow")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.shadowEntryCount")
                .map(String::as_str),
            Some("5")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordHashUserCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordLockedUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordDisabledUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordEmptyUserCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordHashUsers")
                .map(String::as_str),
            Some("root; alice")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.passwordHashAlgorithms")
                .map(String::as_str),
            Some("sha512-crypt; yescrypt")
        );
        assert!(!artifact
            .metadata
            .values()
            .any(|value| value.contains("$6$salt") || value.contains("$y$j9T")));
    }

    #[test]
    fn extracts_unix_gshadow_account_metadata() {
        let bytes = br#"root:*::
wheel:!:root:alice
sudo:!:alice:bob,carol
users:!::alice,bob
"#;
        let source = ChunkedByteSource::new("/image/etc/gshadow", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Unix Account Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.accountConfigType")
                .map(String::as_str),
            Some("unix-gshadow")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localGroupCount")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localGroups")
                .map(String::as_str),
            Some("root; wheel; sudo; users")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.adminGroups")
                .map(String::as_str),
            Some("wheel:admins=root; sudo:admins=alice")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.groupMembers")
                .map(String::as_str),
            Some("wheel=alice; sudo=bob,carol; users=alice,bob")
        );
    }

    #[test]
    fn extracts_bash_command_history_metadata() {
        let bytes = br#"ls -la
sudo cat /etc/shadow
ssh admin@example.com
curl -O https://example.com/tool
"#;
        let source = ChunkedByteSource::new("/image/home/alice/.bash_history", bytes, 256);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "activity");
        assert_eq!(artifact.type_description, "Bash Command History");
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("bash")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandCount")
                .map(String::as_str),
            Some("4")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.networkCommandCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.privilegedCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.fileTransferCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandNames")
                .map(String::as_str),
            Some("ls; sudo; ssh; curl")
        );
    }

    #[test]
    fn extracts_zsh_extended_history_metadata() {
        let bytes = br#": 1717260000:0;git status
: 1717260001:0;rsync -av /a /b
"#;
        let source = ChunkedByteSource::new("/image/Users/alice/.zsh_history", bytes, 128);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "activity");
        assert_eq!(artifact.type_description, "Zsh Command History");
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("zsh")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.fileTransferCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandNames")
                .map(String::as_str),
            Some("git; rsync")
        );
    }

    #[test]
    fn extracts_powershell_command_history_metadata() {
        let bytes = br#"Get-ChildItem C:\
Invoke-WebRequest https://example.com/payload -OutFile payload.bin
runas /user:Administrator cmd
"#;
        let source = ChunkedByteSource::new(
            "/Users/Alice/AppData/Roaming/Microsoft/Windows/PowerShell/PSReadLine/ConsoleHost_history.txt",
            bytes,
            256,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "activity");
        assert_eq!(artifact.type_description, "PowerShell Command History");
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandHistoryType")
                .map(String::as_str),
            Some("powershell")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandCount")
                .map(String::as_str),
            Some("3")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.networkCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.privilegedCommandCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("activity.commandNames")
                .map(String::as_str),
            Some("Get-ChildItem; Invoke-WebRequest; runas")
        );
    }

    #[test]
    fn extracts_iptables_firewall_metadata() {
        let bytes = br#"# sample rules
*filter
:INPUT DROP [0:0]
:FORWARD DROP [0:0]
:OUTPUT ACCEPT [0:0]
-A INPUT -p tcp --dport 22 -j ACCEPT
-A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
COMMIT
"#;
        let source = ChunkedByteSource::new("/image/etc/sysconfig/iptables", bytes, 512);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Linux Firewall Configuration");
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("iptables")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallRuleCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallTables")
                .map(String::as_str),
            Some("filter")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallPolicies")
                .map(String::as_str),
            Some("INPUT:DROP; FORWARD:DROP; OUTPUT:ACCEPT")
        );
    }

    #[test]
    fn extracts_windows_firewall_log_metadata() {
        let bytes = br#"#Version: 1.5
#Fields: date time action protocol src-ip dst-ip src-port dst-port size tcpflags tcpsyn tcpack tcpwin icmptype icmpcode info path
2026-06-01 12:00:00 DROP TCP 10.0.0.5 10.0.0.10 51515 445 60 S 1 0 8192 - - - RECEIVE
2026-06-01 12:00:01 ALLOW UDP 10.0.0.5 8.8.8.8 51516 53 80 - - - - - - - SEND
"#;
        let source = ChunkedByteSource::new(
            "/Windows/System32/LogFiles/Firewall/pfirewall.log",
            bytes,
            512,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Windows Firewall Log");
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("windows-firewall-log")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallLogEntryCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallDroppedCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallAllowedCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallProtocols")
                .map(String::as_str),
            Some("TCP; UDP")
        );
    }

    #[test]
    fn extracts_macos_firewall_preferences_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>globalstate</key><integer>1</integer>
  <key>stealthenabled</key><true/>
  <key>allowsignedenabled</key><false/>
  <key>loggingenabled</key><true/>
  <key>applications</key>
  <array>
    <dict><key>path</key><string>/Applications/Test.app</string></dict>
    <dict><key>path</key><string>/Applications/Other.app</string></dict>
  </array>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Library/Preferences/com.apple.alf.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Firewall Preferences");
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallConfigType")
                .map(String::as_str),
            Some("macos-alf")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallGlobalState")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallStealthEnabled")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallAllowSignedEnabled")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.firewallApplicationRuleCount")
                .map(String::as_str),
            Some("2")
        );
    }

    #[test]
    fn extracts_windows_setup_action_log_metadata() {
        let bytes =
            br#"2026-06-01 12:00:00, Info                  Setup build version: 10.0.26100.1
2026-06-01 12:00:01, Info                  Host OS version: 10.0.22631.3593
2026-06-01 12:00:02, Info                  ComputerName = DESKTOP-CASE01
2026-06-01 12:00:03, Info                  System Manufacturer: Dell Inc.
2026-06-01 12:00:04, Info                  System Product Name: Latitude 7420
2026-06-01 12:00:05, Info                  BIOS Version: 1.32.0
2026-06-01 12:00:06, Info                  Processor Architecture: amd64
"#;
        let source = ChunkedByteSource::new("/Windows/Panther/setupact.log", bytes, 1024);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Windows Setup Log");
        assert_eq!(
            artifact
                .metadata
                .get("system.setupLogType")
                .map(String::as_str),
            Some("setup-action")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupComputerNames")
                .map(String::as_str),
            Some("DESKTOP-CASE01")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupHostOsVersions")
                .map(String::as_str),
            Some("10.0.22631.3593")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupBuildVersions")
                .map(String::as_str),
            Some("10.0.26100.1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupManufacturers")
                .map(String::as_str),
            Some("Dell Inc.")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupModels")
                .map(String::as_str),
            Some("Latitude 7420")
        );
    }

    #[test]
    fn extracts_windows_setupapi_device_log_metadata() {
        let bytes = br#">>>  [Device Install (Hardware initiated) - PCI\VEN_8086&DEV_15F3&SUBSYS_00008086&REV_03]
     dvi:      Device Description: Intel(R) Ethernet Connection
     inf:      Provider: Intel
     inf:      Driver Version: 04/12/2024,1.2.3.4
     inf:      Original Inf Name: oem42.inf
<<<  Section end
"#;
        let source = ChunkedByteSource::new("/Windows/INF/setupapi.dev.log", bytes, 1024);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "Windows Setup Log");
        assert_eq!(
            artifact
                .metadata
                .get("system.setupLogType")
                .map(String::as_str),
            Some("setupapi-dev")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupDeviceInstallCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupDeviceHardwareIds")
                .map(String::as_str),
            Some(r"PCI\VEN_8086&DEV_15F3&SUBSYS_00008086&REV_03")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupDeviceDescriptions")
                .map(String::as_str),
            Some("Intel(R) Ethernet Connection")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupDriverProviders")
                .map(String::as_str),
            Some("Intel")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.setupInfNames")
                .map(String::as_str),
            Some("oem42.inf")
        );
    }

    #[test]
    fn extracts_macos_install_history_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
  <dict>
    <key>date</key><date>2024-01-01T00:00:00Z</date>
    <key>displayName</key><string>macOS Security Response</string>
    <key>displayVersion</key><string>1.0</string>
  </dict>
  <dict>
    <key>date</key><date>2026-06-01T12:34:56Z</date>
    <key>displayName</key><string>macOS Update</string>
    <key>displayVersion</key><string>15.5</string>
    <key>packageIdentifiers</key>
    <array>
      <string>com.apple.pkg.update.os</string>
      <string>com.apple.pkg.update.firmware</string>
    </array>
  </dict>
</array>
</plist>
"#;
        let source = ChunkedByteSource::new("/Library/Receipts/InstallHistory.plist", bytes, 2048);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Install History");
        assert_eq!(
            artifact
                .metadata
                .get("system.installHistoryCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.latestInstallName")
                .map(String::as_str),
            Some("macOS Update")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.latestInstallVersion")
                .map(String::as_str),
            Some("15.5")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.latestInstallDate")
                .map(String::as_str),
            Some("2026-06-01T12:34:56Z")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.latestInstallPackages")
                .map(String::as_str),
            Some("com.apple.pkg.update.os, com.apple.pkg.update.firmware")
        );
    }

    #[test]
    fn extracts_macos_disk_management_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AllDisksAndPartitions</key>
  <array>
    <dict>
      <key>VolumeName</key><string>Macintosh HD</string>
      <key>VolumeUUID</key><string>11111111-2222-3333-4444-555555555555</string>
      <key>BSD Name</key><string>disk3s1</string>
      <key>DAVolumeKind</key><string>apfs</string>
      <key>DAVolumePath</key><string>/</string>
      <key>DAMediaSize</key><integer>512000000000</integer>
    </dict>
    <dict>
      <key>DAVolumeName</key><string>Case Data</string>
      <key>DAVolumeUUID</key><string>aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</string>
      <key>DAMediaBSDName</key><string>disk4s2</string>
      <key>DAMediaContent</key><string>Apple_HFS</string>
      <key>MountPoint</key><string>/Volumes/Case Data</string>
      <key>Size</key><integer>1024</integer>
    </dict>
  </array>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new("/var/db/DiskManagement.plist", bytes, usize::MAX);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Disk Management");
        assert_eq!(
            artifact
                .metadata
                .get("system.volumeCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.volumeNames")
                .map(String::as_str),
            Some("Macintosh HD; Case Data")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.volumeUuids")
                .map(String::as_str),
            Some("11111111-2222-3333-4444-555555555555; aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.diskIdentifiers")
                .map(String::as_str),
            Some("disk3s1; disk4s2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.volumeFilesystems")
                .map(String::as_str),
            Some("apfs; Apple_HFS")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.volumeMounts")
                .map(String::as_str),
            Some("/; /Volumes/Case Data")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.totalVolumeBytes")
                .map(String::as_str),
            Some("512000001024")
        );
        let volumes = artifact
            .metadata
            .get("system.volumes")
            .expect("volume descriptions should be captured");
        assert!(volumes.contains("Macintosh HD (disk3s1, apfs, mounted=/"));
        assert!(volumes.contains("Case Data (disk4s2, Apple_HFS, mounted=/Volumes/Case Data"));
    }

    #[test]
    fn extracts_linux_dmi_manufacturer_model_and_serial_metadata() {
        for (path, metadata_key, expected) in [
            (
                "/mnt/image/sys/class/dmi/id/sys_vendor",
                "system.manufacturer",
                "Dell Inc.",
            ),
            (
                "/mnt/image/sys/class/dmi/id/product_name",
                "system.model",
                "Precision 5680",
            ),
            (
                "/mnt/image/sys/class/dmi/id/product_serial",
                "system.serialNumber",
                "ABC1234",
            ),
            (
                "/mnt/image/sys/devices/virtual/dmi/id/product_uuid",
                "system.uuid",
                "00112233-4455-6677-8899-aabbccddeeff",
            ),
        ] {
            let source = ChunkedByteSource::new(path, format!("{expected}\n").as_bytes(), 4);

            let artifact =
                extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

            assert_eq!(artifact.category, "systeminfo", "{path}");
            assert_eq!(artifact.type_description, "Linux DMI System Information");
            assert_eq!(
                artifact.metadata.get(metadata_key).map(String::as_str),
                Some(expected),
                "{path}"
            );
        }
    }

    #[test]
    fn extracts_macos_system_version_plist_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>ProductName</key><string>macOS</string>
  <key>ProductVersion</key><string>15.5</string>
  <key>ProductBuildVersion</key><string>24F74</string>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Volumes/Macintosh HD/System/Library/CoreServices/SystemVersion.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS System Version Info");
        assert_eq!(
            artifact.metadata.get("system.osFamily").map(String::as_str),
            Some("macos")
        );
        assert_eq!(
            artifact
                .metadata
                .get("os.release.version")
                .map(String::as_str),
            Some("15.5")
        );
        assert_eq!(
            artifact
                .metadata
                .get("os.release.buildId")
                .map(String::as_str),
            Some("24F74")
        );
    }

    #[test]
    fn extracts_macos_preferences_identity_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>System</key>
  <dict>
    <key>System</key>
    <dict>
      <key>ComputerName</key><string>Case MacBook</string>
      <key>HostName</key><string>case-macbook.example.test</string>
      <key>LocalHostName</key><string>Case-MacBook</string>
    </dict>
  </dict>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Volumes/Macintosh HD/Library/Preferences/SystemConfiguration/preferences.plist",
            bytes,
            96,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS System Identity");
        assert_eq!(
            artifact
                .metadata
                .get("system.computerName")
                .map(String::as_str),
            Some("Case MacBook")
        );
        assert_eq!(
            artifact.metadata.get("system.hostname").map(String::as_str),
            Some("case-macbook.example.test")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.localHostname")
                .map(String::as_str),
            Some("Case-MacBook")
        );
    }

    #[test]
    fn extracts_macos_network_interfaces_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Interfaces</key>
  <array>
    <dict>
      <key>BSD Name</key><string>en0</string>
      <key>SCNetworkInterfaceType</key><string>IEEE80211</string>
      <key>SCNetworkInterfaceInfo</key>
      <dict>
        <key>UserDefinedName</key><string>Wi-Fi</string>
      </dict>
      <key>IOMACAddress</key><data>qrvM3e7/</data>
    </dict>
  </array>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Volumes/Macintosh HD/Library/Preferences/SystemConfiguration/NetworkInterfaces.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Network Interfaces");
        assert_eq!(
            artifact
                .metadata
                .get("system.primaryMacAddress")
                .map(String::as_str),
            Some("aa:bb:cc:dd:ee:ff")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.macAddresses")
                .map(String::as_str),
            Some("aa:bb:cc:dd:ee:ff")
        );
        let interfaces = artifact
            .metadata
            .get("system.networkInterfaces")
            .expect("network interface description");
        assert!(interfaces.contains("en0"));
        assert!(interfaces.contains("Wi-Fi"));
        assert!(interfaces.contains("IEEE80211"));
    }

    #[test]
    fn extracts_macos_wifi_preferences_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>KnownNetworks</key>
  <dict>
    <key>wifi.network.ssid.CorpNet</key>
    <dict>
      <key>SSIDString</key><string>CorpNet</string>
      <key>SecurityType</key><string>WPA2 Personal</string>
      <key>AutoJoin</key><true/>
      <key>LastConnected</key><date>2026-06-01T12:34:56Z</date>
    </dict>
    <key>wifi.network.ssid.Guest</key>
    <dict>
      <key>SSIDString</key><string>Guest</string>
      <key>SecurityType</key><string>Open</string>
      <key>AutoJoin</key><false/>
    </dict>
  </dict>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Volumes/Macintosh HD/Library/Preferences/SystemConfiguration/com.apple.airport.preferences.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Wi-Fi Preferences");
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiKnownNetworkCount")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSsids")
                .map(String::as_str),
            Some("CorpNet; Guest")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSecurityTypes")
                .map(String::as_str),
            Some("WPA2 Personal; Open")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiAutoJoinSsids")
                .map(String::as_str),
            Some("CorpNet")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiLastConnected")
                .map(String::as_str),
            Some("CorpNet=2026-06-01T12:34:56Z")
        );
    }

    #[test]
    fn extracts_macos_known_networks_ssid_data_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>KnownNetworks</key>
  <array>
    <dict>
      <key>SSID</key><data>Q2FzZUxhYg==</data>
      <key>SupportedSecurityTypes</key>
      <array>
        <string>WPA3 Personal</string>
        <string>WPA2 Personal</string>
      </array>
      <key>AutoLogin</key><true/>
    </dict>
  </array>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/Volumes/Macintosh HD/Library/Preferences/com.apple.wifi.known-networks.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Wi-Fi Preferences");
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiKnownNetworkCount")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSsids")
                .map(String::as_str),
            Some("CaseLab")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiSecurityTypes")
                .map(String::as_str),
            Some("WPA3 Personal; WPA2 Personal")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.wifiAutoJoinSsids")
                .map(String::as_str),
            Some("CaseLab")
        );
    }

    #[test]
    fn extracts_macos_system_profiler_hardware_identity_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>_items</key>
  <array>
    <dict>
      <key>machine_name</key><string>MacBook Pro</string>
      <key>machine_model</key><string>MacBookPro18,3</string>
      <key>serial_number</key><string>C02TEST12345</string>
      <key>platform_UUID</key><string>00000000-1111-2222-3333-444444444444</string>
      <key>boot_rom_version</key><string>11881.120.56</string>
      <key>smc_version_system</key><string>1.0f0</string>
      <key>cpu_type</key><string>Apple M1 Pro</string>
      <key>current_processor_speed</key><string>3.2 GHz</string>
    </dict>
  </array>
</dict>
</plist>
"#;
        let source =
            ChunkedByteSource::new("/case/SystemProfiler/SPHardwareDataType.plist", bytes, 128);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Hardware Identity");
        assert_eq!(
            artifact.metadata.get("system.osFamily").map(String::as_str),
            Some("macos")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.serialNumber")
                .map(String::as_str),
            Some("C02TEST12345")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.hardwareUuid")
                .map(String::as_str),
            Some("00000000-1111-2222-3333-444444444444")
        );
        assert_eq!(
            artifact.metadata.get("system.model").map(String::as_str),
            Some("MacBook Pro")
        );
        assert_eq!(
            artifact.metadata.get("system.cpuType").map(String::as_str),
            Some("Apple M1 Pro")
        );
    }

    #[test]
    fn extracts_macos_ioplatform_hardware_identity_metadata() {
        let bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>IOPlatformSerialNumber</key><string>FVFTEST98765</string>
  <key>IOPlatformUUID</key><string>aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</string>
  <key>machine_model</key><string>Macmini9,1</string>
</dict>
</plist>
"#;
        let source = ChunkedByteSource::new(
            "/case/IORegistry/IOPlatformExpertDevice.plist",
            bytes,
            usize::MAX,
        );

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(artifact.type_description, "macOS Hardware Identity");
        assert_eq!(
            artifact
                .metadata
                .get("system.serialNumber")
                .map(String::as_str),
            Some("FVFTEST98765")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.hardwareUuid")
                .map(String::as_str),
            Some("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.modelIdentifier")
                .map(String::as_str),
            Some("Macmini9,1")
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
    fn refines_modern_image_artifact_types_from_extension_when_magic_is_short() {
        for (extension, mime_type, description) in [
            (".heic", "image/heic", "HEIC Image"),
            (".heif", "image/heif", "HEIF Image"),
            (".avif", "image/avif", "AVIF Image"),
        ] {
            let file = write_temp_file(extension, b"\0\0\0\x18ftyp");
            let source = LocalFileByteSource::new(file.path());

            let artifact =
                extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

            assert_eq!(artifact.category, "image", "{extension}");
            assert_eq!(
                artifact.mime_type.as_deref(),
                Some(mime_type),
                "{extension}"
            );
            assert_eq!(artifact.type_description, description, "{extension}");
            assert_eq!(artifact.confidence, "medium", "{extension}");
            assert!(
                !artifact.metadata.contains_key("image.width"),
                "{extension}"
            );
            assert!(
                !artifact.metadata.contains_key("image.height"),
                "{extension}"
            );
        }
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
    fn sys_extension_classifies_as_windows_driver_system_artifact() {
        let mut bytes = vec![0u8; 128];
        bytes[0..2].copy_from_slice(b"MZ");
        let file = write_temp_file(".sys", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "system");
        assert_eq!(artifact.type_description, "Windows System Driver");
        assert_eq!(
            artifact.mime_type.as_deref(),
            Some("application/vnd.microsoft.portable-executable")
        );
        assert_eq!(
            artifact.metadata.get("extension").map(String::as_str),
            Some("sys")
        );
    }

    #[test]
    fn sys_driver_artifact_extracts_pe_driver_identity_metadata() {
        let bytes = make_minimal_pe_driver_header();
        let file = write_temp_file(".sys", &bytes);
        let source = LocalFileByteSource::new(file.path());

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "system");
        assert_eq!(
            artifact.metadata.get("pe.format").map(String::as_str),
            Some("portable-executable")
        );
        assert_eq!(
            artifact.metadata.get("pe.isDriver").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            artifact.metadata.get("pe.driverType").map(String::as_str),
            Some("File system minifilter driver")
        );
        assert!(artifact
            .metadata
            .get("pe.driverIndicators")
            .is_some_and(|value| value.contains("file-system filter driver APIs")));
        assert_eq!(
            artifact
                .metadata
                .get("pe.version.CompanyName")
                .map(String::as_str),
            Some("Contoso Driver Labs")
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.version.OriginalFilename")
                .map(String::as_str),
            Some("contosoflt.sys")
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.driverServiceNames")
                .map(String::as_str),
            Some("contosoflt; legacyflt")
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.driverDeviceNames")
                .map(String::as_str),
            Some("ContosoFilter")
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.driverDosDeviceNames")
                .map(String::as_str),
            Some("ContosoFilter")
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.driverRegistryPaths")
                .map(String::as_str),
            Some(
                r"Registry\Machine\System\CurrentControlSet\Services\contosoflt; Registry\Machine\System\ControlSet001\Services\legacyflt\Parameters"
            )
        );
        assert_eq!(
            artifact
                .metadata
                .get("pe.driverPdbPaths")
                .map(String::as_str),
            Some(r"C:\agent\_work\drivers\contosoflt\objfre\amd64\contosoflt.pdb")
        );
        assert_eq!(
            artifact.metadata.get("pe.driverUrls").map(String::as_str),
            Some("https://drivers.example.test/support")
        );
        assert_eq!(
            artifact.metadata.get("pe.driverGuids").map(String::as_str),
            Some("{12345678-9ABC-DEF0-1234-56789ABCDEF0}")
        );
    }

    #[test]
    fn kernel_extension_extensions_classify_as_system_artifacts() {
        for (extension, mime_type, description) in [
            (
                ".ko",
                "application/x-linux-kernel-module",
                "Linux Kernel Module",
            ),
            (
                ".kext",
                "application/x-macos-kernel-extension",
                "macOS Kernel Extension",
            ),
        ] {
            let file = write_temp_file(extension, b"\x7fELF");
            let source = LocalFileByteSource::new(file.path());

            let artifact =
                extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

            assert_eq!(artifact.category, "system", "{extension}");
            assert_eq!(artifact.type_description, description, "{extension}");
            assert_eq!(
                artifact.mime_type.as_deref(),
                Some(mime_type),
                "{extension}"
            );
        }
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
        let source = ChunkedByteSource::new("/image/Windows/System32/config/SAM", &bytes, 257);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(
            artifact.type_description,
            "Windows Registry System Information"
        );
        assert_eq!(
            artifact.mime_type.as_deref(),
            Some("application/x-ms-registry")
        );
        assert_eq!(
            artifact.metadata.get("system.osFamily").map(String::as_str),
            Some("windows")
        );
        assert_eq!(
            artifact
                .metadata
                .get("windows.registryHive")
                .map(String::as_str),
            Some("sam")
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
    fn extracts_windows_software_registry_identity_metadata_beyond_default_header() {
        let bytes = windows_registry_hive_with_utf16_strings(&[
            "ProductName",
            "Windows 11 Pro",
            "DisplayVersion",
            "24H2",
            "CurrentBuildNumber",
            "26100",
            "EditionID",
            "Professional",
            "MachineGuid",
            "00112233-4455-6677-8899-aabbccddeeff",
        ]);
        let source = ChunkedByteSource::new("/image/Windows/System32/config/SOFTWARE", &bytes, 257);

        let artifact =
            extract_normalized_artifact(&source, ArtifactExtractionOptions::default()).unwrap();

        assert_eq!(artifact.category, "systeminfo");
        assert_eq!(
            artifact
                .metadata
                .get("header.bytesRead")
                .map(String::as_str),
            Some("8192")
        );
        assert_eq!(
            artifact
                .metadata
                .get("windows.registryHive")
                .map(String::as_str),
            Some("software")
        );
        assert_eq!(
            artifact.metadata.get("os.release.name").map(String::as_str),
            Some("Windows 11 Pro")
        );
        assert_eq!(
            artifact
                .metadata
                .get("os.release.version")
                .map(String::as_str),
            Some("24H2")
        );
        assert_eq!(
            artifact
                .metadata
                .get("os.release.buildId")
                .map(String::as_str),
            Some("26100")
        );
        assert_eq!(
            artifact
                .metadata
                .get("system.machineGuid")
                .map(String::as_str),
            Some("00112233-4455-6677-8899-aabbccddeeff")
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
