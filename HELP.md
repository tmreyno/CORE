# CORE-FFX User Guide

A practical reference for CORE-FFX workflows, features, and keyboard shortcuts.

---

## Getting Started

1. **Create or open a project** — File → New Project or File → Open Project (`.cffx`)
2. **Set evidence path** — use the toolbar location selector or browse for a directory
3. **Scan for evidence** — click Scan or use the menu (Tools → Scan Evidence)
4. **Explore containers** — expand items in the evidence tree to browse files
5. **Verify integrity** — hash files and compare against stored acquisition hashes

---

## Project Management

### Project Files

- **`.cffx`** — project file storing case metadata, open tabs, evidence paths, and session state (JSON)
- **`.ffxdb`** — per-project SQLite database storing bookmarks, notes, tags, activity logs, hash records, export history, COC records, and evidence collections
- Both files live in the same directory and are created together

### Toolbar Actions

| Action | Description |
|--------|-------------|
| **Save / Save As** | Save the current project (with auto-save toggle) |
| **Location Selector** | Switch between evidence, processed DB, and case document paths |
| **Scan** | Scan the current evidence path for forensic containers |
| **Hash** | Compute hashes for selected files (choose algorithm from dropdown) |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` / `Ctrl+K` | Command Palette |
| `Cmd+S` / `Ctrl+S` | Save Project |
| `Cmd+Shift+S` / `Ctrl+Shift+S` | Save Project As |
| `Cmd+B` / `Ctrl+B` | Toggle Sidebar |
| Arrow keys | Navigate file list |
| Enter | Open selected file/tab |
| Space | Toggle file selection |
| Home / End | Jump to first / last file |
| Escape | Clear filter |

---

## Evidence Formats

### Fully Parsed Containers

| Format | Extensions | Capabilities |
|--------|------------|--------------|
| AD1 | `.ad1`, `.ad2`… | Tree browsing, extraction, hash verification |
| E01/Ex01 | `.E01`, `.Ex01` | Segment verification, VFS mounting, metadata |
| L01/Lx01 | `.L01`, `.Lx01` | Logical image parsing, VFS |
| Raw Images | `.dd`, `.raw`, `.img`, `.001` | Direct byte access, VFS with filesystem parsing |
| UFED | `.ufd`, `.ufdr`, `.ufdx` | Mobile extraction parsing |
| Archives | `.zip`, `.7z`, `.rar`, `.tar`, `.gz`, `.iso`, `.dmg` | Browsing, metadata, extraction |

### Universal File Viewers

| Category | Formats |
|----------|---------|
| Documents | PDF, DOCX, DOC, PPTX, PPT, ODT, ODP, RTF, HTML, Markdown, Text |
| Images | PNG, JPEG, GIF, WebP, HEIC, BMP, TIFF + EXIF metadata |
| Email | EML, MBOX, PST |
| Spreadsheets | XLSX, XLS, CSV, ODS |
| Data | Plist, JSON, XML, SQLite, Windows Registry hives |
| Executables | PE (EXE/DLL), ELF, Mach-O |

### Triage-Only Detection

Identified during scans with basic metadata: AFF/AFF4, VMDK, VHD, VHDX, QCOW2, ISO, DMG, TAR, GZIP, XZ, BZIP2, ZSTD, LZ4

---

## Hashing & Verification

### Hash Algorithms

CORE-FFX supports: MD5, SHA-1, SHA-256, SHA-512, SHA3-256, SHA3-512, BLAKE2b, BLAKE3, xxHash.

Choose between forensic-standard algorithms (MD5/SHA-1 for compatibility with existing case records) and modern algorithms (SHA-256/BLAKE3 for speed and security).

### Stored Hash Comparison

Evidence containers may have embedded hashes from acquisition:

| Container | Stored Hashes |
|-----------|---------------|
| E01/L01 | MD5, SHA1 in header sections |
| AD1 | SHA1 in companion log files |

Results: **green** = match (verified), **red** = mismatch, **yellow** = no stored hash available.

### Reports

Report Wizard steps:

1. **Case Information** — case number, examiner, agency, dates
2. **Evidence Selection** — choose which containers and files to include
3. **Findings** — document analysis results and observations
4. **Preview** — review the full report before export
5. **Export** — generate PDF, DOCX, HTML, or Markdown

---

## Evidence Collection & Chain of Custody

### Evidence Collection

The evidence collection form is an on-site acquisition form for documenting collected items. It opens as a **center-pane tab** (not a modal).

**Entry points:**
- Right-click the report sidebar button → "Evidence Collection…"
- Command Palette (`Cmd+K`) → "Evidence Collection"

The form is schema-driven (JSON template) with auto-save. Linked data (collected items, COC records, evidence files) appears in the **right panel** when a collection tab is active.

### Chain of Custody (COC)

COC records use an **append-only immutability model** for forensic integrity:

| Status | Behavior |
|--------|----------|
| **Draft** (green) | Freely editable — can modify all fields, can be deleted |
| **Locked** (yellow) | Immutable — edits require initials + reason, creating an amendment record |
| **Voided** (red) | Soft-deleted — record persists for audit trail, hidden from active views |

Locking a COC record ensures it cannot be silently altered. All amendments and status changes are recorded in the audit log.

---

## Merge Projects

Combine multiple `.cffx` projects and their `.ffxdb` databases into a single unified project.

**Entry points:**
- Tools → Merge Projects (menu bar)
- Command Palette → "Merge Projects"

**Wizard steps:**
1. **Select** — choose `.cffx` files to merge
2. **Review** — inspect examiners, evidence files, collections, COC, and forms from each project
3. **Execute** — merge with deduplication
4. **Complete** — open the merged project

The wizard gathers examiner names from 16 sources (project owners, session users, COC officers, form submissions, etc.) and auto-suggests the project owner.

---

## Export & Image Creation

The Export Panel (Tools → Export, or `Cmd+K` → "Export") provides four modes:

### Copy Mode
Direct file/folder copy with progress tracking:
- Shows current file being copied
- Displays speed and estimated time
- Preserves file timestamps
- No hashing overhead - fastest option

### Export Mode (Forensic)
Copy with full forensic documentation:
- **SHA-256 hash computed while copying** (shown in progress)
- Hash verification after copy
- JSON manifest file with:
  - Source and destination paths
  - File sizes and timestamps
  - SHA-256 hash for each file
  - Verification status
- Progress shows both copy and hash stages

### Archive Mode
Create 7z archives (see Archive Export Feature below).

### Image Creation Mode
Create forensic disk images:
- **E01 (Physical)**: EnCase Expert Witness Format — supports EnCase 5/6/7 and V2 (Ex01) formats, compression, MD5/SHA1 hashing, case metadata, and segment splitting
- **L01 (Logical)**: EnCase Logical Evidence — collects files and directories into an L01 container with per-file hashes, compression, and case metadata

### Progress Display

The Activity Panel shows detailed progress for all operations:

| Phase | What's Happening |
|-------|------------------|
| Calculating | Scanning files, counting total size |
| Copying + Hashing | Files being copied with inline hash computation |
| Verifying | Destination files being hash-verified |
| Complete | Operation finished with summary |

For forensic export, hashing happens **simultaneously with copying** (no separate pass), which is nearly as fast as plain copy while providing verification.

---

## Hashing Segmented Evidence Containers

### How Segment Hashing Works

When you hash an evidence container in the Evidence Tree, the system automatically handles segmented files:

| Container Type | Hashing Method |
|----------------|----------------|
| **AD1** (.ad1, .ad2, .ad3...) | Hashes all segment files sequentially to produce ONE hash |
| **E01** (.E01, .E02, .E03...) | Reads and decompresses all chunks, hashes uncompressed data |
| **L01** (.L01, .L02...) | Same as E01 - decompresses and hashes content |
| **Raw** (.001, .002...) | Hashes all segment files sequentially |

### AD1 Segment Hashing
- Reads each segment file (.ad1, .ad2, .ad3, etc.) in order
- Hashes the **raw segment bytes** (container-level hash)
- Produces a single hash representing the entire evidence container
- Compare against stored hashes from acquisition logs

### E01/L01 Segment Hashing  
- Opens all segment files (.E01, .E02, etc.)
- Decompresses zlib chunks across all segments
- Hashes the **uncompressed content** (image-level hash)
- Compares against embedded MD5/SHA1 stored in the container

### Progress Display for Segmented Containers

When hashing multi-segment containers:
- Progress shows total bytes across ALL segments
- Speed reflects actual read throughput
- Decompression progress shown for E01/L01 (chunks processed)

Example progress output:
```
Hashing: evidence.E01
Segment 1/5: 45% (chunks 234/512)
Speed: 180 MB/s
```

### Stored Hash Verification

Evidence containers may have embedded hashes from acquisition:

| Container | Stored Hashes |
|-----------|---------------|
| E01/L01 | MD5, SHA1 in header sections |
| AD1 | SHA1 in companion log files |

When stored hashes are present:
- Computed hash is compared automatically
- ✅ Green = Match (integrity verified)
- ❌ Red = Mismatch (possible tampering or corruption)
- ⚠️ Yellow = No stored hash available

---

## Archive Export Feature

The Archive Export feature creates forensic-grade 7z archives for secure evidence transfer and storage.

### Why 7z Format?

- **Industry Standard**: Compatible with 7-Zip, WinRAR, macOS Archive Utility, and all major tools
- **AES-256 Encryption**: Strong password protection for sensitive evidence
- **Split Volumes**: Break large archives into manageable pieces (DVD, USB, cloud upload sizes)
- **Integrity Verification**: Built-in CRC checks ensure data integrity
- **Cross-Platform**: Works on Windows, macOS, and Linux

### Compression Levels

| Level | Name | Speed | Best For |
|-------|------|-------|----------|
| **0** | **Store** | **~500+ MB/s** | **E01/AD1/L01 containers (already compressed)** |
| 1 | Fastest | ~180 MB/s | Text files, logs, uncompressed data |
| 3 | Fast | ~80 MB/s | Mixed content |
| 5 | Normal | ~22 MB/s | General purpose |
| 7 | Maximum | ~12 MB/s | Space-critical storage |
| 9 | Ultra | ~9 MB/s | Maximum compression (slow) |

### Recommended Settings for Forensics

**Default: Store (Level 0)** - This is intentional!

Forensic evidence containers are already internally compressed:

| Container | Internal Compression |
|-----------|---------------------|
| E01/EWF | zlib |
| AD1 | zlib |
| L01/Lx01 | zlib |
| AFF4 | zlib/lz4 |

Attempting to re-compress already-compressed data:
- ❌ Wastes CPU time (hours on large files)
- ❌ Produces the same output size
- ❌ No benefit whatsoever

Store mode packages files at disk I/O speed while still providing:
- ✅ Single archive file
- ✅ AES-256 encryption
- ✅ Split volumes
- ✅ Integrity checks

### When to Use Compression

Use **Fastest (Level 1)** or higher only for:
- Raw disk images (.dd, .raw) that aren't compressed
- Text files, logs, CSV exports
- Uncompressed document collections
- Memory dumps (.mem, .vmem)

### Encryption

Password protection uses AES-256-CBC encryption:
- Virtually zero performance overhead (<1%)
- Encrypts both file content and filenames
- Compatible with all 7z tools

**Best Practices:**
- Use strong passwords (12+ characters, mixed case, numbers, symbols)
- Document passwords securely in case notes
- Test extraction before deleting originals

### Split Archives

For large evidence, split volumes are enabled by default at **2 GB** - ideal for:
- Cloud upload limits (many services cap at 2-5 GB per file)
- USB flash drives (FAT32 has 4GB file limit)
- Email attachments (when zipped further)
- Easier re-transmission if transfer fails

Available split sizes:
- **700 MB**: CD-R size
- **2 GB**: Cloud/USB - **Default & Recommended**
- **4.7 GB**: DVD-R size
- **8.5 GB**: DVD DL size
- **25 GB**: Blu-ray size  
- **50 GB**: Blu-ray DL size
- **No Split**: Single file (for local storage)

Split archives create files like:
```
evidence.7z.001  (first volume)
evidence.7z.002  (second volume)
evidence.7z.003  (etc.)
```

To extract, open the `.001` file - 7z tools automatically read all volumes.

### Performance Expectations

| Evidence Size | Store Time | Fastest Time | Normal Time |
|---------------|------------|--------------|-------------|
| 10 GB | ~20 sec | ~55 sec | ~7.5 min |
| 50 GB | ~100 sec | ~4.5 min | ~38 min |
| 100 GB | ~3.5 min | ~9 min | ~75 min |
| 500 GB | ~17 min | ~46 min | ~6.3 hours |

*Times assume SSD storage. HDD will be disk I/O limited.*

### Activity Panel

While archiving, the right panel shows:
- Current file being processed
- Bytes processed / total
- Speed (MB/s)
- Estimated time remaining
- Progress percentage

### Verification

After archive creation:
1. Open the archive with 7-Zip or similar tool
2. Run "Test Archive" to verify integrity
3. Optionally extract to a test location and compare hashes

### Troubleshooting

**Archive creation is slow:**
- Check compression level (use Store for E01/AD1)
- Verify disk isn't the bottleneck (check Activity Monitor)

**Archive won't open:**
- Verify correct password
- Check all split volumes are present
- Ensure volumes weren't corrupted in transfer

**Out of memory on large archives:**
- Archives >8GB automatically use streaming mode
- Split volumes are auto-enabled for very large files

---

## Further Reading

| Document | Purpose |
|----------|---------|
| [`README.md`](README.md) | Product overview and feature summary |
| [`CODE_BIBLE.md`](CODE_BIBLE.md) | Codebase map, module responsibilities, glossary |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Developer setup, coding standards, PR process |
| [`docs/FORM_TEMPLATE_SYSTEM.md`](docs/FORM_TEMPLATE_SYSTEM.md) | JSON schema form system documentation |
| [`docs/SEVENZIP_FFI_API_REFERENCE.md`](docs/SEVENZIP_FFI_API_REFERENCE.md) | 7z FFI C API reference |
