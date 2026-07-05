// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Background indexing worker for the Tantivy search engine.
//!
//! Crawls container file trees (AD1, L01, Archive, VFS/E01/Raw, UFED) and
//! indexes filenames, metadata, and optionally extracted text content.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tracing::{debug, info, warn};

use crate::ad1;
use crate::archive;
use crate::common::{
    extract_normalized_artifact, read_range_fully, ArtifactExtractionOptions, EvidenceByteSource,
    LocalFileByteSource, NormalizedArtifact,
};
use crate::containers::open_container_entry_source;
use crate::ewf;
use crate::raw;
use crate::viewer::document::binary::{analyze_binary_bytes, BinaryFormat, BinaryInfo};

use super::{classify_extension, is_text_eligible, SearchIndex};

// =============================================================================
// Progress & Cancellation
// =============================================================================

/// Progress data emitted during indexing.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    pub container_path: String,
    pub phase: IndexPhase,
    pub files_indexed: u64,
    pub files_total: u64,
    pub content_extracted: u64,
    pub current_file: String,
    pub percent: f64,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IndexPhase {
    Scanning,
    IndexingMetadata,
    ExtractingContent,
    Committing,
    Done,
}

/// Shared state for tracking indexing progress.
pub struct IndexingState {
    pub files_indexed: AtomicU64,
    pub files_total: AtomicU64,
    pub content_extracted: AtomicU64,
    pub cancel: AtomicBool,
}

impl Default for IndexingState {
    fn default() -> Self {
        Self {
            files_indexed: AtomicU64::new(0),
            files_total: AtomicU64::new(0),
            content_extracted: AtomicU64::new(0),
            cancel: AtomicBool::new(false),
        }
    }
}

impl IndexingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn progress(
        &self,
        container_path: &str,
        phase: IndexPhase,
        current: &str,
    ) -> IndexProgress {
        let total = self.files_total.load(Ordering::Relaxed);
        let indexed = self.files_indexed.load(Ordering::Relaxed);
        let percent = if total > 0 {
            (indexed as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        IndexProgress {
            container_path: container_path.to_string(),
            phase,
            files_indexed: indexed,
            files_total: total,
            content_extracted: self.content_extracted.load(Ordering::Relaxed),
            current_file: current.to_string(),
            percent,
        }
    }
}

// =============================================================================
// Entry types for crawling
// =============================================================================

/// A file entry discovered during crawling (before indexing).
#[derive(Debug, Clone)]
struct CrawledEntry {
    /// Unique ID (container_path + entry_path)
    doc_id: String,
    /// Container file path
    container_path: String,
    /// Container type (e.g., "ad1", "e01", "zip")
    container_type: String,
    /// Path within the container
    entry_path: String,
    /// Filename
    filename: String,
    /// File extension (lowercase, no dot)
    extension: String,
    /// File size in bytes
    size: u64,
    /// Last modified timestamp (unix)
    modified: i64,
    /// Is this a directory?
    is_dir: bool,
    /// File category
    category: String,
    /// Is this entry text-eligible for content extraction?
    text_eligible: bool,
}

// =============================================================================
// Container Crawling — enumerate all files across container types
// =============================================================================

/// Crawl files from an AD1 container.
fn crawl_ad1(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let root = ad1::get_root_children_v2(container_path).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    crawl_ad1_recursive(container_path, &root, &mut entries)?;
    Ok(entries)
}

fn crawl_ad1_recursive(
    container_path: &str,
    items: &[ad1::TreeEntry],
    entries: &mut Vec<CrawledEntry>,
) -> Result<(), String> {
    for item in items {
        let ext = Path::new(&item.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let category = classify_extension(&ext);

        entries.push(CrawledEntry {
            doc_id: format!("{}:{}", container_path, item.path),
            container_path: container_path.to_string(),
            container_type: "ad1".to_string(),
            entry_path: item.path.clone(),
            filename: item.name.clone(),
            extension: ext.clone(),
            size: item.size,
            modified: parse_timestamp_opt(item.modified.as_deref()),
            is_dir: item.is_dir,
            category: category.to_string(),
            text_eligible: !item.is_dir && is_text_eligible(category),
        });

        // Recurse into children
        if item.is_dir && item.child_count.unwrap_or(0) > 0 {
            if let Ok(children) = ad1::get_children_at_addr_v2(
                container_path,
                item.data_addr.unwrap_or(0),
                &item.path,
            ) {
                crawl_ad1_recursive(container_path, &children, entries)?;
            }
        }
    }
    Ok(())
}

/// Crawl files from an L01 container.
fn crawl_l01(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let tree = ewf::parse_l01_file_tree(container_path)
        .map_err(|e| format!("Failed to parse L01: {}", e))?;

    let mut entries = Vec::new();
    for entry in &tree.entries {
        let ext = Path::new(&entry.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let category = classify_extension(&ext);

        entries.push(CrawledEntry {
            doc_id: format!("{}:{}", container_path, entry.path),
            container_path: container_path.to_string(),
            container_type: "l01".to_string(),
            entry_path: entry.path.clone(),
            filename: entry.name.clone(),
            extension: ext.clone(),
            size: entry.size,
            modified: entry.modification_time,
            is_dir: entry.is_directory,
            category: category.to_string(),
            text_eligible: !entry.is_directory && is_text_eligible(category),
        });
    }
    Ok(entries)
}

/// Crawl files from an Archive container (ZIP, 7z, TAR, RAR, etc.).
fn crawl_archive(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let entries_list = archive::libarchive_list_all(container_path)
        .map_err(|e| format!("Failed to list archive: {}", e))?;

    let mut entries = Vec::new();
    for item in &entries_list {
        let ext = Path::new(&item.path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let category = classify_extension(&ext);
        let filename = Path::new(&item.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&item.path)
            .to_string();

        entries.push(CrawledEntry {
            doc_id: format!("{}:{}", container_path, item.path),
            container_path: container_path.to_string(),
            container_type: "archive".to_string(),
            entry_path: item.path.clone(),
            filename,
            extension: ext.clone(),
            size: item.size,
            modified: item.mtime.unwrap_or(0),
            is_dir: item.is_dir,
            category: category.to_string(),
            text_eligible: !item.is_dir && is_text_eligible(category),
        });
    }
    Ok(entries)
}

/// Crawl files from a VFS (E01 / Raw disk image with filesystem).
fn crawl_vfs_ewf(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let vfs = ewf::vfs::EwfVfs::open(container_path)
        .map_err(|e| format!("Failed to open E01 VFS: {:?}", e))?;

    let mut entries = Vec::new();
    crawl_vfs_recursive(&vfs, container_path, "e01", "/", &mut entries, 0);
    Ok(entries)
}

fn crawl_vfs_raw(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let vfs = raw::vfs::RawVfs::open_filesystem(container_path)
        .or_else(|_| raw::vfs::RawVfs::open(container_path))
        .map_err(|e| format!("Failed to open raw VFS: {:?}", e))?;

    let mut entries = Vec::new();
    crawl_vfs_recursive(&vfs, container_path, "raw", "/", &mut entries, 0);
    Ok(entries)
}

fn crawl_vfs_recursive<V: crate::common::vfs::VirtualFileSystem>(
    vfs: &V,
    container_path: &str,
    container_type: &str,
    dir_path: &str,
    entries: &mut Vec<CrawledEntry>,
    depth: usize,
) {
    if depth > MAX_SEARCH_CRAWL_DEPTH {
        warn!(
            "Skipping search VFS crawl directory {}: maximum depth {} exceeded",
            dir_path, MAX_SEARCH_CRAWL_DEPTH
        );
        return;
    }
    if entries.len() >= MAX_SEARCH_CRAWLED_ENTRIES {
        warn!(
            "Stopping search VFS crawl at {} entries",
            MAX_SEARCH_CRAWLED_ENTRIES
        );
        return;
    }

    let listing = match vfs.readdir(dir_path) {
        Ok(items) => items,
        Err(e) => {
            debug!("VFS readdir failed for {}: {:?}", dir_path, e);
            return;
        }
    };

    for item in &listing {
        if entries.len() >= MAX_SEARCH_CRAWLED_ENTRIES {
            warn!(
                "Stopping search VFS crawl at {} entries",
                MAX_SEARCH_CRAWLED_ENTRIES
            );
            return;
        }

        let full_path = if dir_path == "/" {
            format!("/{}", item.name)
        } else {
            format!("{}/{}", dir_path, item.name)
        };

        // DirEntry only has name + is_directory; get size/modified from getattr
        let (file_size, modified_ts) = if !item.is_directory {
            match vfs.getattr(&full_path) {
                Ok(attr) => (
                    attr.size,
                    attr.modified
                        .map(|ns| ns / 1_000_000_000) // nanoseconds → seconds
                        .unwrap_or(0),
                ),
                Err(_) => (0u64, 0i64),
            }
        } else {
            (0u64, 0i64)
        };

        let ext = Path::new(&item.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let category = classify_extension(&ext);

        entries.push(CrawledEntry {
            doc_id: format!("{}:{}", container_path, full_path),
            container_path: container_path.to_string(),
            container_type: container_type.to_string(),
            entry_path: full_path.clone(),
            filename: item.name.clone(),
            extension: ext.clone(),
            size: file_size,
            modified: modified_ts,
            is_dir: item.is_directory,
            category: category.to_string(),
            text_eligible: !item.is_directory && is_text_eligible(category),
        });

        if item.is_directory {
            crawl_vfs_recursive(
                vfs,
                container_path,
                container_type,
                &full_path,
                entries,
                depth + 1,
            );
        }
    }
}

/// Crawl disk files (items directly on the filesystem, e.g., case documents).
fn crawl_disk_files(paths: &[String]) -> Vec<CrawledEntry> {
    let mut entries = Vec::new();
    for path in paths {
        let p = Path::new(path);
        if p.is_dir() {
            crawl_disk_dir(p, &mut entries);
        } else if p.is_file() {
            if let Some(entry) = make_disk_entry(p) {
                entries.push(entry);
            }
        }
    }
    entries
}

fn crawl_disk_dir(dir: &Path, entries: &mut Vec<CrawledEntry>) {
    crawl_disk_dir_limited(dir, entries, 0);
}

fn crawl_disk_dir_limited(dir: &Path, entries: &mut Vec<CrawledEntry>, depth: usize) {
    if depth > MAX_SEARCH_CRAWL_DEPTH {
        warn!(
            "Skipping search disk crawl directory {}: maximum depth {} exceeded",
            dir.display(),
            MAX_SEARCH_CRAWL_DEPTH
        );
        return;
    }
    if entries.len() >= MAX_SEARCH_CRAWLED_ENTRIES {
        warn!(
            "Stopping search disk crawl at {} entries",
            MAX_SEARCH_CRAWLED_ENTRIES
        );
        return;
    }

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if entries.len() >= MAX_SEARCH_CRAWLED_ENTRIES {
                warn!(
                    "Stopping search disk crawl at {} entries",
                    MAX_SEARCH_CRAWLED_ENTRIES
                );
                return;
            }

            let path = entry.path();
            if path.is_dir() {
                crawl_disk_dir_limited(&path, entries, depth + 1);
            } else if let Some(crawled) = make_disk_entry(&path) {
                entries.push(crawled);
            }
        }
    }
}

fn make_disk_entry(path: &Path) -> Option<CrawledEntry> {
    let filename = path.file_name()?.to_str()?.to_string();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let category = classify_extension(&ext);
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    Some(CrawledEntry {
        doc_id: format!("disk:{}", path.display()),
        container_path: "disk".to_string(),
        container_type: "disk".to_string(),
        entry_path: path.to_string_lossy().to_string(),
        filename,
        extension: ext.clone(),
        size: meta.len(),
        modified,
        is_dir: false,
        category: category.to_string(),
        text_eligible: is_text_eligible(category),
    })
}

// =============================================================================
// Content Extraction — read text from files inside containers
// =============================================================================

/// Maximum content size to index per file (256 KB of text).
const MAX_CONTENT_SIZE: usize = 256 * 1024;
/// Maximum file size to read for expensive content extraction.
const MAX_CONTENT_SOURCE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_DOCX_XML_SCAN_BYTES: u64 = MAX_CONTENT_SIZE as u64;
const MAX_INDEX_BINARY_ANALYSIS_BYTES: u64 = 32 * 1024 * 1024;
const MAX_INDEX_METADATA_LIST_ITEMS: usize = 32;
const MAX_INDEX_METADATA_VALUE_CHARS: usize = 180;
const MAX_SEARCH_CRAWL_DEPTH: usize = 128;
const MAX_SEARCH_CRAWLED_ENTRIES: usize = 250_000;

/// Extract searchable content from a file inside a container.
///
/// Returns artifact metadata plus extracted text where available, truncated to
/// MAX_CONTENT_SIZE. This keeps non-text artifacts such as images searchable by
/// type and header metadata without requiring a full file read.
fn extract_content_from_container(container_path: &str, entry: &CrawledEntry) -> String {
    if entry.is_dir {
        return String::new();
    }

    let source = match open_entry_byte_source(container_path, entry) {
        Ok(source) => source,
        Err(e) => {
            debug!(
                "Artifact source open failed for {}: {}",
                entry.entry_path, e
            );
            return String::new();
        }
    };

    let mut artifact = match extract_index_artifact(source.as_ref()) {
        Ok(artifact) => artifact,
        Err(e) => {
            debug!("Artifact extraction failed for {}: {}", entry.entry_path, e);
            return String::new();
        }
    };
    enrich_index_binary_artifact_metadata(source.as_ref(), &mut artifact, entry);

    let mut content_parts = artifact_search_terms(&artifact);

    if let Some(preview) = artifact.content_preview.as_ref() {
        if !preview.is_empty() {
            content_parts.push(preview.clone());
        }
    }

    if !entry.text_eligible || artifact.size > MAX_CONTENT_SOURCE_BYTES {
        return truncate_index_content(content_parts.join("\n"));
    }

    let read_size = match bounded_entry_read_size(source.as_ref(), entry.size) {
        Ok(size) => size,
        Err(e) => {
            debug!("Content read sizing failed for {}: {}", entry.entry_path, e);
            return truncate_index_content(content_parts.join("\n"));
        }
    };

    if read_size == 0 {
        return truncate_index_content(content_parts.join("\n"));
    }

    match extract_entry_text(source.as_ref(), entry, read_size) {
        Ok(extracted_text) if !extracted_text.is_empty() => content_parts.push(extracted_text),
        Ok(_) => {}
        Err(e) => {
            debug!("Content read failed for {}: {}", entry.entry_path, e);
        }
    }

    truncate_index_content(content_parts.join("\n"))
}

fn extract_entry_text(
    source: &dyn EvidenceByteSource,
    entry: &CrawledEntry,
    read_size: usize,
) -> Result<String, String> {
    let bytes = read_range_fully(source, 0, read_size).map_err(|e| e.to_string())?;
    Ok(extract_text_from_bytes(
        &bytes,
        &entry.extension,
        &entry.category,
    ))
}

fn extract_index_artifact(source: &dyn EvidenceByteSource) -> Result<NormalizedArtifact, String> {
    extract_normalized_artifact(
        source,
        ArtifactExtractionOptions {
            header_bytes: 4096,
            preview_bytes: MAX_CONTENT_SIZE,
        },
    )
    .map_err(|e| e.to_string())
}

fn enrich_index_binary_artifact_metadata(
    source: &dyn EvidenceByteSource,
    artifact: &mut NormalizedArtifact,
    entry: &CrawledEntry,
) {
    if !is_index_binary_artifact(artifact, entry) {
        return;
    }

    let size = match source.len() {
        Ok(size) => size,
        Err(error) => {
            debug!(
                "Binary index size lookup failed for {}: {}",
                entry.entry_path, error
            );
            return;
        }
    };
    if size > MAX_INDEX_BINARY_ANALYSIS_BYTES {
        artifact.metadata.insert(
            "binary.indexAnalysisStatus".to_string(),
            "skipped-oversize".to_string(),
        );
        return;
    }
    let Ok(read_len) = usize::try_from(size) else {
        artifact.metadata.insert(
            "binary.indexAnalysisStatus".to_string(),
            "skipped-platform-size".to_string(),
        );
        return;
    };
    let data = match read_range_fully(source, 0, read_len) {
        Ok(data) => data,
        Err(error) => {
            debug!(
                "Binary index read failed for {}: {}",
                entry.entry_path, error
            );
            return;
        }
    };
    let info = match analyze_binary_bytes(source.source_ref().display_id(), &data) {
        Ok(info) => info,
        Err(error) => {
            debug!(
                "Binary index analysis failed for {}: {}",
                entry.entry_path, error
            );
            return;
        }
    };

    artifact
        .metadata
        .extend(index_binary_artifact_metadata_from_info(&info));
    if info.pe_is_driver {
        artifact.category = "system".to_string();
        artifact.type_description = info
            .pe_driver_type
            .as_ref()
            .map(|driver_type| format!("Windows {driver_type}"))
            .unwrap_or_else(|| "Windows Driver Artifact".to_string());
    }
}

fn is_index_binary_artifact(artifact: &NormalizedArtifact, entry: &CrawledEntry) -> bool {
    artifact.category == "executable"
        || artifact
            .mime_type
            .as_deref()
            .is_some_and(|mime| mime.contains("executable") || mime.contains("mach-binary"))
        || is_index_binary_extension(artifact.extension.as_deref())
        || is_index_binary_extension(Some(entry.extension.as_str()))
}

fn is_index_binary_extension(extension: Option<&str>) -> bool {
    matches!(
        extension,
        Some(
            "sys" | "drv" | "ko" | "exe" | "dll" | "ocx" | "efi" | "elf" | "so" | "dylib" | "kext"
        )
    )
}

fn index_binary_artifact_metadata_from_info(info: &BinaryInfo) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "binary.indexAnalysisStatus".to_string(),
        "parsed".to_string(),
    );
    metadata.insert(
        "binary.format".to_string(),
        index_binary_format_name(&info.format).to_string(),
    );
    metadata.insert("binary.architecture".to_string(), info.architecture.clone());
    metadata.insert("binary.is64Bit".to_string(), info.is_64bit.to_string());
    metadata.insert("binary.fileSize".to_string(), info.file_size.to_string());
    metadata.insert(
        "binary.importLibraryCount".to_string(),
        info.imports.len().to_string(),
    );
    metadata.insert(
        "binary.exportCount".to_string(),
        info.exports.len().to_string(),
    );
    metadata.insert(
        "binary.sectionCount".to_string(),
        info.sections.len().to_string(),
    );
    metadata.insert(
        "binary.stringCount".to_string(),
        info.strings.len().to_string(),
    );
    if let Some(entry_point) = info.entry_point {
        metadata.insert(
            "binary.entryPoint".to_string(),
            format!("0x{entry_point:x}"),
        );
    }
    if !info.imports.is_empty() {
        let libraries = info
            .imports
            .iter()
            .take(32)
            .map(|import| import.library.clone())
            .collect::<Vec<_>>();
        insert_index_joined_metadata(&mut metadata, "binary.importLibraries", &libraries);
    }
    if !info.exports.is_empty() {
        let exports = info
            .exports
            .iter()
            .take(32)
            .map(|export| export.name.clone())
            .collect::<Vec<_>>();
        insert_index_joined_metadata(&mut metadata, "binary.exports", &exports);
    }
    if !info.sections.is_empty() {
        let sections = info
            .sections
            .iter()
            .take(32)
            .map(|section| section.name.clone())
            .collect::<Vec<_>>();
        insert_index_joined_metadata(&mut metadata, "binary.sections", &sections);
    }
    if let Some(timestamp) = info.pe_timestamp {
        metadata.insert("pe.timestamp".to_string(), timestamp.to_string());
    }
    if let Some(checksum) = info.pe_checksum {
        metadata.insert("pe.checksum".to_string(), format!("0x{checksum:08x}"));
    }
    if let Some(subsystem) = &info.pe_subsystem {
        metadata.insert("pe.subsystem".to_string(), subsystem.clone());
    }
    if let Some(version) = &info.pe_linker_version {
        metadata.insert("pe.linkerVersion".to_string(), version.clone());
    }
    if let Some(version) = &info.pe_os_version {
        metadata.insert("pe.osVersion".to_string(), version.clone());
    }
    if let Some(version) = &info.pe_image_version {
        metadata.insert("pe.imageVersion".to_string(), version.clone());
    }
    if let Some(version) = &info.pe_subsystem_version {
        metadata.insert("pe.subsystemVersion".to_string(), version.clone());
    }
    if let Some(image_base) = info.pe_image_base {
        metadata.insert("pe.imageBase".to_string(), format!("0x{image_base:x}"));
    }
    if let Some(section_alignment) = info.pe_section_alignment {
        metadata.insert(
            "pe.sectionAlignment".to_string(),
            section_alignment.to_string(),
        );
    }
    if let Some(file_alignment) = info.pe_file_alignment {
        metadata.insert("pe.fileAlignment".to_string(), file_alignment.to_string());
    }
    if let Some(size_of_image) = info.pe_size_of_image {
        metadata.insert("pe.sizeOfImage".to_string(), size_of_image.to_string());
    }
    if let Some(size_of_headers) = info.pe_size_of_headers {
        metadata.insert("pe.sizeOfHeaders".to_string(), size_of_headers.to_string());
    }
    if let Some(characteristics) = &info.pe_dll_characteristics {
        metadata.insert("pe.dllCharacteristics".to_string(), characteristics.clone());
    }
    insert_index_joined_metadata(
        &mut metadata,
        "pe.dllCharacteristicsDetail",
        &info.pe_dll_characteristics_detail,
    );
    if let Some(size) = info.pe_certificate_table_size {
        metadata.insert("pe.certificateTableSize".to_string(), size.to_string());
    }
    metadata.insert("pe.isDriver".to_string(), info.pe_is_driver.to_string());
    if let Some(driver_type) = &info.pe_driver_type {
        metadata.insert("pe.driverType".to_string(), driver_type.clone());
    }
    insert_index_joined_metadata(
        &mut metadata,
        "pe.driverIndicators",
        &info.pe_driver_indicators,
    );
    for (key, value) in &info.pe_version_info {
        metadata.insert(format!("pe.version.{key}"), value.clone());
    }
    if info.pe_is_driver {
        insert_index_pe_driver_string_metadata(&mut metadata, &info.strings);
    }
    metadata
}

fn insert_index_pe_driver_string_metadata(
    metadata: &mut BTreeMap<String, String>,
    strings: &[String],
) {
    let mut service_names = Vec::new();
    let mut device_names = Vec::new();
    let mut dos_device_names = Vec::new();
    let mut registry_paths = Vec::new();
    let mut pdb_paths = Vec::new();
    let mut urls = Vec::new();
    let mut guids = Vec::new();
    let mut indexed_strings = Vec::new();

    for value in strings {
        push_index_unique_limited(
            &mut indexed_strings,
            truncate_index_metadata_value(value, MAX_INDEX_METADATA_VALUE_CHARS),
        );
        if let Some(service_name) = extract_index_windows_driver_service_name(value) {
            push_index_unique_limited(&mut service_names, service_name);
        }
        if let Some(device_name) = extract_index_windows_object_name(value, "\\device\\") {
            push_index_unique_limited(&mut device_names, device_name);
        }
        if let Some(dos_device_name) = extract_index_windows_object_name(value, "\\dosdevices\\") {
            push_index_unique_limited(&mut dos_device_names, dos_device_name);
        }
        if let Some(registry_path) = extract_index_windows_driver_registry_path(value) {
            push_index_unique_limited(&mut registry_paths, registry_path);
        }
        if let Some(pdb_path) = extract_index_windows_driver_pdb_path(value) {
            push_index_unique_limited(&mut pdb_paths, pdb_path);
        }
        if let Some(url) = extract_index_embedded_url(value) {
            push_index_unique_limited(&mut urls, url);
        }
        if let Some(guid) = extract_index_braced_guid(value) {
            push_index_unique_limited(&mut guids, guid);
        }
    }

    insert_index_joined_metadata(metadata, "binary.strings", &indexed_strings);
    insert_index_joined_metadata(metadata, "pe.driverServiceNames", &service_names);
    insert_index_joined_metadata(metadata, "pe.driverDeviceNames", &device_names);
    insert_index_joined_metadata(metadata, "pe.driverDosDeviceNames", &dos_device_names);
    insert_index_joined_metadata(metadata, "pe.driverRegistryPaths", &registry_paths);
    insert_index_joined_metadata(metadata, "pe.driverPdbPaths", &pdb_paths);
    insert_index_joined_metadata(metadata, "pe.driverUrls", &urls);
    insert_index_joined_metadata(metadata, "pe.driverGuids", &guids);
}

fn extract_index_windows_driver_service_name(value: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    for marker in [
        "\\currentcontrolset\\services\\",
        "\\controlset001\\services\\",
        "\\controlset002\\services\\",
        "\\controlset003\\services\\",
    ] {
        if let Some(name) = extract_index_after_marker(&normalized, marker) {
            return Some(name);
        }
    }
    None
}

fn extract_index_windows_object_name(value: &str, marker: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    extract_index_after_marker(&normalized, marker)
}

fn extract_index_after_marker(value: &str, marker: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find(marker)?.checked_add(marker.len())?;
    let rest = value.get(start..)?;
    let end = rest
        .find(|ch: char| ch == '\\' || ch == '/' || ch.is_whitespace() || ch == '\0')
        .unwrap_or(rest.len());
    let candidate = rest.get(..end)?.trim_matches(['"', '\'']);
    (!candidate.is_empty()).then(|| truncate_index_metadata_value(candidate, 120))
}

fn extract_index_windows_driver_registry_path(value: &str) -> Option<String> {
    extract_index_segment_starting_with(value, "\\registry\\machine\\")
        .or_else(|| {
            extract_index_segment_starting_with(value, "system\\currentcontrolset\\services\\")
        })
        .or_else(|| extract_index_segment_starting_with(value, "system\\controlset001\\services\\"))
        .map(|value| value.replace('/', "\\"))
}

fn extract_index_segment_starting_with(value: &str, marker: &str) -> Option<String> {
    let normalized = value.replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    let start = lower.find(marker)?;
    let rest = normalized.get(start..)?;
    let end = rest
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | ',' | ';' | ')' | ']'))
        .unwrap_or(rest.len());
    let candidate = rest
        .get(..end)?
        .trim_matches(['\0', '"', '\'', ':', '.', '\\']);
    (!candidate.is_empty()).then(|| truncate_index_metadata_value(candidate, 180))
}

fn extract_index_windows_driver_pdb_path(value: &str) -> Option<String> {
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
    Some(truncate_index_metadata_value(candidate, 180))
}

fn extract_index_embedded_url(value: &str) -> Option<String> {
    let lower = value.to_ascii_lowercase();
    let start = lower.find("https://").or_else(|| lower.find("http://"))?;
    let raw = value.get(start..)?;
    let end = raw
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '<' | '>' | ')' | ']'))
        .unwrap_or(raw.len());
    let candidate = raw.get(..end)?.trim_end_matches(['.', ',', ';']);
    Some(truncate_index_metadata_value(candidate, 180)).filter(|value| value.contains("://"))
}

fn extract_index_braced_guid(value: &str) -> Option<String> {
    let start = value.find('{')?;
    let end = value.get(start..)?.find('}')?.checked_add(start + 1)?;
    let candidate = value.get(start..end)?;
    is_index_braced_guid(candidate).then(|| candidate.to_ascii_uppercase())
}

fn is_index_braced_guid(value: &str) -> bool {
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

fn push_index_unique_limited(values: &mut Vec<String>, value: String) {
    if values.len() >= MAX_INDEX_METADATA_LIST_ITEMS || value.is_empty() {
        return;
    }
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn truncate_index_metadata_value(value: &str, max_chars: usize) -> String {
    let value = value.trim();
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    value.chars().take(max_chars).collect()
}

fn index_binary_format_name(format: &BinaryFormat) -> &'static str {
    match format {
        BinaryFormat::PE32 => "PE32",
        BinaryFormat::PE64 => "PE64",
        BinaryFormat::ELF32 => "ELF32",
        BinaryFormat::ELF64 => "ELF64",
        BinaryFormat::MachO32 => "Mach-O 32",
        BinaryFormat::MachO64 => "Mach-O 64",
        BinaryFormat::MachOFat => "Universal Binary",
        BinaryFormat::Unknown => "Unknown",
    }
}

fn insert_index_joined_metadata(
    metadata: &mut BTreeMap<String, String>,
    key: &str,
    values: &[String],
) {
    if !values.is_empty() {
        metadata.insert(key.to_string(), values.join("; "));
    }
}

fn artifact_search_terms(artifact: &NormalizedArtifact) -> Vec<String> {
    let mut terms = vec![
        artifact.name.clone(),
        artifact.type_description.clone(),
        artifact.category.clone(),
        artifact.confidence.clone(),
    ];

    if let Some(extension) = &artifact.extension {
        terms.push(extension.clone());
    }
    if let Some(mime_type) = &artifact.mime_type {
        terms.push(mime_type.clone());
    }

    for (key, value) in &artifact.metadata {
        terms.push(key.clone());
        if !value.is_empty() {
            terms.push(value.clone());
            terms.push(format!("{key}:{value}"));
        }
    }

    terms
        .into_iter()
        .map(|term| term.trim().to_string())
        .filter(|term| !term.is_empty())
        .collect()
}

fn truncate_index_content(text: String) -> String {
    if text.len() <= MAX_CONTENT_SIZE {
        return text;
    }

    let mut end = MAX_CONTENT_SIZE;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

/// Open a byte source for a crawled entry.
fn open_entry_byte_source(
    container_path: &str,
    entry: &CrawledEntry,
) -> Result<Box<dyn EvidenceByteSource>, String> {
    match entry.container_type.as_str() {
        "disk" => Ok(Box::new(LocalFileByteSource::new(&entry.entry_path))),
        container_type => open_container_entry_source(
            container_path,
            &entry.entry_path,
            container_type,
            size_hint(entry.size),
        )
        .map_err(|e| format!("Container entry source: {}", e)),
    }
}

fn size_hint(size: u64) -> Option<u64> {
    if size > 0 {
        Some(size)
    } else {
        None
    }
}

fn add_indexed_count(total: u64, count: u64) -> u64 {
    total.saturating_add(count)
}

fn bounded_entry_read_size(
    source: &dyn EvidenceByteSource,
    _known_size: u64,
) -> Result<usize, String> {
    let source_size = source.len().map_err(|e| e.to_string())?;
    let byte_len = source_size.min(MAX_CONTENT_SOURCE_BYTES);

    usize::try_from(byte_len)
        .map_err(|_| format!("Entry is too large to read on this platform: {byte_len} bytes"))
}

/// Extract text content from raw bytes using the file extension to select
/// the appropriate parser.
fn extract_text_from_bytes(data: &[u8], ext: &str, _category: &str) -> String {
    match ext {
        // Plain text (including code, config, markup)
        "txt" | "log" | "md" | "rst" | "tex" | "json" | "xml" | "yaml" | "yml" | "toml" | "ini"
        | "cfg" | "conf" | "env" | "csv" | "tsv" | "html" | "htm" | "xhtml" | "css" | "scss"
        | "less" | "svg" | "py" | "js" | "ts" | "jsx" | "tsx" | "rs" | "c" | "cpp" | "h"
        | "java" | "go" | "rb" | "php" | "swift" | "kt" | "cs" | "sh" | "bat" | "ps1" | "vbs"
        | "pl" | "r" | "m" | "sql" | "readme" | "changelog" | "license" => {
            String::from_utf8_lossy(data).to_string()
        }

        // PDF — use pdf-extract crate
        "pdf" => extract_pdf_text(data),

        // DOCX — use DocumentProcessor
        "docx" => extract_docx_text(data),

        // RTF — strip RTF control words
        "rtf" => {
            let raw = String::from_utf8_lossy(data).to_string();
            strip_rtf_simple(&raw)
        }

        // EML emails
        "eml" | "mbox" => extract_email_text(data),

        // Plist (Apple property list)
        "plist" => {
            // Try text-based plist first
            if data.starts_with(b"<?xml") || data.starts_with(b"bplist") {
                match plist::from_bytes::<plist::Value>(data) {
                    Ok(val) => format!("{:?}", val),
                    Err(_) => String::from_utf8_lossy(data).to_string(),
                }
            } else {
                String::from_utf8_lossy(data).to_string()
            }
        }

        // Fallback: try UTF-8 lossy
        _ => {
            // Only attempt for small text-like files
            if data.len() < 100_000 && is_likely_text(data) {
                String::from_utf8_lossy(data).to_string()
            } else {
                String::new()
            }
        }
    }
}

/// Extract text from PDF bytes using pdf-extract.
fn extract_pdf_text(data: &[u8]) -> String {
    match pdf_extract::extract_text_from_mem(data) {
        Ok(text) => text,
        Err(e) => {
            debug!("PDF text extraction failed: {}", e);
            String::new()
        }
    }
}

/// Extract text from DOCX bytes.
fn extract_docx_text(data: &[u8]) -> String {
    use std::io::Cursor;

    // DOCX is a ZIP containing word/document.xml
    let reader = Cursor::new(data);
    let mut archive = match zip::ZipArchive::new(reader) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };

    let mut text = String::new();
    if let Ok(mut file) = archive.by_name("word/document.xml") {
        if let Some(xml) = read_limited_zip_text(&mut file, MAX_DOCX_XML_SCAN_BYTES) {
            // Simple XML text extraction — get content between <w:t> tags
            for part in xml.split("<w:t") {
                if let Some(content_start) = part.find('>') {
                    let after = &part[content_start + 1..];
                    if let Some(end) = after.find("</w:t>") {
                        text.push_str(&after[..end]);
                        text.push(' ');
                    }
                }
            }
        }
    }
    text
}

fn read_limited_zip_text<R: std::io::Read>(reader: &mut R, max_bytes: u64) -> Option<String> {
    let mut limited_reader =
        std::io::Read::take(std::io::Read::by_ref(reader), max_bytes.saturating_add(1));
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut limited_reader, &mut bytes).ok()?;
    if bytes.len() as u64 > max_bytes {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    Some(text)
}

/// Extract text from EML/MBOX email bytes.
fn extract_email_text(data: &[u8]) -> String {
    match mail_parser::MessageParser::default().parse(data) {
        Some(msg) => {
            let mut text = String::new();
            // Subject
            if let Some(subj) = msg.subject() {
                text.push_str("Subject: ");
                text.push_str(subj);
                text.push('\n');
            }
            // From
            if let Some(from) = msg.from() {
                text.push_str("From: ");
                text.push_str(&format!("{:?}", from));
                text.push('\n');
            }
            // Body text
            if let Some(body) = msg.body_text(0) {
                text.push_str(&body);
            }
            text
        }
        None => String::new(),
    }
}

/// Simple RTF text extraction (strip control words).
fn strip_rtf_simple(rtf: &str) -> String {
    let mut result = String::new();
    let mut in_group = 0i32;
    let mut skip = false;
    let chars: Vec<char> = rtf.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
                in_group += 1;
            }
            '}' => {
                in_group -= 1;
                skip = false;
            }
            '\\' => {
                // Skip control word
                i += 1;
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    i += 1;
                }
                if i < chars.len() && chars[i] == ' ' {
                    i += 1;
                }
                continue;
            }
            _ if !skip && in_group <= 1 => {
                result.push(chars[i]);
            }
            _ => {}
        }
        i += 1;
    }
    result
}

/// Heuristic to check if bytes are likely text (high ASCII ratio).
fn is_likely_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    let check_len = data.len().min(1024);
    let printable = data[..check_len]
        .iter()
        .filter(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
        .count();
    (printable as f64 / check_len as f64) > 0.85
}

/// Parse a timestamp string to unix seconds.
fn parse_timestamp(ts: &str) -> i64 {
    if ts.is_empty() {
        return 0;
    }
    // Try common formats: ISO 8601, "YYYY-MM-DD HH:MM:SS", epoch seconds
    if let Ok(epoch) = ts.parse::<i64>() {
        return epoch;
    }
    // Simple ISO 8601 parse (YYYY-MM-DDTHH:MM:SS)
    // This is a best-effort parse; we don't need precision for search faceting
    0
}

fn parse_timestamp_opt(ts: Option<&str>) -> i64 {
    ts.map(parse_timestamp).unwrap_or(0)
}

// =============================================================================
// Main Indexing Functions
// =============================================================================

/// Index a single container into the search index.
///
/// This is the main indexing function. It:
/// 1. Crawls the container to enumerate all files
/// 2. Indexes metadata (filename, extension, size, category)
/// 3. Optionally extracts and indexes text content
///
/// Returns the number of files indexed.
pub fn index_container(
    search_index: &SearchIndex,
    container_path: &str,
    index_content: bool,
    state: &IndexingState,
) -> Result<u64, String> {
    if state.is_cancelled() {
        return Ok(0);
    }

    info!(
        "Indexing container: {} (content={})",
        container_path, index_content
    );

    // Determine container type and crawl
    let is_ewf = ewf::is_ewf(container_path).unwrap_or(false);
    let is_l01 = ewf::is_l01_file(container_path).unwrap_or(false);
    let is_raw = raw::is_raw(container_path).unwrap_or(false);
    let is_ad1 = ad1::is_ad1(container_path).unwrap_or(false);
    let is_archive = archive::is_archive(container_path).unwrap_or(false);

    let entries = if is_ad1 {
        crawl_ad1(container_path)?
    } else if is_l01 {
        crawl_l01(container_path)?
    } else if is_archive {
        crawl_archive(container_path)?
    } else if is_ewf {
        crawl_vfs_ewf(container_path)?
    } else if is_raw {
        crawl_vfs_raw(container_path)?
    } else {
        warn!("Unknown container type, skipping: {}", container_path);
        return Ok(0);
    };

    state
        .files_total
        .fetch_add(entries.len() as u64, Ordering::Relaxed);

    info!("Crawled {} entries from {}", entries.len(), container_path);

    // Index each entry
    let mut indexed = 0u64;
    for entry in &entries {
        if state.is_cancelled() {
            info!("Indexing cancelled for {}", container_path);
            break;
        }

        // Extract content if requested and eligible
        let content = if index_content && !entry.is_dir {
            let text = extract_content_from_container(container_path, entry);
            if !text.is_empty() {
                state.content_extracted.fetch_add(1, Ordering::Relaxed);
            }
            text
        } else {
            String::new()
        };

        // Add document to index
        if let Err(e) = search_index.add_document(
            &entry.doc_id,
            &entry.container_path,
            &entry.container_type,
            &entry.entry_path,
            &entry.filename,
            &entry.extension,
            &content,
            entry.size,
            entry.modified,
            entry.is_dir,
            &entry.category,
        ) {
            warn!("Failed to index {}: {}", entry.entry_path, e);
        }

        indexed += 1;
        state.files_indexed.fetch_add(1, Ordering::Relaxed);
    }

    // Commit after each container
    search_index.commit()?;

    info!(
        "Indexed {} files from {} ({} with content)",
        indexed,
        container_path,
        state.content_extracted.load(Ordering::Relaxed)
    );

    Ok(indexed)
}

/// Index disk files (e.g., case documents found on the filesystem).
pub fn index_disk_files(
    search_index: &SearchIndex,
    paths: &[String],
    index_content: bool,
    state: &IndexingState,
) -> Result<u64, String> {
    let entries = crawl_disk_files(paths);
    state
        .files_total
        .fetch_add(entries.len() as u64, Ordering::Relaxed);

    let mut indexed = 0u64;
    for entry in &entries {
        if state.is_cancelled() {
            break;
        }

        let content = if index_content && !entry.is_dir {
            let text = extract_content_from_container("disk", entry);
            if !text.is_empty() {
                state.content_extracted.fetch_add(1, Ordering::Relaxed);
            }
            text
        } else {
            String::new()
        };

        if let Err(e) = search_index.add_document(
            &entry.doc_id,
            &entry.container_path,
            &entry.container_type,
            &entry.entry_path,
            &entry.filename,
            &entry.extension,
            &content,
            entry.size,
            entry.modified,
            entry.is_dir,
            &entry.category,
        ) {
            warn!("Failed to index disk file {}: {}", entry.entry_path, e);
        }

        indexed += 1;
        state.files_indexed.fetch_add(1, Ordering::Relaxed);
    }

    search_index.commit()?;
    Ok(indexed)
}

/// Re-index everything: delete all documents and re-crawl all containers.
pub fn rebuild_index(
    search_index: &SearchIndex,
    container_paths: &[String],
    index_content: bool,
    state: &IndexingState,
) -> Result<u64, String> {
    // Clear the entire index
    {
        let mut writer_guard = search_index
            .writer
            .lock()
            .map_err(|e| format!("Writer lock: {}", e))?;
        if let Some(ref mut writer) = *writer_guard {
            writer
                .delete_all_documents()
                .map_err(|e| format!("Failed to clear index: {}", e))?;
            writer
                .commit()
                .map_err(|e| format!("Failed to commit clear: {}", e))?;
        }
    }

    let mut total = 0u64;
    for path in container_paths {
        if state.is_cancelled() {
            break;
        }
        total = add_indexed_count(
            total,
            index_container(search_index, path, index_content, state)?,
        );
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EvidenceSourceError, EvidenceSourceRef, EvidenceSourceResult};
    use std::io::{Cursor, Write};

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
            if offset > self.data.len() as u64 {
                return Err(EvidenceSourceError::InvalidRange {
                    source_id: self.source_ref.display_id(),
                    offset,
                    size: self.data.len() as u64,
                });
            }
            let start = offset as usize;
            if start >= self.data.len() {
                return Ok(Vec::new());
            }
            let requested = size.min(self.max_chunk);
            let end = start.saturating_add(requested).min(self.data.len());
            Ok(self.data[start..end].to_vec())
        }
    }

    struct DeclaredLenByteSource {
        source_ref: EvidenceSourceRef,
        declared_len: u64,
    }

    impl DeclaredLenByteSource {
        fn new(path: &str, declared_len: u64) -> Self {
            Self {
                source_ref: EvidenceSourceRef::LocalFile {
                    path: path.to_string(),
                },
                declared_len,
            }
        }
    }

    impl EvidenceByteSource for DeclaredLenByteSource {
        fn source_ref(&self) -> EvidenceSourceRef {
            self.source_ref.clone()
        }

        fn len(&self) -> EvidenceSourceResult<u64> {
            Ok(self.declared_len)
        }

        fn read_range(&self, _offset: u64, _size: usize) -> EvidenceSourceResult<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    fn write_temp_file(suffix: &str, bytes: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::Builder::new().suffix(suffix).tempfile().unwrap();
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn add_indexed_count_saturates_on_overflow() {
        assert_eq!(add_indexed_count(40, 2), 42);
        assert_eq!(add_indexed_count(u64::MAX - 1, 8), u64::MAX);
    }

    fn disk_entry(path: &Path, category: &str, text_eligible: bool) -> CrawledEntry {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        let size = std::fs::metadata(path).unwrap().len();

        CrawledEntry {
            doc_id: format!("disk:{}", path.display()),
            container_path: "disk".to_string(),
            container_type: "disk".to_string(),
            entry_path: path.to_string_lossy().to_string(),
            filename,
            extension,
            size,
            modified: 0,
            is_dir: false,
            category: category.to_string(),
            text_eligible,
        }
    }

    fn build_docx_with_document_xml(xml: &[u8]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            zip.start_file(
                "word/document.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(xml).unwrap();
            zip.finish().unwrap();
        }
        cursor.into_inner()
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

    fn inline_ascii_value(value: &[u8]) -> u32 {
        let mut bytes = [0u8; 4];
        for (index, byte) in value.iter().take(4).enumerate() {
            bytes[index] = *byte;
        }
        u32::from_le_bytes(bytes)
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

    fn jpeg_with_exif_search_terms() -> Vec<u8> {
        let ifd0_offset = 8usize;
        let ifd0_entries = 3usize;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        push_u16_le(&mut tiff, 42);
        push_u32_le(&mut tiff, ifd0_offset as u32);
        tiff.resize(ifd0_offset + 2 + ifd0_entries * 12 + 4, 0);

        let (make_count, make_offset) = append_ascii(&mut tiff, "CORE");
        let (captured_count, captured_offset) = append_ascii(&mut tiff, "2026:02:16 10:01:00");
        let exif_offset = append_ifd(
            &mut tiff,
            &[TestTiffEntry {
                tag: 0x9003,
                field_type: 2,
                count: captured_count,
                value: captured_offset,
            }],
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

    #[test]
    fn content_extraction_uses_normalized_artifact_text_preview() {
        let file = write_temp_file(".txt", b"normalized artifact search text");
        let entry = disk_entry(file.path(), "text", true);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("Plain Text"));
        assert!(content.contains("text/plain"));
        assert!(content.contains("normalized artifact search text"));
    }

    #[test]
    fn entry_text_extraction_assembles_chunked_source() {
        let bytes = b"alpha beta gamma searchable tail";
        let file = write_temp_file(".txt", bytes);
        let source = ChunkedByteSource::new("chunked.txt", bytes, 5);
        let entry = disk_entry(file.path(), "text", true);

        let content = extract_entry_text(&source, &entry, bytes.len()).unwrap();

        assert_eq!(content, "alpha beta gamma searchable tail");
    }

    #[test]
    fn docx_text_extraction_uses_bounded_xml_scan() {
        let mut xml = vec![b'a'; MAX_DOCX_XML_SCAN_BYTES as usize];
        xml.extend_from_slice(b"<w:t>late searchable text</w:t>");
        let docx = build_docx_with_document_xml(&xml);

        let content = extract_text_from_bytes(&docx, "docx", "document");

        assert!(!content.contains("late searchable text"));
    }

    #[test]
    fn read_limited_zip_text_allows_exact_limit() {
        let mut reader = Cursor::new(b"abc".to_vec());

        let text = read_limited_zip_text(&mut reader, 3).unwrap();

        assert_eq!(text, "abc");
    }

    #[test]
    fn read_limited_zip_text_rejects_oversized_entry() {
        let mut reader = Cursor::new(b"abcd".to_vec());

        let text = read_limited_zip_text(&mut reader, 3);

        assert!(text.is_none());
    }

    #[test]
    fn bounded_entry_read_size_enforces_content_source_cap() {
        let source = DeclaredLenByteSource::new("oversized.txt", MAX_CONTENT_SOURCE_BYTES + 4096);

        let known_size = bounded_entry_read_size(&source, source.len().unwrap()).unwrap();
        let fallback_size = bounded_entry_read_size(&source, 0).unwrap();

        assert_eq!(known_size, MAX_CONTENT_SOURCE_BYTES as usize);
        assert_eq!(fallback_size, MAX_CONTENT_SOURCE_BYTES as usize);
    }

    #[test]
    fn bounded_entry_read_size_uses_live_source_size_for_stale_known_size() {
        let bytes = b"stale crawler size";
        let source = ChunkedByteSource::new("stale.txt", bytes, usize::MAX);

        let read_size = bounded_entry_read_size(&source, bytes.len() as u64 + 4096).unwrap();

        assert_eq!(read_size, bytes.len());
    }

    #[test]
    fn bounded_entry_read_size_uses_live_source_size_when_known_size_is_short() {
        let bytes = b"stale short crawler size";
        let source = ChunkedByteSource::new("stale-short.txt", bytes, usize::MAX);

        let read_size = bounded_entry_read_size(&source, 1).unwrap();

        assert_eq!(read_size, bytes.len());
    }

    #[test]
    fn crawl_disk_dir_limited_skips_beyond_depth_limit() {
        let dir = tempfile::tempdir().unwrap();
        let mut current = dir.path().to_path_buf();
        for index in 0..=MAX_SEARCH_CRAWL_DEPTH + 1 {
            current = current.join(format!("d{index}"));
            std::fs::create_dir(&current).unwrap();
        }
        std::fs::write(current.join("too-deep.txt"), b"search text").unwrap();

        let mut entries = Vec::new();
        crawl_disk_dir(dir.path(), &mut entries);

        assert!(entries.is_empty());
    }

    #[test]
    fn artifact_indicator_metadata_is_searchable() {
        let file = write_temp_file(
            ".txt",
            b"Contact admin@example.com from 192.168.1.10 and visit https://example.com/login",
        );
        let entry = disk_entry(file.path(), "text", true);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("indicators.emailCount:1"));
        assert!(content.contains("indicators.emails:admin@example.com"));
        assert!(content.contains("indicators.ipv4:192.168.1.10"));
        assert!(content.contains("indicators.urls:https://example.com/login"));
    }

    #[test]
    fn binary_index_metadata_flattens_driver_fields_for_search() {
        let mut version_info = BTreeMap::new();
        version_info.insert("CompanyName".to_string(), "Contoso Driver Labs".to_string());
        version_info.insert("OriginalFilename".to_string(), "contosoflt.sys".to_string());

        let info = BinaryInfo {
            path: "case.ad1:/Windows/System32/drivers/contosoflt.sys".to_string(),
            format: BinaryFormat::PE64,
            architecture: "x86_64".to_string(),
            is_64bit: true,
            entry_point: Some(0x140001000),
            imports: vec![crate::viewer::document::binary::ImportInfo {
                library: "fltmgr.sys".to_string(),
                functions: vec!["FltRegisterFilter".to_string()],
                function_count: 1,
            }],
            exports: vec![crate::viewer::document::binary::ExportInfo {
                name: "DriverEntry".to_string(),
                ordinal: None,
                address: 0x1000,
            }],
            sections: vec![crate::viewer::document::binary::SectionInfo {
                name: ".text".to_string(),
                virtual_address: 0x1000,
                virtual_size: 0x2000,
                raw_size: 0x2000,
                characteristics: "0x60000020".to_string(),
                characteristics_detail: vec![
                    "contains-code".to_string(),
                    "executable".to_string(),
                    "readable".to_string(),
                ],
                entropy: Some(6.125),
            }],
            strings: vec![
                "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt".to_string(),
                "\\Registry\\Machine\\System\\ControlSet001\\Services\\legacyflt\\Parameters"
                    .to_string(),
                "\\Device\\ContosoFilter".to_string(),
                "\\DosDevices\\ContosoFilter".to_string(),
                r"C:\agent\_work\drivers\contosoflt\objfre\amd64\contosoflt.pdb".to_string(),
                "https://drivers.example.test/support".to_string(),
                "{12345678-9abc-def0-1234-56789abcdef0}".to_string(),
            ],
            file_size: 4096,
            pe_timestamp: Some(1_717_260_000),
            pe_checksum: Some(0x1234abcd),
            pe_subsystem: Some("Native".to_string()),
            pe_linker_version: Some("14.38".to_string()),
            pe_os_version: Some("10.0".to_string()),
            pe_image_version: Some("10.0".to_string()),
            pe_subsystem_version: Some("10.0".to_string()),
            pe_image_base: Some(0x140000000),
            pe_section_alignment: Some(4096),
            pe_file_alignment: Some(512),
            pe_size_of_image: Some(32768),
            pe_size_of_headers: Some(1024),
            pe_dll_characteristics: Some("0x2140".to_string()),
            pe_dll_characteristics_detail: vec![
                "dynamic-base".to_string(),
                "nx-compatible".to_string(),
                "wdm-driver".to_string(),
            ],
            pe_certificate_table_size: Some(4096),
            pe_is_driver: true,
            pe_driver_type: Some("File system minifilter driver".to_string()),
            pe_driver_indicators: vec![
                "driver file extension".to_string(),
                "file-system filter driver APIs".to_string(),
            ],
            pe_version_info: version_info,
            macho_cpu_type: None,
            macho_filetype: None,
            has_debug_info: false,
            is_stripped: true,
            has_code_signing: true,
        };

        let artifact = NormalizedArtifact {
            id: "driver".to_string(),
            source_ref: crate::common::EvidenceSourceRef::LocalFile {
                path: "/Windows/System32/drivers/contosoflt.sys".to_string(),
            },
            source_id: "/Windows/System32/drivers/contosoflt.sys".to_string(),
            name: "contosoflt.sys".to_string(),
            extension: Some("sys".to_string()),
            size: info.file_size,
            mime_type: None,
            type_description: "Windows Driver Artifact".to_string(),
            category: "system".to_string(),
            confidence: "high".to_string(),
            is_text: false,
            content_preview: None,
            metadata: index_binary_artifact_metadata_from_info(&info),
        };

        let content = artifact_search_terms(&artifact).join("\n");

        assert!(content.contains("binary.indexAnalysisStatus:parsed"));
        assert!(content.contains("binary.format:PE64"));
        assert!(content.contains("binary.importLibraries:fltmgr.sys"));
        assert!(content.contains("binary.exports:DriverEntry"));
        assert!(content.contains("binary.sections:.text"));
        assert!(content.contains("pe.linkerVersion:14.38"));
        assert!(content.contains("pe.imageBase:0x140000000"));
        assert!(content.contains("pe.dllCharacteristics:0x2140"));
        assert!(
            content.contains("pe.dllCharacteristicsDetail:dynamic-base; nx-compatible; wdm-driver")
        );
        assert!(content.contains("pe.certificateTableSize:4096"));
        assert!(content.contains("pe.isDriver:true"));
        assert!(content.contains("pe.driverType:File system minifilter driver"));
        assert!(content
            .contains("pe.driverIndicators:driver file extension; file-system filter driver APIs"));
        assert!(content.contains("pe.version.CompanyName:Contoso Driver Labs"));
        assert!(content.contains("pe.version.OriginalFilename:contosoflt.sys"));
        assert!(content.contains(
            "binary.strings:\\Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt"
        ));
        assert!(content.contains("pe.driverServiceNames:contosoflt; legacyflt"));
        assert!(content.contains("pe.driverDeviceNames:ContosoFilter"));
        assert!(content.contains("pe.driverDosDeviceNames:ContosoFilter"));
        assert!(content.contains("pe.driverRegistryPaths:Registry\\Machine\\System\\CurrentControlSet\\Services\\contosoflt; Registry\\Machine\\System\\ControlSet001\\Services\\legacyflt\\Parameters"));
        assert!(content.contains(
            r"pe.driverPdbPaths:C:\agent\_work\drivers\contosoflt\objfre\amd64\contosoflt.pdb"
        ));
        assert!(content.contains("pe.driverUrls:https://drivers.example.test/support"));
        assert!(content.contains("pe.driverGuids:{12345678-9ABC-DEF0-1234-56789ABCDEF0}"));
    }

    #[test]
    fn pdf_magic_indexes_artifact_metadata_without_plain_text_preview() {
        let file = write_temp_file(".pdf", b"%PDF-1.7\nnot a complete pdf");
        let entry = disk_entry(file.path(), "document", true);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("PDF Document"));
        assert!(content.contains("application/pdf"));
        assert!(content.contains("pdf.version:1.7"));
        assert!(!content.contains("not a complete pdf"));
    }

    #[test]
    fn image_artifact_metadata_is_searchable_even_when_not_text_eligible() {
        let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n"[..]);
        bytes.extend_from_slice(&13u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&320u32.to_be_bytes());
        bytes.extend_from_slice(&200u32.to_be_bytes());
        bytes.extend_from_slice(&[8, 2, 0, 0, 0]);
        let file = write_temp_file(".png", &bytes);
        let entry = disk_entry(file.path(), "image", false);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("PNG Image"));
        assert!(content.contains("image/png"));
        assert!(content.contains("image.dimensions:320x200"));
        assert!(content.contains("image.width:320"));
    }

    #[test]
    fn jpeg_exif_artifact_metadata_is_searchable() {
        let file = write_temp_file(".jpg", &jpeg_with_exif_search_terms());
        let entry = disk_entry(file.path(), "image", false);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("JPEG Image"));
        assert!(content.contains("exif.make:CORE"));
        assert!(content.contains("exif.dateTimeOriginal:2026:02:16 10:01:00"));
        assert!(content.contains("gps.latitude:37.774900"));
        assert!(content.contains("gps.longitude:-122.419400"));
    }

    #[test]
    fn sqlite_artifact_metadata_is_searchable() {
        let mut bytes = vec![0u8; 100];
        bytes[..16].copy_from_slice(b"SQLite format 3\0");
        bytes[16..18].copy_from_slice(&4096u16.to_be_bytes());
        bytes[18] = 1;
        bytes[19] = 1;
        bytes[28..32].copy_from_slice(&12u32.to_be_bytes());
        bytes[56..60].copy_from_slice(&1u32.to_be_bytes());
        let file = write_temp_file(".sqlite", &bytes);
        let entry = disk_entry(file.path(), "database", false);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("SQLite Database"));
        assert!(content.contains("sqlite.pageSize:4096"));
        assert!(content.contains("sqlite.pageCount:12"));
        assert!(content.contains("sqlite.textEncoding:UTF-8"));
    }

    #[test]
    fn email_artifact_metadata_is_searchable() {
        let file = write_temp_file(
            ".eml",
            b"From: Alice <alice@example.com>\r\nTo: Bob <bob@example.com>\r\nSubject: Quarterly update\r\nMessage-ID: <msg-1@example.com>\r\n\r\nBody",
        );
        let entry = disk_entry(file.path(), "email", true);

        let content = extract_content_from_container("disk", &entry);

        assert!(content.contains("Email Message"));
        assert!(content.contains("email.from:Alice <alice@example.com>"));
        assert!(content.contains("email.subject:Quarterly update"));
        assert!(content.contains("email.messageId:<msg-1@example.com>"));
    }

    #[test]
    fn truncate_index_content_preserves_utf8_boundaries() {
        let mut text = "a".repeat(MAX_CONTENT_SIZE - 1);
        text.push('é');
        text.push_str("tail");

        let truncated = truncate_index_content(text);

        assert_eq!(truncated.len(), MAX_CONTENT_SIZE - 1);
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.ends_with('a'));
    }
}
