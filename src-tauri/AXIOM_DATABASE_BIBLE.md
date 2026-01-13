# Magnet AXIOM Database Bible

> **Comprehensive Reference for AXIOM Case Files, Database Schema & Data Structures**
> 
> Version: 1.0 | Last Updated: January 2026
> 
> This document serves as a forensic examiner's guide to understanding AXIOM's internal data storage, file formats, and database architecture.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Case Directory Structure](#2-case-directory-structure)
3. [Core Database Files](#3-core-database-files)
4. [Database Schema Reference](#4-database-schema-reference)
5. [Keyword Search System](#5-keyword-search-system)
6. [Tags & Bookmarks](#6-tags--bookmarks)
7. [Artifact Storage](#7-artifact-storage)
8. [Evidence Sources](#8-evidence-sources)
9. [Attachments & Media](#9-attachments--media)
10. [XML Configuration Files](#10-xml-configuration-files)
11. [Log Files](#11-log-files)
12. [Common SQL Queries](#12-common-sql-queries)
13. [Data Export Considerations](#13-data-export-considerations)
14. [Version Compatibility](#14-version-compatibility)

---

## 1. Overview

### What is Magnet AXIOM?

Magnet AXIOM is a comprehensive digital forensics platform used for acquiring, analyzing, and reporting on digital evidence from computers, mobile devices, cloud services, and vehicles.

### Database Technology

- **Database Engine**: SQLite 3.x
- **Primary Database**: `.mfdb` (Magnet Forensics Database)
- **Attachments Database**: `.attachments` (SQLite)
- **File Extension**: `.mfdb` is a standard SQLite database file

### CORE-FFX Parser Coverage (Current)

CORE-FFX uses this document as a reference, but the current parser only reads a subset of tables and fields:

- Detection: `Case.mfdb`, `Case.mcfc`, `Case Information.xml`/`.txt`
- Case info: `CaseInfo` or `Properties` tables (fallback to filename)
- Evidence sources: `EvidenceSources`, `Sources`, `Evidence`, or `DataSources`
- Artifact categories: `scan_artifact_hit` joined to `artifact_version` (with legacy fallbacks)
- Keywords: `scan_attribute` with `ScanDef` JSON

Other tables listed here are referenced for future coverage.

### Opening AXIOM Databases

```bash
# Using sqlite3 command line
sqlite3 "Case.mfdb"

# Using Python
import sqlite3
conn = sqlite3.connect("Case.mfdb", uri=True)
# Note: Open in read-only mode for forensic integrity
conn = sqlite3.connect("file:Case.mfdb?mode=ro", uri=True)
```

---

## 2. Case Directory Structure

A typical AXIOM case folder contains:

```
<CaseName>/
├── Case.mfdb                           # Primary SQLite database
├── Case.mfdb-shm                       # SQLite shared memory file
├── Case.mfdb-wal                       # SQLite write-ahead log
├── <GUID>.attachments                  # Attachments database
├── <GUID>.attachments-shm              # Attachments shared memory
├── <GUID>.attachments-wal              # Attachments write-ahead log
├── Case Information.xml                # Detailed case metadata (XML)
├── Case Information.txt                # Human-readable case summary
├── Case.mcfc                           # AXIOM Case File Configuration (XML)
├── AXIOMExamine.log                    # Main application log
├── AXIOMExamine.IO.log                 # I/O operations log
├── artifacts.log                       # Artifact parsing log
├── custom_artifacts.log                # Custom artifact log
└── index/                              # Search index directory
    ├── _*.cfs                          # Lucene compound files
    ├── segments_*                      # Lucene segments
    └── write.lock                      # Index lock file
```

### File Descriptions

| File | Description | Format |
|------|-------------|--------|
| `Case.mfdb` | Primary database containing all artifacts, metadata, and case information | SQLite |
| `*.attachments` | Binary attachments, images, documents extracted from evidence | SQLite |
| `Case Information.xml` | Comprehensive XML with search results, evidence sources, keywords | XML |
| `Case Information.txt` | Plain text summary for quick reference | Text |
| `Case.mcfc` | AXIOM project configuration | XML |
| `index/` | Lucene full-text search index | Lucene |

---

## 3. Core Database Files

### 3.1 Case.mfdb (Main Database)

The primary database containing:
- All parsed artifacts
- Case metadata
- Tags and bookmarks
- Keyword search configurations
- Evidence source information
- User annotations and notes

**Size**: Can range from megabytes to hundreds of gigabytes depending on case size.

### 3.2 Attachments Database (`<GUID>.attachments`)

Stores binary content extracted from evidence:
- Images and thumbnails
- Document files
- Media files
- Chat attachments
- Email attachments

**Schema**:
```sql
CREATE TABLE attachment (
    attachment_id TEXT PRIMARY KEY,
    content BLOB,
    content_type TEXT,
    file_name TEXT,
    file_size INTEGER
);
```

---

## 4. Database Schema Reference

### 4.1 Complete Table List

```sql
-- Core artifact tables
artifact                      -- Artifact type definitions
artifact_data                 -- Artifact data schemas
artifact_group                -- Artifact category groupings
artifact_program              -- Source program associations
artifact_program_group        -- Program groupings
artifact_version              -- Artifact version tracking
artifact_version_group        -- Version groupings

-- Case information
case_info                     -- Case metadata
case_info_attribute           -- Case attributes
case_info_note                -- Case-level notes
case_info_source_evidence     -- Evidence source links

-- Hit/artifact data tables
scan_artifact_hit             -- Main artifact hits table
scan_artifact_data            -- Additional artifact data
hit_fragment                  -- Fragment metadata
hit_fragment_string           -- String data values
hit_fragment_int              -- Integer data values
hit_fragment_float            -- Float data values
hit_fragment_date             -- Date/time values
hit_fragment_empty            -- Empty/null markers
hit_fragment_location         -- Location coordinates

-- Tag system
tag                           -- Tag definitions
case_tag                      -- Case-specific tags
hit_case_tag                  -- Tag assignments to hits
source_case_tag               -- Tag assignments to sources

-- Keyword search
keyword_search                -- Keyword search definitions
keyword_search_source_evidence -- Keyword search evidence links
scan_attribute                -- Scan configuration (includes keywords)

-- Evidence sources
source                        -- Evidence source definitions
source_attribute              -- Source attributes
source_attribute_binary_value -- Binary attribute values
source_evidence               -- Evidence items
source_exception              -- Processing exceptions
source_note                   -- Source notes
source_path                   -- File paths

-- Hash and categorization
hit_hash                      -- File hashes
hit_alternative_hashes        -- Additional hash algorithms
hit_media_category            -- Media categorization
media_category                -- Category definitions
media_category_hashset        -- Hash set associations

-- Notes and sets
hit_note                      -- Notes on artifacts
hit_set                       -- Artifact sets
hit_set_member                -- Set memberships
hit_set_member_source         -- Set member sources
hit_set_metadata              -- Set metadata
hit_set_relationship          -- Set relationships

-- Scan/job management
scan                          -- Scan definitions
scan_attribute                -- Scan attributes (CRITICAL)
scan_evidence                 -- Scanned evidence
scan_problem                  -- Processing problems
job                           -- Processing jobs
job_attribute                 -- Job attributes

-- Geolocation
hit_geo_coordinate            -- GPS coordinates
hit_location                  -- Location data

-- Fragment definitions
fragment_categorization       -- Fragment categories
fragment_content              -- Fragment content
fragment_content_type         -- Content types
fragment_definition           -- Fragment field definitions

-- System tables
application_setting           -- Application settings
db_setting                    -- Database settings
product_config                -- Product configuration
product_config_artifact_version -- Product artifact versions
resource                      -- Resources
table_ids                     -- ID management

-- User management
user                          -- User accounts
user_case                     -- User case access
user_case_setting             -- User case settings
user_info                     -- User information
user_setting                  -- User settings

-- Snapshots
snapshot                      -- Case snapshots
snapshot_item                 -- Snapshot items

-- Special features
mitre_attack_rule             -- MITRE ATT&CK rules
yara_rules                    -- YARA rules
project_vic_hash_metadata     -- Project VIC integration
project_vic_hash_segments     -- Project VIC segments
process_item                  -- Process tracking
```

### 4.2 Key Table Schemas

#### scan_artifact_hit (Main Artifacts Table)

```sql
CREATE TABLE scan_artifact_hit (
    hit_id INTEGER PRIMARY KEY,
    artifact_version_id CHAR(32) NOT NULL,
    scan_id CHAR(32) NOT NULL,
    source_id CHAR(32),
    parent_hit_id INTEGER,
    location TEXT,
    deleted INTEGER DEFAULT 0,
    starred INTEGER DEFAULT 0,
    FOREIGN KEY (artifact_version_id) REFERENCES artifact_version(artifact_version_id),
    FOREIGN KEY (scan_id) REFERENCES scan(scan_id)
);
```

#### artifact_version (Artifact Definitions)

```sql
CREATE TABLE artifact_version (
    artifact_version_id CHAR(32) PRIMARY KEY,
    artifact_id CHAR(32) NOT NULL,
    artifact_name TEXT NOT NULL,
    version TEXT,
    schema_xml TEXT,
    FOREIGN KEY (artifact_id) REFERENCES artifact(artifact_id)
);
```

#### hit_fragment_string (Text Data)

```sql
CREATE TABLE hit_fragment_string (
    hit_fragment_id INTEGER PRIMARY KEY,
    hit_id INTEGER NOT NULL,
    value TEXT,
    fragment_definition_id CHAR(32) NOT NULL,
    FOREIGN KEY (hit_id) REFERENCES scan_artifact_hit(hit_id),
    FOREIGN KEY (fragment_definition_id) REFERENCES fragment_definition(fragment_definition_id)
);
```

#### tag (Tag Definitions)

```sql
CREATE TABLE tag (
    tag_id CHAR(32) PRIMARY KEY,
    tag_name TEXT NOT NULL,
    tag_description TEXT,
    tag_color INTEGER NOT NULL,
    tag_type TEXT NOT NULL  -- 'System' or 'User'
);
```

#### case_tag (Case-Specific Tags)

```sql
CREATE TABLE case_tag (
    case_tag_id CHAR(32) PRIMARY KEY,
    case_info_id CHAR(32) NOT NULL,
    tag_id CHAR(32) NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES tag(tag_id)
);
```

#### hit_case_tag (Tag Assignments)

```sql
CREATE TABLE hit_case_tag (
    hit_id INTEGER NOT NULL,
    case_tag_id CHAR(32) NOT NULL,
    PRIMARY KEY (hit_id, case_tag_id),
    FOREIGN KEY (hit_id) REFERENCES scan_artifact_hit(hit_id),
    FOREIGN KEY (case_tag_id) REFERENCES case_tag(case_tag_id)
);
```

---

## 5. Keyword Search System

### 5.1 Keyword Configuration Storage

Keywords are stored in the `scan_attribute` table as JSON within the `ScanDef` attribute:

```sql
SELECT attribute_value 
FROM scan_attribute 
WHERE attribute_name = 'ScanDef';
```

### 5.2 ScanDef JSON Structure

```json
{
  "Keywords": [
    {
      "Value": "search term",
      "Regex": false,
      "IsCaseSensitive": false,
      "EncodingTypes": [1, 2, 3],
      "FromFile": false,
      "FileName": null
    }
  ],
  "KeywordFiles": [
    {
      "FileName": "C:\\path\\to\\keywords.txt",
      "DateAdded": "2025-05-02T13:50:45.198-08:00",
      "RecordCount": 27,
      "Enabled": true,
      "FileId": "00000001-0000-0000-0000-000000000000",
      "EncodingTypes": [1],
      "IsCaseSensitive": false
    }
  ],
  "PrivilegedContentKeywords": [],
  "PrivilegedContentMode": "Off",
  "KeywordSearchType": 0
}
```

### 5.3 Encoding Types

| Value | Encoding |
|-------|----------|
| 1 | UTF-8 |
| 2 | UTF-16 LE |
| 3 | UTF-16 BE |
| 4 | ASCII |
| 5 | Latin-1 |

### 5.4 Important Note on Keyword Files

**Individual keywords from loaded files are NOT stored in the database.**

AXIOM only stores:
- ✅ File path reference
- ✅ Record count (number of keywords)
- ✅ Configuration (case sensitivity, encoding)
- ✅ Date added
- ❌ **NOT** the actual keyword values from the file

The keywords are read from the original file at search time. If the file is moved or deleted, the individual keywords are lost.

### 5.5 keyword_search Table

```sql
CREATE TABLE keyword_search (
    keyword_search_id CHAR(32) PRIMARY KEY,
    keyword_list_file_path CHAR(255),
    search_end_date TIMESTAMP,
    search_type INT
);
```

### 5.6 Keyword Hit Results

Keyword hits appear in artifact data through the `hit_fragment_string` table, where matched content is stored. You can find keyword matches by:

```sql
-- Find artifacts with specific text content
SELECT sah.hit_id, av.artifact_name, hfs.value
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
JOIN hit_fragment_string hfs ON sah.hit_id = hfs.hit_id
WHERE hfs.value LIKE '%search_term%';
```

---

## 6. Tags & Bookmarks

### 6.1 Default System Tags

| Tag Name | Type | Color | Description |
|----------|------|-------|-------------|
| Bookmark | System | -8323328 | General bookmarking |
| Evidence | System | -2555904 | Marked as evidence |
| Of interest | System | -137137 | Items of interest |
| Exception_Tag_Name | Exception | -32768 | Exception handling |

### 6.2 Tag Color Values

Colors are stored as signed 32-bit integers (ARGB format):

```python
def decode_color(color_int):
    """Decode AXIOM color integer to RGB"""
    if color_int < 0:
        color_int = color_int + 2**32
    a = (color_int >> 24) & 0xFF
    r = (color_int >> 16) & 0xFF
    g = (color_int >> 8) & 0xFF
    b = color_int & 0xFF
    return (r, g, b, a)
```

### 6.3 Querying Tagged Artifacts

```sql
-- Get all tagged artifacts with tag names
SELECT 
    sah.hit_id,
    av.artifact_name,
    t.tag_name,
    t.tag_type
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
JOIN hit_case_tag hct ON sah.hit_id = hct.hit_id
JOIN case_tag ct ON hct.case_tag_id = ct.case_tag_id
JOIN tag t ON ct.tag_id = t.tag_id
ORDER BY t.tag_name, sah.hit_id;
```

### 6.4 Creating Custom Tags

Tags require entries in multiple tables:

1. `tag` - Tag definition
2. `case_tag` - Case-specific instance
3. `hit_case_tag` - Assignment to artifacts

---

## 7. Artifact Storage

### 7.1 Artifact Hierarchy

```
artifact (type definition)
  └── artifact_version (versioned schema)
        └── scan_artifact_hit (actual data)
              ├── hit_fragment_string (text values)
              ├── hit_fragment_int (integer values)
              ├── hit_fragment_float (decimal values)
              ├── hit_fragment_date (timestamps)
              └── hit_fragment_location (coordinates)
```

### 7.2 Fragment Definitions

Each artifact type has defined fragments (fields):

```sql
SELECT 
    fd.fragment_definition_id,
    fd.fragment_name,
    fct.content_type_name
FROM fragment_definition fd
LEFT JOIN fragment_content_type fct ON fd.content_type_id = fct.content_type_id
WHERE fd.artifact_version_id = '<artifact_version_id>';
```

### 7.3 Common Artifact Categories

| Category | Examples |
|----------|----------|
| Communication | iMessage, SMS, WhatsApp, Telegram, Email |
| Web | Browser History, Bookmarks, Downloads, Cookies |
| Cloud | Google, iCloud, Dropbox, OneDrive |
| Media | Pictures, Videos, Audio, Screenshots |
| Location | GPS, WiFi Locations, Cell Towers |
| Documents | PDF, Office, Text Files |
| System | Registry, Event Logs, Prefetch |
| Applications | Installed Apps, App Data, Usage |

### 7.4 Querying Artifact Data

```sql
-- Get artifact with all its data
SELECT 
    sah.hit_id,
    av.artifact_name,
    fd.fragment_name,
    COALESCE(hfs.value, 
             CAST(hfi.value AS TEXT), 
             CAST(hff.value AS TEXT),
             hfd.value) as fragment_value
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
LEFT JOIN hit_fragment_string hfs ON sah.hit_id = hfs.hit_id
LEFT JOIN hit_fragment_int hfi ON sah.hit_id = hfi.hit_id
LEFT JOIN hit_fragment_float hff ON sah.hit_id = hff.hit_id
LEFT JOIN hit_fragment_date hfd ON sah.hit_id = hfd.hit_id
LEFT JOIN fragment_definition fd ON 
    COALESCE(hfs.fragment_definition_id, 
             hfi.fragment_definition_id,
             hff.fragment_definition_id,
             hfd.fragment_definition_id) = fd.fragment_definition_id
WHERE sah.hit_id = <hit_id>;
```

---

## 8. Evidence Sources

### 8.1 Source Tables

```sql
-- source: Evidence source definitions
CREATE TABLE source (
    source_id CHAR(32) PRIMARY KEY,
    source_name TEXT,
    source_type TEXT,
    parent_source_id CHAR(32)
);

-- source_evidence: Individual evidence items
CREATE TABLE source_evidence (
    source_evidence_id CHAR(32) PRIMARY KEY,
    source_id CHAR(32),
    evidence_type TEXT,
    evidence_path TEXT,
    evidence_hash TEXT,
    evidence_size INTEGER
);

-- source_attribute: Source metadata
CREATE TABLE source_attribute (
    source_attribute_id CHAR(32) PRIMARY KEY,
    source_id CHAR(32),
    attribute_name TEXT,
    attribute_value TEXT
);
```

### 8.2 Evidence Types

| Type | Description |
|------|-------------|
| Image | Disk/device images (E01, DD, etc.) |
| Mobile | Mobile device extractions |
| Cloud | Cloud service acquisitions |
| Computer | Live computer artifacts |
| Backup | iOS/Android backups |
| Memory | RAM captures |

### 8.3 Querying Evidence Sources

```sql
-- List all evidence sources
SELECT 
    s.source_name,
    se.evidence_type,
    se.evidence_path,
    se.evidence_hash,
    se.evidence_size
FROM source s
JOIN source_evidence se ON s.source_id = se.source_id;
```

---

## 9. Attachments & Media

### 9.1 Attachments Database Structure

The `.attachments` file stores binary content:

```sql
-- In the attachments database
CREATE TABLE attachment (
    attachment_id TEXT PRIMARY KEY,
    content BLOB NOT NULL,
    content_type TEXT,
    original_file_name TEXT,
    file_extension TEXT,
    file_size INTEGER,
    created_date TEXT,
    source_hit_id INTEGER
);
```

### 9.2 Linking Attachments to Artifacts

Attachments are linked via GUIDs stored in `hit_fragment_string`:

```sql
-- Find attachment references
SELECT 
    sah.hit_id,
    hfs.value as attachment_guid
FROM scan_artifact_hit sah
JOIN hit_fragment_string hfs ON sah.hit_id = hfs.hit_id
JOIN fragment_definition fd ON hfs.fragment_definition_id = fd.fragment_definition_id
WHERE fd.fragment_name LIKE '%attachment%' 
   OR fd.fragment_name LIKE '%media%';
```

### 9.3 Media Categorization

```sql
-- Media categories (PhotoDNA, AI classification)
SELECT 
    mc.category_name,
    mc.category_description,
    COUNT(hmc.hit_id) as hit_count
FROM media_category mc
LEFT JOIN hit_media_category hmc ON mc.media_category_id = hmc.media_category_id
GROUP BY mc.category_name;
```

---

## 10. XML Configuration Files

### 10.1 Case Information.xml

Contains comprehensive case data in XML format:

```xml
<?xml version="1.0" encoding="utf-8"?>
<CaseInformation>
  <CaseName>Example Case</CaseName>
  <CaseNumber>2025-001</CaseNumber>
  <Examiner>John Doe</Examiner>
  <Agency>Forensics Lab</Agency>
  <Created>2025-01-01T12:00:00</Created>
  
  <EvidenceSources>
    <Source>
      <Name>iPhone 14 Pro</Name>
      <Type>Mobile</Type>
      <Path>/evidence/iphone.zip</Path>
      <Hash>SHA256:abc123...</Hash>
    </Source>
  </EvidenceSources>
  
  <SearchResults>
    <Result>
      <ArtifactType>iMessage</ArtifactType>
      <Count>1523</Count>
    </Result>
  </SearchResults>
  
  <Keywords>
    <Keyword>
      <Value>suspicious</Value>
      <CaseSensitive>false</CaseSensitive>
      <Regex>false</Regex>
    </Keyword>
  </Keywords>
  
  <KeywordFiles>
    <File>
      <Path>C:\keywords\terms.txt</Path>
      <RecordCount>50</RecordCount>
      <Enabled>true</Enabled>
    </File>
  </KeywordFiles>
</CaseInformation>
```

### 10.2 Case.mcfc (Project Configuration)

```xml
<?xml version="1.0" encoding="utf-8"?>
<MagnetCase>
  <Version>7.0</Version>
  <CaseGuid>abc123-def456-...</CaseGuid>
  <DatabasePath>Case.mfdb</DatabasePath>
  <AttachmentsGuid>xyz789...</AttachmentsGuid>
  <IndexPath>index</IndexPath>
  <Settings>
    <TimeZone>America/Los_Angeles</TimeZone>
    <DateFormat>MM/dd/yyyy</DateFormat>
  </Settings>
</MagnetCase>
```

---

## 11. Log Files

### 11.1 AXIOMExamine.log

Main application log containing:
- Application startup/shutdown
- User actions
- Processing status
- Errors and warnings

```
[2025-01-01 12:00:00.000] [INFO] AXIOM Examine started
[2025-01-01 12:00:05.123] [INFO] Opening case: Example Case
[2025-01-01 12:00:10.456] [DEBUG] Loading artifacts from database
[2025-01-01 12:01:00.789] [WARN] Unable to parse artifact at offset 12345
```

### 11.2 AXIOMExamine.IO.log

I/O operations log:
- File access
- Database queries
- Network requests
- Performance metrics

### 11.3 artifacts.log

Artifact-specific parsing information:
- Parser execution
- Artifact counts
- Parse errors
- Schema validation

---

## 12. Common SQL Queries

### 12.1 Count Artifacts by Type

```sql
SELECT 
    av.artifact_name,
    COUNT(sah.hit_id) as count
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
GROUP BY av.artifact_name
ORDER BY count DESC;
```

### 12.2 Get All Bookmarked Items

```sql
SELECT 
    sah.hit_id,
    av.artifact_name,
    sah.location
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
JOIN hit_case_tag hct ON sah.hit_id = hct.hit_id
JOIN case_tag ct ON hct.case_tag_id = ct.case_tag_id
JOIN tag t ON ct.tag_id = t.tag_id
WHERE t.tag_name = 'Bookmark';
```

### 12.3 Search Text Content

```sql
SELECT DISTINCT
    sah.hit_id,
    av.artifact_name,
    hfs.value
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
JOIN hit_fragment_string hfs ON sah.hit_id = hfs.hit_id
WHERE hfs.value LIKE '%keyword%'
ORDER BY av.artifact_name;
```

### 12.4 Get Timeline Data

```sql
SELECT 
    sah.hit_id,
    av.artifact_name,
    hfd.value as timestamp,
    fd.fragment_name as date_field
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
JOIN hit_fragment_date hfd ON sah.hit_id = hfd.hit_id
JOIN fragment_definition fd ON hfd.fragment_definition_id = fd.fragment_definition_id
WHERE hfd.value IS NOT NULL
ORDER BY hfd.value DESC;
```

### 12.5 List All Tables with Row Counts

```sql
SELECT 
    name as table_name,
    (SELECT COUNT(*) FROM pragma_table_info(name)) as column_count
FROM sqlite_master 
WHERE type = 'table'
ORDER BY name;
```

### 12.6 Export Artifact to CSV Format

```sql
-- Get iMessage data in export-friendly format
SELECT 
    sah.hit_id,
    MAX(CASE WHEN fd.fragment_name = 'Sender' THEN hfs.value END) as sender,
    MAX(CASE WHEN fd.fragment_name = 'Recipient' THEN hfs.value END) as recipient,
    MAX(CASE WHEN fd.fragment_name = 'Message' THEN hfs.value END) as message,
    MAX(CASE WHEN fd.fragment_name = 'Timestamp' THEN hfd.value END) as timestamp
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
LEFT JOIN hit_fragment_string hfs ON sah.hit_id = hfs.hit_id
LEFT JOIN hit_fragment_date hfd ON sah.hit_id = hfd.hit_id
LEFT JOIN fragment_definition fd ON 
    COALESCE(hfs.fragment_definition_id, hfd.fragment_definition_id) = fd.fragment_definition_id
WHERE av.artifact_name = 'iMessage/SMS/MMS'
GROUP BY sah.hit_id;
```

### 12.7 Get Keyword Configuration

```sql
SELECT 
    attribute_name,
    attribute_value
FROM scan_attribute
WHERE attribute_name IN ('ScanDef', 'KeywordsEntered', 'PrivilegedContentKeywords');
```

### 12.8 Find Deleted Artifacts

```sql
SELECT 
    sah.hit_id,
    av.artifact_name,
    sah.location
FROM scan_artifact_hit sah
JOIN artifact_version av ON sah.artifact_version_id = av.artifact_version_id
WHERE sah.deleted = 1;
```

---

## 13. Data Export Considerations

### 13.1 Forensic Integrity

When working with AXIOM databases:

1. **Always work on copies** - Never modify original case files
2. **Use read-only connections** - `sqlite3.connect("file:Case.mfdb?mode=ro", uri=True)`
3. **Document all queries** - Maintain audit trail
4. **Verify hashes** - Check database integrity before/after

### 13.2 Large Database Handling

For large cases (>10GB):
- Use `LIMIT` and `OFFSET` for pagination
- Create indexes for frequent queries
- Consider extracting subsets to temporary databases
- Use streaming for binary exports

### 13.3 Character Encoding

AXIOM uses UTF-8 encoding. When exporting:
- Specify encoding explicitly
- Handle NULL bytes in BLOB data
- Escape special characters for CSV/XML

---

## 14. Version Compatibility

### 14.1 Database Version Tracking

```sql
SELECT * FROM db_setting WHERE setting_name = 'DatabaseVersion';
SELECT * FROM product_config;
```

### 14.2 Schema Changes by Version

| AXIOM Version | Notable Schema Changes |
|---------------|----------------------|
| 6.x | Introduction of media categorization |
| 7.x | Enhanced tag system, MITRE ATT&CK |
| 8.x | Improved cloud artifact support |

### 14.3 Backwards Compatibility

- Newer AXIOM versions can open older databases
- Older versions may not open newer databases
- Schema migrations are automatic on open
- Always note AXIOM version used for processing

---

## Appendix A: Table Reference Quick Guide

### Core Tables

| Table | Purpose | Key Columns |
|-------|---------|-------------|
| `scan_artifact_hit` | Main artifacts | hit_id, artifact_version_id |
| `artifact_version` | Artifact definitions | artifact_name, schema_xml |
| `hit_fragment_string` | Text data | hit_id, value |
| `hit_fragment_date` | Timestamps | hit_id, value |
| `tag` | Tag definitions | tag_name, tag_type |
| `hit_case_tag` | Tag assignments | hit_id, case_tag_id |
| `scan_attribute` | Scan config (keywords) | attribute_name, attribute_value |
| `source` | Evidence sources | source_name, source_type |

### Fragment Tables

| Table | Data Type |
|-------|-----------|
| `hit_fragment_string` | Text/VARCHAR |
| `hit_fragment_int` | Integer |
| `hit_fragment_float` | Decimal/Float |
| `hit_fragment_date` | DateTime |
| `hit_fragment_location` | GPS Coordinates |
| `hit_fragment_empty` | NULL markers |

---

## Appendix B: JSON Schema for ScanDef

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "Keywords": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "Value": { "type": "string" },
          "Regex": { "type": "boolean" },
          "IsCaseSensitive": { "type": "boolean" },
          "EncodingTypes": { 
            "type": "array",
            "items": { "type": "integer" }
          },
          "FromFile": { "type": "boolean" },
          "FileName": { "type": ["string", "null"] }
        }
      }
    },
    "KeywordFiles": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "FileName": { "type": "string" },
          "DateAdded": { "type": "string", "format": "date-time" },
          "RecordCount": { "type": "integer" },
          "Enabled": { "type": "boolean" },
          "FileId": { "type": "string" },
          "EncodingTypes": { "type": "array" },
          "IsCaseSensitive": { "type": "boolean" }
        }
      }
    },
    "PrivilegedContentKeywords": { "type": "array" },
    "PrivilegedContentMode": { "type": "string" }
  }
}
```

---

## Appendix C: Common Artifact Names

### Communication
- `iMessage/SMS/MMS`
- `WhatsApp Messages`
- `Telegram Messages`
- `Facebook Messenger`
- `Signal Messages`
- `Discord Messages`

### Web Activity
- `Chrome Browser History`
- `Safari Browser History`
- `Edge Browser History`
- `Firefox Browser History`
- `Chrome Bookmarks`
- `Chrome Downloads`

### Cloud Services
- `Cloud Google Drive Files`
- `Cloud iCloud Photos`
- `Cloud Dropbox Files`
- `Cloud OneDrive Files`

### Media
- `Pictures`
- `Videos`
- `Audio`
- `Screenshots`
- `Live Photos`

### System
- `Windows Event Logs`
- `Prefetch Files`
- `Registry`
- `Installed Applications`
- `User Accounts`

---

## Appendix D: Error Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Database locked |
| 3 | Schema mismatch |
| 4 | Corrupt database |
| 5 | File not found |
| 6 | Permission denied |

---

## References

- [Magnet Forensics Documentation](https://www.magnetforensics.com/docs/)
- [SQLite Documentation](https://sqlite.org/docs.html)
- [Digital Forensics Wiki](https://forensicswiki.xyz/)

---

*This document is for educational and forensic examination purposes. Always follow proper chain of custody procedures and legal requirements when handling digital evidence.*
