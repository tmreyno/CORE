// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Background indexing worker for the Tantivy search engine.
//!
//! Crawls container file trees (AD1, L01, Archive, VFS/E01/Raw, UFED) and
//! indexes filenames, metadata, and optionally extracted text content.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tracing::{debug, info, warn};

use crate::ad1;
use crate::archive;
use crate::ewf;
use crate::raw;

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

impl IndexingState {
    pub fn new() -> Self {
        Self {
            files_indexed: AtomicU64::new(0),
            files_total: AtomicU64::new(0),
            content_extracted: AtomicU64::new(0),
            cancel: AtomicBool::new(false),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn progress(&self, container_path: &str, phase: IndexPhase, current: &str) -> IndexProgress {
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
            if let Ok(children) =
                ad1::get_children_at_addr_v2(container_path, item.data_addr.unwrap_or(0), &item.path)
            {
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
    crawl_vfs_recursive(&vfs, container_path, "e01", "/", &mut entries);
    Ok(entries)
}

fn crawl_vfs_raw(container_path: &str) -> Result<Vec<CrawledEntry>, String> {
    let vfs = raw::vfs::RawVfs::open_filesystem(container_path)
        .or_else(|_| raw::vfs::RawVfs::open(container_path))
        .map_err(|e| format!("Failed to open raw VFS: {:?}", e))?;

    let mut entries = Vec::new();
    crawl_vfs_recursive(&vfs, container_path, "raw", "/", &mut entries);
    Ok(entries)
}

fn crawl_vfs_recursive<V: crate::common::vfs::VirtualFileSystem>(
    vfs: &V,
    container_path: &str,
    container_type: &str,
    dir_path: &str,
    entries: &mut Vec<CrawledEntry>,
) {
    let listing = match vfs.readdir(dir_path) {
        Ok(items) => items,
        Err(e) => {
            debug!("VFS readdir failed for {}: {:?}", dir_path, e);
            return;
        }
    };

    for item in &listing {
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
            crawl_vfs_recursive(vfs, container_path, container_type, &full_path, entries);
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
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                crawl_disk_dir(&path, entries);
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

/// Extract text content from a file inside a container.
///
/// Returns the extracted text (truncated to MAX_CONTENT_SIZE), or empty string
/// if extraction fails or is unsupported.
fn extract_content_from_container(
    container_path: &str,
    entry: &CrawledEntry,
) -> String {
    if entry.is_dir || !entry.text_eligible {
        return String::new();
    }

    // Skip very large files for content extraction (> 10 MB)
    if entry.size > 10 * 1024 * 1024 {
        return String::new();
    }

    // Read the raw bytes from the container
    let bytes = match read_entry_bytes(container_path, &entry.entry_path, entry.size, &entry.container_type) {
        Ok(data) => data,
        Err(e) => {
            debug!("Content extraction failed for {}: {}", entry.entry_path, e);
            return String::new();
        }
    };

    if bytes.is_empty() {
        return String::new();
    }

    // Try to extract text based on the file extension/category
    let text = extract_text_from_bytes(&bytes, &entry.extension, &entry.category);

    // Truncate to max size
    if text.len() > MAX_CONTENT_SIZE {
        text[..MAX_CONTENT_SIZE].to_string()
    } else {
        text
    }
}

/// Read raw bytes of a file from its container.
fn read_entry_bytes(
    container_path: &str,
    entry_path: &str,
    size: u64,
    container_type: &str,
) -> Result<Vec<u8>, String> {
    match container_type {
        "ad1" => {
            ad1::read_entry_data(container_path, entry_path)
                .map_err(|e| format!("AD1 read: {}", e))
        }
        "l01" => {
            let tree = ewf::parse_l01_file_tree(container_path)
                .map_err(|e| format!("L01 tree: {}", e))?;
            let entry = tree
                .entry_at_path(entry_path)
                .ok_or_else(|| format!("L01 entry not found: {}", entry_path))?;
            let mut handle = ewf::EwfHandle::open(container_path)
                .map_err(|e| format!("L01 handle: {}", e))?;
            let read_size = if entry.size > 0 {
                entry.size as usize
            } else {
                entry.data_size as usize
            };
            handle
                .read_at(entry.data_offset, read_size)
                .map_err(|e| format!("L01 read: {}", e))
        }
        "archive" => {
            archive::libarchive_read_file(container_path, entry_path)
                .map_err(|e| format!("Archive read: {}", e))
        }
        "e01" => {
            use crate::common::vfs::VirtualFileSystem;
            let vfs = ewf::vfs::EwfVfs::open(container_path)
                .map_err(|e| format!("E01 VFS: {:?}", e))?;
            let read_size = if size > 0 {
                size as usize
            } else {
                vfs.file_size(entry_path).unwrap_or(0) as usize
            };
            vfs.read(entry_path, 0, read_size)
                .map_err(|e| format!("E01 read: {:?}", e))
        }
        "raw" => {
            use crate::common::vfs::VirtualFileSystem;
            let vfs = raw::vfs::RawVfs::open_filesystem(container_path)
                .or_else(|_| raw::vfs::RawVfs::open(container_path))
                .map_err(|e| format!("Raw VFS: {:?}", e))?;
            let read_size = if size > 0 {
                size as usize
            } else {
                vfs.file_size(entry_path).unwrap_or(0) as usize
            };
            vfs.read(entry_path, 0, read_size)
                .map_err(|e| format!("Raw read: {:?}", e))
        }
        "disk" => {
            std::fs::read(entry_path).map_err(|e| format!("Disk read: {}", e))
        }
        _ => Err(format!("Unknown container type: {}", container_type)),
    }
}

/// Extract text content from raw bytes using the file extension to select
/// the appropriate parser.
fn extract_text_from_bytes(data: &[u8], ext: &str, _category: &str) -> String {
    match ext {
        // Plain text (including code, config, markup)
        "txt" | "log" | "md" | "rst" | "tex" | "json" | "xml" | "yaml" | "yml" | "toml"
        | "ini" | "cfg" | "conf" | "env" | "csv" | "tsv" | "html" | "htm" | "xhtml" | "css"
        | "scss" | "less" | "svg" | "py" | "js" | "ts" | "jsx" | "tsx" | "rs" | "c" | "cpp"
        | "h" | "java" | "go" | "rb" | "php" | "swift" | "kt" | "cs" | "sh" | "bat" | "ps1"
        | "vbs" | "pl" | "r" | "m" | "sql" | "readme" | "changelog" | "license" => {
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
        let mut xml = String::new();
        if std::io::Read::read_to_string(&mut file, &mut xml).is_ok() {
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

    info!(
        "Crawled {} entries from {}",
        entries.len(),
        container_path
    );

    // Index each entry
    let mut indexed = 0u64;
    for entry in &entries {
        if state.is_cancelled() {
            info!("Indexing cancelled for {}", container_path);
            break;
        }

        // Extract content if requested and eligible
        let content = if index_content && entry.text_eligible {
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

        let content = if index_content && entry.text_eligible {
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
            writer.delete_all_documents()
                .map_err(|e| format!("Failed to clear index: {}", e))?;
            writer.commit()
                .map_err(|e| format!("Failed to commit clear: {}", e))?;
        }
    }

    let mut total = 0u64;
    for path in container_paths {
        if state.is_cancelled() {
            break;
        }
        total += index_container(search_index, path, index_content, state)?;
    }

    Ok(total)
}
