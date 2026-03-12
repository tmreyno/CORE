# Case Data Model

> **Storage:** Dual-file architecture — `.cffx` (JSON project file) + `.ffxdb` (SQLite database)  
> **Schema version:** 10 (10 incremental migrations from v1)  
> **Tables:** 35+ including 3 FTS5 virtual tables

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [.cffx Project File](#cffx-project-file)
3. [.ffxdb Database Schema](#ffxdb-database-schema)
4. [Table Groups](#table-groups)
5. [Evidence Lifecycle](#evidence-lifecycle)
6. [Chain of Custody (Immutability Model)](#chain-of-custody-immutability-model)
7. [Evidence Collections](#evidence-collections)
8. [Schema Migrations](#schema-migrations)
9. [FTS5 Virtual Tables](#fts5-virtual-tables)
10. [Key Files](#key-files)

---

## Architecture Overview

```text
my-case.cffx             ← JSON project file (lightweight, serializable state)
my-case.ffxdb             ← SQLite database (heavy data: COC, hashes, annotations)
my-case.ffxdb-wal         ← SQLite WAL file (auto-managed)
my-case.ffxdb-shm         ← SQLite shared memory (auto-managed)
my-case.ffxdb-index/      ← Tantivy search index (see SEARCH_ARCHITECTURE.md)
```

### Separation of Concerns

| File | Format | Contains | Updated |
|------|--------|----------|---------|
| `.cffx` | JSON | Project metadata, user list, sessions, directory paths, tab state, evidence cache | On save (explicit or auto-save) |
| `.ffxdb` | SQLite (WAL mode) | All forensic records: hashes, verifications, COC, evidence collections, bookmarks, notes, activity log, search state | Continuously (write-through via `dbSync`) |

The `.cffx` is the "project file" that users open/save. The `.ffxdb` is the backing database that stores the bulk of forensic data. Path relationship: the `.cffx` stores a relative `db_path` field pointing to the `.ffxdb`.

---

## .cffx Project File

The `.cffx` file is a JSON serialization of the `FFXProject` struct:

```rust
struct FFXProject {
    // Identity
    version: String,
    project_id: String,
    name: String,
    owner_name: String,
    case_number: String,
    case_name: String,
    description: String,

    // Paths
    root_path: String,         // Directory containing the .cffx
    db_path: String,           // Relative path to .ffxdb

    // Timestamps
    created_at: String,
    saved_at: String,

    // User tracking
    users: Vec<ProjectUser>,
    sessions: Vec<ProjectSession>,
    activity_log: Vec<ActivityLogEntry>,

    // Filesystem
    locations: Option<ProjectLocations>,
    open_directories: Vec<String>,

    // UI state
    tabs: Vec<ProjectTab>,

    // Cached data (for quick reload without full DB scan)
    hash_history: Vec<HashRecord>,
    evidence_cache: Vec<EvidenceFileEntry>,
    case_documents_cache: Vec<String>,
    preview_cache: PreviewCache,
    processed_databases: Vec<ProcessedDatabase>,

    // User-generated content
    bookmarks: Vec<Bookmark>,
    notes: Vec<Note>,
    tags: Vec<Tag>,
    reports: Vec<Report>,
}
```

### ProjectLocations

Tracks the standard forensic folder structure:

```rust
struct ProjectLocations {
    evidence_path: String,
    processed_db_path: String,
    case_documents_path: String,
    export_path: String,
    report_path: String,
}
```

These are created automatically from a folder template on new project setup.

---

## .ffxdb Database Schema

### Schema Versioning

The `schema_meta` table tracks the current version:

```sql
CREATE TABLE schema_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Row: key='schema_version', value='10'
```

On database open, the current version is read and migrations are applied incrementally up to v10.

---

## Table Groups

### Core Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `schema_meta` | Schema version tracking | key, value |
| `users` | User/examiner records | id, name, display_name, role |
| `sessions` | Work sessions | id, user_id, started_at, ended_at, summary |
| `activity_log` | Audit trail | id, session_id, timestamp, user, action, description, file_path, details |

### Evidence Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `evidence_files` | Discovered evidence containers | id, path (UNIQUE), filename, container_type, total_size, segment_count, discovered_at |
| `hashes` | Computed hash results | id, file_id (FK→evidence_files), algorithm, hash_value, computed_at, source (`'computed'`) |
| `verifications` | Hash verification records | id, hash_id (FK→hashes), verified_at, result, expected_hash, actual_hash |

**Relationships:**
```text
evidence_files (1) ──→ (N) hashes ──→ (N) verifications
```

### User Data Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `bookmarks` | File/artifact bookmarks | id, target_type, target_path, container_path, name, color, notes, context, tags, created_by |
| `notes` | Annotations | id, target_type, target_path, container_path, title, content, priority, tags, created_by |
| `tags` | Tag definitions | id, name (UNIQUE), color, description |
| `tag_assignments` | Tag ↔ entity links | id, tag_id (FK), target_type, target_id, assigned_by |

**`target_type` values:** `file`, `artifact`, `database`, `case`, `general`

### UI State Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `tabs` | Open tab state | id, tab_type, title, path, position |
| `ui_state` | Key-value persistence | key (UNIQUE), value, updated_at |
| `saved_searches` | Saved search queries | id, name, query, filters, created_at |
| `recent_searches` | Recent query history | id, query, result_count, searched_at |

### Document Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `case_documents` | Case documents metadata | id, path, filename, document_type, format, case_number, evidence_id, size, modified_at |
| `reports` | Generated reports | id, name, report_type, template_id, format, file_path, generated_at, generated_by |

### Processed Database Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `processed_databases` | Third-party tool databases | id, path, tool_type, case_name, examiner, created_at |
| `processed_db_integrity` | DB integrity records | id, db_id (FK), check_type, result |
| `processed_db_metrics` | DB statistics | id, db_id (FK), metric_name, metric_value |
| `axiom_case_info` | Magnet AXIOM case data | id, db_id (FK), case_number, examiner, created_date |
| `axiom_evidence_sources` | AXIOM evidence sources | id, case_id (FK), source_type, source_path |
| `axiom_search_results` | AXIOM keyword search results | id, case_id (FK), keyword, hit_count |
| `artifact_categories` | Parsed artifact categories | id, db_id (FK), category_name, artifact_count |

### Forensic Workflow Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `export_history` | Export operation tracking | id, export_type, format, source_paths, destination, status, total_size, initiated_by |
| `chain_of_custody` | Legacy COC records | id, case_number, evidence_id, from_person, to_person, transfer_date, purpose, recorded_by |
| `file_classifications` | File categorization | id, file_path, container_path, classification, classified_by |
| `extraction_log` | File extraction records | id, source_path, destination_path, extraction_method |
| `viewer_history` | File view tracking | id, file_path, container_path, viewer_type, viewed_at |
| `annotations` | File annotations | id, file_path, container_path, annotation_type, content, position |
| `evidence_relationships` | File ↔ file links | id, source_path, target_path, relationship_type, description |

### COC System Tables (Schema v4+)

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `coc_items` | Chain of Custody records (Form 7-01) | id, case_number, coc_number, evidence_id, description, **status** (`draft`/`locked`/`voided`), locked_at, locked_by, + 40 Form 7-01 fields |
| `coc_amendments` | Field-level change audit | id, coc_item_id (FK), field_name, old_value, new_value, initials, reason, amended_at |
| `coc_audit_log` | Action-level audit trail | id, coc_item_id (FK), action, performed_by, details, timestamp |
| `coc_transfers` | Transfer chain records | id, coc_id (FK), relinquished_by, received_by, transfer_date, purpose, storage_location, storage_date, method |

### Evidence Collection Tables (Schema v4+)

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `evidence_collections` | On-site acquisition sessions | id, case_number, collecting_officer, collection_date, collection_location, status, authorization |
| `collected_items` | Items acquired in a collection | id, collection_id (FK), evidence_file_id, description, device_type, brand, make, model, serial_number, imei, image_format, 25+ device/forensic fields |

### Form & Conflict Resolution Tables (Schema v6+)

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `form_submissions` | Schema-driven form data | id, template_id, template_version, case_number, data_json, status, created_at, updated_at |
| `evidence_data_alternatives` | Conflict resolution (v10) | id, evidence_file_id, field_name, chosen_source, user_value, container_value, resolved_by |

---

## Evidence Lifecycle

An evidence file moves through these stages in the data model:

```text
1. Discovery
   scan_directory_streaming → DiscoveredFile
   → evidence_files INSERT (path, filename, container_type, total_size)

2. Metadata Loading
   logical_info / logical_info_fast → ContainerInfo
   → fileInfoMap signal (frontend, in-memory)

3. Hashing
   batch_hash → computed hash
   → hashes INSERT (file_id, algorithm, hash_value)

4. Verification
   determineVerification(stored, computed)
   → verifications INSERT (hash_id, result, expected, actual)

5. Indexing
   search_index_container → Tantivy index
   → File is now searchable by name, path, content

6. Export
   export_files / ewf_create_image / l01_create_image
   → export_history INSERT (status tracking)
```

---

## Chain of Custody (Immutability Model)

COC records enforce an **append-only immutability model** for forensic integrity.

### Status Lifecycle

```text
  draft  ──(lock)──▸  locked  ──(void)──▸  voided
    │                    │
    │ (free edit)        │ (amend w/ initials + reason)
    ▼                    ▼
  update               coc_amendments record created
                       coc_audit_log "amend" entry
```

| Status | Editable? | Removable? | UI |
|--------|-----------|------------|-----|
| `draft` | Yes, freely | Yes (hard delete) | Green badge, inputs active |
| `locked` | Via amendment only (initials + reason required) | Void only (soft delete) | Yellow "🔒 Locked" badge, readonly |
| `voided` | No | No (record persists) | Red "Voided" badge, collapsed |

### Amendment Process

When a locked COC item is edited:

1. User attempts to change a field
2. Modal opens requiring **initials** and **reason**
3. Backend validates field name against 24-field allowlist (prevents SQL injection)
4. `coc_amendments` INSERT: old_value, new_value, initials, reason
5. `coc_audit_log` INSERT: action="amend"
6. Field value is updated on the `coc_items` row

### Audit Trail

Every COC action creates an audit entry in `coc_audit_log`:

| Action | Trigger |
|--------|---------|
| `insert` | New COC item created |
| `update` | Draft item edited |
| `lock` | Item locked |
| `amend` | Locked item field changed |
| `void` | Item voided (soft-deleted) |
| `transfer` | Transfer record added |

### Foreign Key Protection

`coc_amendments` and `coc_audit_log` use `ON DELETE RESTRICT` → audit history **cannot** be accidentally deleted.

---

## Evidence Collections

Evidence collections represent **on-site acquisition sessions** linked to COC records and evidence files.

### Data Model

```text
evidence_collections (1)
  ├──→ (N) collected_items
  │         ├── evidence_file_id (FK → evidence_files, nullable)
  │         ├── Device fields (brand, make, model, serial, IMEI)
  │         ├── Forensic fields (image_format, acquisition_method)
  │         └── 25+ additional fields (timestamps, hashes, photo_refs)
  │
  └──→ (N) coc_items (via case_number linkage)
```

### Auto-Enrichment

When evidence containers are loaded, a `createEffect` in `EvidenceCollectionPanel.tsx` automatically enriches collection forms from container metadata:

- **New collections:** Populates header fields + creates collected items per evidence file
- **Existing collections:** Fills only **empty** fields, never overwrites user data

16 enrichable fields: brand, make, model, serial_number, imei, image_format, acquisition_method, connection_method, storage_notes, timestamps, collecting officer, device_type, notes, building.

---

## Schema Migrations

| Version | Changes |
|---------|---------|
| **v1 → v2** | Added processed database tables: `processed_databases`, `processed_db_integrity`, `processed_db_metrics`, `axiom_case_info`, `axiom_evidence_sources`, `axiom_search_results`, `artifact_categories` |
| **v2 → v3** | Added forensic workflow tables: `export_history`, `chain_of_custody`, `file_classifications`, `extraction_log`, `viewer_history`, `annotations`, `evidence_relationships`. Added FTS5 virtual tables. |
| **v3 → v4** | Added COC system: `coc_items`, `coc_transfers`, `evidence_collections`, `collected_items` |
| **v4 → v5** | Added COC immutability: `coc_amendments` (field-level tracking), `coc_audit_log` (action audit). Added `status`, `locked_at`, `locked_by` to `coc_items`. |
| **v5 → v6** | Added `form_submissions` table for schema-driven forms |
| **v6 → v7** | Added `status` column to `evidence_collections` |
| **v7 → v8** | Expanded `collected_items` with 20 new columns (device identification, forensic acquisition, timestamps) |
| **v8 → v9** | Form 7-01 alignment: 15 new columns on `coc_items` (owner/source/contact, collection method, disposition), 2 new columns on `coc_transfers` (storage_location, storage_date) |
| **v9 → v10** | Added `evidence_data_alternatives` table for conflict resolution |

All migrations are **idempotent** and use `ALTER TABLE ... ADD COLUMN` which ignores duplicate columns in SQLite. The migration runner reads the current version from `schema_meta` and applies only the needed migrations.

---

## FTS5 Virtual Tables

Three FTS5 tables enable full-text search on user-generated content within SQLite:

| Virtual Table | Source Table | Indexed Columns |
|--------------|-------------|-----------------|
| `fts_notes` | `notes` | target_path, title, content, tags |
| `fts_bookmarks` | `bookmarks` | target_path, label, description |
| `fts_activity_log` | `activity_log` | action, description, file_path, details |

FTS5 uses SQLite's built-in tokenizer. Queries use `MATCH` syntax and return ranked results. This is separate from the Tantivy full-text search engine which indexes evidence container contents (see [SEARCH_ARCHITECTURE.md](SEARCH_ARCHITECTURE.md)).

---

## Key Files

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src-tauri/src/project/mod.rs` | `FFXProject` struct (.cffx format) |
| `src-tauri/src/project/types.rs` | `ProjectUser`, `ProjectSession`, `ActivityLogEntry`, `ProjectLocations` |
| `src-tauri/src/project_db/schema_tables.rs` | Complete v10 CREATE TABLE statements |
| `src-tauri/src/project_db/schema_migrations.rs` | Incremental migration logic (v1 → v10) |
| `src-tauri/src/project_db/types.rs` | All DB record types (`DbCocItem`, `DbBookmark`, etc.) |
| `src-tauri/src/commands/project_db/` | 119 Tauri commands (modular directory) |

### Frontend (TypeScript)

| File | Purpose |
|------|---------|
| `src/types/project.ts` | `FFXProject` TypeScript interface |
| `src/types/projectDb.ts` | All DB record TypeScript interfaces |
| `src/hooks/project/useProjectDbSync.ts` | Write-through sync to .ffxdb |
| `src/hooks/project/useProjectDbRead.ts` | Seed .ffxdb from .cffx on load |
| `src/hooks/project/useProject.ts` | Project CRUD operations |

---

*Last updated: June 2025*
