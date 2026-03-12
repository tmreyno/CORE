# Search Architecture

> **Engine:** Tantivy 0.22 (Rust full-text search)  
> **Scope:** Filenames, paths, metadata, and optionally file content across all evidence containers  
> **Index location:** `<project>.ffxdb-index/` (MmapDirectory, alongside the `.ffxdb` database)

---

## Table of Contents

1. [Overview](#overview)
2. [Index Schema](#index-schema)
3. [Index Lifecycle](#index-lifecycle)
4. [Container Crawlers](#container-crawlers)
5. [Content Extraction Pipeline](#content-extraction-pipeline)
6. [Query Engine](#query-engine)
7. [Frontend Integration](#frontend-integration)
8. [FTS5 (SQLite) — Complementary Search](#fts5-sqlite--complementary-search)
9. [Key Files](#key-files)

---

## Overview

CORE-FFX has a **two-tier search system**:

| Engine | Scope | Use Case |
|--------|-------|----------|
| **Tantivy** (this doc) | Evidence container contents — filenames, paths, file content | Primary cross-container search |
| **FTS5** (SQLite) | Notes, bookmarks, activity log text | Annotation and audit trail search |

Tantivy indexes are **per-project, per-window**. Each Tauri window can have an independent search index. Indexes are stored as memory-mapped files in a directory alongside the project database.

```text
my-project.cffx          ← project file (JSON)
my-project.ffxdb          ← SQLite database (FTS5 tables inside)
my-project.ffxdb-index/   ← Tantivy index directory (MmapDirectory)
```

---

## Index Schema

The Tantivy index has **12 fields**:

| Field | Tantivy Type | Options | Purpose |
|-------|-------------|---------|---------|
| `doc_id` | STRING | STORED | Unique ID: `{container_path}:{entry_path}` |
| `container_path` | STRING | STORED | Source container file path |
| `container_type` | STRING | STORED | `ad1`, `e01`, `l01`, `archive`, `raw`, `disk` |
| `entry_path` | TEXT | STORED, tokenized (freqs + positions) | Full path within container |
| `filename` | TEXT | STORED, tokenized (freqs + positions) | Filename only — **boosted 3× in queries** |
| `extension` | STRING | STORED | Lowercase file extension |
| `content` | TEXT | Tokenized (positions only, NOT stored) | Extracted text content |
| `size` | u64 | INDEXED, STORED, FAST | File size in bytes |
| `modified` | i64 | INDEXED, STORED, FAST | Last modified timestamp (Unix seconds) |
| `is_dir` | u64 | INDEXED, STORED | 0 = file, 1 = directory |
| `file_category` | STRING | STORED | Classified category (see below) |

### File Categories (16)

Every indexed file is classified into one of 16 categories by `classify_extension()`:

| Category | Extensions |
|----------|-----------|
| `document` | pdf, doc, docx, odt, rtf, pages |
| `spreadsheet` | xls, xlsx, ods, csv, tsv, numbers |
| `presentation` | ppt, pptx, odp, key |
| `email` | eml, msg, mbox, pst, ost |
| `code` | py, js, ts, rs, c, cpp, java, go, rb, php, swift, kt, cs, sh, bat, ps1, sql, r, m |
| `config` | json, xml, yaml, yml, toml, ini, cfg, conf, env, plist |
| `text` | txt, log, md, rst, tex, readme, changelog, license |
| `markup` | html, htm, xhtml, css, scss, less, svg |
| `image` | jpg, jpeg, png, gif, bmp, tiff, webp, ico, heic, heif, raw, cr2, nef, dng |
| `audio` | mp3, wav, flac, aac, ogg, wma, m4a, aiff |
| `video` | mp4, avi, mkv, mov, wmv, flv, webm, m4v, mpg, mpeg |
| `archive` | zip, 7z, rar, tar, gz, bz2, xz, dmg, iso |
| `database` | db, sqlite, sqlite3, mdb, accdb |
| `executable` | exe, dll, so, dylib, app, msi, deb, rpm, dmg |
| `forensic` | e01, ex01, l01, ad1, aff, aff4, raw, dd, img |
| `registry` | reg, dat |
| `other` | everything else |

### Text Eligibility

Content extraction is only attempted for categories that are likely to contain readable text:

```
text, code, config, markup, email, document, spreadsheet
```

All other categories (images, audio, video, executables, etc.) are indexed by metadata only.

---

## Index Lifecycle

```text
Project Load
  → useSearchIndex hook (App.tsx)
    → search_open_index (opens/creates Tantivy index)
    → search_index_all (indexes all discovered containers)
      → For each container: search_index_container
        → Crawl → Extract → Index → Commit
  
Project Close
  → search_close_index (removes from global registry)
```

### Global Registry

`SEARCH_INDEXES` is a `LazyLock<Mutex<HashMap<String, Arc<SearchIndex>>>>` keyed by **Tauri window label**. This supports multi-window project isolation — each window has its own index.

### SearchIndex Operations

| Method | Description |
|--------|-------------|
| `open_or_create(path)` | Opens existing index or creates new one. MmapDirectory, 50 MB writer heap budget. |
| `add_document(...)` | Upserts a document (delete by `doc_id` term + add). |
| `commit()` | Commits pending writes. Reader reloads on `OnCommitWithDelay`. |
| `delete_container(path)` | Deletes all documents from a specific container. |
| `destroy()` | Deletes the entire index directory from disk. |
| `stats()` | Returns document count + approximate disk size. |

---

## Container Crawlers

Each container format has a dedicated crawler that enumerates all files and directories:

| Crawler | Container Type | Method |
|---------|---------------|--------|
| `crawl_ad1` | AD1 | `ad1::get_root_children_v2` + recursive `get_children_at_addr_v2` |
| `crawl_l01` | L01 | `ewf::parse_l01_file_tree` → iterate flat entry list |
| `crawl_archive` | ZIP, 7z, TAR, RAR, ISO | `archive::libarchive_list_all` |
| `crawl_vfs_ewf` | E01/Ex01 | `EwfVfs::open` + recursive `readdir`/`getattr` |
| `crawl_vfs_raw` | Raw/DD | `RawVfs::open_filesystem` + recursive `readdir`/`getattr` |
| `crawl_disk_files` | Filesystem | `std::fs` recursive directory walk |

Each crawler produces a `Vec<CrawledEntry>`:

```rust
struct CrawledEntry {
    doc_id: String,           // "{container_path}:{entry_path}"
    container_path: String,
    container_type: String,
    entry_path: String,
    filename: String,
    extension: String,
    size: u64,
    modified: i64,
    is_dir: bool,
    category: String,         // from classify_extension()
    text_eligible: bool,      // from is_text_eligible()
}
```

### Container Type Auto-Detection

The indexer detects container type by calling format-specific detection functions in priority order:

```text
is_ad1() → is_l01() → is_archive() → is_ewf() → is_raw() → skip
```

---

## Content Extraction Pipeline

When `index_content = true`, the indexer extracts text from files inside containers.

### Extraction Constraints

| Constraint | Value | Rationale |
|-----------|-------|-----------|
| Max content size | 256 KB | Prevent index bloat from large files |
| Max file size for extraction | 10 MB | Skip huge binaries/media |
| Text eligibility | Category-based | Only text-like categories (see above) |

### Extraction Flow

```text
CrawledEntry (text_eligible = true)
  → read_entry_bytes(container_path, entry)
    → 6-arm routing: AD1/L01/Archive/E01-VFS/Raw-VFS/Disk
    → Returns raw bytes (up to 256 KB)
  → extract_text_from_bytes(bytes, extension, category)
    → Dispatch by extension to appropriate parser
  → Truncate to 256 KB
  → Index as "content" field
```

### Content Parsers

| Format | Extensions | Method |
|--------|-----------|--------|
| Plain text | txt, log, md, json, xml, yaml, css, py, js, rs, c, java, go, sh, sql, etc. (50+ extensions) | `String::from_utf8_lossy` |
| PDF | pdf | `pdf_extract::extract_text_from_mem` |
| DOCX | docx | ZIP → `word/document.xml` → `<w:t>` tag extraction |
| RTF | rtf | Simple control-word stripping |
| Email | eml, mbox | `mail_parser::MessageParser` → subject + from + body_text |
| Plist | plist | `plist::from_bytes` → debug format |
| Unknown | * | Heuristic: if >85% printable ASCII in first 1024 bytes, treat as text |

### read_entry_bytes Routing

The 6-arm router extracts file bytes from containers for content indexing:

| Container | Extraction Method |
|-----------|-------------------|
| AD1 | `ad1::read_entry_data_by_path` |
| L01 | `ewf::read_l01_entry_data` via VFS |
| Archive | `archive::libarchive_read_file` |
| E01 (VFS) | `EwfVfs::open` → `read(path, offset, len)` |
| Raw (VFS) | `RawVfs::open_filesystem` → `read(path, offset, len)` |
| Disk | `std::fs::read` (direct file read) |

---

## Query Engine

### Search Options

```typescript
interface SearchOptions {
  query: string;             // User search text
  limit?: number;            // Max results (default: 100)
  containerTypes?: string[]; // Filter: ["ad1", "e01", ...]
  extensions?: string[];     // Filter: ["pdf", "docx", ...]
  categories?: string[];     // Filter: ["document", "email", ...]
  minSize?: number;          // Min file size (bytes)
  maxSize?: number;          // Max file size (bytes)
  includeDirs?: boolean;     // Include directories (default: false)
  searchContent?: boolean;   // Search file content (default: true)
  containerPath?: string;    // Restrict to specific container
}
```

### Query Building

Queries are built as a **BooleanQuery** combining a text sub-query with filter clauses:

```text
BooleanQuery(Must)
  ├── TextQuery (Must) — the user's search terms
  │   └── QueryParser across 3 fields with boosting:
  │       filename  (boost 3.0)  ← highest priority
  │       entry_path (boost 1.5) ← path components
  │       content   (boost 1.0)  ← file body text
  │   + FuzzyTermQuery on filename (1 edit distance)
  │
  ├── ContainerType filter (Should) — OR across selected types
  ├── Extension filter (Should) — OR across selected extensions
  ├── Category filter (Should) — OR across selected categories
  ├── ContainerPath filter (Must) — exact match if specified
  ├── SizeRange filter (Must) — RangeQuery on size field
  └── DirExclusion filter (MustNot) — exclude is_dir=1 if !includeDirs
```

### Ranking

Results are ranked by **BM25** with the field boosts above. The filename field has 3× weight because forensic analysts typically search for specific filenames. The fuzzy search on filename (1 edit distance) catches typos.

### Snippets

Each `SearchHit` includes an HTML snippet with `<b>` highlighting:

1. **Content snippet** (preferred): If the query matched content, uses Tantivy's `SnippetGenerator` to extract the most relevant 150-character window
2. **Filename snippet** (fallback): If no content match, highlights the query terms in the filename

### Faceted Results

`SearchResults` includes aggregated counts for drill-down filtering:

```typescript
interface SearchResults {
  hits: SearchHit[];
  total_hits: number;
  elapsed_ms: number;
  category_counts: FacetCount[];     // [{value: "document", count: 42}, ...]
  container_type_counts: FacetCount[]; // [{value: "e01", count: 15}, ...]
}
```

Facets are computed from the result set (not the full index), so they reflect the current filter state.

---

## Frontend Integration

### Auto-Index Hook (`useSearchIndex`)

Located in `src/hooks/useSearchIndex.ts`, this hook:

1. **Opens** the search index when a project loads (`search_open_index`)
2. **Indexes all containers** when discovered files change (`search_index_all`)
3. **Closes** the index when a project closes (`search_close_index`)
4. **Listens** for `"index-progress"` events to track indexing state

### Search Flow (2-Tier)

The search handler in `useAppActions.ts` implements a 2-tier strategy:

1. **Tantivy query first** — calls `search_query` with the user's search options
2. **Fallback to in-memory filter** — if Tantivy returns no results or the index isn't ready, falls back to filename filtering against the discovered files list

### Frontend API (`src/api/search.ts`)

8 typed `invoke()` wrappers:

| Function | Tauri Command | Purpose |
|----------|--------------|---------|
| `openSearchIndex(dbPath)` | `search_open_index` | Open/create index |
| `closeSearchIndex()` | `search_close_index` | Close and remove from registry |
| `deleteSearchIndex()` | `search_delete_index` | Destroy index directory |
| `getSearchStats()` | `search_get_stats` | Document count + disk size |
| `indexContainer(path, content)` | `search_index_container` | Index one container |
| `indexAllContainers(paths, content)` | `search_index_all` | Index all containers |
| `rebuildSearchIndex(paths, content)` | `search_rebuild_index` | Clear + re-index everything |
| `searchQuery(options)` | `search_query` | Execute a search |
| `listenIndexProgress(cb)` | — (event listener) | Listen for `"index-progress"` events |

### Search UI Components

| Component | File | Purpose |
|-----------|------|---------|
| `SearchPanelComponent` | `src/components/search/SearchPanelComponent.tsx` | Main search panel (query input + results list) |
| `SearchFilters` | `src/components/search/SearchFilters.tsx` | Filter toggles (content search, container types) |
| `SearchResultItem` | `src/components/search/SearchResultItem.tsx` | Result row with snippet, content match badge |

---

## FTS5 (SQLite) — Complementary Search

The `.ffxdb` database includes **3 FTS5 virtual tables** for searching user-generated content:

| FTS5 Table | Source Table | Indexed Columns |
|-----------|-------------|-----------------|
| `fts_notes` | `notes` | target_path, title, content, tags |
| `fts_bookmarks` | `bookmarks` | target_path, label, description |
| `fts_activity_log` | `activity_log` | action, description, file_path, details |

FTS5 search is invoked via `project_db_search_fts` and is separate from the Tantivy search. It covers annotations, bookmarks, and the audit trail — data that lives in SQLite, not in evidence containers.

---

## Key Files

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src-tauri/src/search/mod.rs` | Schema definition (12 fields), `SearchIndex` struct, global registry, `classify_extension()`, `is_text_eligible()` |
| `src-tauri/src/search/indexer.rs` | 6 container crawlers, content extraction (PDF/DOCX/EML/plist/RTF), `index_container()`, `rebuild_index()` |
| `src-tauri/src/search/query.rs` | `search()`, `SearchOptions`, `SearchResults`, `SearchHit`, BM25 + filters, snippet generation |
| `src-tauri/src/commands/search.rs` | 8 Tauri commands: open/close/delete/stats/index/query |

### Frontend (TypeScript)

| File | Purpose |
|------|---------|
| `src/api/search.ts` | Typed invoke wrappers + event listener |
| `src/hooks/useSearchIndex.ts` | Auto-index lifecycle hook |
| `src/hooks/useAppActions.ts` | 2-tier search handler (Tantivy → fallback) |
| `src/components/search/SearchPanelComponent.tsx` | Search panel UI |
| `src/components/search/SearchFilters.tsx` | Filter toggles |
| `src/components/search/SearchResultItem.tsx` | Result row rendering |

---

*Last updated: June 2025*
