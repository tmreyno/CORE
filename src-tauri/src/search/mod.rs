// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Full-text search engine powered by Tantivy.
//!
//! Provides background indexing and instant search across all container types:
//! - Filenames and paths (always indexed)
//! - File content (text-eligible files only, background extraction)
//! - Metadata (size, type, timestamps)
//!
//! The index is stored alongside the project `.ffxdb` file as a directory
//! named `<project>.ffxdb-index/`.

pub mod indexer;
pub mod query;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};
use tracing::{info, warn};

// =============================================================================
// Global Search Index Registry (per-window, keyed by window label)
// =============================================================================

static SEARCH_INDEXES: LazyLock<Mutex<HashMap<String, Arc<SearchIndex>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Get the search index for a window, if one is open.
pub fn get_search_index(window_label: &str) -> Option<Arc<SearchIndex>> {
    SEARCH_INDEXES
        .lock()
        .ok()
        .and_then(|map| map.get(window_label).cloned())
}

/// Register a search index for a window.
pub fn set_search_index(window_label: &str, index: Arc<SearchIndex>) {
    if let Ok(mut map) = SEARCH_INDEXES.lock() {
        map.insert(window_label.to_string(), index);
    }
}

/// Remove and close the search index for a window.
pub fn remove_search_index(window_label: &str) {
    if let Ok(mut map) = SEARCH_INDEXES.lock() {
        if let Some(idx) = map.remove(window_label) {
            // Commit any pending writes before dropping
            if let Err(e) = idx.commit() {
                warn!("Failed to commit search index on close: {}", e);
            }
            info!("Search index closed for window '{}'", window_label);
        }
    }
}

// =============================================================================
// Schema Definition
// =============================================================================

/// Field names used in the Tantivy schema.
pub struct SearchFields {
    pub doc_id: Field,
    pub container_path: Field,
    pub container_type: Field,
    pub entry_path: Field,
    pub filename: Field,
    pub extension: Field,
    pub content: Field,
    pub size: Field,
    pub modified: Field,
    pub is_dir: Field,
    pub file_category: Field,
}

/// Build the Tantivy schema for the search index.
fn build_schema() -> (Schema, SearchFields) {
    let mut builder = Schema::builder();

    // doc_id: unique identifier, stored + indexed for dedup/delete
    let doc_id = builder.add_text_field("doc_id", STRING | STORED);

    // container_path: which container this entry belongs to
    let container_path = builder.add_text_field("container_path", STRING | STORED);

    // container_type: e.g. "e01", "ad1", "zip", "l01", "disk"
    let container_type = builder.add_text_field("container_type", STRING | STORED);

    // entry_path: full path within container (stored for display)
    let entry_path = builder.add_text_field("entry_path", TEXT | STORED);

    // filename: just the file name, tokenized for search
    let filename = builder.add_text_field(
        "filename",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    // extension: file extension (e.g., "pdf", "docx")
    let extension = builder.add_text_field("extension", STRING | STORED);

    // content: extracted text content (indexed, stored for snippets)
    let content = builder.add_text_field(
        "content",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    // size: file size in bytes (fast field for range queries + sorting)
    let size = builder.add_u64_field("size", INDEXED | STORED | FAST);

    // modified: unix timestamp (fast field for date range queries)
    let modified = builder.add_i64_field("modified", INDEXED | STORED | FAST);

    // is_dir: boolean as u64 (0 or 1)
    let is_dir = builder.add_u64_field("is_dir", INDEXED | STORED);

    // file_category: "document", "email", "code", "image", "database", "binary", "other"
    let file_category = builder.add_text_field("file_category", STRING | STORED);

    let schema = builder.build();
    let fields = SearchFields {
        doc_id,
        container_path,
        container_type,
        entry_path,
        filename,
        extension,
        content,
        size,
        modified,
        is_dir,
        file_category,
    };

    (schema, fields)
}

// =============================================================================
// SearchIndex — lifecycle management
// =============================================================================

/// Manages a Tantivy index instance with its schema fields.
pub struct SearchIndex {
    pub index: Index,
    pub fields: SearchFields,
    pub schema: Schema,
    writer: Mutex<Option<IndexWriter>>,
    reader: IndexReader,
    index_path: PathBuf,
}

impl SearchIndex {
    /// Open or create a search index at the given directory path.
    ///
    /// The `index_dir` is typically `<project>.ffxdb-index/`.
    pub fn open_or_create(index_dir: &Path) -> Result<Self, String> {
        let (schema, fields) = build_schema();

        std::fs::create_dir_all(index_dir)
            .map_err(|e| format!("Failed to create search index directory: {}", e))?;

        let mmap_dir = MmapDirectory::open(index_dir)
            .map_err(|e| format!("Failed to open MmapDirectory: {}", e))?;

        let index = if Index::exists(&mmap_dir).unwrap_or(false) {
            info!("Opening existing search index at {:?}", index_dir);
            Index::open(mmap_dir)
                .map_err(|e| format!("Failed to open search index: {}", e))?
        } else {
            info!("Creating new search index at {:?}", index_dir);
            Index::create_in_dir(index_dir, schema.clone())
                .map_err(|e| format!("Failed to create search index: {}", e))?
        };

        // Create a writer with 50 MB heap budget
        let writer = index
            .writer(50_000_000)
            .map_err(|e| format!("Failed to create index writer: {}", e))?;

        // Create a reader with auto-reload for latest commits
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| format!("Failed to create index reader: {}", e))?;

        Ok(Self {
            index,
            fields,
            schema,
            writer: Mutex::new(Some(writer)),
            reader,
            index_path: index_dir.to_path_buf(),
        })
    }

    /// Get a reader (searcher) for the index.
    pub fn searcher(&self) -> tantivy::Searcher {
        self.reader.searcher()
    }

    /// Add a document to the index (does not auto-commit).
    pub fn add_document(
        &self,
        doc_id: &str,
        container_path: &str,
        container_type: &str,
        entry_path: &str,
        filename: &str,
        extension: &str,
        content: &str,
        size: u64,
        modified: i64,
        is_dir: bool,
        file_category: &str,
    ) -> Result<(), String> {
        let mut writer_guard = self
            .writer
            .lock()
            .map_err(|e| format!("Writer lock poisoned: {}", e))?;
        let writer = writer_guard
            .as_mut()
            .ok_or("Index writer has been closed")?;

        // Delete existing document with same doc_id (upsert behavior)
        let doc_id_term = tantivy::Term::from_field_text(self.fields.doc_id, doc_id);
        writer.delete_term(doc_id_term);

        writer
            .add_document(doc!(
                self.fields.doc_id => doc_id,
                self.fields.container_path => container_path,
                self.fields.container_type => container_type,
                self.fields.entry_path => entry_path,
                self.fields.filename => filename,
                self.fields.extension => extension,
                self.fields.content => content,
                self.fields.size => size,
                self.fields.modified => modified,
                self.fields.is_dir => if is_dir { 1u64 } else { 0u64 },
                self.fields.file_category => file_category,
            ))
            .map_err(|e| format!("Failed to add document: {}", e))?;

        Ok(())
    }

    /// Commit all pending writes to disk.
    pub fn commit(&self) -> Result<(), String> {
        let mut writer_guard = self
            .writer
            .lock()
            .map_err(|e| format!("Writer lock poisoned: {}", e))?;
        if let Some(ref mut writer) = *writer_guard {
            writer
                .commit()
                .map_err(|e| format!("Failed to commit index: {}", e))?;
        }
        Ok(())
    }

    /// Delete all documents for a given container path.
    pub fn delete_container(&self, container_path: &str) -> Result<(), String> {
        let mut writer_guard = self
            .writer
            .lock()
            .map_err(|e| format!("Writer lock poisoned: {}", e))?;
        let writer = writer_guard
            .as_mut()
            .ok_or("Index writer has been closed")?;

        let term = tantivy::Term::from_field_text(self.fields.container_path, container_path);
        writer.delete_term(term);

        Ok(())
    }

    /// Delete the entire index from disk.
    pub fn destroy(self) -> Result<(), String> {
        // Drop writer first
        {
            let mut writer_guard = self
                .writer
                .lock()
                .map_err(|e| format!("Writer lock poisoned: {}", e))?;
            *writer_guard = None;
        }
        drop(self.reader);

        if self.index_path.exists() {
            std::fs::remove_dir_all(&self.index_path)
                .map_err(|e| format!("Failed to delete index directory: {}", e))?;
        }
        Ok(())
    }

    /// Get statistics about the index.
    pub fn stats(&self) -> IndexStats {
        let searcher = self.searcher();
        let num_docs = searcher.num_docs() as u64;
        let num_segments = searcher.segment_readers().len() as u64;
        let index_size = dir_size(&self.index_path);

        // Count documents with non-empty content field
        let content_docs = {
            use tantivy::query::AllQuery;
            let count = searcher
                .search(&AllQuery, &TopDocs::with_limit(0))
                .map(|r| r.len())
                .unwrap_or(0);
            count as u64
        };

        IndexStats {
            num_docs,
            num_segments,
            index_size_bytes: index_size,
            content_indexed_docs: content_docs,
        }
    }
}

/// Statistics about a search index.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub num_docs: u64,
    pub num_segments: u64,
    pub index_size_bytes: u64,
    pub content_indexed_docs: u64,
}

/// Calculate total size of a directory tree.
fn dir_size(path: &Path) -> u64 {
    if !path.is_dir() {
        return 0;
    }
    std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| {
                    let meta = e.metadata().ok();
                    if e.path().is_dir() {
                        dir_size(&e.path())
                    } else {
                        meta.map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum()
        })
        .unwrap_or(0)
}

// =============================================================================
// File Category Classification
// =============================================================================

/// Classify a file extension into a search category.
pub fn classify_extension(ext: &str) -> &'static str {
    let lower = ext.to_lowercase();
    match lower.as_str() {
        // Documents
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "pages" => "document",
        // Spreadsheets
        "xls" | "xlsx" | "ods" | "csv" | "tsv" => "spreadsheet",
        // Presentations
        "ppt" | "pptx" | "odp" | "key" => "presentation",
        // Email
        "eml" | "msg" | "mbox" | "pst" | "ost" => "email",
        // Code & Scripts
        "py" | "js" | "ts" | "jsx" | "tsx" | "rs" | "c" | "cpp" | "h" | "java" | "go" | "rb"
        | "php" | "swift" | "kt" | "cs" | "sh" | "bat" | "ps1" | "vbs" | "pl" | "r" | "m"
        | "sql" => "code",
        // Config & Data
        "json" | "xml" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "env" | "plist"
        | "reg" => "config",
        // Text
        "txt" | "log" | "md" | "rst" | "tex" | "readme" | "changelog" | "license" => "text",
        // Markup
        "html" | "htm" | "xhtml" | "svg" | "css" | "scss" | "less" => "markup",
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "tiff" | "tif" | "webp" | "heic"
        | "heif" | "raw" | "cr2" | "nef" | "arw" => "image",
        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => "audio",
        // Video
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => "video",
        // Archives
        "zip" | "7z" | "rar" | "tar" | "gz" | "bz2" | "xz" | "zst" => "archive",
        // Databases
        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "dbf" => "database",
        // Executables
        "exe" | "dll" | "so" | "dylib" | "a" | "o" | "sys" | "drv" | "msi" => "executable",
        // Forensic
        "e01" | "l01" | "ad1" | "dd" | "dmg" | "iso" | "img" | "mem" | "vmdk"
        | "vhd" | "vhdx" => "forensic",
        // Registry
        "dat" | "hiv" => "registry",
        // Everything else
        _ => "other",
    }
}

/// Check if a file category is text-eligible (content can be extracted and indexed).
pub fn is_text_eligible(category: &str) -> bool {
    matches!(
        category,
        "text" | "code" | "config" | "markup" | "email" | "document" | "spreadsheet"
    )
}

/// Check if a file extension represents text-extractable content.
pub fn is_text_extractable_ext(ext: &str) -> bool {
    let category = classify_extension(ext);
    is_text_eligible(category)
}

/// Derive the index directory path from a .ffxdb path.
pub fn index_path_from_ffxdb(ffxdb_path: &str) -> PathBuf {
    PathBuf::from(format!("{}-index", ffxdb_path))
}
