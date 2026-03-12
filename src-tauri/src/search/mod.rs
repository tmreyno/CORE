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
            Index::open(mmap_dir).map_err(|e| format!("Failed to open search index: {}", e))?
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
                .search(&AllQuery, &tantivy::collector::Count)
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
        "e01" | "l01" | "ad1" | "dd" | "dmg" | "iso" | "img" | "mem" | "vmdk" | "vhd" | "vhdx" => {
            "forensic"
        }
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // classify_extension — comprehensive
    // -------------------------------------------------------------------------

    #[test]
    fn classify_documents() {
        for ext in &["pdf", "doc", "docx", "odt", "rtf", "pages"] {
            assert_eq!(classify_extension(ext), "document", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_spreadsheets() {
        for ext in &["xls", "xlsx", "ods", "csv", "tsv"] {
            assert_eq!(classify_extension(ext), "spreadsheet", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_presentations() {
        for ext in &["ppt", "pptx", "odp", "key"] {
            assert_eq!(
                classify_extension(ext),
                "presentation",
                "Failed for {}",
                ext
            );
        }
    }

    #[test]
    fn classify_email() {
        for ext in &["eml", "msg", "mbox", "pst", "ost"] {
            assert_eq!(classify_extension(ext), "email", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_code() {
        for ext in &[
            "py", "js", "ts", "rs", "c", "cpp", "java", "go", "rb", "sql",
        ] {
            assert_eq!(classify_extension(ext), "code", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_config() {
        for ext in &[
            "json", "xml", "yaml", "yml", "toml", "ini", "cfg", "plist", "reg",
        ] {
            assert_eq!(classify_extension(ext), "config", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_text() {
        for ext in &[
            "txt",
            "log",
            "md",
            "rst",
            "tex",
            "readme",
            "changelog",
            "license",
        ] {
            assert_eq!(classify_extension(ext), "text", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_markup() {
        for ext in &["html", "htm", "xhtml", "svg", "css", "scss", "less"] {
            assert_eq!(classify_extension(ext), "markup", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_images() {
        for ext in &[
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic", "raw",
        ] {
            assert_eq!(classify_extension(ext), "image", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_audio() {
        for ext in &["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a"] {
            assert_eq!(classify_extension(ext), "audio", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_video() {
        for ext in &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm"] {
            assert_eq!(classify_extension(ext), "video", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_archives() {
        for ext in &["zip", "7z", "rar", "tar", "gz", "bz2", "xz", "zst"] {
            assert_eq!(classify_extension(ext), "archive", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_databases() {
        for ext in &["db", "sqlite", "sqlite3", "mdb", "accdb", "dbf"] {
            assert_eq!(classify_extension(ext), "database", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_executables() {
        for ext in &["exe", "dll", "so", "dylib", "sys", "msi"] {
            assert_eq!(classify_extension(ext), "executable", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_forensic() {
        for ext in &[
            "e01", "l01", "ad1", "dd", "dmg", "iso", "mem", "vmdk", "vhd",
        ] {
            assert_eq!(classify_extension(ext), "forensic", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_registry() {
        for ext in &["dat", "hiv"] {
            assert_eq!(classify_extension(ext), "registry", "Failed for {}", ext);
        }
    }

    #[test]
    fn classify_unknown_returns_other() {
        assert_eq!(classify_extension("xyz"), "other");
        assert_eq!(classify_extension("foo"), "other");
        assert_eq!(classify_extension(""), "other");
    }

    #[test]
    fn classify_case_insensitive() {
        assert_eq!(classify_extension("PDF"), "document");
        assert_eq!(classify_extension("Jpg"), "image");
        assert_eq!(classify_extension("ZIP"), "archive");
        assert_eq!(classify_extension("RS"), "code");
    }

    // -------------------------------------------------------------------------
    // is_text_eligible
    // -------------------------------------------------------------------------

    #[test]
    fn text_eligible_categories() {
        assert!(is_text_eligible("text"));
        assert!(is_text_eligible("code"));
        assert!(is_text_eligible("config"));
        assert!(is_text_eligible("markup"));
        assert!(is_text_eligible("email"));
        assert!(is_text_eligible("document"));
        assert!(is_text_eligible("spreadsheet"));
    }

    #[test]
    fn non_text_eligible_categories() {
        assert!(!is_text_eligible("image"));
        assert!(!is_text_eligible("video"));
        assert!(!is_text_eligible("audio"));
        assert!(!is_text_eligible("archive"));
        assert!(!is_text_eligible("executable"));
        assert!(!is_text_eligible("forensic"));
        assert!(!is_text_eligible("database"));
        assert!(!is_text_eligible("other"));
    }

    // -------------------------------------------------------------------------
    // is_text_extractable_ext (integration: extension → category → eligible)
    // -------------------------------------------------------------------------

    #[test]
    fn text_extractable_extensions() {
        assert!(is_text_extractable_ext("txt"));
        assert!(is_text_extractable_ext("py"));
        assert!(is_text_extractable_ext("json"));
        assert!(is_text_extractable_ext("html"));
        assert!(is_text_extractable_ext("eml"));
        assert!(is_text_extractable_ext("pdf"));
        assert!(is_text_extractable_ext("csv"));
    }

    #[test]
    fn non_text_extractable_extensions() {
        assert!(!is_text_extractable_ext("jpg"));
        assert!(!is_text_extractable_ext("mp4"));
        assert!(!is_text_extractable_ext("exe"));
        assert!(!is_text_extractable_ext("zip"));
        assert!(!is_text_extractable_ext("e01"));
        assert!(!is_text_extractable_ext("db"));
    }

    // -------------------------------------------------------------------------
    // index_path_from_ffxdb
    // -------------------------------------------------------------------------

    #[test]
    fn index_path_appends_suffix() {
        let result = index_path_from_ffxdb("/path/to/project.ffxdb");
        assert_eq!(result, PathBuf::from("/path/to/project.ffxdb-index"));
    }

    #[test]
    fn index_path_handles_relative() {
        let result = index_path_from_ffxdb("project.ffxdb");
        assert_eq!(result, PathBuf::from("project.ffxdb-index"));
    }

    // -------------------------------------------------------------------------
    // dir_size helper
    // -------------------------------------------------------------------------

    #[test]
    fn dir_size_nonexistent_returns_zero() {
        assert_eq!(dir_size(Path::new("/nonexistent/path/abc123")), 0);
    }

    #[test]
    fn dir_size_empty_temp_dir() {
        let tmp = tempfile::tempdir().unwrap();
        // Empty directory should have size 0
        assert_eq!(dir_size(tmp.path()), 0);
    }

    #[test]
    fn dir_size_with_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("a.txt"), "hello").unwrap(); // 5 bytes
        std::fs::write(tmp.path().join("b.txt"), "world!").unwrap(); // 6 bytes
        assert_eq!(dir_size(tmp.path()), 11);
    }

    #[test]
    fn dir_size_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("file.txt"), "abc").unwrap(); // 3 bytes
        std::fs::write(tmp.path().join("root.txt"), "xy").unwrap(); // 2 bytes
        assert_eq!(dir_size(tmp.path()), 5);
    }

    // -------------------------------------------------------------------------
    // SearchIndex — create, add docs, search, stats, destroy
    // -------------------------------------------------------------------------

    #[test]
    fn search_index_create_and_stats() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        // Empty index
        let stats = idx.stats();
        assert_eq!(stats.num_docs, 0);
        assert_eq!(stats.index_size_bytes > 0, true); // Tantivy writes metadata files

        idx.destroy().unwrap();
    }

    #[test]
    fn search_index_add_and_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "container1:docs/report.pdf",
            "/evidence/case.ad1",
            "ad1",
            "docs/report.pdf",
            "report.pdf",
            "pdf",
            "This is the forensic examination report.",
            1024,
            1700000000,
            false,
            "document",
        )
        .unwrap();

        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let stats = idx.stats();
        assert_eq!(stats.num_docs, 1);

        idx.destroy().unwrap();
    }

    #[test]
    fn search_index_upsert_replaces_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        // Add same doc_id twice with different content
        idx.add_document(
            "c:file.txt",
            "c.ad1",
            "ad1",
            "file.txt",
            "file.txt",
            "txt",
            "version 1",
            100,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c:file.txt",
            "c.ad1",
            "ad1",
            "file.txt",
            "file.txt",
            "txt",
            "version 2",
            200,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let stats = idx.stats();
        // After upsert, should only have 1 document
        assert_eq!(stats.num_docs, 1);

        idx.destroy().unwrap();
    }

    #[test]
    fn search_index_delete_container() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c1:a.txt",
            "/case/c1.ad1",
            "ad1",
            "a.txt",
            "a.txt",
            "txt",
            "",
            100,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c1:b.txt",
            "/case/c1.ad1",
            "ad1",
            "b.txt",
            "b.txt",
            "txt",
            "",
            200,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c2:c.txt",
            "/case/c2.e01",
            "e01",
            "c.txt",
            "c.txt",
            "txt",
            "",
            300,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        assert_eq!(idx.stats().num_docs, 3);

        // Delete all docs from c1
        idx.delete_container("/case/c1.ad1").unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        assert_eq!(idx.stats().num_docs, 1);

        idx.destroy().unwrap();
    }

    #[test]
    fn search_index_reopen_persists_data() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().to_path_buf();

        // Create and add data
        {
            let idx = SearchIndex::open_or_create(&path).unwrap();
            idx.add_document(
                "c:a.txt",
                "c.ad1",
                "ad1",
                "a.txt",
                "a.txt",
                "txt",
                "hello world",
                11,
                0,
                false,
                "text",
            )
            .unwrap();
            idx.commit().unwrap();
            // Drop without destroy — data persists
        }

        // Reopen and verify data is still there
        {
            let idx = SearchIndex::open_or_create(&path).unwrap();
            idx.reader.reload().unwrap();
            assert_eq!(idx.stats().num_docs, 1);
            idx.destroy().unwrap();
        }
    }

    // -------------------------------------------------------------------------
    // Search query integration tests (add docs → search → verify hits)
    // -------------------------------------------------------------------------

    #[test]
    fn search_by_filename() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:photos/vacation.jpg",
            "c.ad1",
            "ad1",
            "photos/vacation.jpg",
            "vacation.jpg",
            "jpg",
            "",
            50000,
            0,
            false,
            "image",
        )
        .unwrap();
        idx.add_document(
            "c:docs/report.pdf",
            "c.ad1",
            "ad1",
            "docs/report.pdf",
            "report.pdf",
            "pdf",
            "",
            1024,
            0,
            false,
            "document",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: "vacation".to_string(),
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec![],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 1);
        assert_eq!(results.hits[0].filename, "vacation.jpg");

        idx.destroy().unwrap();
    }

    #[test]
    fn search_by_content() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:notes.txt",
            "c.ad1",
            "ad1",
            "notes.txt",
            "notes.txt",
            "txt",
            "The suspect was seen near the warehouse at midnight",
            500,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c:other.txt",
            "c.ad1",
            "ad1",
            "other.txt",
            "other.txt",
            "txt",
            "This file contains nothing relevant",
            200,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: "warehouse midnight".to_string(),
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec![],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: true,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert!(!results.hits.is_empty());
        assert_eq!(results.hits[0].filename, "notes.txt");
        assert!(results.hits[0].content_match);

        idx.destroy().unwrap();
    }

    #[test]
    fn search_filter_by_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:a.pdf",
            "c.ad1",
            "ad1",
            "a.pdf",
            "report.pdf",
            "pdf",
            "",
            1000,
            0,
            false,
            "document",
        )
        .unwrap();
        idx.add_document(
            "c:b.docx",
            "c.ad1",
            "ad1",
            "b.docx",
            "report.docx",
            "docx",
            "",
            2000,
            0,
            false,
            "document",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: "report".to_string(),
            limit: 10,
            container_types: vec![],
            extensions: vec!["pdf".to_string()],
            categories: vec![],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 1);
        assert_eq!(results.hits[0].extension, "pdf");

        idx.destroy().unwrap();
    }

    #[test]
    fn search_filter_by_category() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:a.jpg",
            "c.ad1",
            "ad1",
            "photo.jpg",
            "photo.jpg",
            "jpg",
            "",
            50000,
            0,
            false,
            "image",
        )
        .unwrap();
        idx.add_document(
            "c:b.eml",
            "c.ad1",
            "ad1",
            "message.eml",
            "message.eml",
            "eml",
            "",
            2000,
            0,
            false,
            "email",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: String::new(), // All docs
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec!["email".to_string()],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 1);
        assert_eq!(results.hits[0].file_category, "email");

        idx.destroy().unwrap();
    }

    #[test]
    fn search_filter_by_size_range() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:small.txt",
            "c.ad1",
            "ad1",
            "small.txt",
            "small.txt",
            "txt",
            "",
            100,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c:medium.txt",
            "c.ad1",
            "ad1",
            "medium.txt",
            "medium.txt",
            "txt",
            "",
            5000,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.add_document(
            "c:large.txt",
            "c.ad1",
            "ad1",
            "large.txt",
            "large.txt",
            "txt",
            "",
            1_000_000,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: String::new(),
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec![],
            min_size: Some(1000),
            max_size: Some(10000),
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 1);
        assert_eq!(results.hits[0].filename, "medium.txt");

        idx.destroy().unwrap();
    }

    #[test]
    fn search_excludes_directories_by_default() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:docs/", "c.ad1", "ad1", "docs", "docs", "", "", 0, 0, true, "other",
        )
        .unwrap();
        idx.add_document(
            "c:docs/file.txt",
            "c.ad1",
            "ad1",
            "docs/file.txt",
            "file.txt",
            "txt",
            "",
            100,
            0,
            false,
            "text",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        // Without include_dirs
        let opts = query::SearchOptions {
            query: String::new(),
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec![],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 1);
        assert!(!results.hits[0].is_dir);

        idx.destroy().unwrap();
    }

    #[test]
    fn search_facet_counts() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        idx.add_document(
            "c:a.pdf", "c.ad1", "ad1", "a.pdf", "a.pdf", "pdf", "", 100, 0, false, "document",
        )
        .unwrap();
        idx.add_document(
            "c:b.pdf", "c.ad1", "ad1", "b.pdf", "b.pdf", "pdf", "", 200, 0, false, "document",
        )
        .unwrap();
        idx.add_document(
            "c:c.jpg", "c.e01", "e01", "c.jpg", "c.jpg", "jpg", "", 300, 0, false, "image",
        )
        .unwrap();
        idx.commit().unwrap();
        idx.reader.reload().unwrap();

        let opts = query::SearchOptions {
            query: String::new(),
            limit: 10,
            container_types: vec![],
            extensions: vec![],
            categories: vec![],
            min_size: None,
            max_size: None,
            include_dirs: false,
            search_content: false,
            container_path: None,
        };

        let results = query::search(&idx, &opts).unwrap();
        assert_eq!(results.hits.len(), 3);

        // Category facets
        let doc_count = results
            .category_counts
            .iter()
            .find(|f| f.label == "document")
            .map(|f| f.count)
            .unwrap_or(0);
        assert_eq!(doc_count, 2);

        let img_count = results
            .category_counts
            .iter()
            .find(|f| f.label == "image")
            .map(|f| f.count)
            .unwrap_or(0);
        assert_eq!(img_count, 1);

        // Container type facets
        let ad1_count = results
            .container_type_counts
            .iter()
            .find(|f| f.label == "ad1")
            .map(|f| f.count)
            .unwrap_or(0);
        assert_eq!(ad1_count, 2);

        idx.destroy().unwrap();
    }

    // -------------------------------------------------------------------------
    // SearchOptions deserialization
    // -------------------------------------------------------------------------

    #[test]
    fn search_options_defaults() {
        let json = r#"{ "query": "test" }"#;
        let opts: query::SearchOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.query, "test");
        assert_eq!(opts.limit, 100);
        assert!(opts.search_content);
        assert!(!opts.include_dirs);
        assert!(opts.container_types.is_empty());
    }

    // -------------------------------------------------------------------------
    // Global registry
    // -------------------------------------------------------------------------

    #[test]
    fn global_registry_set_get_remove() {
        let tmp = tempfile::tempdir().unwrap();
        let idx = SearchIndex::open_or_create(tmp.path()).unwrap();

        let label = "test-window-registry";
        set_search_index(label, Arc::new(idx));

        assert!(get_search_index(label).is_some());

        remove_search_index(label);
        assert!(get_search_index(label).is_none());
    }

    #[test]
    fn global_registry_missing_returns_none() {
        assert!(get_search_index("nonexistent-window-xyz123").is_none());
    }
}
